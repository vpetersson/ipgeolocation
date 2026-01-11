//! Integration tests for the IP Geolocation API
//!
//! These tests verify the API endpoints work correctly.

use std::sync::Arc;
use std::time::Duration;

use axum::{routing::get, Router};
use tokio::net::TcpListener;

use ipgeolocation::cache::{CacheConfig, GeoCache};
use ipgeolocation::geoip::mock::MockGeoIpReader;
use ipgeolocation::handlers::{
    health_handler, ipgeo_full_handler, ipgeo_handler, llms_txt_handler, openapi_handler,
    root_handler, sitemap_handler, timezone_full_handler, timezone_handler,
    wellknown_ai_plugin_handler, wellknown_openapi_handler, ApiDoc, AppState,
};
use ipgeolocation::models::GeoData;
use ipgeolocation::proto::geolocation;
use prost::Message;
use std::net::SocketAddr;
use utoipa::OpenApi;

/// Test the health endpoint
#[tokio::test]
async fn test_health_endpoint() {
    let app = Router::new().route("/health", get(health_handler));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/health", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "OK");
}

/// Test timezone endpoint with Stockholm coordinates
#[tokio::test]
async fn test_timezone_stockholm() {
    let app = Router::new().route("/timezone", get(timezone_handler));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://{}/timezone?apiKey=test&lat=59.329504&long=18.069532",
            addr
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["timezone"], "Europe/Stockholm");
}

/// Test timezone endpoint with New York coordinates
#[tokio::test]
async fn test_timezone_new_york() {
    let app = Router::new().route("/timezone", get(timezone_handler));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://{}/timezone?apiKey=test&lat=40.7128&long=-74.0060",
            addr
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["timezone"], "America/New_York");
}

/// Test timezone endpoint with Tokyo coordinates
#[tokio::test]
async fn test_timezone_tokyo() {
    let app = Router::new().route("/timezone", get(timezone_handler));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://{}/timezone?apiKey=test&lat=35.6762&long=139.6503",
            addr
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["timezone"], "Asia/Tokyo");
}

/// Test Cache-Control headers are present
#[tokio::test]
async fn test_cache_control_headers() {
    let app = Router::new().route("/timezone", get(timezone_handler));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://{}/timezone?apiKey=test&lat=59.329504&long=18.069532",
            addr
        ))
        .send()
        .await
        .unwrap();

    let cache_control = response.headers().get("cache-control");
    assert!(cache_control.is_some());
    assert_eq!(
        cache_control.unwrap().to_str().unwrap(),
        "public, max-age=1209600"
    );
}

/// Helper to create test app state with mock GeoIP reader
fn create_test_state(mock: MockGeoIpReader) -> AppState {
    let cache = GeoCache::new(CacheConfig::default());
    AppState {
        geoip: Arc::new(mock),
        cache: Arc::new(cache),
        base_url: "https://test.example.com".to_string(),
    }
}

/// Create a minimal test state for handlers that only need base_url
fn create_minimal_test_state() -> AppState {
    let mock = MockGeoIpReader::new();
    let cache = GeoCache::new(CacheConfig::default());
    AppState {
        geoip: Arc::new(mock),
        cache: Arc::new(cache),
        base_url: "https://test.example.com".to_string(),
    }
}

/// Test ipgeo endpoint with valid IP
#[tokio::test]
async fn test_ipgeo_valid_ip() {
    let mock = MockGeoIpReader::new().with_response(
        "8.8.8.8",
        Ok(GeoData {
            latitude: Some(37.751),
            longitude: Some(-97.822),
            city: Some("Mountain View".to_string()),
            country_name: Some("United States".to_string()),
            country_code: Some("US".to_string()),
            state_prov: Some("California".to_string()),
            state_code: Some("CA".to_string()),
            postal_code: Some("94043".to_string()),
            geoname_id: Some(5375480),
        }),
    );

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/ipgeo", get(ipgeo_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/ipgeo?apiKey=test&ip=8.8.8.8", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["city"], "Mountain View");
    assert_eq!(json["country_name"], "United States");
    assert_eq!(json["languages"], "en-US,en");
    assert!(json["latitude"].as_f64().is_some());
}

