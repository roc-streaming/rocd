// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use rocd::drivers::DriverRegistry;
use rocd::dto::DriverId;
use rocd::io_endpoints::EndpointDispatcher;
use rocd::io_streams::StreamDispatcher;
use rocd::p2p::PeerDispatcher;
use rocd::rest_api::RestServer;

use clap::builder::styling::{AnsiColor, Styles};
use clap::{ArgAction, Parser};
use std::net::SocketAddr;
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use time::macros::format_description;
use tracing_subscriber::fmt::time::LocalTime;

const CLI_STYLES: Styles = Styles::styled()
    .header(AnsiColor::BrightWhite.on_default().bold())
    .usage(AnsiColor::BrightWhite.on_default().bold())
    .literal(AnsiColor::BrightBlue.on_default().bold())
    .placeholder(AnsiColor::Yellow.on_default());

#[derive(Parser, Debug)]
#[command(about = "rocd server", styles = CLI_STYLES)]
struct CliArgs {
    /// Address for HTTP server.
    #[arg(short, long, value_name = "HOST:PORT", default_value = "127.0.0.1:4040")]
    addr: String,

    /// Driver for audio devices.
    #[arg(short, long, value_enum, value_name = "DRIVER")]
    driver: Option<DriverId>,

    /// Increase verbosity (can be specified more than once).
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,
}

/// Report error and exit.
macro_rules! oops {
    ($fmt:expr $(,$args:expr)*) => ({
        tracing::error!($fmt, $($args),*);
        process::exit(1);
    });
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    init_tracing(args.verbose);

    tracing::info!("running with {args:?}");

    let addr = match SocketAddr::from_str(&args.addr) {
        Ok(addr) => addr,
        Err(err) => oops!("invalid --addr: {err}"),
    };

    let driver_registry = DriverRegistry::new();

    let driver = match args.driver {
        Some(driver_id) => driver_registry
            .open_driver(driver_id)
            .await
            .inspect_err(|err| oops!("can't open driver {driver_id}: {err}"))
            .unwrap(),
        None => driver_registry
            .open_default_driver()
            .await
            .inspect_err(|err| oops!("can't open default driver: {err}"))
            .unwrap(),
    };

    let peer_dispatcher = Arc::new(PeerDispatcher::new());
    let endpoint_dispatcher = Arc::new(EndpointDispatcher::new(&driver));
    let stream_dispatcher = Arc::new(StreamDispatcher::new(&driver));

    let server =
        Arc::new(RestServer::new(&peer_dispatcher, &endpoint_dispatcher, &stream_dispatcher));

    if let Err(err) = server.start(addr).await {
        oops!("can't start http server: {err}");
    }

    if let Err(err) = server.wait().await {
        oops!("http server failed: {err}");
    }

    driver.close().await;
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
