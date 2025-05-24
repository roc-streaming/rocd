// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use rocd::io_endpoint::EndpointDispatcher;
use rocd::io_stream::StreamDispatcher;
use rocd::rest_api::RestServer;

use clap::{Parser, ValueEnum};
use std::io::{self, Write};
use std::process;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(about = "rocd code generator")]
struct CliArgs {
    /// Dump openapi spec.
    #[arg(long, value_enum, value_name = "FORMAT")]
    openapi: Option<OpenapiFormat>,

    /// Generate openapi client using progenitor.
    #[arg(long, default_value_t = false)]
    client: bool,
}

#[derive(ValueEnum, Clone, Debug)]
enum OpenapiFormat {
    Json,
    Yaml,
}

fn main() {
    let args = CliArgs::parse();

    let server = RestServer::new(
        Arc::new(EndpointDispatcher::new()),
        Arc::new(StreamDispatcher::new()),
    );

    match args.openapi {
        Some(OpenapiFormat::Json) => {
            io::stdout().write_all(server.openapi_json().as_bytes()).unwrap();
            process::exit(0);
        },
        Some(OpenapiFormat::Yaml) => {
            io::stdout().write_all(server.openapi_yaml().as_bytes()).unwrap();
            process::exit(0);
        },
        None => (),
    };

    if args.client {
        // TODO: remove this hack when progenitor will support openapi 3.1.0.
        let hacked_json = server.openapi_json().replace("3.1.0", "3.0.0");

        let spec = serde_json::from_slice(hacked_json.as_bytes()).unwrap();
        let mut generator = progenitor::Generator::default();

        let tokens = generator.generate_tokens(&spec).unwrap();
        let ast = syn::parse2(tokens).unwrap();
        let content = prettyplease::unparse(&ast);

        io::stdout().write_all(content.as_bytes()).unwrap();
        process::exit(0);
    }

    eprintln!("at least one option required");
    process::exit(1);
}
