use crate::devices;
use salvo::logging::Logger;
use salvo::prelude::*;
use std::io::Result;
use std::sync::Arc;

#[endpoint(status_codes(200))]
async fn list_devices(_req: &mut Request, _resp: &mut Response) -> Json<Vec<devices::Device>> {
    //tracing::debug!(req = ?req, "list_devices()"); // TODO: how to print debug level???
    return Json(devices::get_all().await);
}

// TODO
#[endpoint]
async fn get_device(resp: &mut Response) {
    resp.render("device info");
}

// TODO
#[endpoint]
async fn update_device(resp: &mut Response) {
    resp.render("updated device info");
}

pub struct RestServer {
    router: Arc<Router>,
}

impl RestServer {
    pub fn new() -> Self {
        let router = Arc::new(
            Router::new()
                .push(Router::new().get(list_devices)) // default
                .push(Router::with_path("devices").get(list_devices))
                .push(Router::with_path("devices/{uid}").get(get_device))
                .push(Router::with_path("devices/{uid}").patch(update_device)),
        );

        RestServer { router }
    }

    pub async fn serve(&self, host: &str, port: u16) -> Result<()> {
        tracing::info!("starting server at {}:{} ...", host, port);

        let service = Service::new(self.router.clone()).hoop(Logger::new());
        let acceptor = TcpListener::new(format!("{}:{}", host, port)).bind().await;

        Server::new(acceptor).try_serve(service).await
    }
}