/// Test ipgeo endpoint with unknown IP
#[tokio::test]
async fn test_ipgeo_unknown_ip() {
    let mock = MockGeoIpReader::new(); // No responses configured, all lookups fail

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/ipgeo", get(ipgeo_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/ipgeo?apiKey=test&ip=0.0.0.0", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["city"], "");
    assert_eq!(json["country_name"], "");
    assert_eq!(json["languages"], "");
}

/// Test ipgeo endpoint caching
#[tokio::test]
async fn test_ipgeo_caching() {
    let mock = MockGeoIpReader::new().with_response(
        "1.1.1.1",
        Ok(GeoData {
            latitude: Some(51.5074),
            longitude: Some(-0.1278),
            city: Some("London".to_string()),
            country_name: Some("United Kingdom".to_string()),
            country_code: Some("GB".to_string()),
            state_prov: Some("England".to_string()),
            state_code: Some("ENG".to_string()),
            postal_code: None,
            geoname_id: Some(2643743),
        }),
    );

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/ipgeo", get(ipgeo_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // First request
    let response1 = client
        .get(format!("http://{}/ipgeo?apiKey=test&ip=1.1.1.1", addr))
        .send()
        .await
        .unwrap();
    assert_eq!(response1.status(), 200);
    let json1: serde_json::Value = response1.json().await.unwrap();
    assert_eq!(json1["city"], "London");

    // Second request (should be cached)
    let response2 = client
        .get(format!("http://{}/ipgeo?apiKey=test&ip=1.1.1.1", addr))
        .send()
        .await
        .unwrap();
    assert_eq!(response2.status(), 200);
    let json2: serde_json::Value = response2.json().await.unwrap();
    assert_eq!(json2["city"], "London");
}

/// Test ipgeo endpoint Cache-Control headers
#[tokio::test]
async fn test_ipgeo_cache_control_headers() {
    let mock = MockGeoIpReader::new().with_response(
        "8.8.4.4",
        Ok(GeoData {
            latitude: Some(37.0),
            longitude: Some(-97.0),
            city: None,
            country_name: Some("United States".to_string()),
            country_code: Some("US".to_string()),
            state_prov: None,
            state_code: None,
            postal_code: None,
            geoname_id: None,
        }),
    );

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/ipgeo", get(ipgeo_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/ipgeo?apiKey=test&ip=8.8.4.4", addr))
        .send()
        .await
        .unwrap();

    let cache_control = response.headers().get("cache-control");
    assert!(cache_control.is_some());
    assert_eq!(
        cache_control.unwrap().to_str().unwrap(),
        "public, max-age=1209600"
    );
}

/// Test ipgeo endpoint with missing coordinates (no timezone can be determined)
#[tokio::test]
async fn test_ipgeo_no_coordinates() {
    let mock = MockGeoIpReader::new().with_response(
        "192.0.2.1",
        Ok(GeoData {
            latitude: None, // No coordinates
            longitude: None,
            city: Some("Unknown".to_string()),
            country_name: Some("Reserved".to_string()),
            country_code: Some("XX".to_string()),
            state_prov: None,
            state_code: None,
            postal_code: None,
            geoname_id: None,
        }),
    );

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/ipgeo", get(ipgeo_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/ipgeo?apiKey=test&ip=192.0.2.1", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    // With no coordinates, timezone should be empty
    assert_eq!(json["time_zone"]["name"], "");
    // But city and country should still be present
    assert_eq!(json["city"], "Unknown");
    assert_eq!(json["country_name"], "Reserved");
}

/// Test ipgeo endpoint with partial coordinates (only latitude)
#[tokio::test]
async fn test_ipgeo_partial_coordinates() {
    let mock = MockGeoIpReader::new().with_response(
        "198.51.100.1",
        Ok(GeoData {
            latitude: Some(40.0), // Only latitude
            longitude: None,      // No longitude
            city: Some("Partial".to_string()),
            country_name: Some("Test".to_string()),
            country_code: Some("TE".to_string()),
            state_prov: None,
            state_code: None,
            postal_code: None,
            geoname_id: None,
        }),
    );

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/ipgeo", get(ipgeo_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/ipgeo?apiKey=test&ip=198.51.100.1", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    // With partial coordinates, timezone should be empty
    assert_eq!(json["time_zone"]["name"], "");
}

