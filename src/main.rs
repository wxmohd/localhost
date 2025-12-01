mod cgi;
mod config;
mod error;
mod http;
mod router;
mod server;
mod session;

use config::Config;
use server::Server;
use std::env;
use std::process;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    let config_path = if args.len() > 1 {
        &args[1]
    } else {
        "config/default.conf"
    };

    println!("localhost HTTP Server v0.1.0");
    println!("Loading configuration from: {}", config_path);

    // Load configuration
    let config = match Config::load(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            process::exit(1);
        }
    };

    // Print server info
    println!("\nServer configuration:");
    for server in &config.servers {
        println!("  - Host: {}", server.host);
        println!("    Ports: {:?}", server.ports);
        println!("    Root: {}", server.root);
        println!("    Routes: {}", server.routes.len());
    }

    println!("\nStarting server...");

    // Run the server
    if let Err(e) = Server::run(config) {
        eprintln!("Server error: {}", e);
        process::exit(1);
    }
}
