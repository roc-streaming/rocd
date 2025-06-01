// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::drivers::driver::*;
use crate::drivers::error::*;
use crate::dto::DriverId;

use async_trait::async_trait;
use pipewire::main_loop::MainLoop;
use std::rc::Rc;
use std::sync::Arc;
use std::thread::{self};
use tokio::sync::Mutex;
use tokio::task;

/// Driver implementation for pipewire.
pub struct PipewireDriver {
    conn: Arc<PwConn>,
}

#[async_trait]
impl Driver for PipewireDriver {
    async fn open() -> DriverResult<Arc<dyn Driver>> {
        tracing::debug!("opening pipewire driver");

        let conn = PwConn::open().await?;

        Ok(Arc::new(PipewireDriver { conn: Arc::new(conn) }))
    }

    async fn close(self: Arc<Self>) {
        tracing::debug!("closing pipewire driver");

        self.conn.close().await;
    }

    fn id(&self) -> DriverId {
        DriverId::Pipewire
    }
}

/// Async task for pipewire mainloop.
#[derive(Debug)]
struct PwTask {
    cmd: PwCmd,
    out: tokio::sync::oneshot::Sender<PwCmdOut>,
}

#[derive(Debug)]
enum PwCmd {
    Close,
}

#[derive(Debug)]
enum PwCmdOut {
    None,
}

/// Async connector to pipewire mainloop.
struct PwConn {
    task_tx: pipewire::channel::Sender<PwTask>,
    thread_handle: Mutex<Option<thread::JoinHandle<()>>>,
}

impl PwConn {
    async fn open() -> DriverResult<PwConn> {
        let (open_tx, open_rx) = tokio::sync::oneshot::channel();
        let (task_tx, task_rx) = pipewire::channel::channel();

        let thread_handle = thread::spawn(move || {
            let pw_loop = match PwLoop::new() {
                Ok(pw_loop) => {
                    tracing::trace!("sending open ok");
                    open_tx.send(Ok(())).unwrap();
                    Rc::new(pw_loop)
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

        Ok(PwConn { task_tx, thread_handle: Mutex::new(Some(thread_handle)) })
    }

    async fn close(self: &Arc<Self>) {
        let thread_handle = { self.thread_handle.lock().await.take() };

        if let Some(thread_handle) = thread_handle {
            _ = self.task(PwCmd::Close).await;

            tracing::trace!("waiting thread");
            task::spawn_blocking(move || {
                thread_handle.join().expect("pipewire thread panicked");
            })
            .await
            .expect("task panicked");
        }
    }

    async fn task(self: &Arc<Self>, cmd: PwCmd) -> DriverResult<PwCmdOut> {
        let (out_tx, out_rx) = tokio::sync::oneshot::channel();

        tracing::trace!("scheduling task");
        let send_result = task::spawn_blocking({
            let this = Arc::clone(self);
            move || this.task_tx.send(PwTask { cmd, out: out_tx })
        })
        .await
        .expect("task panicked");

        send_result.map_err(|_err| DriverError::ConnectionError)?;

        tracing::trace!("waiting task result");
        let recv_result = out_rx.await;

        recv_result.map_err(|_err| DriverError::ConnectionError)
    }
}

/// Single-threaded pipewire mainloop.
struct PwLoop {
    mainloop: MainLoop,
}

impl PwLoop {
    fn new() -> DriverResult<PwLoop> {
        tracing::debug!("creating mainloop");

        let mainloop =
            MainLoop::new(None).map_err(|err| DriverError::OpenError(err.to_string()))?;

        Ok(PwLoop { mainloop })
    }

    fn run(self: &Rc<Self>, task_rx: pipewire::channel::Receiver<PwTask>) {
        tracing::debug!("running mainloop");

        let _rx_handle = task_rx.attach(self.mainloop.loop_(), {
            let this = Rc::clone(self);
            move |task| this.process_task(task)
        });

        self.mainloop.run()
    }

    fn process_task(self: &Rc<Self>, task: PwTask) {
        tracing::trace!("processing command: {:?}", task.cmd);
        let cmd_out = self.process_cmd(&task.cmd);

        tracing::trace!("sending result: {:?}", cmd_out);
        task.out.send(cmd_out).unwrap();
    }

    fn process_cmd(self: &Rc<Self>, cmd: &PwCmd) -> PwCmdOut {
        match cmd {
            PwCmd::Close => {
                self.mainloop.quit();
                PwCmdOut::None
            },
        }
    }
}
