//! MCP Resource handlers for IP geolocation API
//!
//! This module provides MCP resource implementations that expose
//! API metadata, schemas, and documentation.

use std::collections::HashMap;

use async_trait::async_trait;
use mcp_protocol_sdk::core::error::McpResult;
use mcp_protocol_sdk::core::resource::ResourceHandler;
use mcp_protocol_sdk::protocol::types::{Resource as ResourceInfo, ResourceContents};
use serde_json::json;

use super::schemas;
use super::tools::BULK_LOOKUP_MAX_IPS;

/// Resource URI prefix for geoip resources
pub const RESOURCE_URI_PREFIX: &str = "geoip://";

/// Get the schema resource content
pub fn get_schema_resource() -> ResourceContents {
    let schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "IP Geolocation API Response Schemas",
        "description": "JSON Schema definitions for all response types from the IP Geolocation API",
        "definitions": {
            "IpGeoResponseSimple": schemas::ip_geo_response_simple_schema(),
            "IpGeoResponseFull": schemas::ip_geo_response_full_schema(),
            "TimezoneResponseSimple": schemas::timezone_response_simple_schema(),
            "TimezoneResponseFull": schemas::timezone_response_full_schema(),
            "BulkLookupResult": schemas::bulk_lookup_response_schema()
        }
    });

    ResourceContents::Text {
        uri: format!("{}schema", RESOURCE_URI_PREFIX),
        text: serde_json::to_string_pretty(&schema).unwrap(),
        mime_type: Some("application/json".to_string()),
        meta: None,
    }
}

/// Get the data source resource content
pub fn get_data_source_resource() -> ResourceContents {
    let content = json!({
        "title": "IP Geolocation Data Sources",
        "description": "Information about the data sources used by this API",
        "sources": {
            "ip_geolocation": {
                "name": "MaxMind GeoLite2-City",
                "description": "Free IP geolocation database providing city-level accuracy",
                "license": "CC BY-SA 4.0",
                "attribution": "This product includes GeoLite2 Data created by MaxMind, available from https://www.maxmind.com",
                "update_frequency": "Weekly (typically Tuesday)",
                "coverage": "Global IP address space",
                "accuracy": "City-level for most IPs, country-level guaranteed"
            },
            "timezone_boundaries": {
                "name": "tzf-rs",
                "description": "Timezone boundary lookup library compiled from OpenStreetMap timezone data",
                "license": "MIT",
                "source_data": "Timezone boundaries derived from OpenStreetMap",
                "update_frequency": "Compiled into binary at build time"
            },
            "country_metadata": {
                "name": "Embedded country dataset",
                "description": "Static dataset of country metadata including currencies, languages, and calling codes",
                "fields": [
                    "Country name (common and official)",
                    "ISO 3166-1 alpha-2 and alpha-3 codes",
                    "Continent",
                    "Capital city",
                    "Currency (code, name, symbol)",
                    "Languages",
                    "Calling code",
                    "Top-level domain",
                    "EU membership status",
                    "Flag emoji"
                ]
            }
        }
    });

    ResourceContents::Text {
        uri: format!("{}data-source", RESOURCE_URI_PREFIX),
        text: serde_json::to_string_pretty(&content).unwrap(),
        mime_type: Some("application/json".to_string()),
        meta: None,
    }
}

/// Get the limits resource content
pub fn get_limits_resource() -> ResourceContents {
    let content = json!({
        "title": "API Limits and Constraints",
        "description": "Information about rate limits and operational constraints",
        "limits": {
            "bulk_lookup": {
                "max_ips_per_request": BULK_LOOKUP_MAX_IPS,
                "description": "Maximum number of IP addresses that can be looked up in a single bulk request"
            },
            "cache": {
                "description": "Responses are cached to improve performance",
                "cache_ttl_seconds": 3600,
                "cache_control_header": "public, max-age=1209600",
                "note": "IP geolocation data changes infrequently, so aggressive caching is safe"
            },
            "rate_limiting": {
                "enabled": false,
                "description": "No rate limiting is currently enforced. The API is designed for high-throughput usage."
            },
            "coordinate_ranges": {
                "latitude": {
                    "min": -90,
                    "max": 90
                },
                "longitude": {
                    "min": -180,
                    "max": 180
                }
            }
        },
        "supported_formats": {
            "ip_addresses": ["IPv4", "IPv6"],
            "response_formats": ["simple", "full"],
            "content_types": ["application/json", "application/x-protobuf"]
        }
    });

    ResourceContents::Text {
        uri: format!("{}limits", RESOURCE_URI_PREFIX),
        text: serde_json::to_string_pretty(&content).unwrap(),
        mime_type: Some("application/json".to_string()),
        meta: None,
    }
}

