//! Time handling and duration utilities.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Get the current Unix timestamp in seconds
#[must_use]
pub fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Get the current Unix timestamp in milliseconds
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn unix_timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Format a duration in a human-readable way
#[must_use]
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    if secs >= 86400 {
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        format!("{days}d {hours}h")
    } else if secs >= 3600 {
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        format!("{hours}h {minutes}m")
    } else if secs >= 60 {
        let minutes = secs / 60;
        let seconds = secs % 60;
        format!("{minutes}m {seconds}s")
    } else if secs > 0 {
        format!("{secs}.{millis:03}s")
    } else {
        format!("{millis}ms")
    }
}

/// Parse a duration from a human-readable string.
///
/// Supports single units (`"100ms"`, `"5s"`, `"2m"`, `"1h"`, `"1d"`) as well
/// as compound expressions separated by whitespace (`"1h 30m"`,
/// `"2d 6h 30m 500ms"`). A bare number without a suffix is treated as
/// fractional seconds.
///
/// # Errors
///
/// Returns an error if any chunk cannot be parsed.
pub fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Empty duration string".to_string());
    }

    let chunks: Vec<&str> = s.split_whitespace().collect();

    // Fast path: single chunk (most common case, preserves fractional-second support)
    if chunks.len() == 1 {
        return parse_single_duration(chunks[0]);
    }

    let mut total = Duration::ZERO;
    for chunk in chunks {
        total += parse_single_duration(chunk)?;
    }
    Ok(total)
}

/// Parse a single duration chunk such as `"100ms"` or `"2h"`.
fn parse_single_duration(s: &str) -> Result<Duration, String> {
    if let Some(num_str) = s.strip_suffix("ms") {
        let num: u64 = num_str
            .parse()
            .map_err(|_| format!("Invalid milliseconds value: {num_str}"))?;
        Ok(Duration::from_millis(num))
    } else if let Some(num_str) = s.strip_suffix('s') {
        let num: f64 = num_str
            .parse()
            .map_err(|_| format!("Invalid seconds value: {num_str}"))?;
        Ok(Duration::from_secs_f64(num))
    } else if let Some(num_str) = s.strip_suffix('m') {
        let num: u64 = num_str
            .parse()
            .map_err(|_| format!("Invalid minutes value: {num_str}"))?;
        Ok(Duration::from_secs(num * 60))
    } else if let Some(num_str) = s.strip_suffix('h') {
        let num: u64 = num_str
            .parse()
            .map_err(|_| format!("Invalid hours value: {num_str}"))?;
        Ok(Duration::from_secs(num * 3600))
    } else if let Some(num_str) = s.strip_suffix('d') {
        let num: u64 = num_str
            .parse()
            .map_err(|_| format!("Invalid days value: {num_str}"))?;
        Ok(Duration::from_secs(num * 86400))
    } else {
        // Bare number — treat as fractional seconds
        let num: f64 = s
            .parse()
            .map_err(|_| format!("Unknown duration chunk: {s}"))?;
        Ok(Duration::from_secs_f64(num))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("100ms").unwrap(), Duration::from_millis(100));
        assert_eq!(parse_duration("5s").unwrap(), Duration::from_secs(5));
        assert_eq!(parse_duration("2m").unwrap(), Duration::from_secs(120));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("1d").unwrap(), Duration::from_secs(86400));
    }

    #[test]
    fn test_parse_duration_compound() {
        assert_eq!(
            parse_duration("1h 30m").unwrap(),
            Duration::from_secs(3600 + 1800)
        );
        assert_eq!(
            parse_duration("2d 6h 30m 500ms").unwrap(),
            Duration::from_secs(2 * 86400 + 6 * 3600 + 30 * 60) + Duration::from_millis(500)
        );
        assert_eq!(
            parse_duration("  5s  200ms  ").unwrap(),
            Duration::from_secs(5) + Duration::from_millis(200)
        );
    }

    #[test]
    fn test_parse_duration_errors() {
        assert!(parse_duration("").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("1h abc").is_err());
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration(Duration::from_secs(5)), "5.000s");
        assert_eq!(format_duration(Duration::from_secs(65)), "1m 5s");
        assert_eq!(format_duration(Duration::from_secs(3665)), "1h 1m");
    }
}
