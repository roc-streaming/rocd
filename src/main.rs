// Crate-level options.
#![allow(dead_code)] // TODO: remove it later

mod devices;
mod models;
mod rest;

use crate::rest::RestServer;
use clap::Parser;
use std::io::{Write, stdout};
use std::process::exit;

#[derive(Parser, Debug)]
#[command(about = "rocd server")]
struct CLI {
    /// server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// server port
    #[arg(long, default_value_t = 3000)]
    port: u16,

    /// dump OpenAPI specification in JSON format to stdout and exit
    #[arg(long, default_value_t = false)]
    dump_openapi: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let opts = CLI::parse();

    let server = RestServer::new();
    if opts.dump_openapi {
        if let Err(_) = stdout().write_all(server.openapi_json().as_bytes()) {
            exit(1);
        }
        exit(0);
    }

    tracing::info!("starting server with options {opts:?}");

    if let Err(err) = server.serve(&opts.host, opts.port).await {
        tracing::error!("http server failed: {}", err);
        exit(1);
    }
}
