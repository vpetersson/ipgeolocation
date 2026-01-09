//! Timezone utilities for calculating offsets, DST, and current time

use chrono::{DateTime, Offset, TimeZone, Utc};
use chrono_tz::Tz;

/// Timezone details including offset and DST information
#[derive(Debug, Clone)]
pub struct TimezoneDetails {
    pub name: String,
    pub offset_hours: i32,
    pub offset_with_dst_hours: i32,
    pub current_time: String,
    pub current_time_unix: f64,
    pub is_dst: bool,
    pub dst_exists: bool,
    pub dst_savings_hours: i32,
}

/// Get detailed timezone information for a given IANA timezone name
#[must_use]
pub fn get_timezone_details(tz_name: &str) -> Option<TimezoneDetails> {
    let tz: Tz = tz_name.parse().ok()?;
    let now_utc = Utc::now();
    let now_local = now_utc.with_timezone(&tz);

    // Get the current offset
    let offset = now_local.offset();
    let offset_secs = offset.fix().local_minus_utc();
    let offset_hours = offset_secs / 3600;

    // Check if DST exists for this timezone by comparing winter and summer
    let (is_dst, dst_exists, dst_savings) = check_dst(&tz, &now_utc);

    // Calculate offset with DST (current offset already includes DST if active)
    let offset_with_dst_hours = offset_hours;

    // Format current time
    let current_time = now_local.format("%Y-%m-%d %H:%M:%S%.3f%z").to_string();
    let current_time_unix =
        now_utc.timestamp() as f64 + (now_utc.timestamp_subsec_millis() as f64 / 1000.0);

    Some(TimezoneDetails {
        name: tz_name.to_string(),
        offset_hours,
        offset_with_dst_hours,
        current_time,
        current_time_unix,
        is_dst,
        dst_exists,
        dst_savings_hours: dst_savings,
    })
}

/// Check if DST exists and is currently active for a timezone
fn check_dst(tz: &Tz, now_utc: &DateTime<Utc>) -> (bool, bool, i32) {
    // Check January and July to determine if DST exists
    let year = now_utc
        .format("%Y")
        .to_string()
        .parse::<i32>()
        .unwrap_or(2024);

    let jan = tz.with_ymd_and_hms(year, 1, 15, 12, 0, 0).single();
    let jul = tz.with_ymd_and_hms(year, 7, 15, 12, 0, 0).single();

    match (jan, jul) {
        (Some(jan_dt), Some(jul_dt)) => {
            let jan_offset = jan_dt.offset().fix().local_minus_utc();
            let jul_offset = jul_dt.offset().fix().local_minus_utc();

            let dst_exists = jan_offset != jul_offset;
            let dst_savings = (jul_offset - jan_offset).abs() / 3600;

            // Determine if currently in DST
            let current_offset = now_utc.with_timezone(tz).offset().fix().local_minus_utc();
            let is_dst = dst_exists && current_offset == jul_offset.max(jan_offset);

            (is_dst, dst_exists, dst_savings)
        }
        _ => (false, false, 0),
    }
}

/// Get just the current offset in hours for a timezone
#[must_use]
pub fn get_timezone_offset(tz_name: &str) -> Option<i32> {
    let tz: Tz = tz_name.parse().ok()?;
    let now_utc = Utc::now();
    let now_local = now_utc.with_timezone(&tz);
    let offset_secs = now_local.offset().fix().local_minus_utc();
    Some(offset_secs / 3600)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_timezone_details_new_york() {
        let details = get_timezone_details("America/New_York").unwrap();
        assert_eq!(details.name, "America/New_York");
        // Offset should be -5 or -4 depending on DST
        assert!(details.offset_hours >= -5 && details.offset_hours <= -4);
        assert!(details.dst_exists);
    }

    #[test]
    fn test_get_timezone_details_london() {
        let details = get_timezone_details("Europe/London").unwrap();
        assert_eq!(details.name, "Europe/London");
        // Offset should be 0 or 1 depending on DST
        assert!(details.offset_hours >= 0 && details.offset_hours <= 1);
        assert!(details.dst_exists);
    }

    #[test]
    fn test_get_timezone_details_tokyo() {
        let details = get_timezone_details("Asia/Tokyo").unwrap();
        assert_eq!(details.name, "Asia/Tokyo");
        assert_eq!(details.offset_hours, 9);
        // Japan doesn't use DST
        assert!(!details.dst_exists);
        assert!(!details.is_dst);
    }

    #[test]
    fn test_get_timezone_details_invalid() {
        assert!(get_timezone_details("Invalid/Timezone").is_none());
    }

    #[test]
    fn test_get_timezone_offset() {
        let offset = get_timezone_offset("Asia/Tokyo").unwrap();
        assert_eq!(offset, 9);
    }

    #[test]
    fn test_get_timezone_offset_utc() {
        let offset = get_timezone_offset("UTC").unwrap();
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_get_timezone_offset_invalid() {
        assert!(get_timezone_offset("Not/A/Timezone").is_none());
    }

    #[test]
    fn test_timezone_details_has_current_time() {
        let details = get_timezone_details("Europe/Stockholm").unwrap();
        assert!(!details.current_time.is_empty());
        assert!(details.current_time_unix > 0.0);
    }
}
