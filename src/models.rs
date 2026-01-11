use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Query parameters for the /ipgeo endpoint
#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct IpGeoQuery {
    /// API key (accepted but not validated)
    #[serde(default)]
    pub api_key: Option<String>,
    /// IP address to lookup (IPv4 or IPv6)
    #[param(example = "8.8.8.8")]
    pub ip: String,
    /// Fields to include (optional, use "*" or "location" for full format)
    #[serde(default)]
    pub fields: Option<String>,
}

/// Query parameters for the /timezone endpoint
#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct TimezoneQuery {
    /// API key (accepted but not validated)
    #[serde(default)]
    pub api_key: Option<String>,
    /// Latitude coordinate (-90 to 90)
    #[param(example = 59.329504, minimum = -90, maximum = 90)]
    pub lat: f64,
    /// Longitude coordinate (-180 to 180)
    #[param(example = 18.069532, minimum = -180, maximum = 180)]
    pub long: f64,
}

// ============================================================================
// Full API Response (Extended Format)
// ============================================================================

/// Detailed location information including continent, country, and city data
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[schema(example = json!({
    "continent_code": "NA",
    "continent_name": "North America",
    "country_code2": "US",
    "country_code3": "USA",
    "country_name": "United States",
    "country_name_official": "United States of America",
    "country_capital": "Washington, D.C.",
    "state_prov": "California",
    "state_code": "US-CA",
    "city": "Mountain View",
    "zipcode": "94043",
    "latitude": "37.75100",
    "longitude": "-97.82200",
    "is_eu": false,
    "country_flag": "/static/flags/us.svg",
    "geoname_id": "5375480",
    "country_emoji": "🇺🇸"
}))]
pub struct LocationInfo {
    /// Two-letter continent code (e.g., "NA", "EU")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continent_code: Option<String>,
    /// Full continent name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continent_name: Option<String>,
    /// ISO 3166-1 alpha-2 country code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code2: Option<String>,
    /// ISO 3166-1 alpha-3 country code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code3: Option<String>,
    /// Common country name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_name: Option<String>,
    /// Official country name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_name_official: Option<String>,
    /// Capital city of the country
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_capital: Option<String>,
    /// State or province name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_prov: Option<String>,
    /// State code with country prefix (e.g., "US-CA")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_code: Option<String>,
    /// District or subdivision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub district: Option<String>,
    /// City name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Postal/ZIP code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zipcode: Option<String>,
    /// Latitude as string with 5 decimal places
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<String>,
    /// Longitude as string with 5 decimal places
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<String>,
    /// Whether the country is in the European Union
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_eu: Option<bool>,
    /// Path to country flag SVG (e.g., "/static/flags/us.svg")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_flag: Option<String>,
    /// GeoNames ID for the location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geoname_id: Option<String>,
    /// Country flag emoji
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_emoji: Option<String>,
}

/// Country metadata including calling code, TLD, and languages
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[schema(example = json!({
    "calling_code": "+1",
    "tld": ".us",
    "languages": ["en-US", "es-US"]
}))]
pub struct CountryMetadataInfo {
    /// International calling code (e.g., "+1")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calling_code: Option<String>,
    /// Country top-level domain (e.g., ".us")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tld: Option<String>,
    /// List of language codes spoken in the country
    #[serde(skip_serializing_if = "Option::is_none")]
    pub languages: Option<Vec<String>>,
}

