// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use rocd::rest_api::RestController;

use clap::{ArgGroup, Parser, ValueEnum};
use std::fs;
use std::path::Path;
use std::process;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(about = "rocd code generator")]
#[clap(group(
    ArgGroup::new("generator")
        .required(true)
        .args(&["openapi", "progenitor"]),
))]
struct CliArgs {
    /// Dump openapi spec.
    #[arg(long, value_enum, value_name = "FORMAT")]
    openapi: Option<OpenapiFormat>,

    /// Generate openapi client using progenitor.
    #[arg(long, default_value_t = false)]
    progenitor: bool,

    /// Output file name.
    #[arg(short, long)]
    output: String,
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

fn write_file(path: &str, content: &str) {
    let p = Path::new(path);
    if p.exists() && fs::read_to_string(p).unwrap_or_default() == content {
        // Don't write file if it wasn't actually changed, to avoid
        // updating modification time and trigerring rebuild.
        return;
    }

    fs::write(p, content).unwrap();
}

fn main() {
    let args = CliArgs::parse();

    match args.openapi {
        Some(OpenapiFormat::Json) => {
            write_file(&args.output, &generate_json_spec());
            process::exit(0);
        },
        Some(OpenapiFormat::Yaml) => {
            write_file(&args.output, &generate_yaml_spec());
            process::exit(0);
        },
        None => (),
    };

    if args.progenitor {
        write_file(&args.output, &generate_rust_client());
        process::exit(0);
    }

    process::exit(1);
}
