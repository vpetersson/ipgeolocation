use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use std::net::IpAddr;

use crate::cache::SharedGeoCache;
use crate::country_data::{get_country_metadata, get_flag_path};
use crate::geoip::SharedGeoIpReader;
use crate::languages::get_languages;
use crate::models::{
    CountryMetadataInfo, CurrencyInfo, GeoData, IpGeoQuery, IpGeoResponse, IpGeoResponseFull,
    LocationInfo, TimeZoneInfo, TimeZoneInfoFull, TimezoneQuery, TimezoneResponse,
    TimezoneResponseFull,
};
use crate::timezone::lookup_timezone;
use crate::tz_utils::get_timezone_details;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub geoip: SharedGeoIpReader,
    pub cache: SharedGeoCache,
}

/// API error response
#[derive(serde::Serialize)]
pub struct ApiError {
    pub error: String,
    pub code: &'static str,
}

/// Cache-Control header value for responses (2 weeks)
/// IP geolocation data changes infrequently, so aggressive caching is safe
const CACHE_CONTROL: &str = "public, max-age=1209600";

/// Validate IP address format
fn validate_ip(ip: &str) -> Result<(), ApiError> {
    ip.parse::<IpAddr>().map_err(|_| ApiError {
        error: format!("Invalid IP address: {}", ip),
        code: "INVALID_IP",
    })?;
    Ok(())
}

/// Validate latitude range (-90 to 90)
fn validate_latitude(lat: f64) -> Result<(), ApiError> {
    if !(-90.0..=90.0).contains(&lat) {
        return Err(ApiError {
            error: format!("Latitude must be between -90 and 90, got: {}", lat),
            code: "INVALID_LATITUDE",
        });
    }
    Ok(())
}

/// Validate longitude range (-180 to 180)
fn validate_longitude(lng: f64) -> Result<(), ApiError> {
    if !(-180.0..=180.0).contains(&lng) {
        return Err(ApiError {
            error: format!("Longitude must be between -180 and 180, got: {}", lng),
            code: "INVALID_LONGITUDE",
        });
    }
    Ok(())
}

/// Build full response from GeoData
fn build_full_response(ip: &str, geo_data: &GeoData) -> IpGeoResponseFull {
    let country_code = geo_data.country_code.as_deref();
    let country_meta = get_country_metadata(country_code);

    // Get timezone from coordinates if available
    let timezone_name = match (geo_data.latitude, geo_data.longitude) {
        (Some(lat), Some(lng)) => lookup_timezone(lat, lng),
        _ => None,
    };

    // Get timezone details
    let tz_details = timezone_name.as_ref().and_then(|tz| get_timezone_details(tz));

    IpGeoResponseFull {
        ip: Some(ip.to_string()),
        location: Some(LocationInfo {
            continent_code: country_meta.map(|m| m.continent_code.to_string()),
            continent_name: country_meta.map(|m| m.continent_name.to_string()),
            country_code2: geo_data.country_code.clone(),
            country_code3: country_meta.map(|m| m.iso_code3.to_string()),
            country_name: geo_data.country_name.clone(),
            country_name_official: country_meta.map(|m| m.official_name.to_string()),
            country_capital: country_meta.map(|m| m.capital.to_string()),
            state_prov: geo_data.state_prov.clone(),
            state_code: geo_data.state_code.as_ref().map(|sc| {
                format!("{}-{}", geo_data.country_code.as_deref().unwrap_or(""), sc)
            }),
            district: None,
            city: geo_data.city.clone(),
            zipcode: geo_data.postal_code.clone(),
            latitude: geo_data.latitude.map(|l| format!("{:.5}", l)),
            longitude: geo_data.longitude.map(|l| format!("{:.5}", l)),
            is_eu: country_meta.map(|m| m.is_eu),
            country_flag: country_code.map(get_flag_path),
            geoname_id: geo_data.geoname_id.map(|id| id.to_string()),
            country_emoji: country_meta.map(|m| m.flag_emoji.to_string()),
        }),
        country_metadata: Some(CountryMetadataInfo {
            calling_code: country_meta.map(|m| m.calling_code.to_string()),
            tld: country_meta.map(|m| m.tld.to_string()),
            languages: country_meta.map(|m| {
                m.languages.split(',').map(|s| s.to_string()).collect()
            }),
        }),
        currency: country_meta.map(|m| CurrencyInfo {
            code: Some(m.currency_code.to_string()),
            name: Some(m.currency_name.to_string()),
            symbol: Some(m.currency_symbol.to_string()),
        }),
        time_zone: tz_details.map(|tz| TimeZoneInfoFull {
            name: Some(tz.name),
            offset: Some(tz.offset_hours),
            offset_with_dst: Some(tz.offset_with_dst_hours),
            current_time: Some(tz.current_time),
            current_time_unix: Some(tz.current_time_unix),
            is_dst: Some(tz.is_dst),
            dst_savings: Some(tz.dst_savings_hours),
            dst_exists: Some(tz.dst_exists),
        }),
    }
}

