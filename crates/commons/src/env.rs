//! Environment variable utilities.
//!
//! Provides typed access to environment variables with defaults and validation.
//!
//! # Example
//!
//! ```rust
//! use commons::env::{get_env, get_env_or, require_env};
//!
//! // Get optional env var
//! let port: Option<u16> = get_env("PORT");
//!
//! // Get with default
//! let host: String = get_env_or("HOST", "localhost".to_string());
//!
//! // Require env var (panics if missing)
//! // let api_key: String = require_env("API_KEY");
//! ```

use std::env;
use std::str::FromStr;

/// Error type for environment variable operations.
///
/// The `expected` field in [`ParseError`](EnvError::ParseError) is populated
/// via [`std::any::type_name`], which is not guaranteed to be stable across
/// compiler versions. It is intended for human-readable diagnostics only —
/// do not match on its string value programmatically.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum EnvError {
    /// Variable is not set.
    NotSet(String),
    /// Variable value cannot be parsed.
    ParseError {
        var: String,
        value: String,
        /// Human-readable type name (from `std::any::type_name`).
        /// Not stable across compiler versions — for display only.
        expected: String,
    },
    /// Variable value is empty.
    Empty(String),
}

impl std::fmt::Display for EnvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotSet(var) => write!(f, "Environment variable not set: {var}"),
            Self::ParseError {
                var,
                value,
                expected,
            } => {
                write!(f, "Cannot parse {var}={value} as {expected}")
            }
            Self::Empty(var) => write!(f, "Environment variable is empty: {var}"),
        }
    }
}

impl std::error::Error for EnvError {}

/// Get an environment variable, parsed to the specified type.
///
/// Returns `None` if the variable is not set or cannot be parsed.
///
/// # Example
///
/// ```rust
/// use commons::env::get_env;
///
/// let port: Option<u16> = get_env("PORT");
/// let debug: Option<bool> = get_env("DEBUG");
/// ```
#[must_use]
pub fn get_env<T>(key: &str) -> Option<T>
where
    T: FromStr,
{
    env::var(key).ok().and_then(|v| v.parse().ok())
}

/// Get an environment variable or return a default value.
///
/// # Example
///
/// ```rust
/// use commons::env::get_env_or;
///
/// let port: u16 = get_env_or("PORT", 8080);
/// let host: String = get_env_or("HOST", "localhost".to_string());
/// ```
#[must_use]
pub fn get_env_or<T>(key: &str, default: T) -> T
where
    T: FromStr,
{
    get_env(key).unwrap_or(default)
}

/// Get an environment variable, returning an error if not set or invalid.
///
/// # Errors
///
/// Returns an error if the variable is not set, empty, or cannot be parsed.
///
/// # Example
///
/// ```rust
/// use commons::env::try_get_env;
///
/// let port: Result<u16, _> = try_get_env("PORT");
/// ```
pub fn try_get_env<T>(key: &str) -> Result<T, EnvError>
where
    T: FromStr,
{
    let value = env::var(key).map_err(|_| EnvError::NotSet(key.to_string()))?;

    if value.is_empty() {
        return Err(EnvError::Empty(key.to_string()));
    }

    value.parse().map_err(|_| EnvError::ParseError {
        var: key.to_string(),
        value,
        expected: std::any::type_name::<T>().to_string(),
    })
}

/// Require an environment variable, panicking if not set.
///
/// # Panics
///
/// Panics if the variable is not set or cannot be parsed.
///
/// # Example
///
/// ```rust,no_run
/// use commons::env::require_env;
///
/// let api_key: String = require_env("API_KEY");
/// ```
#[must_use]
pub fn require_env<T>(key: &str) -> T
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    env::var(key)
        .unwrap_or_else(|_| panic!("Required environment variable not set: {key}"))
        .parse()
        .unwrap_or_else(|e| panic!("Cannot parse environment variable {key}: {e:?}"))
}

/// Get an environment variable as a string.
#[must_use]
pub fn get_string(key: &str) -> Option<String> {
    env::var(key).ok().filter(|s| !s.is_empty())
}

