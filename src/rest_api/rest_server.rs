// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::io_endpoint::EndpointDispatcher;
use crate::io_stream::StreamDispatcher;
use crate::rest_api::rest_controller::RestController;

use salvo::logging::Logger;
use salvo::prelude::*;
use std::io::Result;
use std::sync::Arc;

pub struct RestServer {
    controller: Arc<RestController>,
    router: Arc<Router>,
    openapi: OpenApi,
}

impl RestServer {
    pub fn new(
        endpoint_dispatcher: Arc<EndpointDispatcher>, stream_dispatcher: Arc<StreamDispatcher>,
    ) -> Self {
        let controller = Arc::new(RestController::new(endpoint_dispatcher, stream_dispatcher));

        let mut router = controller.router();

        let openapi =
            OpenApi::new("rocd REST API", env!("CARGO_PKG_VERSION")).merge_router(&router);

        router = router
            .push(openapi.clone().into_router("/openapi/openapi.json"))
            .push(ReDoc::new("/openapi/openapi.json").into_router("/openapi/"));

        RestServer { controller, router: Arc::new(router), openapi }
    }

    pub async fn serve(&self, host: &str, port: u16) -> Result<()> {
        tracing::info!("starting server at {}:{} ...", host, port);

        let service = Service::new(self.router.clone()).hoop(Logger::new());
        let acceptor = TcpListener::new(format!("{}:{}", host, port)).bind().await;

        Server::new(acceptor).try_serve(service).await
    }

    pub fn openapi_json(&self) -> String {
        self.openapi.to_pretty_json().unwrap() + "\n"
    }

    pub fn openapi_yaml(&self) -> String {
        self.openapi.to_yaml().unwrap()
    }
}