/// Build simple response from GeoData
fn build_simple_response(geo_data: &GeoData) -> IpGeoResponse {
    let timezone_name = match (geo_data.latitude, geo_data.longitude) {
        (Some(lat), Some(lng)) => lookup_timezone(lat, lng).unwrap_or_default(),
        _ => String::new(),
    };

    let languages = get_languages(geo_data.country_code.as_deref());

    IpGeoResponse {
        latitude: geo_data.latitude,
        longitude: geo_data.longitude,
        city: geo_data.city.clone().unwrap_or_default(),
        country_name: geo_data.country_name.clone().unwrap_or_default(),
        time_zone: TimeZoneInfo {
            name: timezone_name,
        },
        languages,
    }
}

/// Handler for GET /ipgeo
/// Returns geolocation data for a given IP address
/// Supports both simple format (backward compatible) and full format
pub async fn ipgeo_handler(
    State(state): State<AppState>,
    Query(params): Query<IpGeoQuery>,
) -> impl IntoResponse {
    let ip = params.ip.trim();

    // Validate IP address
    if let Err(e) = validate_ip(ip) {
        return (
            StatusCode::BAD_REQUEST,
            [(header::CACHE_CONTROL, CACHE_CONTROL)],
            Json(serde_json::to_value(e).unwrap()),
        );
    }

    // Check cache first (only for simple format)
    if params.fields.is_none() {
        if let Some(cached) = state.cache.get(ip) {
            return (
                StatusCode::OK,
                [(header::CACHE_CONTROL, CACHE_CONTROL)],
                Json(serde_json::to_value(cached).unwrap()),
            );
        }
    }

    // Lookup in MaxMind database
    let geo_result = state.geoip.lookup(ip);

    // Determine response format based on fields parameter
    let use_full_format = params
        .fields
        .as_ref()
        .map(|f| f.contains('*') || f.contains("location"))
        .unwrap_or(false);

    if use_full_format {
        // Full response format
        let response = match geo_result {
            Ok(geo_data) => build_full_response(ip, &geo_data),
            Err(_) => IpGeoResponseFull {
                ip: Some(ip.to_string()),
                ..Default::default()
            },
        };

        (
            StatusCode::OK,
            [(header::CACHE_CONTROL, CACHE_CONTROL)],
            Json(serde_json::to_value(response).unwrap()),
        )
    } else {
        // Simple response format (backward compatible)
        let response = match geo_result {
            Ok(geo_data) => build_simple_response(&geo_data),
            Err(_) => IpGeoResponse::default(),
        };

        // Cache the simple response
        state.cache.insert(ip.to_string(), response.clone());

        (
            StatusCode::OK,
            [(header::CACHE_CONTROL, CACHE_CONTROL)],
            Json(serde_json::to_value(response).unwrap()),
        )
    }
}

/// Handler for GET /v1/ipgeo (full format by default)
/// Returns full geolocation data with extended fields
pub async fn ipgeo_full_handler(
    State(state): State<AppState>,
    Query(params): Query<IpGeoQuery>,
) -> impl IntoResponse {
    let ip = params.ip.trim();

    // Validate IP address
    if let Err(e) = validate_ip(ip) {
        return (
            StatusCode::BAD_REQUEST,
            [(header::CACHE_CONTROL, CACHE_CONTROL)],
            Json(serde_json::to_value(e).unwrap()),
        );
    }

    // Lookup in MaxMind database
    let geo_result = state.geoip.lookup(ip);

    let response = match geo_result {
        Ok(geo_data) => build_full_response(ip, &geo_data),
        Err(_) => IpGeoResponseFull {
            ip: Some(ip.to_string()),
            ..Default::default()
        },
    };

    (
        StatusCode::OK,
        [(header::CACHE_CONTROL, CACHE_CONTROL)],
        Json(serde_json::to_value(response).unwrap()),
    )
}

