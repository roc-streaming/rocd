// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
#![allow(non_upper_case_globals)]

use crate::drivers::driver::*;
use crate::drivers::error::*;
use crate::dto::DriverId;

use async_trait::async_trait;
use libspa::param::ParamType;
use libspa::pod::deserialize::PodDeserializer;
use libspa::pod::{Object, Pod, Value};
use libspa::utils::dict::DictRef;
use libspa::utils::result::AsyncSeq;
use libspa_sys::SPA_PROP_device;
use pipewire::context::ContextRc;
use pipewire::core::{CoreRc, Info};
use pipewire::main_loop::MainLoopRc;
use pipewire::node::{Node, NodeListener};
use pipewire::registry::{GlobalObject, RegistryRc};
use pipewire::types::ObjectType;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::thread::{self};
use tokio::sync::Mutex;
use tokio::task;

/// Driver implementation for pipewire.
pub struct PipewireDriver {
    task_tx: pipewire::channel::Sender<PwTask>,
    thread_handle: Mutex<Option<thread::JoinHandle<()>>>,
}

#[async_trait]
impl Driver for PipewireDriver {
    async fn open() -> DriverResult<Arc<dyn Driver>> {
        tracing::debug!("opening pipewire driver");

        let (open_tx, open_rx) = tokio::sync::oneshot::channel();
        let (task_tx, task_rx) = pipewire::channel::channel();

        let thread_handle = thread::spawn(move || {
            let pw_loop = match PwLoop::open_and_connect() {
                Ok(pw_loop) => {
                    tracing::trace!("sending open ok");
                    open_tx.send(Ok(())).unwrap();
                    pw_loop
                },
                Err(err) => {
                    tracing::trace!("sending open err");
                    open_tx.send(Err(err)).unwrap();
                    return;
                },
            };

            pw_loop.run(task_rx);
        });

        tracing::trace!("waiting open result");
        let open_result = open_rx.await.map_err(|_err| DriverError::ConnectionError)?;
        open_result?;

        Ok(Arc::new(PipewireDriver {
            task_tx,
            thread_handle: Mutex::new(Some(thread_handle)),
        }))
    }

    async fn close(self: Arc<Self>) {
        tracing::debug!("closing pipewire driver");

        let thread_handle = { self.thread_handle.lock().await.take() };

        if let Some(thread_handle) = thread_handle {
            _ = self.round_trip(PwReq::Close).await;

            tracing::trace!("waiting thread");
            task::spawn_blocking(move || {
                thread_handle.join().expect("pipewire thread panicked");
            })
            .await
            .expect("task panicked");
        }
    }

    fn id(&self) -> DriverId {
        DriverId::Pipewire
    }
}

impl PipewireDriver {
    /// Send PwReq to pipewire thread and wait PwResp.
    /// PwReq + PwResp are packed into PwTask.
    async fn round_trip(self: &Arc<Self>, req: PwReq) -> DriverResult<PwResp> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

        tracing::trace!("scheduling task");
        self.task_tx
            .send(PwTask { req, resp_tx })
            .map_err(|_err| DriverError::ConnectionError)?;

        tracing::trace!("waiting task result");
        resp_rx.await.map_err(|_err| DriverError::ConnectionError)
    }
}

/// Async task for pipewire mainloop.
#[derive(Debug)]
struct PwTask {
    req: PwReq,
    resp_tx: tokio::sync::oneshot::Sender<PwResp>,
}

#[derive(Debug)]
enum PwReq {
    Close,
}

#[derive(Debug)]
enum PwResp {
    None,
}

struct PwDev {
    node: Node,
    node_listener: NodeListener,
    props: PwProps,
}

#[derive(Debug, Default, PartialEq)]
struct PwProps {
    device: String,
    volume: f32,
}

/// Single-threaded pipewire mainloop.
struct PwLoop {
    mainloop: MainLoopRc,
    context: ContextRc,
    conn: RefCell<Option<PwConn>>,
}

struct PwConn {
    core: CoreRc,
    core_listener: pipewire::core::Listener,
    registry: RegistryRc,
    registry_listener: pipewire::registry::Listener,
    devices: HashMap<u32, PwDev>,
}

impl PwLoop {
    fn open_and_connect() -> DriverResult<Rc<PwLoop>> {
        tracing::debug!("creating mainloop");

        let mainloop = MainLoopRc::new(None).map_err(|err| {
            DriverError::OpenError(format!("can't open pipewire main loop: {err}"))
        })?;

        let context = ContextRc::new(&mainloop, None).map_err(|err| {
            DriverError::OpenError(format!("can't open pipewire context: {err}"))
        })?;

        let pw_loop = Rc::new(PwLoop { mainloop, context, conn: RefCell::new(None) });
        pw_loop.reconnect()?;

        Ok(pw_loop)
    }

    fn reconnect(self: &Rc<Self>) -> DriverResult<()> {
        tracing::debug!("connecting to pipewire");

        // reset previous connection, if any
        drop(self.conn.take());

        let core = self.context.connect_rc(None).map_err(|err| {
            DriverError::OpenError(format!("can't connect to pipewire: {err}"))
        })?;

        let core_listener = core
            .add_listener_local()
            .info({
                let this = Rc::clone(self);
                move |info| this.on_core_info(info)
            })
            .done({
                let this = Rc::clone(self);
                move |id, seq| this.on_core_done(id, seq)
            })
            .error({
                let this = Rc::clone(self);
                move |id, seq, res, msg| this.on_core_error(id, seq, res, msg)
            })
            .register();

        let registry = core.get_registry_rc().map_err(|err| {
            DriverError::OpenError(format!("can't get pipewire registry: {err}"))
        })?;

        let registry_listener = registry
            .add_listener_local()
            .global({
                let this = Rc::clone(self);
                move |obj| this.on_registry_add(obj)
            })
            .global_remove({
                let this = Rc::clone(self);
                move |obj_id| this.on_registry_remove(obj_id)
            })
            .register();

        _ = self.conn.borrow_mut().insert(PwConn {
            core,
            core_listener,
            registry,
            registry_listener,
            devices: HashMap::new(),
        });
        Ok(())
    }