/// Currency information for the country
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[schema(example = json!({
    "code": "USD",
    "name": "US Dollar",
    "symbol": "$"
}))]
pub struct CurrencyInfo {
    /// ISO 4217 currency code (e.g., "USD")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Currency name (e.g., "US Dollar")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Currency symbol (e.g., "$")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

/// Detailed timezone information including DST data
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[schema(example = json!({
    "name": "America/Los_Angeles",
    "offset": -8,
    "offset_with_dst": -7,
    "current_time": "2024-01-15 14:30:00.123-0800",
    "current_time_unix": 1705355400.123,
    "is_dst": false,
    "dst_savings": 1,
    "dst_exists": true
}))]
pub struct TimeZoneInfoFull {
    /// IANA timezone name (e.g., "America/Los_Angeles")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// UTC offset in hours (without DST)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    /// UTC offset in hours (with DST if active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_with_dst: Option<i32>,
    /// Current local time in the timezone
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_time: Option<String>,
    /// Current time as Unix timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_time_unix: Option<f64>,
    /// Whether daylight saving time is currently active
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_dst: Option<bool>,
    /// DST offset in hours (typically 1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_savings: Option<i32>,
    /// Whether DST is observed in this timezone
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_exists: Option<bool>,
}

/// Full IP geolocation response with extended location, currency, and timezone data
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[schema(example = json!({
    "ip": "8.8.8.8",
    "location": {
        "continent_code": "NA",
        "country_code2": "US",
        "country_name": "United States",
        "city": "Mountain View"
    },
    "currency": {
        "code": "USD",
        "name": "US Dollar",
        "symbol": "$"
    },
    "time_zone": {
        "name": "America/Los_Angeles",
        "offset": -8
    }
}))]
pub struct IpGeoResponseFull {
    /// The queried IP address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    /// Detailed location information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<LocationInfo>,
    /// Country metadata (calling code, TLD, languages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_metadata: Option<CountryMetadataInfo>,
    /// Currency information for the country
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<CurrencyInfo>,
    /// Detailed timezone information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<TimeZoneInfoFull>,
}

// ============================================================================
// Simple API Response (backward compatible with original spec)
// ============================================================================

/// Simple timezone information with just the timezone name
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({"name": "America/New_York"}))]
pub struct TimeZoneInfo {
    /// IANA timezone name (e.g., "America/New_York")
    pub name: String,
}

/// Simple IP geolocation response with basic location data
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "latitude": 37.751,
    "longitude": -97.822,
    "city": "Mountain View",
    "country_name": "United States",
    "time_zone": {"name": "America/Chicago"},
    "languages": "en-US,en"
}))]
pub struct IpGeoResponse {
    /// Latitude of the location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    /// Longitude of the location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    /// City name (empty string if unknown)
    pub city: String,
    /// Country name (empty string if unknown)
    pub country_name: String,
    /// Timezone information
    pub time_zone: TimeZoneInfo,
    /// Comma-separated language codes for the country
    pub languages: String,
}

impl Default for IpGeoResponse {
    fn default() -> Self {
        Self {
            latitude: None,
            longitude: None,
            city: String::new(),
            country_name: String::new(),
            time_zone: TimeZoneInfo {
                name: String::new(),
            },
            languages: String::new(),
        }
    }
}

/// Simple timezone response with just the timezone name
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({"timezone": "Europe/Stockholm"}))]
pub struct TimezoneResponse {
    /// IANA timezone name
    pub timezone: String,
}

/// Extended timezone response with offset and DST information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "timezone": "Europe/Stockholm",
    "offset": 1,
    "offset_with_dst": 2,
    "current_time": "2024-01-15 23:30:00.123+0100",
    "current_time_unix": 1705355400.123,
    "is_dst": false,
    "dst_exists": true
}))]
pub struct TimezoneResponseFull {
    /// IANA timezone name
    pub timezone: String,
    /// UTC offset in hours (without DST)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    /// UTC offset in hours (with DST if active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_with_dst: Option<i32>,
    /// Current local time in the timezone
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_time: Option<String>,
    /// Current time as Unix timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_time_unix: Option<f64>,
    /// Whether daylight saving time is currently active
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_dst: Option<bool>,
    /// Whether DST is observed in this timezone
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_exists: Option<bool>,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "error": "Invalid IP address: not-an-ip",
    "code": "INVALID_IP"
}))]
pub struct ApiErrorResponse {
    /// Human-readable error message
    pub error: String,
    /// Machine-readable error code (INVALID_IP, INVALID_LATITUDE, INVALID_LONGITUDE)
    pub code: String,
}

/// Geolocation data extracted from MaxMind database
#[derive(Debug, Clone)]
pub struct GeoData {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub city: Option<String>,
    pub country_name: Option<String>,
    pub country_code: Option<String>,
    pub state_prov: Option<String>,
    pub state_code: Option<String>,
    pub postal_code: Option<String>,
    pub geoname_id: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipgeo_response_default() {
        let response = IpGeoResponse::default();
        assert!(response.latitude.is_none());
        assert!(response.longitude.is_none());
        assert_eq!(response.city, "");
        assert_eq!(response.country_name, "");
        assert_eq!(response.time_zone.name, "");
        assert_eq!(response.languages, "");
    }

