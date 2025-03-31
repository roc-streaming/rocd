mod devices;
mod rest;

use crate::rest::RestServer;
use std::process::exit;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let server = RestServer::new();

    if let Err(err) = server.serve("127.0.0.1", 3000).await {
        eprintln!("http server failed: {}", err);
        exit(1);
    }
}
