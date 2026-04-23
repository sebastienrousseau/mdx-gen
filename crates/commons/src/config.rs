//! Configuration management utilities.
//!
//! This module provides a flexible configuration system that supports
//! loading from files, environment variables, and defaults.
//!
//! # Example
//!
//! ```rust,no_run
//! use commons::config::Config;
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize)]
//! struct AppConfig {
//!     name: String,
//!     port: u16,
//! }
//!
//! let config = Config::from_file("config.toml").unwrap();
//! let app_config: AppConfig = config.parse().unwrap();
//! ```

use serde::de::DeserializeOwned;
use std::path::Path;

/// Configuration loading and management.
///
/// The TOML content is parsed once on creation and cached internally,
/// so repeated calls to [`get`](Config::get) and [`has_key`](Config::has_key)
/// are cheap lookups rather than full re-parses.
#[derive(Debug, Clone)]
pub struct Config {
    /// Raw TOML content.
    content: String,
    /// Pre-parsed TOML value tree for efficient lookups.
    parsed: toml::Value,
}

impl Config {
    /// Create a new configuration from TOML string content.
    ///
    /// # Arguments
    ///
    /// * `content` - TOML formatted configuration string
    ///
    /// # Example
    ///
    /// ```rust
    /// use commons::config::Config;
    ///
    /// let config = Config::new(r#"
    ///     name = "app"
    ///     port = 8080
    /// "#);
    /// ```
    #[must_use]
    pub fn new(content: &str) -> Self {
        let parsed =
            toml::from_str(content).unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()));
        Self {
            content: content.to_string(),
            parsed,
        }
    }

    /// Load configuration from a TOML file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use commons::config::Config;
    ///
    /// let config = Config::from_file("config.toml").unwrap();
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError::FileRead(format!("{}: {}", path.as_ref().display(), e)))?;
        let parsed =
            toml::from_str(&content).unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()));
        Ok(Self { content, parsed })
    }

    /// Parse the configuration into a typed struct.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be parsed into the target type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use commons::config::Config;
    /// use serde::Deserialize;
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct MyConfig {
    ///     name: String,
    /// }
    ///
    /// let config = Config::new("name = \"test\"");
    /// let parsed: MyConfig = config.parse().unwrap();
    /// assert_eq!(parsed.name, "test");
    /// ```
    pub fn parse<T: DeserializeOwned>(&self) -> Result<T, ConfigError> {
        toml::from_str(&self.content).map_err(|e| ConfigError::Parse(e.to_string()))
    }

    /// Get a value from the configuration by key path.
    ///
    /// Supports nested keys using dot notation: "section.key"
    ///
    /// # Example
    ///
    /// ```rust
    /// use commons::config::Config;
    ///
    /// let config = Config::new(r#"
    ///     [server]
    ///     port = 8080
    /// "#);
    /// let port: Option<i64> = config.get("server.port");
    /// assert_eq!(port, Some(8080));
    /// ```
    #[must_use]
    pub fn get<T: FromTomlValue>(&self, key: &str) -> Option<T> {
        let mut current = &self.parsed;

        for part in key.split('.') {
            current = current.get(part)?;
        }

        T::from_toml_value(current)
    }

    /// Check if a key exists in the configuration.
    #[must_use]
    pub fn has_key(&self, key: &str) -> bool {
        self.get::<toml::Value>(key).is_some()
    }

    /// Get the raw TOML content.
    #[must_use]
    pub fn raw(&self) -> &str {
        &self.content
    }
}

/// Error type for configuration operations.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Failed to read configuration file.
    #[error("Failed to read config file: {0}")]
    FileRead(String),

    /// Failed to parse configuration.
    #[error("Failed to parse config: {0}")]
    Parse(String),

    /// Missing required configuration key.
    #[error("Missing required config key: {0}")]
    MissingKey(String),
}

/// Trait for converting TOML values to Rust types.
pub trait FromTomlValue: Sized {
    /// Convert from a TOML value.
    fn from_toml_value(value: &toml::Value) -> Option<Self>;
}

impl FromTomlValue for String {
    fn from_toml_value(value: &toml::Value) -> Option<Self> {
        value.as_str().map(Self::from)
    }
}

impl FromTomlValue for i64 {
    fn from_toml_value(value: &toml::Value) -> Option<Self> {
        value.as_integer()
    }
}

impl FromTomlValue for f64 {
    fn from_toml_value(value: &toml::Value) -> Option<Self> {
        value.as_float()
    }
}

impl FromTomlValue for bool {
    fn from_toml_value(value: &toml::Value) -> Option<Self> {
        value.as_bool()
    }
}

impl FromTomlValue for toml::Value {
    fn from_toml_value(value: &toml::Value) -> Option<Self> {
        Some(value.clone())
    }
}

impl<T: FromTomlValue> FromTomlValue for Vec<T> {
    fn from_toml_value(value: &toml::Value) -> Option<Self> {
        value
            .as_array()
            .map(|arr| arr.iter().filter_map(T::from_toml_value).collect())
    }
}

/// Builder for creating configurations programmatically.
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    values: toml::map::Map<String, toml::Value>,
}

impl ConfigBuilder {
    /// Create a new configuration builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a string value.
    #[must_use]
    pub fn set_string(mut self, key: &str, value: &str) -> Self {
        self.values
            .insert(key.to_string(), toml::Value::String(value.to_string()));
        self
    }

    /// Set an integer value.
    #[must_use]
    pub fn set_int(mut self, key: &str, value: i64) -> Self {
        self.values
            .insert(key.to_string(), toml::Value::Integer(value));
        self
    }

    /// Set a boolean value.
    #[must_use]
    pub fn set_bool(mut self, key: &str, value: bool) -> Self {
        self.values
            .insert(key.to_string(), toml::Value::Boolean(value));
        self
    }

    /// Build the configuration.
    #[must_use]
    pub fn build(self) -> Config {
        let parsed = toml::Value::Table(self.values);
        let content = toml::to_string_pretty(&parsed).unwrap_or_default();
        Config { content, parsed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        port: u16,
    }

    #[test]
    fn test_parse_config() {
        let config = Config::new(
            r#"
            name = "test"
            port = 8080
        "#,
        );
        let parsed: TestConfig = config.parse().unwrap();
        assert_eq!(parsed.name, "test");
        assert_eq!(parsed.port, 8080);
    }

    #[test]
    fn test_get_nested_key() {
        let config = Config::new(
            r#"
            [server]
            host = "localhost"
            port = 3000
        "#,
        );
        assert_eq!(
            config.get::<String>("server.host"),
            Some("localhost".into())
        );
        assert_eq!(config.get::<i64>("server.port"), Some(3000));
    }

    #[test]
    fn test_get_array() {
        let config = Config::new(
            r#"
            allowed_hosts = ["localhost", "127.0.0.1"]
            ports = [8080, 8443]
        "#,
        );
        assert_eq!(
            config.get::<Vec<String>>("allowed_hosts"),
            Some(vec!["localhost".to_string(), "127.0.0.1".to_string()])
        );
        assert_eq!(config.get::<Vec<i64>>("ports"), Some(vec![8080, 8443]));
        assert_eq!(config.get::<Vec<String>>("missing"), None);
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .set_string("name", "app")
            .set_int("port", 8080)
            .set_bool("debug", true)
            .build();

        assert_eq!(config.get::<String>("name"), Some("app".into()));
        assert_eq!(config.get::<i64>("port"), Some(8080));
        assert_eq!(config.get::<bool>("debug"), Some(true));
    }
}
