// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::io_endpoint::EndpointDispatcher;
use crate::io_stream::StreamDispatcher;
use crate::rest_api::controller::RestController;
use crate::rest_api::error::RestError;

use axum::Router;
use axum_server::Server;
use std::io;
use std::net::SocketAddr;
use std::result;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use utoipa_redoc::{Redoc, Servable};

type Result<T> = result::Result<T, RestError>;

type ServerHandle = axum_server::Handle;
type TaskHandle = tokio::task::JoinHandle<io::Result<()>>;

pub struct RestServer {
    controller: Arc<RestController>,
    state: Mutex<ServerState>,
}

struct ServerState {
    router: Option<Router>,
    server_handle: Option<ServerHandle>,
    task_handle: Option<TaskHandle>,
}

impl RestServer {
    /// Create unstarted server.
    pub fn new(
        endpoint_dispatcher: Arc<EndpointDispatcher>, stream_dispatcher: Arc<StreamDispatcher>,
    ) -> Self {
        let controller = Arc::new(RestController::new(endpoint_dispatcher, stream_dispatcher));

        let (mut router, api) = controller.router().split_for_parts();

        router = router
            // serve html docs
            .merge(Redoc::with_url("/openapi", api));

        RestServer {
            controller,
            state: Mutex::new(ServerState {
                router: Some(router),
                server_handle: None,
                task_handle: None,
            }),
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
    ///
    /// Returned address is the same as passed to argument, except that
    /// if the port was zero, the actual port is returned.
    pub async fn start(self: &Arc<Self>, addr: SocketAddr) -> Result<SocketAddr> {
        let mut locked_state = self.state.lock().await;

        let router = locked_state.router.take().ok_or(RestError::StateError)?;

        let tcp_listener = TcpListener::bind(addr)
            .await
            .map_err(|err| RestError::BindError(err))?
            .into_std()
            .map_err(|err| RestError::BindError(err))?;

        let resolved_addr =
            tcp_listener.local_addr().map_err(|err| RestError::BindError(err))?;

        tracing::info!("starting server at http://{} ...", resolved_addr);

        let server_handle = ServerHandle::new();
        let server = Server::from_tcp(tcp_listener).handle(server_handle.clone());

        let task_handle =
            tokio::spawn(async move { server.serve(router.into_make_service()).await });

        locked_state.server_handle = Some(server_handle);
        locked_state.task_handle = Some(task_handle);

        Ok(resolved_addr)
    }

    /// Tell server to stop.
    ///
    /// May be called multiple times, at any time, from any thread.
    ///
    /// stop().await doesn't wait for server to finish, it just tells
    /// server that it should stop and returns.
    ///
    /// Call wait() to wait server to finish.
    pub async fn stop(self: &Arc<Self>) {
        let server_handle = {
            let mut locked_state = self.state.lock().await;
            locked_state.server_handle.take()
        };

        if let Some(handle) = server_handle {
            tracing::debug!("stopping server");
            handle.shutdown();
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
            locked_state.task_handle.take()
        }
        .ok_or(RestError::StateError)?;

        tracing::debug!("waiting server");
        task_handle
            .await
            .map_err(|err| RestError::TokioError(err))?
            .map_err(|err| RestError::ServeError(err))?;

        Ok(())
    }
}
