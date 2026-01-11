//! Protobuf types and conversion utilities
//!
//! This module provides protobuf serialization for API responses.

use prost::Message;

// Include the generated protobuf types
pub mod geolocation {
    include!(concat!(env!("OUT_DIR"), "/geolocation.rs"));
}

use crate::models;

/// Convert IpGeoResponse to protobuf
impl From<&models::IpGeoResponse> for geolocation::IpGeoResponse {
    fn from(resp: &models::IpGeoResponse) -> Self {
        Self {
            latitude: resp.latitude,
            longitude: resp.longitude,
            city: resp.city.clone(),
            country_name: resp.country_name.clone(),
            time_zone: Some(geolocation::TimeZoneInfo {
                name: resp.time_zone.name.clone(),
            }),
            languages: resp.languages.clone(),
        }
    }
}

/// Convert IpGeoResponseFull to protobuf
impl From<&models::IpGeoResponseFull> for geolocation::IpGeoResponseFull {
    fn from(resp: &models::IpGeoResponseFull) -> Self {
        Self {
            ip: resp.ip.clone(),
            location: resp.location.as_ref().map(|l| geolocation::LocationInfo {
                continent_code: l.continent_code.clone(),
                continent_name: l.continent_name.clone(),
                country_code2: l.country_code2.clone(),
                country_code3: l.country_code3.clone(),
                country_name: l.country_name.clone(),
                country_name_official: l.country_name_official.clone(),
                country_capital: l.country_capital.clone(),
                state_prov: l.state_prov.clone(),
                state_code: l.state_code.clone(),
                district: l.district.clone(),
                city: l.city.clone(),
                zipcode: l.zipcode.clone(),
                latitude: l.latitude.clone(),
                longitude: l.longitude.clone(),
                is_eu: l.is_eu,
                country_flag: l.country_flag.clone(),
                geoname_id: l.geoname_id.clone(),
                country_emoji: l.country_emoji.clone(),
            }),
            country_metadata: resp.country_metadata.as_ref().map(|m| {
                geolocation::CountryMetadataInfo {
                    calling_code: m.calling_code.clone(),
                    tld: m.tld.clone(),
                    languages: m.languages.clone().unwrap_or_default(),
                }
            }),
            currency: resp.currency.as_ref().map(|c| geolocation::CurrencyInfo {
                code: c.code.clone(),
                name: c.name.clone(),
                symbol: c.symbol.clone(),
            }),
            time_zone: resp
                .time_zone
                .as_ref()
                .map(|t| geolocation::TimeZoneInfoFull {
                    name: t.name.clone(),
                    offset: t.offset,
                    offset_with_dst: t.offset_with_dst,
                    current_time: t.current_time.clone(),
                    current_time_unix: t.current_time_unix,
                    is_dst: t.is_dst,
                    dst_savings: t.dst_savings,
                    dst_exists: t.dst_exists,
                }),
        }
    }
}

/// Convert TimezoneResponse to protobuf
impl From<&models::TimezoneResponse> for geolocation::TimezoneResponse {
    fn from(resp: &models::TimezoneResponse) -> Self {
        Self {
            timezone: resp.timezone.clone(),
        }
    }
}

/// Convert TimezoneResponseFull to protobuf
impl From<&models::TimezoneResponseFull> for geolocation::TimezoneResponseFull {
    fn from(resp: &models::TimezoneResponseFull) -> Self {
        Self {
            timezone: resp.timezone.clone(),
            offset: resp.offset,
            offset_with_dst: resp.offset_with_dst,
            current_time: resp.current_time.clone(),
            current_time_unix: resp.current_time_unix,
            is_dst: resp.is_dst,
            dst_exists: resp.dst_exists,
        }
    }
}

/// Encode a protobuf message to bytes
pub fn encode_proto<T: Message>(msg: &T) -> Vec<u8> {
    msg.encode_to_vec()
}

/// Canonical content type for protobuf responses.
///
/// This is used for all protobuf response bodies. The server also accepts
/// `application/protobuf` in `Accept` headers for compatibility.
pub const PROTOBUF_CONTENT_TYPE: &str = "application/x-protobuf";

/// Check if request accepts protobuf
///
/// Accepts both `application/x-protobuf` (canonical) and `application/protobuf` (legacy).
/// Responses are always sent with `PROTOBUF_CONTENT_TYPE`.
pub fn accepts_protobuf(accept: Option<&str>) -> bool {
    accept
        .map(|a| a.contains("application/x-protobuf") || a.contains("application/protobuf"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{IpGeoResponse, TimeZoneInfo, TimezoneResponse};

    #[test]
    fn test_ipgeo_response_to_proto() {
        let resp = IpGeoResponse {
            latitude: Some(37.751),
            longitude: Some(-97.822),
            city: "Mountain View".to_string(),
            country_name: "United States".to_string(),
            time_zone: TimeZoneInfo {
                name: "America/Chicago".to_string(),
            },
            languages: "en-US,en".to_string(),
        };

        let proto: geolocation::IpGeoResponse = (&resp).into();
        assert_eq!(proto.latitude, Some(37.751));
        assert_eq!(proto.city, "Mountain View");
        assert_eq!(proto.time_zone.unwrap().name, "America/Chicago");
    }

    #[test]
    fn test_timezone_response_to_proto() {
        let resp = TimezoneResponse {
            timezone: "Europe/Stockholm".to_string(),
        };

        let proto: geolocation::TimezoneResponse = (&resp).into();
        assert_eq!(proto.timezone, "Europe/Stockholm");
    }

    #[test]
    fn test_encode_proto() {
        let proto = geolocation::TimezoneResponse {
            timezone: "Europe/Stockholm".to_string(),
        };

        let bytes = encode_proto(&proto);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_accepts_protobuf() {
        assert!(accepts_protobuf(Some("application/x-protobuf")));
        assert!(accepts_protobuf(Some("application/protobuf")));
        assert!(accepts_protobuf(Some(
            "application/json, application/x-protobuf"
        )));
        assert!(!accepts_protobuf(Some("application/json")));
        assert!(!accepts_protobuf(None));
    }
}
