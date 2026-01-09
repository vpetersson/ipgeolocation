use maxminddb::{geoip2, Reader};
use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

use crate::models::GeoData;

#[derive(Error, Debug)]
pub enum GeoIpError {
    #[error("Failed to open MaxMind database: {0}")]
    DatabaseOpen(#[from] maxminddb::MaxMindDbError),
    #[error("Invalid IP address: {0}")]
    InvalidIp(#[from] std::net::AddrParseError),
    #[error("IP address not found in database")]
    NotFound,
}

/// Trait for IP geolocation lookup
pub trait GeoIpLookup: Send + Sync {
    /// Lookup geolocation data for an IP address string
    fn lookup(&self, ip_str: &str) -> Result<GeoData, GeoIpError>;
}

/// Wrapper around MaxMind database reader
pub struct GeoIpReader {
    reader: Reader<Vec<u8>>,
}

impl GeoIpReader {
    /// Open a MaxMind database from the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, GeoIpError> {
        let reader = Reader::open_readfile(path)?;
        Ok(Self { reader })
    }

    /// Lookup geolocation data for an IP address
    fn lookup_ip(&self, ip: IpAddr) -> Result<GeoData, GeoIpError> {
        let lookup_result = self.reader.lookup(ip)?;

        // Check if data was found
        if !lookup_result.has_data() {
            return Err(GeoIpError::NotFound);
        }

        // Decode the result as City
        let city: geoip2::City = lookup_result
            .decode()
            .map_err(GeoIpError::DatabaseOpen)?
            .ok_or(GeoIpError::NotFound)?;

        // In maxminddb 0.27+, nested structs are not wrapped in Option
        // but their fields are still Option
        let latitude = city.location.latitude;
        let longitude = city.location.longitude;

        // City name from the names struct
        let city_name = city.city.names.english.map(String::from);

        // Country info
        let country_name = city.country.names.english.map(String::from);
        let country_code = city.country.iso_code.map(String::from);

        // Extract subdivisions (state/province)
        let subdivision = city.subdivisions.first();
        let state_prov = subdivision
            .and_then(|s| s.names.english)
            .map(String::from);
        let state_code = subdivision
            .and_then(|s| s.iso_code)
            .map(String::from);

        // Extract postal code
        let postal_code = city.postal.code.map(String::from);

        // Extract geoname_id from city
        let geoname_id = city.city.geoname_id;

        Ok(GeoData {
            latitude,
            longitude,
            city: city_name,
            country_name,
            country_code,
            state_prov,
            state_code,
            postal_code,
            geoname_id,
        })
    }
}

impl GeoIpLookup for GeoIpReader {
    fn lookup(&self, ip_str: &str) -> Result<GeoData, GeoIpError> {
        let ip: IpAddr = ip_str.parse()?;
        self.lookup_ip(ip)
    }
}

/// Shared GeoIP reader wrapped in Arc for thread-safe access
pub type SharedGeoIpReader = Arc<dyn GeoIpLookup>;

/// Mock GeoIP reader for testing
pub mod mock {
    use super::*;

    #[derive(Default)]
    pub struct MockGeoIpReader {
        pub responses: std::collections::HashMap<String, Result<GeoData, GeoIpError>>,
    }

    impl MockGeoIpReader {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn with_response(mut self, ip: &str, response: Result<GeoData, GeoIpError>) -> Self {
            self.responses.insert(ip.to_string(), response);
            self
        }
    }

    impl GeoIpLookup for MockGeoIpReader {
        fn lookup(&self, ip_str: &str) -> Result<GeoData, GeoIpError> {
            match self.responses.get(ip_str) {
                Some(Ok(data)) => Ok(data.clone()),
                Some(Err(_)) => Err(GeoIpError::NotFound),
                None => Err(GeoIpError::NotFound),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::mock::MockGeoIpReader;

    #[test]
    fn test_invalid_ip() {
        let result = "not-an-ip".parse::<IpAddr>();
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_ipv4_parse() {
        let result = "8.8.8.8".parse::<IpAddr>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "8.8.8.8");
    }

    #[test]
    fn test_valid_ipv6_parse() {
        let result = "2001:4860:4860::8888".parse::<IpAddr>();
        assert!(result.is_ok());
    }

    #[test]
    fn test_geo_data_default() {
        let data = GeoData {
            latitude: Some(37.751),
            longitude: Some(-97.822),
            city: Some("Test City".to_string()),
            country_name: Some("Test Country".to_string()),
            country_code: Some("TC".to_string()),
            state_prov: Some("Test State".to_string()),
            state_code: Some("TS".to_string()),
            postal_code: Some("12345".to_string()),
            geoname_id: Some(123456),
        };
        assert_eq!(data.latitude, Some(37.751));
        assert_eq!(data.longitude, Some(-97.822));
        assert_eq!(data.city, Some("Test City".to_string()));
        assert_eq!(data.country_name, Some("Test Country".to_string()));
        assert_eq!(data.country_code, Some("TC".to_string()));
    }

    #[test]
    fn test_geo_data_none() {
        let data = GeoData {
            latitude: None,
            longitude: None,
            city: None,
            country_name: None,
            country_code: None,
            state_prov: None,
            state_code: None,
            postal_code: None,
            geoname_id: None,
        };
        assert!(data.latitude.is_none());
        assert!(data.city.is_none());
    }

    #[test]
    fn test_geoip_error_display() {
        let err = GeoIpError::NotFound;
        assert_eq!(format!("{}", err), "IP address not found in database");
    }

    #[test]
    fn test_geoip_error_invalid_ip() {
        let parse_err = "invalid".parse::<IpAddr>().unwrap_err();
        let err = GeoIpError::InvalidIp(parse_err);
        assert!(format!("{}", err).contains("Invalid IP address"));
    }

    #[test]
    fn test_mock_geoip_reader_found() {
        let mock = MockGeoIpReader::new()
            .with_response("8.8.8.8", Ok(GeoData {
                latitude: Some(37.751),
                longitude: Some(-97.822),
                city: Some("Mountain View".to_string()),
                country_name: Some("United States".to_string()),
                country_code: Some("US".to_string()),
                state_prov: Some("California".to_string()),
                state_code: Some("CA".to_string()),
                postal_code: Some("94043".to_string()),
                geoname_id: Some(5375480),
            }));

        let result = mock.lookup("8.8.8.8");
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.city, Some("Mountain View".to_string()));
        assert_eq!(data.country_code, Some("US".to_string()));
    }

    #[test]
    fn test_mock_geoip_reader_not_found() {
        let mock = MockGeoIpReader::new();
        let result = mock.lookup("1.2.3.4");
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_geoip_reader_with_error() {
        let mock = MockGeoIpReader::new()
            .with_response("0.0.0.0", Err(GeoIpError::NotFound));

        let result = mock.lookup("0.0.0.0");
        assert!(result.is_err());
    }
}
