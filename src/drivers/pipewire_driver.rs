// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::drivers::driver::*;
use crate::drivers::error::*;
use crate::dto::DriverId;

use async_trait::async_trait;
use pipewire::main_loop::MainLoop;
use std::sync::Arc;
use std::thread::{self};
use tokio::sync::Mutex;
use tokio::task;

/// Driver implementation for pipewire.
pub struct PipewireDriver {
    worker: Arc<PipewireLoop>,
    worker_handle: Mutex<Option<thread::JoinHandle<()>>>,
}

#[async_trait]
impl Driver for PipewireDriver {
    async fn open() -> DriverResult<Arc<dyn Driver>> {
        tracing::debug!("opening pipewire driver");

        let (worker, worker_handle) = PipewireLoop::start().await?;

        let driver = Arc::new(PipewireDriver {
            worker,
            worker_handle: Mutex::new(Some(worker_handle)),
        });

        Ok(driver)
    }

    async fn close(self: Arc<Self>) {
        let worker_handle = { self.worker_handle.lock().await.take() };

        if let Some(worker_handle) = worker_handle {
            tracing::debug!("closing pipewire driver");

            _ = self.worker.exec(Cmd::Close).await;

            task::spawn_blocking(move || {
                worker_handle.join().expect("pipewire thread panicked");
            })
            .await
            .expect("task panicked");
        }
    }

    fn id(&self) -> DriverId {
        DriverId::Pipewire
    }
}

#[derive(Debug)]
struct PipewireTask {
    cmd: Cmd,
    output: tokio::sync::oneshot::Sender<CmdOutput>,
}

#[derive(Debug)]
enum Cmd {
    Close,
}

#[derive(Debug)]
enum CmdOutput {
    None,
}

struct PipewireLoop {
    task_sender: pipewire::channel::Sender<PipewireTask>,
}

impl PipewireLoop {
    /// Start pipewire mainloop.
    async fn start() -> DriverResult<(Arc<PipewireLoop>, thread::JoinHandle<()>)> {
        let (open_sender, open_receiver) = tokio::sync::oneshot::channel();
        let (task_sender, task_receiver) = pipewire::channel::channel();

        let worker = Arc::new(PipewireLoop { task_sender });

        let worker_handle = thread::spawn({
            let worker = Arc::clone(&worker);
            move || worker.run_loop(open_sender, task_receiver)
        });

        let open_result = open_receiver.await.map_err(|_err| DriverError::ConnectionError)?;
        open_result?;

        Ok((worker, worker_handle))
    }

    /// Execute command on pipewire mainloop and return its result.
    async fn exec(self: &Arc<Self>, cmd: Cmd) -> DriverResult<CmdOutput> {
        let (result_tx, result_rx) = tokio::sync::oneshot::channel();

        // send task to main loop
        let send_result = task::spawn_blocking({
            let self_clone = Arc::clone(self);
            move || {
                // writing to pipewire::channel may block
                self_clone.task_sender.send(PipewireTask { cmd, output: result_tx })
            }
        })
        .await
        .expect("task panicked");

        send_result.map_err(|_err| DriverError::ConnectionError)?;

        // wait result from main loop
        let recv_result = result_rx.await;

        recv_result.map_err(|_err| DriverError::ConnectionError)
    }

    /// Mainloop thread.
    /// Communicates with pipewire and executes tasks.
    fn run_loop(
        self: &Arc<Self>, open_sender: tokio::sync::oneshot::Sender<DriverResult<()>>,
        task_receiver: pipewire::channel::Receiver<PipewireTask>,
    ) {
        tracing::debug!("entering pipewire mainloop");

        let mainloop = match MainLoop::new(None) {
            Ok(mainloop) => {
                tracing::trace!("sending open ok");
                open_sender.send(Ok(())).unwrap();
                mainloop
            },
            Err(err) => {
                tracing::trace!("sending open err");
                open_sender.send(Err(DriverError::OpenError(err.to_string()))).unwrap();
                return;
            },
        };

        let _handle = task_receiver.attach(mainloop.loop_(), {
            let self_clone = Arc::clone(&self);
            let mainloop_clone = mainloop.clone();

            move |task| {
                tracing::trace!("received task");
                let cmd_out = self_clone.process_cmd(&mainloop_clone, &task.cmd);

                tracing::trace!("sending task result");
                task.output.send(cmd_out).unwrap();
            }
        });

        mainloop.run();

        tracing::debug!("leaving pipewire mainloop");
    }

    /// Process command from task.
    fn process_cmd(self: &Arc<Self>, mainloop: &MainLoop, cmd: &Cmd) -> CmdOutput {
        tracing::debug!("processing command {:?}", cmd);

        match cmd {
            Cmd::Close => {
                mainloop.quit();
                CmdOutput::None
            },
        }
    }
}
