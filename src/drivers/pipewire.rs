// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::drivers::driver::*;
use crate::drivers::error::*;
use crate::dto::DriverId;

use async_trait::async_trait;
use libspa::utils::dict::DictRef;
use libspa::utils::result::AsyncSeq;
use pipewire::context::Context;
use pipewire::core::{Core, Info};
use pipewire::main_loop::MainLoop;
use pipewire::registry::{GlobalObject, Registry};
use std::cell::RefCell;
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

/// Single-threaded pipewire mainloop.
struct PwLoop {
    mainloop: MainLoop,
    context: Context,
    conn: RefCell<Option<PwConn>>,
}

struct PwConn {
    core: Core,
    core_listener: pipewire::core::Listener,
    registry: Registry,
    registry_listener: pipewire::registry::Listener,
}

impl PwLoop {
    fn open_and_connect() -> DriverResult<Rc<PwLoop>> {
        tracing::debug!("creating mainloop");

        let mainloop = MainLoop::new(None).map_err(|err| {
            DriverError::OpenError(format!("can't open pipewire main loop: {err}"))
        })?;

        let context = Context::new(&mainloop).map_err(|err| {
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

        let core = self.context.connect(None).map_err(|err| {
            DriverError::OpenError(format!("can't connect to pipewire: {err}"))
        })?;

        let core_listener = core
            .add_listener_local()
            .info({
                let this = Rc::clone(self);
                move |info| this.on_info(info)
            })
            .done({
                let this = Rc::clone(self);
                move |id, seq| this.on_done(id, seq)
            })
            .error({
                let this = Rc::clone(self);
                move |id, seq, res, msg| this.on_error(id, seq, res, msg)
            })
            .register();

        let registry = core.get_registry().map_err(|err| {
            DriverError::OpenError(format!("can't get pipewire registry: {err}"))
        })?;

        let registry_listener = registry
            .add_listener_local()
            .global({
                let this = Rc::clone(self);
                move |obj| this.on_add(obj)
            })
            .global_remove({
                let this = Rc::clone(self);
                move |obj_id| this.on_remove(obj_id)
            })
            .register();

        _ = self.conn.borrow_mut().insert(PwConn {
            core,
            core_listener,
            registry,
            registry_listener,
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

    fn on_info(self: &Rc<Self>, info: &Info) {
        tracing::trace!("on_info: {:?}", info);

        tracing::debug!(
            "connected to pipewire: version={} cookie={}",
            info.version(),
            info.cookie()
        );
    }

    fn on_done(self: &Rc<Self>, id: u32, seq: AsyncSeq) {
        tracing::trace!("on_done: id={:?} seq={:?}", id, seq);

        // TODO: handle initial sync
    }

    fn on_error(self: &Rc<Self>, id: u32, seq: i32, res: i32, msg: &str) {
        tracing::trace!("on_error id={:?} seq={:?} res={:?} msg={:?}", id, seq, res, msg);

        tracing::warn!("got error from pipewire: {}", msg);

        // TODO: reconnect, emit event (if id == 0)
    }

    fn on_add(self: &Rc<Self>, obj: &GlobalObject<&DictRef>) {
        tracing::trace!("on_add: obj={:?}", obj);

        tracing::debug!("object added: {}", obj.id);
        // TODO: update list, emit event
    }

    fn on_remove(self: &Rc<Self>, obj_id: u32) {
        tracing::trace!("on_remove: obj_id={:?}", obj_id);

        tracing::debug!("object removed: {}", obj_id);
        // TODO: update list, emit event
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
