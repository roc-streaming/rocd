// Crate-level options.
#![allow(dead_code)] // TODO: remove it later

mod devices;
mod errors;
mod models;
mod rest;

use crate::errors::ParseError;
use crate::rest::RestServer;
use clap::{Parser, ValueEnum};
use regex_static::static_regex;
use std::io::{Write, stdout};
use std::process::exit;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(about = "rocd server")]
struct CLI {
    /// host:port to run http server at
    #[arg(short, long, value_name = "HOST:PORT", default_value = "127.0.0.1:4040")]
    addr: String,

    /// dump OpenAPI specification in JSON format to stdout and exit
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

    let opts = CLI::parse();

    let server = RestServer::new();

    match opts.dump_openapi {
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

    tracing::info!("starting server with options {opts:?}");

    let (host, port) = match parse_addr(&opts.addr) {
        Ok((host, port)) => (host, port),
        Err(err) => {
            tracing::error!("invalid --addr: {}", err);
            exit(1);
        },
    };

    if let Err(err) = server.serve(&host, port).await {
        tracing::error!("http server failed: {}", err);
        exit(1);
    }
}

fn parse_addr(hs: &str) -> Result<(&str, u16), ParseError> {
    let re = static_regex!(r"^(.+):(\d+)$");

    let (host, port_str) = match re.captures(hs) {
        Some(cap) => (cap.get(1).unwrap().as_str(), cap.get(2).unwrap().as_str()),
        None => {
            return Err(ParseError::new("bad format: expected 'hostname:port' or 'ipaddr:port'"));
        },
    };

    let port = match u16::from_str(&port_str) {
        Ok(n) => n,
        Err(err) => return Err(ParseError::new(format!("bad port: {}", err))),
    };

    Ok((host, port))
}
