// Crate-level options.
#![allow(dead_code)] // TODO: remove it later

mod devices;
mod models;
mod rest;

use crate::rest::RestServer;
use std::env::args;
use std::io::{Write, stdout};
use std::process::exit;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let server = RestServer::new();

    if args().last().unwrap_or("".into()) == "--dump-openapi" {
        if let Err(_) = stdout().write_all(server.openapi_json().as_bytes()) {
            exit(1);
        }

        exit(0);
    }

    if let Err(err) = server.serve("127.0.0.1", 3000).await {
        eprintln!("http server failed: {}", err);
        exit(1);
    }
}