    fn run(self: &Rc<Self>, task_rx: pipewire::channel::Receiver<PwTask>) {
        tracing::debug!("entering mainloop");

        let _task_handler = task_rx.attach(self.mainloop.loop_(), {
            let this = Rc::clone(self);
            move |task| this.on_task(task)
        });

        self.mainloop.run();

        tracing::debug!("leaving mainloop");
    }

    /// Called by core_listener when connected to core.
    fn on_core_info(self: &Rc<Self>, info: &Info) {
        tracing::trace!("on_core_info: {:?}", info);

        tracing::debug!(
            "connected to pipewire: version={} cookie={}",
            info.version(),
            info.cookie()
        );
    }

    /// Called by core_listener after core.sync.
    fn on_core_done(self: &Rc<Self>, id: u32, seq: AsyncSeq) {
        tracing::trace!("on_core_done: id={:?} seq={:?}", id, seq);

        // TODO: handle initial sync
    }

    /// Called by core_listener on asynchronous error.
    fn on_core_error(self: &Rc<Self>, id: u32, seq: i32, res: i32, msg: &str) {
        tracing::trace!("on_core_error id={:?} seq={:?} res={:?} msg={:?}", id, seq, res, msg);

        tracing::warn!("got error from pipewire: {}", msg);

        // TODO: reconnect, emit event (if id == 0)
    }

    /// Called by registry_listener when a global is added.
    /// The most interesting type of 'global' is 'node'.
    /// Audio devices are nodes.
    fn on_registry_add(self: &Rc<Self>, obj: &GlobalObject<&DictRef>) {
        tracing::trace!("on_registry_add: obj={:?}", obj);

        let mut conn_ref = self.conn.borrow_mut();
        if !conn_ref.is_some() {
            return;
        }
        let conn = conn_ref.as_mut().unwrap();

        match obj.type_ {
            ObjectType::Node => {
                let node: Node = conn.registry.bind(obj).unwrap();

                let node_listener = node
                    .add_listener_local()
                    .param({
                        let this = Rc::clone(self);
                        move |seq, param_type, param_index, _next, param| {
                            this.on_node_param(seq, param_type, param_index, param);
                        }
                    })
                    .register();

                node.subscribe_params(&[ParamType::Props]);

                conn.devices
                    .insert(obj.id, PwDev { node, node_listener, props: PwProps::default() });
            },

            _ => (),
        };

        tracing::debug!("object added: {}", obj.id);
        // TODO: update list, emit event
    }

    /// Called by registry_listener when a global is removed.
    fn on_registry_remove(self: &Rc<Self>, obj_id: u32) {
        tracing::trace!("on_registry_remove: obj_id={:?}", obj_id);

        tracing::debug!("object removed: {}", obj_id);
        // TODO: update list, emit event
    }

    /// Called by node_listener when node parameter is changed.
    /// The most interesting type of node parameter is 'Props'.
    /// Things like device muted state are represented as fields inside
    /// 'Props' parameter of 'node' object of the device.
    fn on_node_param(
        self: &Rc<Self>, seq: i32, param_type: ParamType, param_index: u32,
        param: Option<&Pod>,
    ) {
        tracing::trace!(
            "on_node_param: seq={:?} param_type={:?} param_index={:?}",
            seq,
            param_type,
            param_index
        );

        match param_type {
            ParamType::Props if param.is_some() && param.unwrap().is_object() => {
                if let Ok((_, value)) =
                    PodDeserializer::deserialize_any_from(param.unwrap().as_bytes())
                {
                    if let Value::Object(obj) = value {
                        self.on_node_props(obj);
                    }
                }
            },
            _ => (),
        }
    }

    /// Called from on_node_param() when node 'Props' parameter is changed.
    /// 'Value' contains decoded properties.
    fn on_node_props(self: &Rc<Self>, obj: Object) {
        tracing::trace!("on_node_props: obj={:?}", obj);

        let mut props = PwProps::default();

        for prop in obj.properties {
            match prop.key {
                SPA_PROP_device => {
                    if let Value::String(value) = prop.value {
                        props.device = from_pw_string(value);
                    }
                },
                _ => (),
            }
        }

        tracing::debug!("on_node_props: props={:?}", props);

        // TODO: read relevant properties into PwProps
    }

    fn on_task(self: &Rc<Self>, task: PwTask) {
        tracing::trace!("on_task: {:?}", task.req);
        let resp = self.request(&task.req);

        tracing::trace!("task response: {:?}", resp);
        task.resp_tx.send(resp).unwrap();
    }

    fn request(self: &Rc<Self>, req: &PwReq) -> PwResp {
        match req {
            PwReq::Close => {
                self.mainloop.quit();
                PwResp::None
            },
        }
    }
}

fn from_pw_string(mut s: String) -> String {
    while s.ends_with('\0') {
        s.pop();
    }
    s
}
