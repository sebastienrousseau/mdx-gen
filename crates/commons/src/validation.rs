//! Input validation utilities.
//!
//! Provides common validation functions for strings, numbers, and other types.
//!
//! # Example
//!
//! ```rust
//! use commons::validation::{is_valid_email, is_valid_url, validate_length};
//!
//! assert!(is_valid_email("user@example.com"));
//! assert!(is_valid_url("https://example.com"));
//! assert!(validate_length("hello", 1, 10).is_ok());
//! ```

use std::net::IpAddr;

/// Validation error types.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum ValidationError {
    /// Value is empty when it shouldn't be.
    Empty,
    /// Value is too short.
    TooShort { min: usize, actual: usize },
    /// Value is too long.
    TooLong { max: usize, actual: usize },
    /// Value is below minimum.
    BelowMin { min: String, actual: String },
    /// Value is above maximum.
    AboveMax { max: String, actual: String },
    /// Value doesn't match expected pattern.
    InvalidPattern { pattern: String },
    /// Value is not in allowed set.
    NotInSet { allowed: Vec<String> },
    /// Custom validation error.
    Custom(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Value cannot be empty"),
            Self::TooShort { min, actual } => {
                write!(f, "Value too short: minimum {min}, got {actual}")
            }
            Self::TooLong { max, actual } => {
                write!(f, "Value too long: maximum {max}, got {actual}")
            }
            Self::BelowMin { min, actual } => {
                write!(f, "Value below minimum: min {min}, got {actual}")
            }
            Self::AboveMax { max, actual } => {
                write!(f, "Value above maximum: max {max}, got {actual}")
            }
            Self::InvalidPattern { pattern } => {
                write!(f, "Value doesn't match pattern: {pattern}")
            }
            Self::NotInSet { allowed } => {
                write!(f, "Value not in allowed set: {allowed:?}")
            }
            Self::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Result type for validation operations.
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Validate that a string is not empty.
///
/// # Errors
///
/// Returns `ValidationError::Empty` if the trimmed value is empty.
pub fn validate_not_empty(value: &str) -> ValidationResult<&str> {
    if value.trim().is_empty() {
        Err(ValidationError::Empty)
    } else {
        Ok(value)
    }
}

/// Validate string length is within bounds.
///
/// # Errors
///
/// Returns `ValidationError::TooShort` or `ValidationError::TooLong` if out of range.
pub const fn validate_length(value: &str, min: usize, max: usize) -> ValidationResult<&str> {
    let len = value.len();
    if len < min {
        Err(ValidationError::TooShort { min, actual: len })
    } else if len > max {
        Err(ValidationError::TooLong { max, actual: len })
    } else {
        Ok(value)
    }
}

/// Validate that a number is within range.
///
/// # Errors
///
/// Returns `ValidationError::BelowMin` or `ValidationError::AboveMax` if out of range.
pub fn validate_range<T>(value: T, min: T, max: T) -> ValidationResult<T>
where
    T: PartialOrd + std::fmt::Display + Copy,
{
    if value < min {
        Err(ValidationError::BelowMin {
            min: min.to_string(),
            actual: value.to_string(),
        })
    } else if value > max {
        Err(ValidationError::AboveMax {
            max: max.to_string(),
            actual: value.to_string(),
        })
    } else {
        Ok(value)
    }
}

/// Check if a string looks like a valid email address.
///
/// This is a simple check, not RFC 5322 compliant.
#[must_use]
pub fn is_valid_email(email: &str) -> bool {
    let email = email.trim();

    // Must contain exactly one @
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let (local, domain) = (parts[0], parts[1]);

    // Local part checks
    if local.is_empty() || local.len() > 64 {
        return false;
    }

    // Domain checks
    if domain.is_empty() || domain.len() > 255 {
        return false;
    }

    // Domain must contain at least one dot
    if !domain.contains('.') {
        return false;
    }

    // No consecutive dots
    if email.contains("..") {
        return false;
    }

    true
}

/// Check if a string looks like a valid URL.
///
/// Rejects whitespace, bare dots (e.g. `http://.`), and URLs without
/// a meaningful host. For full RFC 3986 compliance, use the
/// [`url`](https://crates.io/crates/url) crate.
#[must_use]
pub fn is_valid_url(url: &str) -> bool {
    let url = url.trim();

    // Must start with http:// or https://
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"));

    let Some(rest) = rest else {
        return false;
    };

    // Must have content after the scheme
    if rest.is_empty() {
        return false;
    }

    // No whitespace allowed
    if rest.contains(char::is_whitespace) {
        return false;
    }

    // Extract the host (before any path, query, or fragment)
    let host = rest.split('/').next().unwrap_or(rest);
    let host = host.split('?').next().unwrap_or(host);
    let host = host.split('#').next().unwrap_or(host);

    // Strip port from host (handle IPv6 bracketed addresses)
    let host_without_port = if host.starts_with('[') {
        host.split(']')
            .next()
            .map_or(host, |h| h.trim_start_matches('['))
    } else {
        host.rsplit_once(':').map_or(host, |(h, _)| h)
    };

    // Allow "localhost" as a valid host
    if host_without_port.eq_ignore_ascii_case("localhost") {
        return true;
    }

    // Host must contain a dot and have content on both sides
    host_without_port
        .find('.')
        .is_some_and(|dot_pos| dot_pos > 0 && dot_pos < host_without_port.len() - 1)
}

/// Check if a string is a valid IP address (v4 or v6).
#[must_use]
pub fn is_valid_ip(ip: &str) -> bool {
    ip.trim().parse::<IpAddr>().is_ok()
}

/// Check if a string is a valid IPv4 address.
#[must_use]
pub fn is_valid_ipv4(ip: &str) -> bool {
    ip.trim().parse::<std::net::Ipv4Addr>().is_ok()
}

/// Check if a string is a valid IPv6 address.
#[must_use]
pub fn is_valid_ipv6(ip: &str) -> bool {
    ip.trim().parse::<std::net::Ipv6Addr>().is_ok()
}

/// Check if a string contains only alphanumeric characters.
#[must_use]
pub fn is_alphanumeric(s: &str) -> bool {
    !s.is_empty() && s.chars().all(char::is_alphanumeric)
}

/// Check if a string contains only ASCII alphanumeric characters and underscores.
#[must_use]
pub fn is_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();

    // First character must be letter or underscore
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }

    // Rest can be alphanumeric or underscore
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Check if a string is a valid semantic version.
///
/// Supports optional `v` prefix, pre-release labels (`-alpha.1`), and
/// build metadata (`+build.42`). The three core version components
/// (major, minor, patch) must be non-negative integers.
#[must_use]
pub fn is_valid_semver(version: &str) -> bool {
    let version = version.trim().strip_prefix('v').unwrap_or(version);

    // Split off pre-release and build metadata before parsing the core
    let core_version = version.split(&['-', '+'][..]).next().unwrap_or(version);

    let parts: Vec<&str> = core_version.split('.').collect();

    if parts.len() != 3 {
        return false;
    }

    parts.iter().all(|part| part.parse::<u64>().is_ok())
}