// ============================================================================
// V1 API Tests (Extended Format)
// ============================================================================

/// Test v1/ipgeo endpoint returns full format
#[tokio::test]
async fn test_v1_ipgeo_full_format() {
    let mock = MockGeoIpReader::new().with_response(
        "8.8.8.8",
        Ok(GeoData {
            latitude: Some(37.751),
            longitude: Some(-97.822),
            city: Some("Mountain View".to_string()),
            country_name: Some("United States".to_string()),
            country_code: Some("US".to_string()),
            state_prov: Some("California".to_string()),
            state_code: Some("CA".to_string()),
            postal_code: Some("94043".to_string()),
            geoname_id: Some(5375480),
        }),
    );

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/v1/ipgeo", get(ipgeo_full_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/v1/ipgeo?apiKey=test&ip=8.8.8.8", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();

    // Check IP is returned
    assert_eq!(json["ip"], "8.8.8.8");

    // Check location object
    assert_eq!(json["location"]["country_code2"], "US");
    assert_eq!(json["location"]["country_code3"], "USA");
    assert_eq!(json["location"]["country_name"], "United States");
    assert_eq!(json["location"]["city"], "Mountain View");
    assert_eq!(json["location"]["state_prov"], "California");
    assert_eq!(json["location"]["continent_code"], "NA");
    assert_eq!(json["location"]["continent_name"], "North America");
    assert_eq!(json["location"]["is_eu"], false);

    // Check country_metadata object
    assert_eq!(json["country_metadata"]["calling_code"], "+1");
    assert_eq!(json["country_metadata"]["tld"], ".us");
    assert!(json["country_metadata"]["languages"].is_array());

    // Check currency object
    assert_eq!(json["currency"]["code"], "USD");
    assert_eq!(json["currency"]["name"], "US Dollar");
    assert_eq!(json["currency"]["symbol"], "$");

    // Check time_zone object
    assert!(json["time_zone"]["name"].is_string());
    assert!(json["time_zone"]["offset"].is_number());
    assert!(json["time_zone"]["current_time"].is_string());
    assert!(json["time_zone"]["current_time_unix"].is_number());
}

/// Test v1/timezone endpoint returns full details
#[tokio::test]
async fn test_v1_timezone_full_format() {
    let app = Router::new().route("/v1/timezone", get(timezone_full_handler));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://{}/v1/timezone?apiKey=test&lat=59.329504&long=18.069532",
            addr
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();

    // Check timezone details
    assert_eq!(json["timezone"], "Europe/Stockholm");
    assert!(json["offset"].is_number());
    assert!(json["current_time"].is_string());
    assert!(json["current_time_unix"].is_number());
    assert!(json["is_dst"].is_boolean());
    assert!(json["dst_exists"].is_boolean());
}

/// Test ipgeo endpoint with invalid IP address returns 400
#[tokio::test]
async fn test_ipgeo_invalid_ip() {
    let mock = MockGeoIpReader::new();

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/ipgeo", get(ipgeo_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/ipgeo?apiKey=test&ip=not-an-ip", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 400);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["code"], "INVALID_IP");
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("Invalid IP address"));
}

/// Test timezone endpoint with invalid latitude returns 400
#[tokio::test]
async fn test_timezone_invalid_latitude() {
    let app = Router::new().route("/timezone", get(timezone_handler));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://{}/timezone?apiKey=test&lat=91.0&long=18.0",
            addr
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 400);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["code"], "INVALID_LATITUDE");
    assert!(json["error"].as_str().unwrap().contains("Latitude"));
}

/// Test timezone endpoint with invalid longitude returns 400
#[tokio::test]
async fn test_timezone_invalid_longitude() {
    let app = Router::new().route("/timezone", get(timezone_handler));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://{}/timezone?apiKey=test&lat=59.0&long=181.0",
            addr
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 400);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["code"], "INVALID_LONGITUDE");
    assert!(json["error"].as_str().unwrap().contains("Longitude"));
}

