use axum::{
    body::Body,
    extract::{ConnectInfo, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use prost::Message;
use std::net::{IpAddr, SocketAddr};
use utoipa::OpenApi;

use crate::cache::SharedGeoCache;
use crate::country_data::{get_country_metadata, get_flag_path};
use crate::geoip::SharedGeoIpReader;
use crate::languages::get_languages;
use crate::models::{
    ApiErrorResponse, CountryMetadataInfo, CurrencyInfo, GeoData, IpGeoQuery, IpGeoResponse,
    IpGeoResponseFull, LocationInfo, TimeZoneInfo, TimeZoneInfoFull, TimezoneQuery,
    TimezoneResponse, TimezoneResponseFull,
};
use crate::proto::{accepts_protobuf, geolocation, PROTOBUF_CONTENT_TYPE};
use crate::timezone::lookup_timezone;
use crate::tz_utils::get_timezone_details;

/// OpenAPI documentation for the IP Geolocation API
#[derive(OpenApi)]
#[openapi(
    info(
        title = "IP Geolocation API",
        description = "A high-performance IP geolocation and timezone lookup API.\n\nProvides:\n- IP address to geographic location mapping\n- Coordinate-based timezone lookups\n- Country metadata (currency, languages, calling codes)\n\nAll responses are cached for 2 weeks (Cache-Control: public, max-age=1209600).",
        version = "1.0.0",
        license(name = "MIT", url = "https://opensource.org/licenses/MIT")
    ),
    servers(
        (url = "https://geoip.vpetersson.com", description = "Production server")
    ),
    paths(
        root_handler,
        ipgeo_handler,
        ipgeo_full_handler,
        timezone_handler,
        timezone_full_handler,
        health_handler,
    ),
    components(schemas(
        IpGeoResponse,
        IpGeoResponseFull,
        TimezoneResponse,
        TimezoneResponseFull,
        LocationInfo,
        CountryMetadataInfo,
        CurrencyInfo,
        TimeZoneInfo,
        TimeZoneInfoFull,
        ApiErrorResponse,
    ))
)]
pub struct ApiDoc;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub geoip: SharedGeoIpReader,
    pub cache: SharedGeoCache,
    /// Base URL for the API (used in OpenAPI spec, sitemap, etc.)
    pub base_url: String,
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

/// Helper to get Accept header value from HeaderMap
fn get_accept_header(headers: &HeaderMap) -> Option<&str> {
    headers.get(header::ACCEPT).and_then(|v| v.to_str().ok())
}

/// Build OK response with content negotiation (JSON or Protobuf)
fn build_response<T, P>(response: &T, proto_response: P, use_protobuf: bool) -> Response<Body>
where
    T: serde::Serialize,
    P: Message,
{
    if use_protobuf {
        let body = proto_response.encode_to_vec();
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, PROTOBUF_CONTENT_TYPE)
            .header(header::CACHE_CONTROL, CACHE_CONTROL)
            .body(Body::from(body))
            .unwrap()
    } else {
        let body = serde_json::to_vec(response).unwrap();
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .header(header::CACHE_CONTROL, CACHE_CONTROL)
            .body(Body::from(body))
            .unwrap()
    }
}

/// Build error response with content negotiation (JSON or Protobuf)
fn build_error_response(error: &ApiError, use_protobuf: bool) -> Response<Body> {
    let proto_error = geolocation::ApiError {
        error: error.error.clone(),
        code: error.code.to_string(),
    };

    if use_protobuf {
        let body = proto_error.encode_to_vec();
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header(header::CONTENT_TYPE, PROTOBUF_CONTENT_TYPE)
            .header(header::CACHE_CONTROL, CACHE_CONTROL)
            .body(Body::from(body))
            .unwrap()
    } else {
        let body = serde_json::to_vec(error).unwrap();
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .header(header::CACHE_CONTROL, CACHE_CONTROL)
            .body(Body::from(body))
            .unwrap()
    }
}

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

