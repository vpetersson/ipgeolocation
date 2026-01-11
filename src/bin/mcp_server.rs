//! MCP Server binary for IP Geolocation API
//!
//! This binary provides an MCP (Model Context Protocol) server that exposes
//! the IP geolocation and timezone lookup functionality to MCP clients.
//!
//! ## Usage
//!
//! ```bash
//! # STDIO mode (for local MCP clients like Claude Desktop)
//! mcp_server --transport stdio
//!
//! # SSE mode (for HTTP-based MCP clients)
//! mcp_server --transport sse --bind 0.0.0.0:3001
//! ```
//!
//! ## Environment Variables
//!
//! - `GEOIP_DB_PATH` - Path to MaxMind GeoLite2-City.mmdb (default: data/GeoLite2-City.mmdb)
//! - `RUST_LOG` - Log level (default: info)

use std::env;
use std::process;
use std::sync::Arc;

use mcp_protocol_sdk::server::McpServer;
use mcp_protocol_sdk::transport::StdioServerTransport;

use ipgeolocation::geoip::{GeoIpReader, SharedGeoIpReader};
use ipgeolocation::mcp::{
    schemas, GeoIpBulkLookupHandler, GeoIpLookupHandler, GeoIpLookupSelfHandler,
    GeoIpResourceHandler, TimezoneLookupHandler,
};

/// Print usage information
fn print_usage() {
    eprintln!(
        r#"IP Geolocation MCP Server

USAGE:
    mcp_server --transport <MODE> [OPTIONS]

TRANSPORT MODES:
    stdio           STDIO transport for local MCP clients (e.g., Claude Desktop)
    sse             HTTP/SSE transport for network MCP clients

OPTIONS:
    --bind <ADDR>   Bind address for SSE mode (default: 0.0.0.0:3001)
    --help          Print this help message

ENVIRONMENT VARIABLES:
    GEOIP_DB_PATH   Path to MaxMind GeoLite2-City.mmdb (default: data/GeoLite2-City.mmdb)
    RUST_LOG        Log level (default: info)

EXAMPLES:
    # Start STDIO server for Claude Desktop
    mcp_server --transport stdio

    # Start SSE server on port 3001
    mcp_server --transport sse --bind 0.0.0.0:3001
"#
    );
}

/// Parse command line arguments
fn parse_args() -> (String, String) {
    let args: Vec<String> = env::args().collect();

    let mut transport = String::new();
    let mut bind_addr = "0.0.0.0:3001".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--transport" => {
                if i + 1 < args.len() {
                    transport = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --transport requires a value");
                    process::exit(1);
                }
            }
            "--bind" => {
                if i + 1 < args.len() {
                    bind_addr = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --bind requires a value");
                    process::exit(1);
                }
            }
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_usage();
                process::exit(1);
            }
        }
    }

    if transport.is_empty() {
        eprintln!("Error: --transport is required");
        print_usage();
        process::exit(1);
    }

    if transport != "stdio" && transport != "sse" {
        eprintln!("Error: transport must be 'stdio' or 'sse'");
        process::exit(1);
    }

    (transport, bind_addr)
}

/// Initialize the GeoIP reader
fn init_geoip() -> SharedGeoIpReader {
    let db_path =
        env::var("GEOIP_DB_PATH").unwrap_or_else(|_| "data/GeoLite2-City.mmdb".to_string());

    match GeoIpReader::open(&db_path) {
        Ok(reader) => Arc::new(reader),
        Err(e) => {
            eprintln!("Failed to open GeoIP database at {}: {}", db_path, e);
            eprintln!("Please set GEOIP_DB_PATH to a valid MaxMind GeoLite2-City.mmdb file");
            process::exit(1);
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let (transport, bind_addr) = parse_args();
    let _ = bind_addr; // Reserved for future SSE transport implementation

    // Initialize GeoIP reader
    let geoip = init_geoip();

    // Create the MCP server (must be mutable to call start())
    let mut server = McpServer::new(
        "ip-geolocation-mcp".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
    );

    // Register tools
    if let Err(e) = server
        .add_tool(
            "geoip_lookup".to_string(),
            Some(
                "Look up geographic location for an IP address. Returns city, country, \
                 coordinates, timezone, currency, and other location metadata."
                    .to_string(),
            ),
            schemas::geoip_lookup_input_schema(),
            GeoIpLookupHandler {
                geoip: geoip.clone(),
            },
        )
        .await
    {
        eprintln!("Failed to register geoip_lookup tool: {}", e);
        process::exit(1);
    }

    if let Err(e) = server
        .add_tool(
            "geoip_bulk_lookup".to_string(),
            Some(
                "Look up geographic locations for multiple IP addresses in a single request. \
                 Maximum 100 IPs per request. Returns results and errors separately."
                    .to_string(),
            ),
            schemas::geoip_bulk_lookup_input_schema(),
            GeoIpBulkLookupHandler {
                geoip: geoip.clone(),
            },
        )
        .await
    {
        eprintln!("Failed to register geoip_bulk_lookup tool: {}", e);
        process::exit(1);
    }

    if let Err(e) = server
        .add_tool(
            "geoip_lookup_self".to_string(),
            Some(
                "Look up geographic location for the caller's IP address. \
                 Only available via SSE transport - returns error on STDIO."
                    .to_string(),
            ),
            schemas::geoip_lookup_self_input_schema(),
            GeoIpLookupSelfHandler {
                geoip: geoip.clone(),
                caller_ip: None, // Will be set per-request in SSE transport
            },
        )
        .await
    {
        eprintln!("Failed to register geoip_lookup_self tool: {}", e);
        process::exit(1);
    }

    if let Err(e) = server
        .add_tool(
            "timezone_lookup".to_string(),
            Some(
                "Look up IANA timezone for geographic coordinates. Returns timezone name, \
                 current offset, DST information, and current local time."
                    .to_string(),
            ),
            schemas::timezone_lookup_input_schema(),
            TimezoneLookupHandler,
        )
        .await
    {
        eprintln!("Failed to register timezone_lookup tool: {}", e);
        process::exit(1);
    }

    // Register resources
    if let Err(e) = server
        .add_resource(
            "geoip".to_string(),
            "geoip://".to_string(),
            GeoIpResourceHandler,
        )
        .await
    {
        eprintln!("Failed to register geoip resources: {}", e);
        process::exit(1);
    }

    // Start the server based on transport
    match transport.as_str() {
        "stdio" => {
            tracing::info!("Starting MCP server with STDIO transport");

            let stdio_transport = StdioServerTransport::new();

            // Start the server with the transport - this wires up the request handlers
            if let Err(e) = server.start(stdio_transport).await {
                eprintln!("Failed to start MCP server: {}", e);
                process::exit(1);
            }

            tracing::info!("MCP server is running. Press Ctrl+C to stop.");

            // Wait for shutdown signal
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for ctrl-c");

            tracing::info!("Shutting down MCP server...");
        }
        "sse" => {
            // SSE transport is not yet fully implemented in mcp-protocol-sdk 0.5
            // For now, we'll print a message and exit
            eprintln!("SSE transport is not yet available in this version.");
            eprintln!("Please use the HTTP/MCP endpoint on the main server instead: POST /mcp");
            process::exit(1);
        }
        _ => unreachable!(),
    }
}