/// Validate that a value is in an allowed set.
///
/// # Errors
///
/// Returns `ValidationError::NotInSet` if the value is not in the allowed set.
pub fn validate_in_set<T>(value: &T, allowed: &[T]) -> ValidationResult<()>
where
    T: PartialEq + std::fmt::Display,
{
    if allowed.contains(value) {
        Ok(())
    } else {
        Err(ValidationError::NotInSet {
            allowed: allowed.iter().map(ToString::to_string).collect(),
        })
    }
}

/// Builder for composing multiple validations.
#[derive(Debug, Default)]
pub struct Validator {
    errors: Vec<(String, ValidationError)>,
}

impl Validator {
    /// Create a new validator.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a validation check.
    pub fn check<F>(&mut self, field: &str, validation: F) -> &mut Self
    where
        F: FnOnce() -> Result<(), ValidationError>,
    {
        if let Err(e) = validation() {
            self.errors.push((field.to_string(), e));
        }
        self
    }

    /// Check if validation passed.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get all errors.
    #[must_use]
    pub fn errors(&self) -> &[(String, ValidationError)] {
        &self.errors
    }

    /// Finish validation and return result.
    ///
    /// # Errors
    ///
    /// Returns the list of validation errors if any checks failed.
    pub fn finish(self) -> Result<(), Vec<(String, ValidationError)>> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_not_empty() {
        assert!(validate_not_empty("hello").is_ok());
        assert!(validate_not_empty("").is_err());
        assert!(validate_not_empty("   ").is_err());
    }

