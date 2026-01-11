//! MCP (Model Context Protocol) server module for IP Geolocation API
//!
//! This module provides MCP server functionality, exposing the IP geolocation
//! and timezone lookup capabilities as MCP tools and resources.
//!
//! ## Tools
//!
//! - `geoip_lookup` - Look up geographic location for an IP address
//! - `geoip_bulk_lookup` - Look up multiple IP addresses (max 100)
//! - `geoip_lookup_self` - Look up the caller's IP (HTTP transport only)
//! - `timezone_lookup` - Look up timezone for coordinates
//!
//! ## Resources
//!
//! - `geoip://schema` - JSON Schema for response types
//! - `geoip://data-source` - Information about data sources
//! - `geoip://limits` - API limits and constraints
//! - `geoip://privacy` - Privacy and data handling information
//!
//! ## Transports
//!
//! ### STDIO (for local clients like Claude Desktop)
//!
//! ```bash
//! mcp_server --transport stdio
//! ```
//!
//! ### HTTP (integrated into main server)
//!
//! MCP is available on the same port as the REST API via:
//! - `POST /mcp` - JSON-RPC endpoint
//! - `GET /mcp/sse` - Server-Sent Events for notifications
//! - `GET /mcp/info` - Server capabilities and discovery

pub mod axum_handlers;
pub mod resources;
pub mod schemas;
pub mod tools;

// Axum handler exports (for HTTP integration)
pub use axum_handlers::{
    mcp_batch_handler, mcp_info_handler, mcp_jsonrpc_handler, mcp_sse_handler, McpState,
};

// Resource exports
pub use resources::{list_resource_infos, read_resource, GeoIpResourceHandler};

// Tool exports
pub use tools::{
    handle_geoip_bulk_lookup, handle_geoip_lookup, handle_geoip_lookup_self,
    handle_timezone_lookup, GeoIpBulkLookupHandler, GeoIpLookupHandler, GeoIpLookupSelfHandler,
    McpErrorCode, McpToolContext, TimezoneLookupHandler, BULK_LOOKUP_MAX_IPS,
};