/// Test v1/timezone with invalid coordinates returns 400
#[tokio::test]
async fn test_v1_timezone_invalid_coords() {
    let app = Router::new().route("/v1/timezone", get(timezone_full_handler));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://{}/v1/timezone?apiKey=test&lat=-95.0&long=18.0",
            addr
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 400);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["code"], "INVALID_LATITUDE");
}

/// Test v1/ipgeo with invalid IP returns 400
#[tokio::test]
async fn test_v1_ipgeo_invalid_ip() {
    let mock = MockGeoIpReader::new();

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/v1/ipgeo", get(ipgeo_full_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/v1/ipgeo?apiKey=test&ip=invalid", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 400);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["code"], "INVALID_IP");
}

/// Test v1/ipgeo with EU country (check is_eu flag)
#[tokio::test]
async fn test_v1_ipgeo_eu_country() {
    let mock = MockGeoIpReader::new().with_response(
        "1.2.3.4",
        Ok(GeoData {
            latitude: Some(52.52),
            longitude: Some(13.405),
            city: Some("Berlin".to_string()),
            country_name: Some("Germany".to_string()),
            country_code: Some("DE".to_string()),
            state_prov: Some("Berlin".to_string()),
            state_code: Some("BE".to_string()),
            postal_code: Some("10115".to_string()),
            geoname_id: Some(2950159),
        }),
    );

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/v1/ipgeo", get(ipgeo_full_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/v1/ipgeo?apiKey=test&ip=1.2.3.4", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();

    // Germany is in EU
    assert_eq!(json["location"]["is_eu"], true);
    assert_eq!(json["location"]["country_code2"], "DE");
    assert_eq!(json["currency"]["code"], "EUR");
    assert_eq!(json["currency"]["symbol"], "€");
}

// ============================================================================
// Root Endpoint Tests (/ - client IP geolocation)
// ============================================================================

/// Test root endpoint returns geolocation for direct connection IP
#[tokio::test]
async fn test_root_endpoint_direct_ip() {
    // The mock will receive 127.0.0.1 as the client IP from direct connection
    let mock = MockGeoIpReader::new().with_response(
        "127.0.0.1",
        Ok(GeoData {
            latitude: Some(37.751),
            longitude: Some(-97.822),
            city: Some("Test City".to_string()),
            country_name: Some("United States".to_string()),
            country_code: Some("US".to_string()),
            state_prov: None,
            state_code: None,
            postal_code: None,
            geoname_id: None,
        }),
    );

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/", get(root_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["city"], "Test City");
    assert_eq!(json["country_name"], "United States");
    assert_eq!(json["languages"], "en-US,en");
}

/// Test root endpoint with X-Forwarded-For header
#[tokio::test]
async fn test_root_endpoint_x_forwarded_for() {
    // The mock should receive the IP from X-Forwarded-For header
    let mock = MockGeoIpReader::new().with_response(
        "203.0.113.1",
        Ok(GeoData {
            latitude: Some(51.5074),
            longitude: Some(-0.1278),
            city: Some("London".to_string()),
            country_name: Some("United Kingdom".to_string()),
            country_code: Some("GB".to_string()),
            state_prov: None,
            state_code: None,
            postal_code: None,
            geoname_id: None,
        }),
    );

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/", get(root_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/", addr))
        .header("X-Forwarded-For", "203.0.113.1, 10.0.0.1")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["city"], "London");
    assert_eq!(json["country_name"], "United Kingdom");
}

/// Test root endpoint with X-Real-IP header
#[tokio::test]
async fn test_root_endpoint_x_real_ip() {
    let mock = MockGeoIpReader::new().with_response(
        "198.51.100.1",
        Ok(GeoData {
            latitude: Some(35.6762),
            longitude: Some(139.6503),
            city: Some("Tokyo".to_string()),
            country_name: Some("Japan".to_string()),
            country_code: Some("JP".to_string()),
            state_prov: None,
            state_code: None,
            postal_code: None,
            geoname_id: None,
        }),
    );

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/", get(root_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/", addr))
        .header("X-Real-IP", "198.51.100.1")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["city"], "Tokyo");
    assert_eq!(json["country_name"], "Japan");
}