/// Extract client IP from request headers, checking proxy headers first
pub fn extract_client_ip(headers: &HeaderMap, connect_info: Option<SocketAddr>) -> String {
    // Check Cloudflare header first
    if let Some(cf_ip) = headers
        .get("CF-Connecting-IP")
        .and_then(|v| v.to_str().ok())
    {
        return cf_ip.to_string();
    }

    // Check X-Real-IP (common with nginx)
    if let Some(real_ip) = headers.get("X-Real-IP").and_then(|v| v.to_str().ok()) {
        return real_ip.to_string();
    }

    // Check X-Forwarded-For (take first IP in chain)
    if let Some(forwarded_for) = headers.get("X-Forwarded-For").and_then(|v| v.to_str().ok()) {
        if let Some(first_ip) = forwarded_for.split(',').next() {
            return first_ip.trim().to_string();
        }
    }

    // Fall back to direct connection IP
    connect_info
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "-".to_string())
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
    let tz_details = timezone_name
        .as_ref()
        .and_then(|tz| get_timezone_details(tz));

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
            state_code: geo_data
                .state_code
                .as_ref()
                .map(|sc| format!("{}-{}", geo_data.country_code.as_deref().unwrap_or(""), sc)),
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
            languages: country_meta
                .map(|m| m.languages.split(',').map(|s| s.to_string()).collect()),
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

/// Get geolocation for an IP address (simple format)
///
/// Returns basic geographic location data for a given IP address.
/// Use the `fields` parameter with "*" or "location" for full format response.
/// Supports content negotiation: use Accept: application/x-protobuf for protobuf response.
#[utoipa::path(
    get,
    path = "/ipgeo",
    params(IpGeoQuery),
    responses(
        (status = 200, description = "Successful geolocation lookup", body = IpGeoResponse),
        (status = 400, description = "Invalid IP address", body = ApiErrorResponse)
    ),
    tag = "IP Geolocation"
)]
pub async fn ipgeo_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<IpGeoQuery>,
) -> Response<Body> {
    let ip = params.ip.trim();
    let use_protobuf = accepts_protobuf(get_accept_header(&headers));

    // Validate IP address
    if let Err(e) = validate_ip(ip) {
        return build_error_response(&e, use_protobuf);
    }

    // Check cache first (only for simple format and JSON)
    if params.fields.is_none() && !use_protobuf {
        if let Some(cached) = state.cache.get(ip) {
            let proto: geolocation::IpGeoResponse = (&cached).into();
            return build_response(&cached, proto, use_protobuf);
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

        let proto: geolocation::IpGeoResponseFull = (&response).into();
        build_response(&response, proto, use_protobuf)
    } else {
        // Simple response format (backward compatible)
        let response = match geo_result {
            Ok(geo_data) => build_simple_response(&geo_data),
            Err(_) => IpGeoResponse::default(),
        };

        // Cache the simple response (JSON only)
        if !use_protobuf {
            state.cache.insert(ip.to_string(), response.clone());
        }

        let proto: geolocation::IpGeoResponse = (&response).into();
        build_response(&response, proto, use_protobuf)
    }
}

/// Get geolocation for an IP address (full format)
///
/// Returns comprehensive location data with extended fields including
/// continent, country metadata, currency, and detailed timezone information.
/// Supports content negotiation: use Accept: application/x-protobuf for protobuf response.
#[utoipa::path(
    get,
    path = "/v1/ipgeo",
    params(IpGeoQuery),
    responses(
        (status = 200, description = "Successful geolocation lookup", body = IpGeoResponseFull),
        (status = 400, description = "Invalid IP address", body = ApiErrorResponse)
    ),
    tag = "IP Geolocation"
)]
pub async fn ipgeo_full_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<IpGeoQuery>,
) -> Response<Body> {
    let ip = params.ip.trim();
    let use_protobuf = accepts_protobuf(get_accept_header(&headers));

    // Validate IP address
    if let Err(e) = validate_ip(ip) {
        return build_error_response(&e, use_protobuf);
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

    let proto: geolocation::IpGeoResponseFull = (&response).into();
    build_response(&response, proto, use_protobuf)
}

/// Get timezone for coordinates (simple format)
///
/// Returns timezone name for given geographic coordinates.
/// Supports content negotiation: use Accept: application/x-protobuf for protobuf response.
#[utoipa::path(
    get,
    path = "/timezone",
    params(TimezoneQuery),
    responses(
        (status = 200, description = "Successful timezone lookup", body = TimezoneResponse),
        (status = 400, description = "Invalid coordinates", body = ApiErrorResponse)
    ),
    tag = "Timezone"
)]
pub async fn timezone_handler(
    headers: HeaderMap,
    Query(params): Query<TimezoneQuery>,
) -> Response<Body> {
    let use_protobuf = accepts_protobuf(get_accept_header(&headers));

    // Validate coordinates
    if let Err(e) = validate_latitude(params.lat) {
        return build_error_response(&e, use_protobuf);
    }
    if let Err(e) = validate_longitude(params.long) {
        return build_error_response(&e, use_protobuf);
    }

    let timezone = lookup_timezone(params.lat, params.long).unwrap_or_default();
    let response = TimezoneResponse { timezone };

    let proto: geolocation::TimezoneResponse = (&response).into();
    build_response(&response, proto, use_protobuf)
}