    #[test]
    fn test_ipgeo_response_serialize() {
        let response = IpGeoResponse {
            latitude: Some(37.751),
            longitude: Some(-97.822),
            city: "Test City".to_string(),
            country_name: "Test Country".to_string(),
            time_zone: TimeZoneInfo {
                name: "America/Chicago".to_string(),
            },
            languages: "en-US,en".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("37.751"));
        assert!(json.contains("Test City"));
        assert!(json.contains("America/Chicago"));
    }

    #[test]
    fn test_ipgeo_response_deserialize() {
        let json = r#"{"latitude":37.751,"longitude":-97.822,"city":"Test","country_name":"TC","time_zone":{"name":"America/Chicago"},"languages":"en"}"#;
        let response: IpGeoResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.latitude, Some(37.751));
        assert_eq!(response.city, "Test");
    }

    #[test]
    fn test_timezone_response_serialize() {
        let response = TimezoneResponse {
            timezone: "Europe/Stockholm".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Europe/Stockholm"));
    }

    #[test]
    fn test_timezone_query_deserialize() {
        let query: TimezoneQuery =
            serde_urlencoded::from_str("apiKey=test&lat=59.329504&long=18.069532").unwrap();
        assert_eq!(query.api_key, Some("test".to_string()));
        assert!((query.lat - 59.329504).abs() < 0.0001);
        assert!((query.long - 18.069532).abs() < 0.0001);
    }

    #[test]
    fn test_ipgeo_query_deserialize() {
        let query: IpGeoQuery = serde_urlencoded::from_str("apiKey=test&ip=8.8.8.8").unwrap();
        assert_eq!(query.api_key, Some("test".to_string()));
        assert_eq!(query.ip, "8.8.8.8");
    }

    #[test]
    fn test_ipgeo_query_without_apikey() {
        let query: IpGeoQuery = serde_urlencoded::from_str("ip=8.8.8.8").unwrap();
        assert!(query.api_key.is_none());
        assert_eq!(query.ip, "8.8.8.8");
    }

    #[test]
    fn test_timezone_info_clone() {
        let tz = TimeZoneInfo {
            name: "Europe/London".to_string(),
        };
        let cloned = tz.clone();
        assert_eq!(tz.name, cloned.name);
    }

    #[test]
    fn test_geo_data_clone() {
        let data = GeoData {
            latitude: Some(51.5074),
            longitude: Some(-0.1278),
            city: Some("London".to_string()),
            country_name: Some("United Kingdom".to_string()),
            country_code: Some("GB".to_string()),
            state_prov: Some("England".to_string()),
            state_code: Some("ENG".to_string()),
            postal_code: Some("SW1A".to_string()),
            geoname_id: Some(2643743),
        };
        let cloned = data.clone();
        assert_eq!(data.city, cloned.city);
        assert_eq!(data.latitude, cloned.latitude);
    }

    #[test]
    fn test_full_response_serialize() {
        let response = IpGeoResponseFull {
            ip: Some("8.8.8.8".to_string()),
            location: Some(LocationInfo {
                country_code2: Some("US".to_string()),
                country_name: Some("United States".to_string()),
                city: Some("Mountain View".to_string()),
                latitude: Some("37.386".to_string()),
                longitude: Some("-122.084".to_string()),
                ..Default::default()
            }),
            country_metadata: Some(CountryMetadataInfo {
                calling_code: Some("+1".to_string()),
                languages: Some(vec!["en-US".to_string()]),
                ..Default::default()
            }),
            currency: Some(CurrencyInfo {
                code: Some("USD".to_string()),
                name: Some("US Dollar".to_string()),
                symbol: Some("$".to_string()),
            }),
            time_zone: Some(TimeZoneInfoFull {
                name: Some("America/Los_Angeles".to_string()),
                offset: Some(-8),
                ..Default::default()
            }),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("8.8.8.8"));
        assert!(json.contains("Mountain View"));
        assert!(json.contains("USD"));
    }
}
