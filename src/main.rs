use axum::{
    extract::ConnectInfo,
    routing::{get, post},
    Router,
};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ipgeolocation::cache::{CacheConfig, GeoCache};
use ipgeolocation::geoip::GeoIpReader;
use ipgeolocation::handlers::{
    extract_client_ip, health_handler, ipgeo_full_handler, ipgeo_handler, llms_txt_handler,
    openapi_handler, robots_txt_handler, root_handler, sitemap_handler, timezone_full_handler,
    timezone_handler, wellknown_ai_plugin_handler, wellknown_openapi_handler, AppState,
};
use ipgeolocation::http3::{run_http3_server, Http3Config};
use ipgeolocation::mcp::{
    mcp_batch_handler, mcp_info_handler, mcp_jsonrpc_handler, mcp_sse_handler, McpState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ipgeolocation=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration from environment
    let bind_address: SocketAddr = env::var("BIND_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse()
        .expect("Invalid BIND_ADDRESS");

    let geoip_db_path =
        env::var("GEOIP_DB_PATH").unwrap_or_else(|_| "data/GeoLite2-City.mmdb".to_string());

    let static_dir = env::var("STATIC_DIR").unwrap_or_else(|_| "static".to_string());

    let cache_size: u64 = env::var("CACHE_SIZE")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .expect("Invalid CACHE_SIZE");

    let cache_ttl_secs: u64 = env::var("CACHE_TTL_SECS")
        .unwrap_or_else(|_| "3600".to_string())
        .parse()
        .expect("Invalid CACHE_TTL_SECS");

    // Base URL for API documentation (OpenAPI, sitemap, etc.)
    let base_url =
        env::var("BASE_URL").unwrap_or_else(|_| "https://geoip.vpetersson.com".to_string());

    // HTTP/3 configuration (optional)
    let http3_enabled = env::var("HTTP3_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);
    let http3_bind_address: SocketAddr = env::var("HTTP3_BIND_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:443".to_string())
        .parse()
        .unwrap_or_else(|_| "0.0.0.0:443".parse().unwrap());
    let tls_cert_path = env::var("TLS_CERT_PATH").unwrap_or_else(|_| "cert.pem".to_string());
    let tls_key_path = env::var("TLS_KEY_PATH").unwrap_or_else(|_| "key.pem".to_string());

    // Initialize GeoIP reader
    tracing::info!("Loading GeoIP database from: {}", geoip_db_path);
    let geoip_reader = GeoIpReader::open(&geoip_db_path).map_err(|e| {
        format!(
            "Failed to open GeoIP database at '{}': {}",
            geoip_db_path, e
        )
    })?;

    // Initialize cache
    let cache_config = CacheConfig {
        max_capacity: cache_size,
        ttl: Duration::from_secs(cache_ttl_secs),
    };
    let cache = GeoCache::new(cache_config);

    // Create shared GeoIP reader
    let geoip: Arc<ipgeolocation::geoip::GeoIpReader> = Arc::new(geoip_reader);

    // Create shared state for REST API
    let state = AppState {
        geoip: geoip.clone(),
        cache: Arc::new(cache),
        base_url: base_url.clone(),
    };

    // Create shared state for MCP
    let mcp_state = McpState::new(geoip);

    // Build MCP router (nested under /mcp)
    let mcp_router = Router::new()
        .route("/", post(mcp_jsonrpc_handler))
        .route("/batch", post(mcp_batch_handler))
        .route("/sse", get(mcp_sse_handler))
        .route("/info", get(mcp_info_handler))
        .with_state(mcp_state);

    // Build main router with access logging
    let app = Router::new()
        // Root endpoint - returns geolocation for client's IP
        .route("/", get(root_handler))
        // Simple format endpoints (backward compatible)
        .route("/ipgeo", get(ipgeo_handler))
        .route("/timezone", get(timezone_handler))
        // Full format endpoints (extended format)
        .route("/v1/ipgeo", get(ipgeo_full_handler))
        .route("/v1/timezone", get(timezone_full_handler))
        // Health check
        .route("/health", get(health_handler))
        // API documentation for LLMs and agents
        .route("/openapi.yaml", get(openapi_handler))
        .route("/llms.txt", get(llms_txt_handler))
        .route("/sitemap.xml", get(sitemap_handler))
        .route("/robots.txt", get(robots_txt_handler))
        // .well-known discovery endpoints
        .route("/.well-known/openapi.yaml", get(wellknown_openapi_handler))
        .route(
            "/.well-known/ai-plugin.json",
            get(wellknown_ai_plugin_handler),
        )
        // MCP endpoints (Model Context Protocol)
        .nest("/mcp", mcp_router)
        // Static files (flags, etc.)
        .nest_service("/static", ServeDir::new(&static_dir))
        .with_state(state)
        // Access logging layer with proxy-aware client IP extraction
        // Silences logging for favicon.ico (expected 404 from browsers)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    // Skip creating detailed spans for favicon.ico
                    if request.uri().path() == "/favicon.ico" {
                        return tracing::debug_span!("favicon");
                    }

                    let connect_info = request
                        .extensions()
                        .get::<ConnectInfo<SocketAddr>>()
                        .map(|ci| ci.0);

                    let client_ip = extract_client_ip(request.headers(), connect_info);

                    tracing::info_span!(
                        "request",
                        method = %request.method(),
                        uri = %request.uri(),
                        client_ip = %client_ip,
                    )
                })
                .on_request(|request: &axum::http::Request<_>, _span: &tracing::Span| {
                    // Skip logging for favicon.ico
                    if request.uri().path() == "/favicon.ico" {
                        return;
                    }
                    tracing::info!(
                        method = %request.method(),
                        uri = %request.uri(),
                        "started processing request"
                    );
                })
                .on_response(
                    |response: &axum::http::Response<_>,
                     latency: std::time::Duration,
                     span: &tracing::Span| {
                        // Skip logging for favicon.ico (check span name)
                        if span.metadata().map(|m| m.name()) == Some("favicon") {
                            return;
                        }
                        tracing::info!(
                            status = %response.status().as_u16(),
                            latency_ms = %latency.as_millis(),
                            "finished processing request"
                        );
                    },
                ),
        );

    tracing::info!("Starting server on {}", bind_address);
    tracing::info!("Endpoints:");
    tracing::info!("  GET /                - Geolocation for client's IP");
    tracing::info!("  GET /ipgeo           - Simple IP geolocation");
    tracing::info!("  GET /timezone        - Simple timezone lookup");
    tracing::info!("  GET /v1/ipgeo        - Full IP geolocation (extended format)");
    tracing::info!("  GET /v1/timezone     - Full timezone details");
    tracing::info!("  GET /static/flags/*  - Country flag SVGs");
    tracing::info!("  GET /health          - Health check");
    tracing::info!("  GET /openapi.yaml    - OpenAPI specification");
    tracing::info!("  GET /llms.txt        - LLM-friendly documentation");
    tracing::info!("  GET /sitemap.xml     - Sitemap for discovery");
    tracing::info!("  GET /robots.txt      - Robots.txt for crawlers");
    tracing::info!("  GET /.well-known/openapi.yaml   - OpenAPI (well-known)");
    tracing::info!("  GET /.well-known/ai-plugin.json - AI plugin manifest");
    tracing::info!("");
    tracing::info!("MCP (Model Context Protocol):");
    tracing::info!("  POST /mcp            - JSON-RPC endpoint");
    tracing::info!("  POST /mcp/batch      - Batch JSON-RPC endpoint");
    tracing::info!("  GET  /mcp/sse        - Server-Sent Events");
    tracing::info!("  GET  /mcp/info       - Server info and capabilities");
    tracing::info!("");
    tracing::info!(
        "Content negotiation: Use Accept: application/x-protobuf for protobuf responses"
    );

    // Start HTTP/3 server if enabled
    if http3_enabled {
        let http3_config = Http3Config {
            bind_address: http3_bind_address,
            cert_path: tls_cert_path,
            key_path: tls_key_path,
        };
        tracing::info!(
            "HTTP/3 enabled on {} (requires TLS certificates)",
            http3_bind_address
        );
        tokio::spawn(async move {
            if let Err(e) = run_http3_server(http3_config).await {
                tracing::error!("HTTP/3 server error: {}", e);
            }
        });
    }

    // Start HTTP/1.1 + HTTP/2 server with client address extraction
    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