/// Handler for GET /timezone
/// Returns timezone for given coordinates (simple format)
pub async fn timezone_handler(Query(params): Query<TimezoneQuery>) -> impl IntoResponse {
    // Validate coordinates
    if let Err(e) = validate_latitude(params.lat) {
        return (
            StatusCode::BAD_REQUEST,
            [(header::CACHE_CONTROL, CACHE_CONTROL)],
            Json(serde_json::to_value(e).unwrap()),
        );
    }
    if let Err(e) = validate_longitude(params.long) {
        return (
            StatusCode::BAD_REQUEST,
            [(header::CACHE_CONTROL, CACHE_CONTROL)],
            Json(serde_json::to_value(e).unwrap()),
        );
    }

    let timezone = lookup_timezone(params.lat, params.long).unwrap_or_default();

    (
        StatusCode::OK,
        [(header::CACHE_CONTROL, CACHE_CONTROL)],
        Json(serde_json::to_value(TimezoneResponse { timezone }).unwrap()),
    )
}

/// Handler for GET /v1/timezone
/// Returns full timezone details for given coordinates
pub async fn timezone_full_handler(Query(params): Query<TimezoneQuery>) -> impl IntoResponse {
    // Validate coordinates
    if let Err(e) = validate_latitude(params.lat) {
        return (
            StatusCode::BAD_REQUEST,
            [(header::CACHE_CONTROL, CACHE_CONTROL)],
            Json(serde_json::to_value(e).unwrap()),
        );
    }
    if let Err(e) = validate_longitude(params.long) {
        return (
            StatusCode::BAD_REQUEST,
            [(header::CACHE_CONTROL, CACHE_CONTROL)],
            Json(serde_json::to_value(e).unwrap()),
        );
    }

    let timezone_name = lookup_timezone(params.lat, params.long);

    let response = match &timezone_name {
        Some(tz_name) => {
            let details = get_timezone_details(tz_name);
            TimezoneResponseFull {
                timezone: tz_name.clone(),
                offset: details.as_ref().map(|d| d.offset_hours),
                offset_with_dst: details.as_ref().map(|d| d.offset_with_dst_hours),
                current_time: details.as_ref().map(|d| d.current_time.clone()),
                current_time_unix: details.as_ref().map(|d| d.current_time_unix),
                is_dst: details.as_ref().map(|d| d.is_dst),
                dst_exists: details.as_ref().map(|d| d.dst_exists),
            }
        }
        None => TimezoneResponseFull {
            timezone: String::new(),
            offset: None,
            offset_with_dst: None,
            current_time: None,
            current_time_unix: None,
            is_dst: None,
            dst_exists: None,
        },
    };

    (
        StatusCode::OK,
        [(header::CACHE_CONTROL, CACHE_CONTROL)],
        Json(serde_json::to_value(response).unwrap()),
    )
}