/// Test root endpoint with CF-Connecting-IP header (Cloudflare)
#[tokio::test]
async fn test_root_endpoint_cf_connecting_ip() {
    let mock = MockGeoIpReader::new().with_response(
        "192.0.2.1",
        Ok(GeoData {
            latitude: Some(52.52),
            longitude: Some(13.405),
            city: Some("Berlin".to_string()),
            country_name: Some("Germany".to_string()),
            country_code: Some("DE".to_string()),
            state_prov: None,
            state_code: None,
            postal_code: None,
            geoname_id: None,
        }),
    );

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/", get(root_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/", addr))
        .header("CF-Connecting-IP", "192.0.2.1")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["city"], "Berlin");
    assert_eq!(json["country_name"], "Germany");
}

/// Test root endpoint Cache-Control headers
#[tokio::test]
async fn test_root_endpoint_cache_control_headers() {
    let mock = MockGeoIpReader::new().with_response(
        "127.0.0.1",
        Ok(GeoData {
            latitude: Some(40.0),
            longitude: Some(-74.0),
            city: Some("Test".to_string()),
            country_name: Some("Test Country".to_string()),
            country_code: Some("TC".to_string()),
            state_prov: None,
            state_code: None,
            postal_code: None,
            geoname_id: None,
        }),
    );

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/", get(root_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/", addr))
        .send()
        .await
        .unwrap();

    let cache_control = response.headers().get("cache-control");
    assert!(cache_control.is_some());
    assert_eq!(
        cache_control.unwrap().to_str().unwrap(),
        "public, max-age=1209600"
    );
}

/// Test root endpoint with unknown IP returns empty response
#[tokio::test]
async fn test_root_endpoint_unknown_ip() {
    let mock = MockGeoIpReader::new(); // No responses configured

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/", get(root_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["city"], "");
    assert_eq!(json["country_name"], "");
    assert_eq!(json["languages"], "");
}

// ============================================================================
// LLM/Agent Documentation Endpoint Tests
// ============================================================================

