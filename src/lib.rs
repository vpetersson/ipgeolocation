pub mod cache;
pub mod country_data;
pub mod geoip;
pub mod handlers;
pub mod http3;
pub mod languages;
pub mod mcp;
pub mod models;
pub mod proto;
pub mod timezone;
pub mod tz_utils;

use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};

/// CORS policy for the public API: any origin may read these read-only
/// geolocation responses (GET/HEAD only), so browser apps — for example static
/// sites hosted elsewhere — can call the API directly instead of proxying it.
/// Shared by the server (`main.rs`) and the integration tests.
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::HEAD])
}