/// Get an environment variable as a boolean.
///
/// Recognizes: "true", "1", "yes", "on" as true (case-insensitive).
/// Everything else is false.
#[must_use]
pub fn get_bool(key: &str) -> bool {
    env::var(key).is_ok_and(|v| {
        v == "1"
            || v.eq_ignore_ascii_case("true")
            || v.eq_ignore_ascii_case("yes")
            || v.eq_ignore_ascii_case("on")
    })
}

/// Get an environment variable as a list, split by a delimiter.
///
/// # Example
///
/// ```rust
/// use commons::env::get_list;
///
/// // If FEATURES="a,b,c"
/// // let features: Vec<String> = get_list("FEATURES", ",");
/// // features == ["a", "b", "c"]
/// ```
#[must_use]
pub fn get_list(key: &str, delimiter: &str) -> Vec<String> {
    env::var(key)
        .map(|v| {
            v.split(delimiter)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Check if an environment variable is set (and non-empty).
#[must_use]
pub fn is_set(key: &str) -> bool {
    env::var(key).map(|v| !v.is_empty()).unwrap_or(false)
}

/// Get the current environment name (development, staging, production).
///
/// Checks `ENV`, `ENVIRONMENT`, `RUST_ENV`, `APP_ENV` in order.
#[must_use]
pub fn get_environment() -> String {
    for key in &["ENV", "ENVIRONMENT", "RUST_ENV", "APP_ENV"] {
        if let Some(env) = get_string(key) {
            return env.to_lowercase();
        }
    }
    "development".to_string()
}

/// Check if running in production environment.
#[must_use]
pub fn is_production() -> bool {
    let env = get_environment();
    env == "production" || env == "prod"
}

/// Check if running in development environment.
#[must_use]
pub fn is_development() -> bool {
    let env = get_environment();
    env == "development" || env == "dev" || env.is_empty()
}

/// Check if running in test environment.
#[must_use]
pub fn is_test() -> bool {
    let env = get_environment();
    env == "test" || env == "testing"
}

/// Environment configuration builder.
#[derive(Debug, Default)]
pub struct EnvConfig {
    vars: Vec<(String, Option<String>)>,
}

impl EnvConfig {
    /// Create a new environment configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a required variable.
    #[must_use]
    pub fn require(&mut self, key: &str) -> &mut Self {
        self.vars.push((key.to_string(), None));
        self
    }

    /// Add an optional variable with a default.
    #[must_use]
    pub fn optional(&mut self, key: &str, default: &str) -> &mut Self {
        self.vars.push((key.to_string(), Some(default.to_string())));
        self
    }

    /// Validate all required variables are set.
    ///
    /// Returns a list of missing required variables.
    #[must_use]
    pub fn validate(&self) -> Vec<String> {
        self.vars
            .iter()
            .filter(|(_, default)| default.is_none())
            .filter(|(key, _)| !is_set(key))
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// Check if configuration is valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_env_missing() {
        let value: Option<String> = get_env("NONEXISTENT_VAR_12345");
        assert_eq!(value, None);
    }

    #[test]
    fn test_get_env_or_default() {
        let value: u16 = get_env_or("NONEXISTENT_PORT", 3000);
        assert_eq!(value, 3000);
    }

    #[test]
    fn test_get_bool_missing() {
        assert!(!get_bool("NONEXISTENT_BOOL_VAR"));
    }

    #[test]
    fn test_is_set_missing() {
        assert!(!is_set("NONEXISTENT_VAR_99999"));
    }

    #[test]
    fn test_get_list_missing() {
        let list = get_list("NONEXISTENT_LIST_VAR", ",");
        assert!(list.is_empty());
    }

    #[test]
    fn test_env_config_validation() {
        let mut config = EnvConfig::new();
        let _ = config
            .require("DEFINITELY_NOT_SET_VAR")
            .optional("OPTIONAL_VAR", "default");

        let missing = config.validate();
        assert_eq!(missing, vec!["DEFINITELY_NOT_SET_VAR"]);
        assert!(!config.is_valid());
    }

    #[test]
    fn test_get_environment_default() {
        // Without any ENV vars set, should return "development"
        let env = get_environment();
        assert!(!env.is_empty());
    }

    #[test]
    fn test_try_get_env_missing() {
        let result: Result<String, EnvError> = try_get_env("NONEXISTENT_TRY_VAR");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EnvError::NotSet(_)));
    }
}
