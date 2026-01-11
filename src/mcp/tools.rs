//! MCP Tool handlers for IP geolocation operations
//!
//! This module provides MCP tool implementations that wrap the existing
//! geolocation functionality.

use std::collections::HashMap;
use std::net::IpAddr;

use async_trait::async_trait;
use mcp_protocol_sdk::core::error::McpResult;
use mcp_protocol_sdk::core::tool::ToolHandler;
use mcp_protocol_sdk::protocol::types::{CallToolResult, ContentBlock};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::country_data::{get_country_metadata, get_flag_path};
use crate::geoip::{GeoIpError, SharedGeoIpReader};
use crate::languages::get_languages;
use crate::models::{
    CountryMetadataInfo, CurrencyInfo, GeoData, IpGeoResponse, IpGeoResponseFull, LocationInfo,
    TimeZoneInfo, TimeZoneInfoFull, TimezoneResponse, TimezoneResponseFull,
};
use crate::timezone::lookup_timezone;
use crate::tz_utils::get_timezone_details;

/// Maximum number of IPs allowed in a bulk lookup
pub const BULK_LOOKUP_MAX_IPS: usize = 100;

/// Error codes for MCP tools
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpErrorCode {
    InvalidIp,
    PrivateIp,
    NotFound,
    BulkLimitExceeded,
    InvalidLatitude,
    InvalidLongitude,
    StdioNoCallerIp,
}

impl McpErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidIp => "INVALID_IP",
            Self::PrivateIp => "PRIVATE_IP",
            Self::NotFound => "NOT_FOUND",
            Self::BulkLimitExceeded => "BULK_LIMIT_EXCEEDED",
            Self::InvalidLatitude => "INVALID_LATITUDE",
            Self::InvalidLongitude => "INVALID_LONGITUDE",
            Self::StdioNoCallerIp => "STDIO_NO_CALLER_IP",
        }
    }
}

/// Input parameters for geoip_lookup tool
#[derive(Debug, Deserialize)]
pub struct GeoIpLookupInput {
    pub ip: String,
    #[serde(default = "default_format")]
    pub format: String,
}

/// Input parameters for geoip_bulk_lookup tool
#[derive(Debug, Deserialize)]
pub struct GeoIpBulkLookupInput {
    pub ips: Vec<String>,
    #[serde(default = "default_format")]
    pub format: String,
}

/// Input parameters for geoip_lookup_self tool
#[derive(Debug, Deserialize)]
pub struct GeoIpLookupSelfInput {
    #[serde(default = "default_format")]
    pub format: String,
}

/// Input parameters for timezone_lookup tool
#[derive(Debug, Deserialize)]
pub struct TimezoneLookupInput {
    pub lat: f64,
    pub lon: f64,
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "full".to_string()
}

/// Result of a bulk lookup for a single IP
#[derive(Debug, Serialize)]
pub struct BulkLookupError {
    pub ip: String,
    pub code: String,
    pub message: String,
}

/// Result of a bulk lookup operation
#[derive(Debug, Serialize)]
pub struct BulkLookupResult {
    pub results: Vec<IpGeoResponseFull>,
    pub errors: Vec<BulkLookupError>,
}

/// Check if an IP address is private/loopback
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            ipv4.is_loopback()
                || ipv4.is_private()
                || ipv4.is_link_local()
                || ipv4.is_broadcast()
                || ipv4.is_unspecified()
        }
        IpAddr::V6(ipv6) => ipv6.is_loopback() || ipv6.is_unspecified(),
    }
}

/// Validate an IP address string
fn validate_ip(ip_str: &str) -> Result<IpAddr, (McpErrorCode, String)> {
    ip_str.parse::<IpAddr>().map_err(|_| {
        (
            McpErrorCode::InvalidIp,
            format!("Invalid IP address: {}", ip_str),
        )
    })
}

/// Build a simple response from GeoData
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

/// Build a full response from GeoData
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

/// Create a text content block
fn text_content(text: String) -> ContentBlock {
    ContentBlock::Text {
        text,
        annotations: None,
        meta: None,
    }
}

