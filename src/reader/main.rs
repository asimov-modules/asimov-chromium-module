// This is free and unencumbered software released into the public domain.

#[cfg(not(feature = "std"))]
compile_error!("asimov-chromium-reader requires the 'std' feature");

use asimov_module::SysexitsError::{self, *};
use clap::Parser;
use clientele::StandardOptions;
use std::{error::Error, io::Read};

/// asimov-chromium-reader
#[derive(Debug, Parser)]
#[command(arg_required_else_help = false)]
struct Options {
    #[clap(flatten)]
    flags: StandardOptions,
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
    let mut buffer = String::new();
    std::io::stdin().lock().read_to_string(&mut buffer)?;
    let mut input: serde_json::Value = serde_json::from_str(&buffer)?;

    // Convert Arc sidebar format to Chromium format
    if let Some(sidebar) = input.get("sidebar") {
        if let Some(_containers) = sidebar.get("containers") {
            input = asimov_chromium_module::specialized::arc::convert_arc_bookmarks_to_chromium(
                input,
                Some("Default"),
            )?;
        }
    }

    // Transform JSON to JSON-LD:
    let transform = asimov_chromium_module::BookmarksTransform::new()?;
    let output = transform.execute(input)?;

    // Serialize the output JSON-LD:
    if cfg!(feature = "pretty") {
        colored_json::write_colored_json(&output, &mut std::io::stdout())?;
        println!();
    } else {
        println!("{}", serde_json::to_string(&output).unwrap());
    }

    Ok(EX_OK)
}
