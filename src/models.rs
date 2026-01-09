use serde::{Deserialize, Serialize};

/// Query parameters for the /ipgeo endpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpGeoQuery {
    /// API key (accepted but not validated)
    #[serde(default)]
    pub api_key: Option<String>,
    /// IP address to lookup
    pub ip: String,
    /// Fields to include (optional, comma-separated)
    #[serde(default)]
    pub fields: Option<String>,
}

/// Query parameters for the /timezone endpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimezoneQuery {
    /// API key (accepted but not validated)
    #[serde(default)]
    pub api_key: Option<String>,
    /// Latitude coordinate
    pub lat: f64,
    /// Longitude coordinate
    pub long: f64,
}

// ============================================================================
// Full API Response (Extended Format)
// ============================================================================

/// Location information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocationInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continent_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continent_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code3: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_name_official: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_capital: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_prov: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub district: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zipcode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_eu: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_flag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geoname_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_emoji: Option<String>,
}

/// Country metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CountryMetadataInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calling_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tld: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub languages: Option<Vec<String>>,
}

/// Currency information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CurrencyInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

/// Timezone information (full version)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimeZoneInfoFull {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_with_dst: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_time_unix: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_dst: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_savings: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_exists: Option<bool>,
}

/// Full IP Geolocation response (extended format)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IpGeoResponseFull {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<LocationInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_metadata: Option<CountryMetadataInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<CurrencyInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<TimeZoneInfoFull>,
}

// ============================================================================
// Simple API Response (backward compatible with original spec)
// ============================================================================

/// Simple timezone information nested in IpGeoResponse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeZoneInfo {
    pub name: String,
}

/// Simple response for the /ipgeo endpoint (original spec)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpGeoResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    pub city: String,
    pub country_name: String,
    pub time_zone: TimeZoneInfo,
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

/// Response for the /timezone endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimezoneResponse {
    pub timezone: String,
}

/// Extended timezone response with full details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimezoneResponseFull {
    pub timezone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_with_dst: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_time_unix: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_dst: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_exists: Option<bool>,
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
        let query: TimezoneQuery = serde_urlencoded::from_str("apiKey=test&lat=59.329504&long=18.069532").unwrap();
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