/// Create an error CallToolResult
fn error_result(code: McpErrorCode, message: &str) -> CallToolResult {
    let error_json = serde_json::json!({
        "error": message,
        "code": code.as_str()
    });

    CallToolResult {
        content: vec![text_content(
            serde_json::to_string_pretty(&error_json).unwrap(),
        )],
        is_error: Some(true),
        structured_content: None,
        meta: None,
    }
}

/// Create a success CallToolResult with structured content
fn success_result<T: Serialize>(result: &T) -> CallToolResult {
    let json_str = serde_json::to_string_pretty(result).unwrap();
    let structured = serde_json::to_value(result).ok();

    CallToolResult {
        content: vec![text_content(json_str)],
        is_error: Some(false),
        structured_content: structured,
        meta: None,
    }
}

/// Tool handler for geoip_lookup
pub struct GeoIpLookupHandler {
    pub geoip: SharedGeoIpReader,
}

#[async_trait]
impl ToolHandler for GeoIpLookupHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let args = serde_json::to_value(arguments).unwrap_or_default();
        Ok(handle_geoip_lookup(&self.geoip, args))
    }
}

/// Tool handler for geoip_bulk_lookup
pub struct GeoIpBulkLookupHandler {
    pub geoip: SharedGeoIpReader,
}

#[async_trait]
impl ToolHandler for GeoIpBulkLookupHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let args = serde_json::to_value(arguments).unwrap_or_default();
        Ok(handle_geoip_bulk_lookup(&self.geoip, args))
    }
}

/// Tool handler for geoip_lookup_self
pub struct GeoIpLookupSelfHandler {
    pub geoip: SharedGeoIpReader,
    pub caller_ip: Option<String>,
}

#[async_trait]
impl ToolHandler for GeoIpLookupSelfHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let args = serde_json::to_value(arguments).unwrap_or_default();
        Ok(handle_geoip_lookup_self(
            &self.geoip,
            self.caller_ip.as_deref(),
            args,
        ))
    }
}

/// Tool handler for timezone_lookup
pub struct TimezoneLookupHandler;

#[async_trait]
impl ToolHandler for TimezoneLookupHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let args = serde_json::to_value(arguments).unwrap_or_default();
        Ok(handle_timezone_lookup(args))
    }
}

/// Handle geoip_lookup tool call
pub fn handle_geoip_lookup(geoip: &SharedGeoIpReader, args: Value) -> CallToolResult {
    // Parse input
    let input: GeoIpLookupInput = match serde_json::from_value(args) {
        Ok(i) => i,
        Err(e) => {
            return error_result(McpErrorCode::InvalidIp, &format!("Invalid input: {}", e));
        }
    };

    // Validate IP
    let ip = match validate_ip(&input.ip) {
        Ok(ip) => ip,
        Err((code, msg)) => return error_result(code, &msg),
    };

    // Check for private IP
    if is_private_ip(&ip) {
        return error_result(
            McpErrorCode::PrivateIp,
            &format!("Private/loopback IP address not supported: {}", input.ip),
        );
    }

    // Lookup in MaxMind database
    let geo_result = geoip.lookup(&input.ip);

    match geo_result {
        Ok(geo_data) => {
            if input.format == "simple" {
                let response = build_simple_response(&geo_data);
                success_result(&response)
            } else {
                let response = build_full_response(&input.ip, &geo_data);
                success_result(&response)
            }
        }
        Err(GeoIpError::NotFound) => error_result(
            McpErrorCode::NotFound,
            &format!("IP address not found in database: {}", input.ip),
        ),
        Err(e) => error_result(McpErrorCode::InvalidIp, &format!("Lookup error: {}", e)),
    }
}

