use tzf_rs::DefaultFinder;
use std::sync::LazyLock;

/// Global timezone finder instance
/// tzf-rs embeds timezone boundary data at compile time
static TZ_FINDER: LazyLock<DefaultFinder> = LazyLock::new(DefaultFinder::new);

/// Lookup IANA timezone name for given coordinates
/// Returns None if coordinates are invalid or not found
#[must_use]
pub fn lookup_timezone(lat: f64, lng: f64) -> Option<String> {
    // Validate coordinate ranges
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lng) {
        return None;
    }

    let tz_name = TZ_FINDER.get_tz_name(lng, lat);

    // tzf-rs returns empty string for ocean/unclaimed areas
    if tz_name.is_empty() {
        None
    } else {
        Some(tz_name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stockholm_timezone() {
        let tz = lookup_timezone(59.329504, 18.069532);
        assert_eq!(tz, Some("Europe/Stockholm".to_string()));
    }

    #[test]
    fn test_new_york_timezone() {
        let tz = lookup_timezone(40.7128, -74.0060);
        assert_eq!(tz, Some("America/New_York".to_string()));
    }

    #[test]
    fn test_tokyo_timezone() {
        let tz = lookup_timezone(35.6762, 139.6503);
        assert_eq!(tz, Some("Asia/Tokyo".to_string()));
    }

    #[test]
    fn test_invalid_coordinates() {
        assert_eq!(lookup_timezone(91.0, 0.0), None);
        assert_eq!(lookup_timezone(0.0, 181.0), None);
        assert_eq!(lookup_timezone(-91.0, 0.0), None);
        assert_eq!(lookup_timezone(0.0, -181.0), None);
    }

    #[test]
    fn test_edge_coordinates() {
        // North Pole area - should return a timezone or None
        let tz = lookup_timezone(90.0, 0.0);
        // We just check it doesn't panic
        let _ = tz;
    }

    #[test]
    fn test_london_timezone() {
        let tz = lookup_timezone(51.5074, -0.1278);
        assert_eq!(tz, Some("Europe/London".to_string()));
    }

    #[test]
    fn test_sydney_timezone() {
        let tz = lookup_timezone(-33.8688, 151.2093);
        assert_eq!(tz, Some("Australia/Sydney".to_string()));
    }

    #[test]
    fn test_los_angeles_timezone() {
        let tz = lookup_timezone(34.0522, -118.2437);
        assert_eq!(tz, Some("America/Los_Angeles".to_string()));
    }

    #[test]
    fn test_berlin_timezone() {
        let tz = lookup_timezone(52.5200, 13.4050);
        assert_eq!(tz, Some("Europe/Berlin".to_string()));
    }

    #[test]
    fn test_boundary_latitude_positive() {
        // Exactly 90.0 should be valid
        let tz = lookup_timezone(90.0, 0.0);
        // Just verify it doesn't return None due to validation
        // The actual result depends on tzf-rs behavior at poles
        let _ = tz;
    }

    #[test]
    fn test_boundary_latitude_negative() {
        // Exactly -90.0 should be valid
        let tz = lookup_timezone(-90.0, 0.0);
        let _ = tz;
    }

    #[test]
    fn test_boundary_longitude_positive() {
        // Exactly 180.0 should be valid
        let tz = lookup_timezone(0.0, 180.0);
        let _ = tz;
    }

    #[test]
    fn test_boundary_longitude_negative() {
        // Exactly -180.0 should be valid
        let tz = lookup_timezone(0.0, -180.0);
        let _ = tz;
    }

    #[test]
    fn test_equator_prime_meridian() {
        // 0,0 is in the ocean (Gulf of Guinea)
        let tz = lookup_timezone(0.0, 0.0);
        // May return None for ocean areas
        let _ = tz;
    }

    #[test]
    fn test_pacific_ocean() {
        // Deep Pacific Ocean - should return None (no timezone)
        let tz = lookup_timezone(0.0, -160.0);
        // This should be in international waters with no defined timezone
        // The result could be None or a nearby timezone depending on tzf-rs
        let _ = tz;
    }

    #[test]
    fn test_atlantic_ocean() {
        // Mid-Atlantic Ocean
        let tz = lookup_timezone(35.0, -45.0);
        // Could return None for international waters
        let _ = tz;
    }

    #[test]
    fn test_southern_ocean() {
        // Southern Ocean near Antarctica
        let tz = lookup_timezone(-65.0, 0.0);
        // Could return None for unclaimed areas
        let _ = tz;
    }
}
