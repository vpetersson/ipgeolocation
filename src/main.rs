use axum::{routing::get, Router};
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
    health_handler, ipgeo_full_handler, ipgeo_handler, timezone_full_handler, timezone_handler,
    AppState,
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

    // Create shared state
    let state = AppState {
        geoip: Arc::new(geoip_reader),
        cache: Arc::new(cache),
    };

    // Build router with static file serving for flags
    let app = Router::new()
        // Simple format endpoints (backward compatible)
        .route("/ipgeo", get(ipgeo_handler))
        .route("/timezone", get(timezone_handler))
        // Full format endpoints (extended format)
        .route("/v1/ipgeo", get(ipgeo_full_handler))
        .route("/v1/timezone", get(timezone_full_handler))
        // Health check
        .route("/health", get(health_handler))
        // Static files (flags, etc.)
        .nest_service("/static", ServeDir::new(&static_dir))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    tracing::info!("Starting server on {}", bind_address);
    tracing::info!("Endpoints:");
    tracing::info!("  GET /ipgeo           - Simple IP geolocation");
    tracing::info!("  GET /timezone        - Simple timezone lookup");
    tracing::info!("  GET /v1/ipgeo        - Full IP geolocation (extended format)");
    tracing::info!("  GET /v1/timezone     - Full timezone details");
    tracing::info!("  GET /static/flags/*  - Country flag SVGs");
    tracing::info!("  GET /health          - Health check");

    // Start server
    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