/// Handle geoip_bulk_lookup tool call
pub fn handle_geoip_bulk_lookup(geoip: &SharedGeoIpReader, args: Value) -> CallToolResult {
    // Parse input
    let input: GeoIpBulkLookupInput = match serde_json::from_value(args) {
        Ok(i) => i,
        Err(e) => {
            return error_result(McpErrorCode::InvalidIp, &format!("Invalid input: {}", e));
        }
    };

    // Check bulk limit
    if input.ips.len() > BULK_LOOKUP_MAX_IPS {
        return error_result(
            McpErrorCode::BulkLimitExceeded,
            &format!(
                "Bulk lookup limit exceeded: {} IPs provided, maximum is {}",
                input.ips.len(),
                BULK_LOOKUP_MAX_IPS
            ),
        );
    }

    let mut results = Vec::new();
    let mut errors = Vec::new();

    for ip_str in &input.ips {
        // Validate IP
        match validate_ip(ip_str) {
            Ok(ip) => {
                if is_private_ip(&ip) {
                    errors.push(BulkLookupError {
                        ip: ip_str.clone(),
                        code: McpErrorCode::PrivateIp.as_str().to_string(),
                        message: "Private/loopback IP address not supported".to_string(),
                    });
                    continue;
                }

                match geoip.lookup(ip_str) {
                    Ok(geo_data) => {
                        let response = build_full_response(ip_str, &geo_data);
                        results.push(response);
                    }
                    Err(GeoIpError::NotFound) => {
                        errors.push(BulkLookupError {
                            ip: ip_str.clone(),
                            code: McpErrorCode::NotFound.as_str().to_string(),
                            message: "IP address not found in database".to_string(),
                        });
                    }
                    Err(e) => {
                        errors.push(BulkLookupError {
                            ip: ip_str.clone(),
                            code: McpErrorCode::InvalidIp.as_str().to_string(),
                            message: format!("Lookup error: {}", e),
                        });
                    }
                }
            }
            Err((code, msg)) => {
                errors.push(BulkLookupError {
                    ip: ip_str.clone(),
                    code: code.as_str().to_string(),
                    message: msg,
                });
            }
        }
    }

    let bulk_result = BulkLookupResult { results, errors };
    success_result(&bulk_result)
}

/// Handle geoip_lookup_self tool call
/// Note: This only works with HTTP/SSE transport. Returns error for STDIO.
pub fn handle_geoip_lookup_self(
    geoip: &SharedGeoIpReader,
    caller_ip: Option<&str>,
    args: Value,
) -> CallToolResult {
    // Parse input
    let input: GeoIpLookupSelfInput = match serde_json::from_value(args) {
        Ok(i) => i,
        Err(e) => {
            return error_result(McpErrorCode::InvalidIp, &format!("Invalid input: {}", e));
        }
    };

    // Check if we have a caller IP (only available via HTTP/SSE transport)
    let ip_str = match caller_ip {
        Some(ip) => ip.to_string(),
        None => {
            return error_result(
                McpErrorCode::StdioNoCallerIp,
                "geoip_lookup_self is not available over STDIO transport. \
                 Use geoip_lookup with an explicit IP address instead, \
                 or use SSE transport which provides caller IP information.",
            );
        }
    };

    // Validate IP
    let ip = match validate_ip(&ip_str) {
        Ok(ip) => ip,
        Err((code, msg)) => return error_result(code, &msg),
    };

    // Check for private IP
    if is_private_ip(&ip) {
        return error_result(
            McpErrorCode::PrivateIp,
            &format!("Private/loopback IP address not supported: {}", ip_str),
        );
    }

    // Lookup in MaxMind database
    let geo_result = geoip.lookup(&ip_str);

    match geo_result {
        Ok(geo_data) => {
            if input.format == "simple" {
                let response = build_simple_response(&geo_data);
                success_result(&response)
            } else {
                let response = build_full_response(&ip_str, &geo_data);
                success_result(&response)
            }
        }
        Err(GeoIpError::NotFound) => error_result(
            McpErrorCode::NotFound,
            &format!("IP address not found in database: {}", ip_str),
        ),
        Err(e) => error_result(McpErrorCode::InvalidIp, &format!("Lookup error: {}", e)),
    }
}

/// Handle timezone_lookup tool call
pub fn handle_timezone_lookup(args: Value) -> CallToolResult {
    // Parse input
    let input: TimezoneLookupInput = match serde_json::from_value(args) {
        Ok(i) => i,
        Err(e) => {
            return error_result(
                McpErrorCode::InvalidLatitude,
                &format!("Invalid input: {}", e),
            );
        }
    };

    // Validate latitude
    if !(-90.0..=90.0).contains(&input.lat) {
        return error_result(
            McpErrorCode::InvalidLatitude,
            &format!("Latitude must be between -90 and 90, got: {}", input.lat),
        );
    }

    // Validate longitude
    if !(-180.0..=180.0).contains(&input.lon) {
        return error_result(
            McpErrorCode::InvalidLongitude,
            &format!("Longitude must be between -180 and 180, got: {}", input.lon),
        );
    }

    // Look up timezone
    let timezone_name = lookup_timezone(input.lat, input.lon);

    if input.format == "simple" {
        let response = TimezoneResponse {
            timezone: timezone_name.clone().unwrap_or_default(),
        };
        success_result(&response)
    } else {
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
        success_result(&response)
    }
}

