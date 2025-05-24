// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::*;
use crate::io_port::PortDispatcher;
use crate::io_stream::StreamDispatcher;
use std::sync::Arc;

use salvo::oapi::extract::*;
use salvo::prelude::*;
use salvo_craft_macros::craft;

pub struct RestController {
    port_dispatcher: Arc<PortDispatcher>,
    stream_dispatcher: Arc<StreamDispatcher>,
}

#[craft]
impl RestController {
    pub fn new(
        port_dispatcher: Arc<PortDispatcher>, stream_dispatcher: Arc<StreamDispatcher>,
    ) -> Self {
        RestController { port_dispatcher, stream_dispatcher }
    }

    pub fn router(self: &Arc<Self>) -> Router {
        Router::new().push(
            // peers
            Router::with_path("peers/self").push(
                // self
                Router::with_path("self")
                    // ports
                    .push(Router::with_path("ports").get(self.list_ports()))
                    .push(
                        Router::with_path("ports/{uid}")
                            .get(self.read_port())
                            .put(self.update_port()),
                    ),
            ),
        )
    }

    // ports

    #[craft(endpoint(operation_id = "list_ports", status_codes(200)))]
    async fn list_ports(self: &Arc<Self>) -> Json<Vec<PortDescriptor>> {
        Json(self.port_dispatcher.get_all().await)
    }

    #[craft(endpoint(operation_id = "read_port"))]
    async fn read_port(self: &Arc<Self>, uid: PathParam<&str>) -> Json<PortDescriptor> {
        Json(self.port_dispatcher.get_port(uid.into_inner()).await)
    }

    #[craft(endpoint(operation_id = "update_port"))]
    async fn update_port(self: &Arc<Self>, uid: PathParam<&str>) -> Json<PortDescriptor> {
        Json(self.port_dispatcher.get_port(uid.into_inner()).await)
    }

    // streams

    #[craft(endpoint(operation_id = "list_streams", status_codes(200)))]
    async fn list_streams(self: &Arc<Self>) -> Json<Vec<StreamDescriptor>> {
        Json(self.stream_dispatcher.get_all().await)
    }

    #[craft(endpoint(operation_id = "read_stream"))]
    async fn read_stream(self: &Arc<Self>, uid: PathParam<&str>) -> Json<StreamDescriptor> {
        Json(self.stream_dispatcher.get_stream(uid.into_inner()).await)
    }

    #[craft(endpoint(operation_id = "update_stream"))]
    async fn update_stream(self: &Arc<Self>, uid: PathParam<&str>) -> Json<StreamDescriptor> {
        Json(self.stream_dispatcher.get_stream(uid.into_inner()).await)
    }
}
