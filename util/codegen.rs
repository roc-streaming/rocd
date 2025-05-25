// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use rocd::rest_api::RestController;

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

fn generate_json_spec() -> String {
    let controller = Arc::new(RestController::new_noop());
    let openapi = controller.openapi();

    openapi.to_pretty_json().unwrap() + "\n"
}

fn generate_yaml_spec() -> String {
    let controller = Arc::new(RestController::new_noop());
    let openapi = controller.openapi();

    openapi.to_yaml().unwrap()
}

fn generate_rust_client() -> String {
    let mut json_spec = generate_json_spec();
    // TODO: remove this hack when progenitor will support openapi 3.1.0.
    json_spec = json_spec.replace("3.1.0", "3.0.0");

    let spec = serde_json::from_slice(json_spec.as_bytes()).unwrap();

    let mut generator = progenitor::Generator::new(
        progenitor::GenerationSettings::default().with_derive("PartialEq"),
    );

    let tokens = generator.generate_tokens(&spec).unwrap();
    let ast = syn::parse2(tokens).unwrap();

    prettyplease::unparse(&ast)
}

fn main() {
    let args = CliArgs::parse();

    match args.openapi {
        Some(OpenapiFormat::Json) => {
            io::stdout().write_all(generate_json_spec().as_bytes()).unwrap();
            process::exit(0);
        },
        Some(OpenapiFormat::Yaml) => {
            io::stdout().write_all(generate_yaml_spec().as_bytes()).unwrap();
            process::exit(0);
        },
        None => (),
    };

    if args.client {
        io::stdout().write_all(generate_rust_client().as_bytes()).unwrap();
        process::exit(0);
    }

    eprintln!("at least one option required");
    process::exit(1);
}