/// MCP Tool context holding shared state
pub struct McpToolContext {
    pub geoip: SharedGeoIpReader,
    /// Caller IP address (only set for SSE transport)
    pub caller_ip: Option<String>,
}

impl McpToolContext {
    pub fn new(geoip: SharedGeoIpReader) -> Self {
        Self {
            geoip,
            caller_ip: None,
        }
    }

    pub fn with_caller_ip(mut self, ip: Option<String>) -> Self {
        self.caller_ip = ip;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geoip::mock::MockGeoIpReader;
    use std::sync::Arc;

    fn mock_geoip() -> SharedGeoIpReader {
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
        Arc::new(mock)
    }

    #[test]
    fn test_validate_ip_valid() {
        assert!(validate_ip("8.8.8.8").is_ok());
        assert!(validate_ip("2001:4860:4860::8888").is_ok());
    }

    #[test]
    fn test_validate_ip_invalid() {
        assert!(validate_ip("not-an-ip").is_err());
        assert!(validate_ip("256.1.2.3").is_err());
    }

    #[test]
    fn test_is_private_ip() {
        assert!(is_private_ip(&"127.0.0.1".parse().unwrap()));
        assert!(is_private_ip(&"192.168.1.1".parse().unwrap()));
        assert!(is_private_ip(&"10.0.0.1".parse().unwrap()));
        assert!(is_private_ip(&"172.16.0.1".parse().unwrap()));
        assert!(!is_private_ip(&"8.8.8.8".parse().unwrap()));
    }

    #[test]
    fn test_handle_geoip_lookup_valid() {
        let geoip = mock_geoip();
        let args = serde_json::json!({ "ip": "8.8.8.8" });
        let result = handle_geoip_lookup(&geoip, args);
        assert!(!result.is_error.unwrap_or(true));
    }

    #[test]
    fn test_handle_geoip_lookup_invalid_ip() {
        let geoip = mock_geoip();
        let args = serde_json::json!({ "ip": "not-an-ip" });
        let result = handle_geoip_lookup(&geoip, args);
        assert!(result.is_error.unwrap_or(false));
    }

    #[test]
    fn test_handle_geoip_lookup_private_ip() {
        let geoip = mock_geoip();
        let args = serde_json::json!({ "ip": "127.0.0.1" });
        let result = handle_geoip_lookup(&geoip, args);
        assert!(result.is_error.unwrap_or(false));
    }

    #[test]
    fn test_handle_bulk_lookup_exceeds_limit() {
        let geoip = mock_geoip();
        let ips: Vec<String> = (0..101).map(|i| format!("8.8.8.{}", i % 256)).collect();
        let args = serde_json::json!({ "ips": ips });
        let result = handle_geoip_bulk_lookup(&geoip, args);
        assert!(result.is_error.unwrap_or(false));
    }

    #[test]
    fn test_handle_geoip_lookup_self_no_caller_ip() {
        let geoip = mock_geoip();
        let args = serde_json::json!({});
        let result = handle_geoip_lookup_self(&geoip, None, args);
        assert!(result.is_error.unwrap_or(false));
    }

    #[test]
    fn test_handle_timezone_lookup_valid() {
        let args = serde_json::json!({ "lat": 59.329504, "lon": 18.069532 });
        let result = handle_timezone_lookup(args);
        assert!(!result.is_error.unwrap_or(true));
    }

    #[test]
    fn test_handle_timezone_lookup_invalid_lat() {
        let args = serde_json::json!({ "lat": 91.0, "lon": 0.0 });
        let result = handle_timezone_lookup(args);
        assert!(result.is_error.unwrap_or(false));
    }

    #[test]
    fn test_handle_timezone_lookup_invalid_lon() {
        let args = serde_json::json!({ "lat": 0.0, "lon": 181.0 });
        let result = handle_timezone_lookup(args);
        assert!(result.is_error.unwrap_or(false));
    }
}
