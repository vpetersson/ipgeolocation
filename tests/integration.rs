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
    health_handler, ipgeo_full_handler, ipgeo_handler, timezone_full_handler, timezone_handler,
    AppState,
};
use ipgeolocation::models::GeoData;

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
