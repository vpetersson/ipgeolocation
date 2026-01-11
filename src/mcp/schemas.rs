//! JSON Schema definitions for MCP tools
//!
//! These schemas define the input and output structures for MCP tool calls.

use serde_json::{json, Value};

/// JSON Schema for geoip_lookup tool input
pub fn geoip_lookup_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "ip": {
                "type": "string",
                "description": "IPv4 or IPv6 address to lookup"
            },
            "format": {
                "type": "string",
                "enum": ["simple", "full"],
                "default": "full",
                "description": "Response format: 'simple' for basic data, 'full' for comprehensive details"
            }
        },
        "required": ["ip"]
    })
}

/// JSON Schema for geoip_bulk_lookup tool input
pub fn geoip_bulk_lookup_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "ips": {
                "type": "array",
                "items": {
                    "type": "string"
                },
                "maxItems": 100,
                "description": "Array of IPv4 or IPv6 addresses to lookup (max 100)"
            },
            "format": {
                "type": "string",
                "enum": ["simple", "full"],
                "default": "full",
                "description": "Response format: 'simple' for basic data, 'full' for comprehensive details"
            }
        },
        "required": ["ips"]
    })
}

/// JSON Schema for geoip_lookup_self tool input
pub fn geoip_lookup_self_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "format": {
                "type": "string",
                "enum": ["simple", "full"],
                "default": "full",
                "description": "Response format: 'simple' for basic data, 'full' for comprehensive details"
            }
        }
    })
}

/// JSON Schema for timezone_lookup tool input
pub fn timezone_lookup_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "lat": {
                "type": "number",
                "minimum": -90,
                "maximum": 90,
                "description": "Latitude coordinate (-90 to 90)"
            },
            "lon": {
                "type": "number",
                "minimum": -180,
                "maximum": 180,
                "description": "Longitude coordinate (-180 to 180)"
            },
            "format": {
                "type": "string",
                "enum": ["simple", "full"],
                "default": "full",
                "description": "Response format: 'simple' for timezone name only, 'full' for comprehensive details"
            }
        },
        "required": ["lat", "lon"]
    })
}

/// JSON Schema for simple IP geolocation response
pub fn ip_geo_response_simple_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "latitude": {
                "type": ["number", "null"],
                "description": "Latitude of the location"
            },
            "longitude": {
                "type": ["number", "null"],
                "description": "Longitude of the location"
            },
            "city": {
                "type": "string",
                "description": "City name (empty if unknown)"
            },
            "country_name": {
                "type": "string",
                "description": "Country name (empty if unknown)"
            },
            "time_zone": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "IANA timezone name"
                    }
                }
            },
            "languages": {
                "type": "string",
                "description": "Comma-separated language codes for the country"
            }
        }
    })
}

/// JSON Schema for full IP geolocation response
pub fn ip_geo_response_full_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "ip": {
                "type": "string",
                "description": "The queried IP address"
            },
            "location": {
                "type": "object",
                "properties": {
                    "continent_code": { "type": "string", "description": "Two-letter continent code" },
                    "continent_name": { "type": "string", "description": "Full continent name" },
                    "country_code2": { "type": "string", "description": "ISO 3166-1 alpha-2 country code" },
                    "country_code3": { "type": "string", "description": "ISO 3166-1 alpha-3 country code" },
                    "country_name": { "type": "string", "description": "Common country name" },
                    "country_name_official": { "type": "string", "description": "Official country name" },
                    "country_capital": { "type": "string", "description": "Capital city" },
                    "state_prov": { "type": "string", "description": "State or province name" },
                    "state_code": { "type": "string", "description": "State code with country prefix" },
                    "district": { "type": "string", "description": "District or subdivision" },
                    "city": { "type": "string", "description": "City name" },
                    "zipcode": { "type": "string", "description": "Postal/ZIP code" },
                    "latitude": { "type": "string", "description": "Latitude as string" },
                    "longitude": { "type": "string", "description": "Longitude as string" },
                    "is_eu": { "type": "boolean", "description": "Whether the country is in the EU" },
                    "country_flag": { "type": "string", "description": "Path to country flag SVG" },
                    "geoname_id": { "type": "string", "description": "GeoNames ID" },
                    "country_emoji": { "type": "string", "description": "Country flag emoji" }
                }
            },
            "country_metadata": {
                "type": "object",
                "properties": {
                    "calling_code": { "type": "string", "description": "International calling code" },
                    "tld": { "type": "string", "description": "Top-level domain" },
                    "languages": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Language codes spoken in the country"
                    }
                }
            },
            "currency": {
                "type": "object",
                "properties": {
                    "code": { "type": "string", "description": "ISO 4217 currency code" },
                    "name": { "type": "string", "description": "Currency name" },
                    "symbol": { "type": "string", "description": "Currency symbol" }
                }
            },
            "time_zone": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "IANA timezone name" },
                    "offset": { "type": "integer", "description": "UTC offset in hours (without DST)" },
                    "offset_with_dst": { "type": "integer", "description": "UTC offset in hours (with DST)" },
                    "current_time": { "type": "string", "description": "Current local time" },
                    "current_time_unix": { "type": "number", "description": "Current time as Unix timestamp" },
                    "is_dst": { "type": "boolean", "description": "Whether DST is active" },
                    "dst_savings": { "type": "integer", "description": "DST offset in hours" },
                    "dst_exists": { "type": "boolean", "description": "Whether DST is observed" }
                }
            }
        }
    })
}

/// JSON Schema for bulk lookup response
pub fn bulk_lookup_response_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "results": {
                "type": "array",
                "items": ip_geo_response_full_schema(),
                "description": "Successful lookup results"
            },
            "errors": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "ip": { "type": "string", "description": "The IP that failed" },
                        "code": { "type": "string", "description": "Error code" },
                        "message": { "type": "string", "description": "Error message" }
                    }
                },
                "description": "Failed lookups"
            }
        }
    })
}

/// JSON Schema for simple timezone response
pub fn timezone_response_simple_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "timezone": {
                "type": "string",
                "description": "IANA timezone name"
            }
        }
    })
}

/// JSON Schema for full timezone response
pub fn timezone_response_full_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "timezone": { "type": "string", "description": "IANA timezone name" },
            "offset": { "type": "integer", "description": "UTC offset in hours (without DST)" },
            "offset_with_dst": { "type": "integer", "description": "UTC offset in hours (with DST)" },
            "current_time": { "type": "string", "description": "Current local time" },
            "current_time_unix": { "type": "number", "description": "Current time as Unix timestamp" },
            "is_dst": { "type": "boolean", "description": "Whether DST is active" },
            "dst_exists": { "type": "boolean", "description": "Whether DST is observed" }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geoip_lookup_input_schema_has_required_fields() {
        let schema = geoip_lookup_input_schema();
        assert!(schema["properties"]["ip"].is_object());
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("ip")));
    }

    #[test]
    fn test_geoip_bulk_lookup_input_schema_has_max_items() {
        let schema = geoip_bulk_lookup_input_schema();
        assert_eq!(schema["properties"]["ips"]["maxItems"], 100);
    }

    #[test]
    fn test_timezone_lookup_input_schema_has_coordinate_bounds() {
        let schema = timezone_lookup_input_schema();
        assert_eq!(schema["properties"]["lat"]["minimum"], -90);
        assert_eq!(schema["properties"]["lat"]["maximum"], 90);
        assert_eq!(schema["properties"]["lon"]["minimum"], -180);
        assert_eq!(schema["properties"]["lon"]["maximum"], 180);
    }
}
