// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use rocd::io_endpoint::EndpointDispatcher;
use rocd::io_stream::StreamDispatcher;
use rocd::rest_api::RestServer;

use clap::{ArgAction, Parser};
use std::net::SocketAddr;
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use time::macros::format_description;
use tracing_subscriber::fmt::time::LocalTime;

#[derive(Parser, Debug)]
#[command(about = "rocd server")]
struct CliArgs {
    /// host:port to run http server at
    #[arg(short, long, value_name = "HOST:PORT", default_value = "127.0.0.1:4040")]
    addr: String,

    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    init_tracing(args.verbose);

    tracing::info!("starting server with options {args:?}");

    let addr = match SocketAddr::from_str(&args.addr) {
        Ok(addr) => addr,
        Err(err) => {
            tracing::error!("invalid --addr: {err}");
            process::exit(1);
        },
    };

    let endpoint_dispatcher = Arc::new(EndpointDispatcher::new());
    let stream_dispatcher = Arc::new(StreamDispatcher::new());

    let server = Arc::new(RestServer::new(endpoint_dispatcher, stream_dispatcher));

    if let Err(err) = server.start(addr).await {
        tracing::error!("http server failed to start: {err}");
        process::exit(1);
    }

    if let Err(err) = server.wait().await {
        tracing::error!("http server failed: {err}");
        process::exit(1);
    }
}

fn init_tracing(verbosity: u8) {
    let level = match verbosity {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::fmt()
        .with_timer(LocalTime::new(format_description!(
            "[year]-[month]-[day] [hour repr:24]:[minute]:[second].[subsecond digits:3]"
        )))
        .with_max_level(level)
        .init();
}
