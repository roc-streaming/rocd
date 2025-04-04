// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::devices;
use crate::models::Device;
use salvo::logging::Logger;
use salvo::oapi::extract::*;
use salvo::prelude::*;
use salvo_craft_macros::craft;
use std::io::Result;
use std::sync::Arc;

pub struct RestServer {
    controller: Arc<Controller>,
    router: Arc<Router>,
    openapi: OpenApi,
}

impl RestServer {
    pub fn new() -> Self {
        // controller methods implement request handlers (endpoints)
        let controller = Arc::new(Controller::new());

        // router maps paths to controller methods
        let router = Router::new()
            // Device API
            .push(Router::with_path("devices").get(controller.list_devices()))
            .push(
                Router::with_path("devices/{uid}")
                    .get(controller.read_device())
                    .put(controller.update_device()),
            );

        // add auto-generate openapi endpoints
        let version = env!("CARGO_PKG_VERSION"); // compile-time env read
        let openapi = OpenApi::new("rocd REST API", version).merge_router(&router);
        let router = router
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

// craft requires pub
pub struct Controller {}

#[craft]
impl Controller {
    fn new() -> Self {
        Controller {}
    }

    #[craft(endpoint(operation_id = "list_devices", status_codes(200)))]
    async fn list_devices(self: &Arc<Self>) -> Json<Vec<Device>> {
        Json(devices::get_all().await)
    }

    #[craft(endpoint(operation_id = "read_device"))]
    async fn read_device(self: &Arc<Self>, uid: PathParam<&str>) -> Json<Device> {
        Json(devices::get_device(uid.into_inner()).await)
    }

    #[craft(endpoint(operation_id = "update_device"))]
    async fn update_device(self: &Arc<Self>, uid: PathParam<&str>) -> Json<Device> {
        Json(devices::get_device(uid.into_inner()).await)
    }
}