    #[test]
    fn test_validate_length() {
        assert!(validate_length("hello", 1, 10).is_ok());
        assert!(validate_length("hi", 5, 10).is_err());
        assert!(validate_length("hello world!", 1, 5).is_err());
    }

    #[test]
    fn test_validate_range() {
        assert!(validate_range(5, 1, 10).is_ok());
        assert!(validate_range(0, 1, 10).is_err());
        assert!(validate_range(15, 1, 10).is_err());
    }

    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("user.name@example.co.uk"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email("user@@example.com"));
    }

    #[test]
    fn test_is_valid_url() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://example.com/path"));
        assert!(is_valid_url("https://example.com/path?q=1#frag"));
        assert!(!is_valid_url("example.com"));
        assert!(!is_valid_url("ftp://example.com"));
        assert!(!is_valid_url("https://"));
        assert!(!is_valid_url("http://."));
        assert!(!is_valid_url("https://invalid space.com"));
        assert!(!is_valid_url("http://.com"));
        assert!(!is_valid_url("http://com."));

        // Localhost support
        assert!(is_valid_url("http://localhost"));
        assert!(is_valid_url("http://localhost:8080"));
        assert!(is_valid_url("http://localhost/path"));
        assert!(is_valid_url("http://localhost:8080/path?q=1"));
        assert!(is_valid_url("https://LOCALHOST"));

        // Port stripping on regular domains
        assert!(is_valid_url("http://example.com:8080"));

        // Empty host with port should fail
        assert!(!is_valid_url("http://:8080"));
    }

    #[test]
    fn test_is_valid_ip() {
        assert!(is_valid_ip("192.168.1.1"));
        assert!(is_valid_ip("::1"));
        assert!(is_valid_ip("2001:db8::1"));
        assert!(!is_valid_ip("not an ip"));
        assert!(!is_valid_ip("256.1.1.1"));
    }

    #[test]
    fn test_is_identifier() {
        assert!(is_identifier("hello"));
        assert!(is_identifier("_private"));
        assert!(is_identifier("camelCase"));
        assert!(is_identifier("snake_case"));
        assert!(is_identifier("with123"));
        assert!(!is_identifier("123start"));
        assert!(!is_identifier("has-dash"));
        assert!(!is_identifier(""));
    }

    #[test]
    fn test_is_valid_semver() {
        assert!(is_valid_semver("1.0.0"));
        assert!(is_valid_semver("v1.0.0"));
        assert!(is_valid_semver("0.1.0"));
        assert!(is_valid_semver("1.0.0-alpha"));
        assert!(is_valid_semver("1.0.0-alpha.1"));
        assert!(is_valid_semver("1.0.0-rc.2"));
        assert!(is_valid_semver("1.0.0+build.42"));
        assert!(is_valid_semver("1.0.0-beta+exp.sha.5114f85"));
        assert!(!is_valid_semver("1.0"));
        assert!(!is_valid_semver("1"));
        assert!(!is_valid_semver("a.b.c"));
    }

    #[test]
    fn test_validator() {
        let mut v = Validator::new();
        v.check("email", || {
            if is_valid_email("test@example.com") {
                Ok(())
            } else {
                Err(ValidationError::InvalidPattern {
                    pattern: "email".to_string(),
                })
            }
        });
        assert!(v.is_valid());
        assert!(v.finish().is_ok());
    }
}
