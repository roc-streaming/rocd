// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::io_endpoints::EndpointDispatcher;
use crate::io_streams::StreamDispatcher;
use crate::p2p::PeerDispatcher;
use crate::rest_api::api_controller::ApiController;
use crate::rest_api::doc_controller::DocController;
use crate::rest_api::error::ServerError;

use axum::extract::Request;
use axum::{Router, ServiceExt};
use axum_server::Server;
use std::io;
use std::net::SocketAddr;
use std::result;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower::Layer;
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::trace::TraceLayer;

type Result<T> = result::Result<T, ServerError>;

type ServerHandle = axum_server::Handle;
type TaskHandle = tokio::task::JoinHandle<io::Result<()>>;

/// Runs HTTP server with REST API and OpenAPI docs.
pub struct RestServer {
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
        peer_dispatcher: &Arc<PeerDispatcher>, endpoint_dispatcher: &Arc<EndpointDispatcher>,
        stream_dispatcher: &Arc<StreamDispatcher>,
    ) -> Self {
        let mut router = Router::new();
        let spec;

        {
            let api_controller = Arc::new(ApiController::new(
                peer_dispatcher,
                endpoint_dispatcher,
                stream_dispatcher,
            ));

            let (api_router, api_spec) = api_controller.router_with_spec();

            router = router.merge(api_router);
            spec = api_spec;
        }

        {
            let doc_controller = Arc::new(DocController::new(spec));
            let doc_router = doc_controller.router();

            router = router.merge(doc_router);
        }

        RestServer {
            state: Mutex::new(ServerState {
                router: Some(router),
                server_handle: None,
                task_handle: None,
            }),
        }
    }

    /// Runs http server with given router.
    async fn serve_with_router(server: Server, router: Router) -> io::Result<()> {
        let service = router;

        // add TraceLayer layer
        let service = service.layer(TraceLayer::new_for_http());

        // add NormalizePathLayer layer
        let service = ServiceExt::<Request>::into_make_service(
            NormalizePathLayer::trim_trailing_slash().layer(service),
        );

        server.serve(service).await
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

        let router = locked_state.router.take().ok_or(ServerError::StateError)?;

        let tcp_listener = TcpListener::bind(addr)
            .await
            .map_err(|err| ServerError::BindError(err))?
            .into_std()
            .map_err(|err| ServerError::BindError(err))?;

        let resolved_addr =
            tcp_listener.local_addr().map_err(|err| ServerError::BindError(err))?;

        tracing::info!("starting server at http://{}", resolved_addr);

        let server_handle = ServerHandle::new();
        let server = Server::from_tcp(tcp_listener).handle(server_handle.clone());

        let task_handle =
            tokio::spawn(async move { RestServer::serve_with_router(server, router).await });

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
        .ok_or(ServerError::StateError)?;

        tracing::debug!("waiting server");
        task_handle
            .await
            .map_err(|err| ServerError::TokioError(err))?
            .map_err(|err| ServerError::ServeError(err))?;

        Ok(())
    }
}