/// Health check endpoint
pub async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ip_valid() {
        assert!(validate_ip("8.8.8.8").is_ok());
        assert!(validate_ip("2001:4860:4860::8888").is_ok());
        assert!(validate_ip("192.168.1.1").is_ok());
    }

    #[test]
    fn test_validate_ip_invalid() {
        assert!(validate_ip("not-an-ip").is_err());
        assert!(validate_ip("256.256.256.256").is_err());
        assert!(validate_ip("").is_err());
    }

    #[test]
    fn test_validate_latitude_valid() {
        assert!(validate_latitude(0.0).is_ok());
        assert!(validate_latitude(90.0).is_ok());
        assert!(validate_latitude(-90.0).is_ok());
        assert!(validate_latitude(45.5).is_ok());
    }

    #[test]
    fn test_validate_latitude_invalid() {
        assert!(validate_latitude(91.0).is_err());
        assert!(validate_latitude(-91.0).is_err());
        assert!(validate_latitude(180.0).is_err());
    }

    #[test]
    fn test_validate_longitude_valid() {
        assert!(validate_longitude(0.0).is_ok());
        assert!(validate_longitude(180.0).is_ok());
        assert!(validate_longitude(-180.0).is_ok());
        assert!(validate_longitude(45.5).is_ok());
    }

    #[test]
    fn test_validate_longitude_invalid() {
        assert!(validate_longitude(181.0).is_err());
        assert!(validate_longitude(-181.0).is_err());
        assert!(validate_longitude(360.0).is_err());
    }

    #[test]
    fn test_timezone_lookup_stockholm() {
        let tz = lookup_timezone(59.329504, 18.069532);
        assert_eq!(tz, Some("Europe/Stockholm".to_string()));
    }

    #[test]
    fn test_timezone_lookup_new_york() {
        let tz = lookup_timezone(40.7128, -74.0060);
        assert_eq!(tz, Some("America/New_York".to_string()));
    }

    #[test]
    fn test_timezone_lookup_invalid() {
        let tz = lookup_timezone(999.0, 999.0);
        assert!(tz.is_none());
    }

    #[test]
    fn test_get_languages_us() {
        let langs = get_languages(Some("US"));
        assert_eq!(langs, "en-US,en");
    }

    #[test]
    fn test_get_languages_unknown() {
        let langs = get_languages(Some("XX"));
        assert_eq!(langs, "");
    }

    #[test]
    fn test_app_state_clone() {
        fn assert_clone<T: Clone>() {}
        assert_clone::<AppState>();
    }

    #[test]
    fn test_build_simple_response() {
        let geo_data = GeoData {
            latitude: Some(51.5074),
            longitude: Some(-0.1278),
            city: Some("London".to_string()),
            country_name: Some("United Kingdom".to_string()),
            country_code: Some("GB".to_string()),
            state_prov: None,
            state_code: None,
            postal_code: None,
            geoname_id: None,
        };

        let response = build_simple_response(&geo_data);

        assert_eq!(response.latitude, Some(51.5074));
        assert_eq!(response.city, "London");
        assert_eq!(response.time_zone.name, "Europe/London");
    }

    #[test]
    fn test_build_full_response() {
        let geo_data = GeoData {
            latitude: Some(37.751),
            longitude: Some(-97.822),
            city: Some("Test City".to_string()),
            country_name: Some("United States".to_string()),
            country_code: Some("US".to_string()),
            state_prov: Some("Kansas".to_string()),
            state_code: Some("KS".to_string()),
            postal_code: Some("67401".to_string()),
            geoname_id: Some(123456),
        };

        let response = build_full_response("8.8.8.8", &geo_data);

        assert_eq!(response.ip, Some("8.8.8.8".to_string()));
        assert!(response.location.is_some());
        let location = response.location.unwrap();
        assert_eq!(location.country_code2, Some("US".to_string()));
        assert_eq!(location.country_code3, Some("USA".to_string()));
        assert_eq!(location.is_eu, Some(false));
    }

    #[test]
    fn test_ipgeo_response_default_empty_strings() {
        let response = IpGeoResponse::default();

        assert!(response.latitude.is_none());
        assert!(response.longitude.is_none());
        assert!(response.city.is_empty());
        assert!(response.country_name.is_empty());
        assert!(response.time_zone.name.is_empty());
        assert!(response.languages.is_empty());
    }

    #[test]
    fn test_timezone_response_creation() {
        let response = TimezoneResponse {
            timezone: "Asia/Tokyo".to_string(),
        };
        assert_eq!(response.timezone, "Asia/Tokyo");
    }

    #[test]
    fn test_cache_control_constant() {
        assert_eq!(CACHE_CONTROL, "public, max-age=1209600");
    }

    #[test]
    fn test_country_metadata_us() {
        let meta = get_country_metadata(Some("US")).unwrap();
        assert_eq!(meta.name, "United States");
        assert_eq!(meta.currency_code, "USD");
    }

    #[test]
    fn test_flag_path() {
        let path = get_flag_path("US");
        assert_eq!(path, "/static/flags/us.svg");
    }

    #[test]
    fn test_api_error_serialization() {
        let error = ApiError {
            error: "Test error".to_string(),
            code: "TEST_ERROR",
        };
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("Test error"));
        assert!(json.contains("TEST_ERROR"));
    }
}