/// Get the privacy resource content
pub fn get_privacy_resource() -> ResourceContents {
    let content = json!({
        "title": "Privacy Information",
        "description": "Information about data handling and privacy practices",
        "privacy_practices": {
            "ip_logging": {
                "enabled": false,
                "description": "IP addresses are NOT logged or stored. All lookups are stateless."
            },
            "pii_retention": {
                "enabled": false,
                "description": "No personally identifiable information is retained. Lookups are processed in memory and results are not persisted."
            },
            "stateless_operation": {
                "description": "Each lookup is independent and stateless. No session tracking or user identification is performed."
            },
            "data_sharing": {
                "third_parties": false,
                "description": "Query data is not shared with any third parties. All processing happens locally using embedded databases."
            },
            "cache_behavior": {
                "description": "Responses may be cached in memory for performance. Cache entries contain only the lookup result, not the requesting client's information."
            }
        },
        "data_sources": {
            "note": "IP geolocation data is sourced from MaxMind GeoLite2. This data maps IP addresses to approximate geographic locations but does not identify individuals."
        },
        "private_ip_handling": {
            "description": "Private IP addresses (RFC 1918, loopback, link-local) are rejected and not processed. These addresses have no meaningful geographic location."
        }
    });

    ResourceContents::Text {
        uri: format!("{}privacy", RESOURCE_URI_PREFIX),
        text: serde_json::to_string_pretty(&content).unwrap(),
        mime_type: Some("application/json".to_string()),
        meta: None,
    }
}

/// List of all available resource infos
pub fn list_resource_infos() -> Vec<ResourceInfo> {
    vec![
        ResourceInfo {
            uri: format!("{}schema", RESOURCE_URI_PREFIX),
            name: "API Response Schemas".to_string(),
            description: Some("JSON Schema definitions for all response types".to_string()),
            mime_type: Some("application/json".to_string()),
            annotations: None,
            size: None,
            title: None,
            meta: None,
        },
        ResourceInfo {
            uri: format!("{}data-source", RESOURCE_URI_PREFIX),
            name: "Data Sources".to_string(),
            description: Some(
                "Information about MaxMind GeoLite2, tzf-rs, and other data sources".to_string(),
            ),
            mime_type: Some("application/json".to_string()),
            annotations: None,
            size: None,
            title: None,
            meta: None,
        },
        ResourceInfo {
            uri: format!("{}limits", RESOURCE_URI_PREFIX),
            name: "API Limits".to_string(),
            description: Some(
                "Bulk lookup cap (100), cache TTL, and operational constraints".to_string(),
            ),
            mime_type: Some("application/json".to_string()),
            annotations: None,
            size: None,
            title: None,
            meta: None,
        },
        ResourceInfo {
            uri: format!("{}privacy", RESOURCE_URI_PREFIX),
            name: "Privacy Information".to_string(),
            description: Some("No IP logging, no PII retention, stateless lookups".to_string()),
            mime_type: Some("application/json".to_string()),
            annotations: None,
            size: None,
            title: None,
            meta: None,
        },
    ]
}

/// Read a resource by URI
pub fn read_resource(uri: &str) -> Option<ResourceContents> {
    match uri {
        "geoip://schema" => Some(get_schema_resource()),
        "geoip://data-source" => Some(get_data_source_resource()),
        "geoip://limits" => Some(get_limits_resource()),
        "geoip://privacy" => Some(get_privacy_resource()),
        _ => None,
    }
}

/// Resource handler for GeoIP resources
pub struct GeoIpResourceHandler;

#[async_trait]
impl ResourceHandler for GeoIpResourceHandler {
    async fn read(
        &self,
        uri: &str,
        _params: &HashMap<String, String>,
    ) -> McpResult<Vec<ResourceContents>> {
        match read_resource(uri) {
            Some(contents) => Ok(vec![contents]),
            None => Err(mcp_protocol_sdk::core::error::McpError::ResourceNotFound(
                format!("Resource not found: {}", uri),
            )),
        }
    }

    async fn list(&self) -> McpResult<Vec<ResourceInfo>> {
        Ok(list_resource_infos())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_resource_infos_returns_four() {
        let resources = list_resource_infos();
        assert_eq!(resources.len(), 4);
    }

    #[test]
    fn test_list_resource_infos_has_correct_uris() {
        let resources = list_resource_infos();
        let uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();
        assert!(uris.contains(&"geoip://schema"));
        assert!(uris.contains(&"geoip://data-source"));
        assert!(uris.contains(&"geoip://limits"));
        assert!(uris.contains(&"geoip://privacy"));
    }

    #[test]
    fn test_read_resource_schema() {
        let content = read_resource("geoip://schema");
        assert!(content.is_some());
    }

    #[test]
    fn test_read_resource_data_source() {
        let content = read_resource("geoip://data-source");
        assert!(content.is_some());
    }

    #[test]
    fn test_read_resource_limits() {
        let content = read_resource("geoip://limits");
        assert!(content.is_some());
    }

    #[test]
    fn test_read_resource_privacy() {
        let content = read_resource("geoip://privacy");
        assert!(content.is_some());
    }

    #[test]
    fn test_read_resource_unknown() {
        let content = read_resource("geoip://unknown");
        assert!(content.is_none());
    }

    #[test]
    fn test_schema_resource_contains_definitions() {
        let content = get_schema_resource();
        // Verify it's valid JSON with definitions
        if let ResourceContents::Text { text, .. } = content {
            let json: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert!(json["definitions"]["IpGeoResponseSimple"].is_object());
            assert!(json["definitions"]["IpGeoResponseFull"].is_object());
        } else {
            panic!("Expected Text content");
        }
    }

    #[test]
    fn test_limits_resource_has_bulk_cap() {
        let content = get_limits_resource();
        if let ResourceContents::Text { text, .. } = content {
            let json: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(
                json["limits"]["bulk_lookup"]["max_ips_per_request"],
                BULK_LOOKUP_MAX_IPS
            );
        } else {
            panic!("Expected Text content");
        }
    }
}
