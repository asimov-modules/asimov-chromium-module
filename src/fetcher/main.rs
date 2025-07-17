// This is free and unencumbered software released into the public domain.

#[cfg(not(feature = "std"))]
compile_error!("asimov-chromium-fetcher requires the 'std' feature");

use asimov_chromium_module::browsers;
use asimov_module::SysexitsError::{self, *};
use clap::Parser;
use clientele::StandardOptions;
use serde_json::Value;
use std::{error::Error, io::Read};

/// asimov-chromium-fetcher
#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
struct Options {
    #[clap(flatten)]
    flags: StandardOptions,

    /// The browser bookmarks URL to fetch (e.g., chrome://bookmarks, brave://bookmarks/Profile 5)
    url: String,
}

fn main() -> Result<SysexitsError, Box<dyn Error>> {
    // Load environment variables from `.env`:
    asimov_module::dotenv().ok();

    // Expand wildcards and @argfiles:
    let args = asimov_module::args_os()?;

    // Parse command-line options:
    let options = Options::parse_from(args);

    // Handle the `--version` flag:
    if options.flags.version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(EX_OK);
    }

    // Handle the `--license` flag:
    if options.flags.license {
        print!("{}", include_str!("../../UNLICENSE"));
        return Ok(EX_OK);
    }

    // Configure logging & tracing:
    #[cfg(feature = "tracing")]
    asimov_module::init_tracing_subscriber(&options.flags).expect("failed to initialize logging");

    // Parse the input JSON:
    let input_url = &options.url;
    let outputs: Vec<Value> = if input_url.starts_with("-") {
        let mut input_buffer = String::new();
        std::io::stdin().lock().read_to_string(&mut input_buffer)?;
        vec![serde_json::from_str(&input_buffer)?]
    } else {
        browsers::fetch_bookmarks(input_url)?
    };

    // Transform JSON to JSON-LD:
    let transform = asimov_chromium_module::BookmarksTransform::new()?;
    for input in outputs {
        let output = transform.execute(input)?;
        // Serialize the output JSON-LD:
        if cfg!(feature = "pretty") {
            colored_json::write_colored_json(&output, &mut std::io::stdout())?;
            println!();
        } else {
            println!("{}", serde_json::to_string(&output).unwrap());
        }
    }

    Ok(EX_OK)
}