/// Test OpenAPI specification endpoint returns valid YAML
#[tokio::test]
async fn test_openapi_endpoint() {
    let state = create_minimal_test_state();
    let app = Router::new()
        .route("/openapi.yaml", get(openapi_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/openapi.yaml", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Check content type
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    assert!(content_type
        .unwrap()
        .to_str()
        .unwrap()
        .contains("application/yaml"));

    // Check content contains expected OpenAPI structure
    let body = response.text().await.unwrap();
    assert!(body.contains("openapi: 3.1"));
    assert!(body.contains("IP Geolocation API"));
    assert!(body.contains("/ipgeo"));
    assert!(body.contains("/timezone"));
    assert!(body.contains("/v1/ipgeo"));
    assert!(body.contains("/v1/timezone"));
    assert!(body.contains("/health"));
}

/// Test OpenAPI specification is valid YAML
#[tokio::test]
async fn test_openapi_valid_yaml() {
    let state = create_minimal_test_state();
    let app = Router::new()
        .route("/openapi.yaml", get(openapi_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/openapi.yaml", addr))
        .send()
        .await
        .unwrap();

    let body = response.text().await.unwrap();

    // Parse as YAML to validate syntax using yaml-rust2
    let yaml_docs = yaml_rust2::YamlLoader::load_from_str(&body);
    assert!(yaml_docs.is_ok(), "OpenAPI spec should be valid YAML");

    let docs = yaml_docs.unwrap();
    assert!(!docs.is_empty(), "Should have at least one YAML document");

    let yaml = &docs[0];
    assert_eq!(yaml["openapi"].as_str(), Some("3.1.0"));
    assert!(!yaml["paths"].is_badvalue());
    assert!(!yaml["components"]["schemas"].is_badvalue());
}

/// Test OpenAPI spec is generated from code (not static file)
#[tokio::test]
async fn test_openapi_generated_from_code() {
    // Verify ApiDoc generates valid spec
    let spec = ApiDoc::openapi();
    let yaml = spec.to_yaml().unwrap();

    // Should contain all our endpoints
    assert!(yaml.contains("/ipgeo"));
    assert!(yaml.contains("/timezone"));
    assert!(yaml.contains("/v1/ipgeo"));
    assert!(yaml.contains("/v1/timezone"));
    assert!(yaml.contains("/health"));

    // Should contain our schemas
    assert!(yaml.contains("IpGeoResponse"));
    assert!(yaml.contains("TimezoneResponse"));
    assert!(yaml.contains("ApiErrorResponse"));
}

/// Test LLM documentation endpoint returns valid content
#[tokio::test]
async fn test_llms_txt_endpoint() {
    let app = Router::new().route("/llms.txt", get(llms_txt_handler));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/llms.txt", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Check content type
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    assert!(content_type
        .unwrap()
        .to_str()
        .unwrap()
        .contains("text/plain"));

    // Check content contains expected documentation
    let body = response.text().await.unwrap();
    assert!(body.contains("IP Geolocation API"));
    assert!(body.contains("/ipgeo"));
    assert!(body.contains("/timezone"));
    assert!(body.contains("openapi.yaml"));
    assert!(body.contains("curl"));
}

/// Test LLM documentation contains all required sections
#[tokio::test]
async fn test_llms_txt_content_structure() {
    let app = Router::new().route("/llms.txt", get(llms_txt_handler));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/llms.txt", addr))
        .send()
        .await
        .unwrap();

    let body = response.text().await.unwrap();

    // Check required sections
    assert!(body.contains("What This API Does"));
    assert!(body.contains("When to Use This API"));
    assert!(body.contains("Quick Reference"));
    assert!(body.contains("Examples"));
    assert!(body.contains("Error Handling"));
    assert!(body.contains("API Discovery"));
}

/// Test sitemap.xml endpoint returns valid XML
#[tokio::test]
async fn test_sitemap_endpoint() {
    let state = create_minimal_test_state();
    let app = Router::new()
        .route("/sitemap.xml", get(sitemap_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/sitemap.xml", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Check content type
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    assert!(content_type
        .unwrap()
        .to_str()
        .unwrap()
        .contains("application/xml"));

    // Check XML content
    let body = response.text().await.unwrap();
    assert!(body.contains("<?xml version"));
    assert!(body.contains("<urlset"));
    assert!(body.contains("test.example.com")); // Uses base_url from state
    assert!(body.contains("/ipgeo"));
    assert!(body.contains("/timezone"));
    assert!(body.contains("/openapi.yaml"));
    assert!(body.contains("/llms.txt"));
}

/// Test .well-known/openapi.yaml endpoint
#[tokio::test]
async fn test_wellknown_openapi_endpoint() {
    let state = create_minimal_test_state();
    let app = Router::new()
        .route("/.well-known/openapi.yaml", get(wellknown_openapi_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/.well-known/openapi.yaml", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Check content type
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    assert!(content_type
        .unwrap()
        .to_str()
        .unwrap()
        .contains("application/yaml"));

    // Check it's the same as /openapi.yaml
    let body = response.text().await.unwrap();
    assert!(body.contains("openapi: 3.1"));
    assert!(body.contains("IP Geolocation API"));
}

/// Test .well-known/ai-plugin.json endpoint
#[tokio::test]
async fn test_wellknown_ai_plugin_endpoint() {
    let state = create_minimal_test_state();
    let app = Router::new()
        .route(
            "/.well-known/ai-plugin.json",
            get(wellknown_ai_plugin_handler),
        )
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/.well-known/ai-plugin.json", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Check content type
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    assert!(content_type
        .unwrap()
        .to_str()
        .unwrap()
        .contains("application/json"));

    // Check JSON structure
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["schema_version"], "v1");
    assert_eq!(json["name_for_model"], "ip_geolocation");
    assert!(!json["description_for_model"].as_str().unwrap().is_empty());
    assert_eq!(json["api"]["type"], "openapi");
    assert!(json["api"]["url"]
        .as_str()
        .unwrap()
        .contains("openapi.yaml"));
}

// ============================================================================
// Protobuf Response Tests
// ============================================================================

/// Test timezone endpoint with protobuf Accept header
#[tokio::test]
async fn test_timezone_protobuf_response() {
    let app = Router::new().route("/timezone", get(timezone_handler));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://{}/timezone?lat=59.329504&long=18.069532",
            addr
        ))
        .header("Accept", "application/x-protobuf")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Check content type is protobuf
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    assert!(content_type
        .unwrap()
        .to_str()
        .unwrap()
        .contains("application/x-protobuf"));

    // Decode protobuf response
    let bytes = response.bytes().await.unwrap();
    let proto = geolocation::TimezoneResponse::decode(bytes).unwrap();
    assert_eq!(proto.timezone, "Europe/Stockholm");
}

/// Test ipgeo endpoint with protobuf Accept header
#[tokio::test]
async fn test_ipgeo_protobuf_response() {
    let mock = MockGeoIpReader::new().with_response(
        "8.8.8.8",
        Ok(GeoData {
            latitude: Some(37.751),
            longitude: Some(-97.822),
            city: Some("Mountain View".to_string()),
            country_name: Some("United States".to_string()),
            country_code: Some("US".to_string()),
            state_prov: Some("California".to_string()),
            state_code: Some("CA".to_string()),
            postal_code: Some("94043".to_string()),
            geoname_id: Some(5375480),
        }),
    );

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/ipgeo", get(ipgeo_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/ipgeo?ip=8.8.8.8", addr))
        .header("Accept", "application/x-protobuf")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Check content type is protobuf
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    assert!(content_type
        .unwrap()
        .to_str()
        .unwrap()
        .contains("application/x-protobuf"));

    // Decode protobuf response
    let bytes = response.bytes().await.unwrap();
    let proto = geolocation::IpGeoResponse::decode(bytes).unwrap();
    assert_eq!(proto.city, "Mountain View");
    assert_eq!(proto.country_name, "United States");
    assert!(proto.latitude.is_some());
}

/// Test v1/ipgeo endpoint with protobuf Accept header
#[tokio::test]
async fn test_v1_ipgeo_protobuf_response() {
    let mock = MockGeoIpReader::new().with_response(
        "1.1.1.1",
        Ok(GeoData {
            latitude: Some(51.5074),
            longitude: Some(-0.1278),
            city: Some("London".to_string()),
            country_name: Some("United Kingdom".to_string()),
            country_code: Some("GB".to_string()),
            state_prov: None,
            state_code: None,
            postal_code: None,
            geoname_id: None,
        }),
    );

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/v1/ipgeo", get(ipgeo_full_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/v1/ipgeo?ip=1.1.1.1", addr))
        .header("Accept", "application/x-protobuf")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Check content type is protobuf
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    assert!(content_type
        .unwrap()
        .to_str()
        .unwrap()
        .contains("application/x-protobuf"));

    // Decode protobuf response
    let bytes = response.bytes().await.unwrap();
    let proto = geolocation::IpGeoResponseFull::decode(bytes).unwrap();
    assert_eq!(proto.ip, Some("1.1.1.1".to_string()));
    assert!(proto.location.is_some());
    let location = proto.location.unwrap();
    assert_eq!(location.city, Some("London".to_string()));
}

/// Test error response in protobuf format
#[tokio::test]
async fn test_error_protobuf_response() {
    let mock = MockGeoIpReader::new();

    let state = create_test_state(mock);
    let app = Router::new()
        .route("/ipgeo", get(ipgeo_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/ipgeo?ip=invalid-ip", addr))
        .header("Accept", "application/x-protobuf")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 400);

    // Check content type is protobuf
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    assert!(content_type
        .unwrap()
        .to_str()
        .unwrap()
        .contains("application/x-protobuf"));

    // Decode protobuf error response
    let bytes = response.bytes().await.unwrap();
    let proto = geolocation::ApiError::decode(bytes).unwrap();
    assert_eq!(proto.code, "INVALID_IP");
    assert!(proto.error.contains("Invalid IP address"));
}
