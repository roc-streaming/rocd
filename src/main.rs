// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use rocd::io_endpoint::EndpointDispatcher;
use rocd::io_stream::StreamDispatcher;
use rocd::rest_api::{self, RestServer};

use clap::Parser;
use std::process;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(about = "rocd server")]
struct CliArgs {
    /// host:port to run http server at
    #[arg(short, long, value_name = "HOST:PORT", default_value = "127.0.0.1:4040")]
    addr: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let args = CliArgs::parse();

    tracing::info!("starting server with options {args:?}");

    let (host, port) = match rest_api::parse_addr(&args.addr) {
        Ok((host, port)) => (host, port),
        Err(err) => {
            tracing::error!("invalid --addr: {err}");
            process::exit(1);
        },
    };

    let endpoint_dispatcher = Arc::new(EndpointDispatcher::new());
    let stream_dispatcher = Arc::new(StreamDispatcher::new());

    let server = Arc::new(RestServer::new(endpoint_dispatcher, stream_dispatcher));

    if let Err(err) = server.start(host, port).await {
        tracing::error!("http server failed to start: {err}");
        process::exit(1);
    }

    if let Err(err) = server.wait().await {
        tracing::error!("http server failed: {err}");
        process::exit(1);
    }
}