/// Get timezone for coordinates (full format)
///
/// Returns comprehensive timezone details including current time,
/// UTC offset, and daylight saving time information.
/// Supports content negotiation: use Accept: application/x-protobuf for protobuf response.
#[utoipa::path(
    get,
    path = "/v1/timezone",
    params(TimezoneQuery),
    responses(
        (status = 200, description = "Successful timezone lookup", body = TimezoneResponseFull),
        (status = 400, description = "Invalid coordinates", body = ApiErrorResponse)
    ),
    tag = "Timezone"
)]
pub async fn timezone_full_handler(
    headers: HeaderMap,
    Query(params): Query<TimezoneQuery>,
) -> Response<Body> {
    let use_protobuf = accepts_protobuf(get_accept_header(&headers));

    // Validate coordinates
    if let Err(e) = validate_latitude(params.lat) {
        return build_error_response(&e, use_protobuf);
    }
    if let Err(e) = validate_longitude(params.long) {
        return build_error_response(&e, use_protobuf);
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

    let proto: geolocation::TimezoneResponseFull = (&response).into();
    build_response(&response, proto, use_protobuf)
}

/// Get geolocation for client's IP
///
/// Returns geolocation data for the requesting client's IP address.
/// Automatically detects the client IP from CF-Connecting-IP, X-Real-IP,
/// X-Forwarded-For headers, or the direct connection.
/// Supports content negotiation: use Accept: application/x-protobuf for protobuf response.
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Successful geolocation lookup", body = IpGeoResponse)
    ),
    tag = "IP Geolocation"
)]
pub async fn root_handler(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response<Body> {
    let ip = extract_client_ip(&headers, Some(addr));
    let use_protobuf = accepts_protobuf(get_accept_header(&headers));

    // Validate IP address (should always be valid from extraction, but be safe)
    if let Err(e) = validate_ip(&ip) {
        return build_error_response(&e, use_protobuf);
    }

    // Check cache first (JSON only)
    if !use_protobuf {
        if let Some(cached) = state.cache.get(&ip) {
            let proto: geolocation::IpGeoResponse = (&cached).into();
            return build_response(&cached, proto, use_protobuf);
        }
    }

    // Lookup in MaxMind database
    let geo_result = state.geoip.lookup(&ip);

    // Simple response format (same as /ipgeo)
    let response = match geo_result {
        Ok(geo_data) => build_simple_response(&geo_data),
        Err(_) => IpGeoResponse::default(),
    };

    // Cache the response (JSON only)
    if !use_protobuf {
        state.cache.insert(ip, response.clone());
    }

    let proto: geolocation::IpGeoResponse = (&response).into();
    build_response(&response, proto, use_protobuf)
}

/// Health check endpoint
///
/// Returns OK if the service is running.
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = String, example = json!("OK"))
    ),
    tag = "Health"
)]
pub async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// OpenAPI specification handler
///
/// Returns the OpenAPI 3.0 specification generated from the code.
pub async fn openapi_handler(State(state): State<AppState>) -> impl IntoResponse {
    let mut openapi = ApiDoc::openapi();

    // Update server URL from environment
    openapi.servers = Some(vec![utoipa::openapi::Server::new(&state.base_url)]);

    let spec = openapi.to_yaml().unwrap();
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/yaml; charset=utf-8")],
        spec,
    )
}

