//! Headless demo for the Nomad Web Engine.
//!
//! A simple CLI application that loads a URL and prints the extracted text content.

use nomad_core::Engine;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <url> [--json]", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!("  {} https://example.com", args[0]);
        eprintln!("  {} https://example.com --json  (output display list JSON)", args[0]);
        process::exit(1);
    }

    let url = &args[1];
    let show_json = args.len() > 2 && args[2] == "--json";

    // Create the engine
    let mut engine = match Engine::new() {
        Ok(engine) => engine,
        Err(e) => {
            eprintln!("Failed to initialize engine: {}", e);
            process::exit(1);
        }
    };

    println!("Loading: {}", url);
    
    if show_json {
        // Set viewport width
        engine.set_viewport_width(800.0);
        
        // Load URL
        match engine.load_url(url) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Error loading URL: {}", e);
                process::exit(1);
            }
        }
        
        // Get display list JSON
        match engine.get_display_list_bytes() {
            Ok(bytes) => {
                match String::from_utf8(bytes) {
                    Ok(json) => {
                        println!("\nDisplay List JSON:");
                        println!("{}", json);
                    },
                    Err(e) => {
                        eprintln!("Error converting bytes to string: {}", e);
                        process::exit(1);
                    }
                }
            },
            Err(e) => {
                eprintln!("Error getting display list: {}", e);
                process::exit(1);
            }
        }
    } else {
        // Original behavior - print text
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
}
