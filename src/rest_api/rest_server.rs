// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::io_endpoint::EndpointDispatcher;
use crate::io_stream::StreamDispatcher;
use crate::rest_api::rest_controller::RestController;

use salvo::logging::Logger;
use salvo::prelude::*;
use salvo::server::{Server, ServerHandle};
use std::io::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

pub struct RestServer {
    controller: Arc<RestController>,
    router: Arc<Router>,
    state: Mutex<ServerState>,
}

struct ServerState {
    /// used to stop running server
    stop_handle: Option<ServerHandle>,
    /// used to wait server results
    task_handle: Option<JoinHandle<Result<()>>>,
}

impl RestServer {
    /// Create unstarted server.
    pub fn new(
        endpoint_dispatcher: Arc<EndpointDispatcher>, stream_dispatcher: Arc<StreamDispatcher>,
    ) -> Self {
        let controller = Arc::new(RestController::new(endpoint_dispatcher, stream_dispatcher));

        // api router
        let mut router = controller.router();

        let openapi_spec = controller.openapi();
        let openapi_docs = ReDoc::new("/openapi/openapi.json");

        router = router
            // serve json spec
            .push(openapi_spec.into_router("/openapi/openapi.json"))
            // serve html docs
            .push(openapi_docs.into_router("/openapi/"));

        RestServer {
            controller,
            router: Arc::new(router),
            state: Mutex::new(ServerState { stop_handle: None, task_handle: None }),
        }
    }

    /// Bind to address and run server in background.
    ///
    /// Must be called once after new().
    ///
    /// start().await doesn't wait for server to finish, it just waits
    /// when bind is complete and server is started.
    ///
    /// Call wait() to wait server to finish.
    pub async fn start(self: &Arc<Self>, host: &str, port: u16) -> Result<SocketAddr> {
        let mut locked_state = self.state.lock().await;

        if locked_state.stop_handle.is_some() {
            panic!("bad state");
        }

        let str_addr = format!("{}:{}", host, port);

        let tcp_acceptor = TcpListener::new(str_addr).bind().await;
        let tcp_addr = tcp_acceptor.local_addr()?;

        tracing::info!("starting server at {} ...", tcp_addr);

        let service = Service::new(self.router.clone()).hoop(Logger::new());
        let server = Server::new(tcp_acceptor);

        // run server in background
        let stop_handle = server.handle();
        let task_handle = tokio::spawn(async move {
            server.try_serve(service).await
        });

        locked_state.stop_handle = Some(stop_handle);
        locked_state.task_handle = Some(task_handle);

        Ok(tcp_addr)
    }

    /// Tell server to stop.
    ///
    /// May be called multiple times, at any time, from any thread.
    ///
    /// stop().await doesn't wait for server to finish, it just tells
    /// server that it should stop.
    ///
    /// Call wait() to wait server to finish.
    pub async fn stop(self: &Arc<Self>) {
        let stop_handle = {
            let mut locked_state = self.state.lock().await;
            locked_state.stop_handle.take()
        };

        if let Some(handle) = stop_handle {
            tracing::debug!("stopping server");
            handle.stop_forcible();
        }
    }

    /// Wait until server is finished and return result.
    ///
    /// Must be called once after start().
    ///
    /// Won't return until server fails or stop() is called.
    pub async fn wait(self: &Arc<Self>) -> Result<()> {
        let task_handle = {
            let mut locked_state = self.state.lock().await;

            if locked_state.task_handle.is_none() {
                panic!("bad state");
            }

            locked_state.task_handle.take().unwrap()
        };

        tracing::debug!("waiting server");
        task_handle.await.unwrap()
    }
}
