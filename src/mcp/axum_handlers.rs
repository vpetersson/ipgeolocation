//! Axum handlers for MCP (Model Context Protocol) integration
//!
//! This module provides HTTP handlers that implement the MCP protocol
//! using JSON-RPC over HTTP, allowing MCP clients to connect via the
//! same port as the REST API.

use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::{IntoResponse, Response, Sse},
    Json,
};
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::broadcast;

use crate::geoip::SharedGeoIpReader;
use crate::handlers::extract_client_ip;

use super::resources::{list_resource_infos, read_resource};
use super::schemas;
use super::tools::{
    handle_geoip_bulk_lookup, handle_geoip_lookup, handle_geoip_lookup_self, handle_timezone_lookup,
};

/// MCP server state for Axum handlers
#[derive(Clone)]
pub struct McpState {
    pub geoip: SharedGeoIpReader,
    /// Broadcast channel for SSE notifications (optional)
    pub notification_tx: broadcast::Sender<McpNotification>,
}

impl McpState {
    pub fn new(geoip: SharedGeoIpReader) -> Self {
        let (notification_tx, _) = broadcast::channel(100);
        Self {
            geoip,
            notification_tx,
        }
    }
}

/// JSON-RPC request structure
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

/// JSON-RPC response structure
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error structure
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP notification for SSE
#[derive(Debug, Clone, Serialize)]
pub struct McpNotification {
    pub method: String,
    pub params: Value,
}

// JSON-RPC error codes
const INVALID_REQUEST: i32 = -32600;
const METHOD_NOT_FOUND: i32 = -32601;
const INVALID_PARAMS: i32 = -32602;

/// MCP server information
fn server_info() -> Value {
    json!({
        "name": "ip-geolocation-mcp",
        "version": env!("CARGO_PKG_VERSION"),
        "protocolVersion": "2024-11-05"
    })
}

/// MCP server capabilities
fn server_capabilities() -> Value {
    json!({
        "tools": {
            "listChanged": false
        },
        "resources": {
            "subscribe": false,
            "listChanged": false
        },
        "prompts": null,
        "logging": null
    })
}

/// List of available tools
fn list_tools() -> Value {
    json!({
        "tools": [
            {
                "name": "geoip_lookup",
                "description": "Look up geographic location for an IP address. Returns city, country, coordinates, timezone, currency, and other location metadata.",
                "inputSchema": schemas::geoip_lookup_input_schema()
            },
            {
                "name": "geoip_bulk_lookup",
                "description": "Look up geographic locations for multiple IP addresses in a single request. Maximum 100 IPs per request. Returns results and errors separately.",
                "inputSchema": schemas::geoip_bulk_lookup_input_schema()
            },
            {
                "name": "geoip_lookup_self",
                "description": "Look up geographic location for the caller's IP address. Available via HTTP transport.",
                "inputSchema": schemas::geoip_lookup_self_input_schema()
            },
            {
                "name": "timezone_lookup",
                "description": "Look up IANA timezone for geographic coordinates. Returns timezone name, current offset, DST information, and current local time.",
                "inputSchema": schemas::timezone_lookup_input_schema()
            }
        ]
    })
}

/// List of available resources
fn list_resources() -> Value {
    let resources: Vec<Value> = list_resource_infos()
        .into_iter()
        .map(|r| {
            json!({
                "uri": r.uri,
                "name": r.name,
                "description": r.description,
                "mimeType": r.mime_type
            })
        })
        .collect();

    json!({ "resources": resources })
}

/// Handle MCP JSON-RPC request
pub async fn mcp_jsonrpc_handler(
    State(state): State<McpState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<JsonRpcRequest>,
) -> Response {
    // Validate JSON-RPC version
    if request.jsonrpc != "2.0" {
        return Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: INVALID_REQUEST,
                message: "Invalid JSON-RPC version".to_string(),
                data: None,
            }),
        })
        .into_response();
    }

    // Extract caller IP for geoip_lookup_self
    let caller_ip = extract_client_ip(&headers, Some(addr));

    // Route to appropriate handler
    let response = match request.method.as_str() {
        // MCP lifecycle methods
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": server_info(),
                "capabilities": server_capabilities()
            })),
            error: None,
        },

        "initialized" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(json!({})),
            error: None,
        },

        "ping" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(json!({})),
            error: None,
        },

        // Tool methods
        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(list_tools()),
            error: None,
        },

        "tools/call" => handle_tool_call(&state.geoip, &caller_ip, request.id, request.params),

        // Resource methods
        "resources/list" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(list_resources()),
            error: None,
        },

        "resources/read" => handle_resource_read(request.id, request.params),

        // Unknown method
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: METHOD_NOT_FOUND,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        },
    };

    Json(response).into_response()
}

/// Handle tools/call method
fn handle_tool_call(
    geoip: &SharedGeoIpReader,
    caller_ip: &str,
    id: Value,
    params: Option<Value>,
) -> JsonRpcResponse {
    let params = match params {
        Some(p) => p,
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: INVALID_PARAMS,
                    message: "Missing params".to_string(),
                    data: None,
                }),
            };
        }
    };

    let tool_name = params.get("name").and_then(|n| n.as_str());
    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    let tool_result = match tool_name {
        Some("geoip_lookup") => handle_geoip_lookup(geoip, arguments),
        Some("geoip_bulk_lookup") => handle_geoip_bulk_lookup(geoip, arguments),
        Some("geoip_lookup_self") => handle_geoip_lookup_self(geoip, Some(caller_ip), arguments),
        Some("timezone_lookup") => handle_timezone_lookup(arguments),
        Some(name) => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: METHOD_NOT_FOUND,
                    message: format!("Unknown tool: {}", name),
                    data: None,
                }),
            };
        }
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: INVALID_PARAMS,
                    message: "Missing tool name".to_string(),
                    data: None,
                }),
            };
        }
    };

    // Convert CallToolResult to JSON-RPC response
    let content: Vec<Value> = tool_result
        .content
        .iter()
        .map(|c| match c {
            mcp_protocol_sdk::protocol::types::ContentBlock::Text { text, .. } => {
                json!({ "type": "text", "text": text })
            }
            _ => json!({ "type": "unknown" }),
        })
        .collect();

    if tool_result.is_error.unwrap_or(false) {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "content": content,
                "isError": true
            })),
            error: None,
        }
    } else {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "content": content,
                "isError": false,
                "structuredContent": tool_result.structured_content
            })),
            error: None,
        }
    }
}

