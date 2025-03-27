use salvo::prelude::*;

#[handler]
async fn list_devices(req: &mut Request, resp: &mut Response) {
    tracing::info!(req = ?req, "list_devices()");
    resp.render("dev1 dev2");
}

#[handler]
async fn get_device(resp: &mut Response) {
    resp.render("device info");
}

#[handler]
async fn update_device(resp: &mut Response) {
    resp.render("updated device info");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    println!("server starting ...");

    let router = Router::new()
        .push(Router::new().get(list_devices)) // default
        .push(Router::with_path("devices").get(list_devices))
        .push(Router::with_path("devices/{uid}").get(get_device))
        .push(Router::with_path("devices/{uid}").patch(update_device));
    let acceptor = TcpListener::new("127.0.0.1:3000").bind().await;
    Server::new(acceptor).serve(router).await;
}