/// LLM-friendly documentation handler
///
/// Returns plain text documentation optimized for LLM consumption.
pub async fn llms_txt_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        include_str!("../llms.txt"),
    )
}

/// Robots.txt handler
///
/// Returns robots.txt for search engine crawlers.
pub async fn robots_txt_handler() -> impl IntoResponse {
    let robots = r#"User-agent: *
Allow: /
Allow: /openapi.yaml
Allow: /llms.txt
Allow: /sitemap.xml

# API endpoints - allow crawling for discovery
Allow: /ipgeo
Allow: /timezone
Allow: /v1/ipgeo
Allow: /v1/timezone

# Sitemap location
Sitemap: https://geoip.vpetersson.com/sitemap.xml
"#;

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        robots,
    )
}

/// Sitemap XML handler
///
/// Returns a sitemap.xml for search engine and agent discovery.
pub async fn sitemap_handler(State(state): State<AppState>) -> impl IntoResponse {
    let base = &state.base_url;
    let sitemap = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>{base}/</loc>
    <changefreq>monthly</changefreq>
    <priority>1.0</priority>
  </url>
  <url>
    <loc>{base}/openapi.yaml</loc>
    <changefreq>monthly</changefreq>
    <priority>0.8</priority>
  </url>
  <url>
    <loc>{base}/llms.txt</loc>
    <changefreq>monthly</changefreq>
    <priority>0.8</priority>
  </url>
  <url>
    <loc>{base}/ipgeo</loc>
    <changefreq>monthly</changefreq>
    <priority>0.9</priority>
  </url>
  <url>
    <loc>{base}/v1/ipgeo</loc>
    <changefreq>monthly</changefreq>
    <priority>0.9</priority>
  </url>
  <url>
    <loc>{base}/timezone</loc>
    <changefreq>monthly</changefreq>
    <priority>0.9</priority>
  </url>
  <url>
    <loc>{base}/v1/timezone</loc>
    <changefreq>monthly</changefreq>
    <priority>0.9</priority>
  </url>
</urlset>"#
    );

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        sitemap,
    )
}

/// Well-known OpenAPI handler (redirects to /openapi.yaml)
///
/// Serves the OpenAPI spec from the standard .well-known location.
pub async fn wellknown_openapi_handler(State(state): State<AppState>) -> impl IntoResponse {
    let mut openapi = ApiDoc::openapi();

    // Update server URL from environment
    openapi.servers = Some(vec![utoipa::openapi::Server::new(&state.base_url)]);

    let spec = openapi.to_yaml().unwrap();
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/yaml; charset=utf-8")],
        spec,
    )
}

/// Well-known AI plugin manifest
///
/// Returns an AI plugin manifest for ChatGPT-style agent discovery.
pub async fn wellknown_ai_plugin_handler(State(state): State<AppState>) -> impl IntoResponse {
    let base = &state.base_url;
    let manifest = serde_json::json!({
        "schema_version": "v1",
        "name_for_human": "IP Geolocation API",
        "name_for_model": "ip_geolocation",
        "description_for_human": "Get geographic location data from IP addresses and timezone information from coordinates.",
        "description_for_model": "Use this API to look up geographic location (city, country, coordinates, timezone) for any IP address, or to get timezone information from latitude/longitude coordinates. Supports both simple and detailed response formats.",
        "auth": {
            "type": "none"
        },
        "api": {
            "type": "openapi",
            "url": format!("{}/openapi.yaml", base)
        },
        "logo_url": format!("{}/static/flags/un.svg", base),
        "contact_email": "support@example.com",
        "legal_info_url": format!("{}/", base)
    });

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        Json(manifest),
    )
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

    #[test]
    fn test_openapi_spec_generation() {
        let spec = ApiDoc::openapi();
        let yaml = spec.to_yaml().unwrap();
        assert!(yaml.contains("IP Geolocation API"));
        assert!(yaml.contains("/ipgeo"));
        assert!(yaml.contains("/timezone"));
        assert!(yaml.contains("/v1/ipgeo"));
        assert!(yaml.contains("/v1/timezone"));
    }
}
