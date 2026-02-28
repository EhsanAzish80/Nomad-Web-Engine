//! Headless demo for the Nomad Web Engine.
//!
//! A simple CLI application that loads a URL and prints the extracted text content.

use nomad_core::Engine;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <url>", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!("  {} https://example.com", args[0]);
        process::exit(1);
    }

    let url = &args[1];

    // Create the engine
    let mut engine = match Engine::new() {
        Ok(engine) => engine,
        Err(e) => {
            eprintln!("Failed to initialize engine: {}", e);
            process::exit(1);
        }
    };

    // Load the URL and print the text
    println!("Loading: {}", url);
    println!("---");

    match engine.load_url_and_print(url) {
        Ok(_) => {}
        Err(e) => {
            eprintln!();
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
