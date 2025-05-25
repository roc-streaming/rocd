// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::*;
use crate::io_endpoint::EndpointDispatcher;
use crate::io_stream::StreamDispatcher;

use salvo::oapi::extract::*;
use salvo::prelude::*;
use salvo_craft_macros::craft;
use std::sync::Arc;

pub struct RestController {
    endpoint_dispatcher: Option<Arc<EndpointDispatcher>>,
    stream_dispatcher: Option<Arc<StreamDispatcher>>,
}

#[craft]
impl RestController {
    /// Create controller.
    pub fn new(
        endpoint_dispatcher: Arc<EndpointDispatcher>, stream_dispatcher: Arc<StreamDispatcher>,
    ) -> Self {
        RestController {
            endpoint_dispatcher: Some(endpoint_dispatcher),
            stream_dispatcher: Some(stream_dispatcher),
        }
    }

    /// Create no-op controller.
    pub fn new_noop() -> Self {
        RestController { endpoint_dispatcher: None, stream_dispatcher: None }
    }

    /// Construct openapi specification for this controller.
    pub fn openapi(self: &Arc<Self>) -> OpenApi {
        OpenApi::new("rocd REST API", env!("CARGO_PKG_VERSION")).merge_router(&self.router())
    }

    /// Construct http router for this controller.
    pub fn router(self: &Arc<Self>) -> Router {
        Router::new()
            // peers
            .push(
                Router::with_path("peers")
                    // self
                    .push(
                        Router::with_path("self")
                            // endpoints
                            .push(Router::with_path("endpoints").get(self.list_endpoints()))
                            .push(
                                Router::with_path("endpoints/{uid}")
                                    .get(self.read_endpoint())
                                    .put(self.update_endpoint()),
                            ),
                    ),
            )
            // streams
            .push(Router::with_path("streams").get(self.list_streams()))
            .push(
                Router::with_path("streams/{uid}")
                    .get(self.read_stream())
                    .put(self.update_stream()),
            )
    }

    fn endpoint_dispatch(&self) -> &EndpointDispatcher {
        self.endpoint_dispatcher.as_ref().unwrap()
    }

    fn stream_dispatch(&self) -> &StreamDispatcher {
        self.stream_dispatcher.as_ref().unwrap()
    }

    // endpoints

    #[craft(endpoint(operation_id = "list_endpoints", status_codes(200)))]
    async fn list_endpoints(self: &Arc<Self>) -> Json<Vec<EndpointSpec>> {
        Json(self.endpoint_dispatch().get_all().await)
    }

    #[craft(endpoint(operation_id = "read_endpoint"))]
    async fn read_endpoint(self: &Arc<Self>, uid: PathParam<&str>) -> Json<EndpointSpec> {
        Json(self.endpoint_dispatch().get_endpoint(uid.into_inner()).await)
    }

    #[craft(endpoint(operation_id = "update_endpoint"))]
    async fn update_endpoint(self: &Arc<Self>, uid: PathParam<&str>) -> Json<EndpointSpec> {
        Json(self.endpoint_dispatch().get_endpoint(uid.into_inner()).await)
    }

    // streams

    #[craft(endpoint(operation_id = "list_streams", status_codes(200)))]
    async fn list_streams(self: &Arc<Self>) -> Json<Vec<StreamSpec>> {
        Json(self.stream_dispatch().get_all().await)
    }

    #[craft(endpoint(operation_id = "read_stream"))]
    async fn read_stream(self: &Arc<Self>, uid: PathParam<&str>) -> Json<StreamSpec> {
        Json(self.stream_dispatch().get_stream(uid.into_inner()).await)
    }

    #[craft(endpoint(operation_id = "update_stream"))]
    async fn update_stream(self: &Arc<Self>, uid: PathParam<&str>) -> Json<StreamSpec> {
        Json(self.stream_dispatch().get_stream(uid.into_inner()).await)
    }
}
