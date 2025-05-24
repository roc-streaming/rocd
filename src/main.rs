// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
mod dto;
mod io_endpoint;
mod io_stream;
mod rest_api;
mod vault;

use crate::io_endpoint::EndpointDispatcher;
use crate::io_stream::StreamDispatcher;
use crate::rest_api::RestServer;
use clap::{Parser, ValueEnum};
use std::io::{Write, stdout};
use std::process::exit;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(about = "rocd server")]
struct CliArgs {
    /// host:port to run http server at
    #[arg(short, long, value_name = "HOST:PORT", default_value = "127.0.0.1:4040")]
    addr: String,

    /// print openapi spec and exit
    #[arg(long, value_enum, value_name = "FORMAT")]
    dump_openapi: Option<OpenapiFormat>,
}

#[derive(ValueEnum, Clone, Debug)]
enum OpenapiFormat {
    Json,
    Yaml,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let args = CliArgs::parse();

    let endpoint_dispatcher = Arc::new(EndpointDispatcher::new());
    let stream_dispatcher = Arc::new(StreamDispatcher::new());
    let server = RestServer::new(endpoint_dispatcher, stream_dispatcher);

    match args.dump_openapi {
        Some(OpenapiFormat::Json) => {
            if stdout().write_all(server.openapi_json().as_bytes()).is_err() {
                exit(1);
            }
            exit(0);
        },
        Some(OpenapiFormat::Yaml) => {
            if stdout().write_all(server.openapi_yaml().as_bytes()).is_err() {
                exit(1);
            }
            exit(0);
        },
        None => (),
    };

    tracing::info!("starting server with options {args:?}");

    let (host, port) = match rest_api::parse_addr(&args.addr) {
        Ok((host, port)) => (host, port),
        Err(err) => {
            tracing::error!("invalid --addr: {err}");
            exit(1);
        },
    };

    if let Err(err) = server.serve(host, port).await {
        tracing::error!("http server failed: {err}");
        exit(1);
    }
}