/// Handle resources/read method
fn handle_resource_read(id: Value, params: Option<Value>) -> JsonRpcResponse {
    let params = match params {
        Some(p) => p,
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: INVALID_PARAMS,
                    message: "Missing params".to_string(),
                    data: None,
                }),
            };
        }
    };

    let uri = match params.get("uri").and_then(|u| u.as_str()) {
        Some(u) => u,
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: INVALID_PARAMS,
                    message: "Missing uri parameter".to_string(),
                    data: None,
                }),
            };
        }
    };

    match read_resource(uri) {
        Some(contents) => {
            let content_json = match contents {
                mcp_protocol_sdk::protocol::types::ResourceContents::Text {
                    uri,
                    text,
                    mime_type,
                    ..
                } => {
                    json!({
                        "uri": uri,
                        "mimeType": mime_type,
                        "text": text
                    })
                }
                mcp_protocol_sdk::protocol::types::ResourceContents::Blob {
                    uri,
                    blob,
                    mime_type,
                    ..
                } => {
                    json!({
                        "uri": uri,
                        "mimeType": mime_type,
                        "blob": blob
                    })
                }
            };

            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "contents": [content_json]
                })),
                error: None,
            }
        }
        None => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: INVALID_PARAMS,
                message: format!("Resource not found: {}", uri),
                data: None,
            }),
        },
    }
}

/// Handle batch JSON-RPC requests
pub async fn mcp_batch_handler(
    State(state): State<McpState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(requests): Json<Vec<JsonRpcRequest>>,
) -> Response {
    let caller_ip = extract_client_ip(&headers, Some(addr));

    let responses: Vec<JsonRpcResponse> = requests
        .into_iter()
        .map(|request| {
            // Simplified handling - in production, this should be async
            handle_single_request(&state.geoip, &caller_ip, request)
        })
        .collect();

    Json(responses).into_response()
}

fn handle_single_request(
    geoip: &SharedGeoIpReader,
    caller_ip: &str,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    if request.jsonrpc != "2.0" {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: INVALID_REQUEST,
                message: "Invalid JSON-RPC version".to_string(),
                data: None,
            }),
        };
    }

    match request.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": server_info(),
                "capabilities": server_capabilities()
            })),
            error: None,
        },
        "initialized" | "ping" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(json!({})),
            error: None,
        },
        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(list_tools()),
            error: None,
        },
        "tools/call" => handle_tool_call(geoip, caller_ip, request.id, request.params),
        "resources/list" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(list_resources()),
            error: None,
        },
        "resources/read" => handle_resource_read(request.id, request.params),
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: METHOD_NOT_FOUND,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        },
    }
}

/// SSE endpoint for MCP notifications (optional, for real-time updates)
pub async fn mcp_sse_handler(
    State(state): State<McpState>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    let mut rx = state.notification_tx.subscribe();

    let stream = async_stream::stream! {
        // Send initial connection event
        yield Ok(axum::response::sse::Event::default()
            .event("connected")
            .data(serde_json::to_string(&server_info()).unwrap()));

        // Stream notifications
        loop {
            match rx.recv().await {
                Ok(notification) => {
                    let event = axum::response::sse::Event::default()
                        .event(&notification.method)
                        .data(serde_json::to_string(&notification.params).unwrap());
                    yield Ok(event);
                }
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("ping"),
    )
}

/// MCP server info endpoint (for discovery)
pub async fn mcp_info_handler() -> impl IntoResponse {
    Json(json!({
        "name": "ip-geolocation-mcp",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "MCP",
        "protocolVersion": "2024-11-05",
        "transports": ["http", "stdio"],
        "endpoints": {
            "jsonrpc": "/mcp",
            "sse": "/mcp/sse",
            "info": "/mcp/info"
        },
        "capabilities": server_capabilities(),
        "tools": list_tools()["tools"],
        "resources": list_resources()["resources"]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_info() {
        let info = server_info();
        assert_eq!(info["name"], "ip-geolocation-mcp");
        assert!(info["protocolVersion"].as_str().is_some());
    }

    #[test]
    fn test_list_tools() {
        let tools = list_tools();
        let tools_arr = tools["tools"].as_array().unwrap();
        assert_eq!(tools_arr.len(), 4);

        let names: Vec<&str> = tools_arr
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"geoip_lookup"));
        assert!(names.contains(&"geoip_bulk_lookup"));
        assert!(names.contains(&"geoip_lookup_self"));
        assert!(names.contains(&"timezone_lookup"));
    }

    #[test]
    fn test_list_resources() {
        let resources = list_resources();
        let resources_arr = resources["resources"].as_array().unwrap();
        assert_eq!(resources_arr.len(), 4);
    }

    #[test]
    fn test_server_capabilities() {
        let caps = server_capabilities();
        assert!(caps["tools"].is_object());
        assert!(caps["resources"].is_object());
    }
}
