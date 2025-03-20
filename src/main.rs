use salvo::logging::Logger;
use salvo::prelude::*;

mod devices;

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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    tracing::info!("server starting ...");

    let router = Router::new()
        .push(Router::new().get(list_devices)) // default
        .push(Router::with_path("devices").get(list_devices))
        .push(Router::with_path("devices/{uid}").get(get_device))
        .push(Router::with_path("devices/{uid}").patch(update_device));
    let service = Service::new(router).hoop(Logger::new());

    let acceptor = TcpListener::new("127.0.0.1:3000").bind().await;

    Server::new(acceptor).serve(service).await;
}
