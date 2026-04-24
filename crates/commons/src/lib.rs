#![forbid(unsafe_code)]
//! # Commons
//!
//! Shared Rust utilities and common patterns for the Sebastien Rousseau ecosystem.
//!
//! This crate provides reusable components, traits, and utilities used across
//! multiple Rust projects in the ecosystem.
//!
//! ## Features
//!
//! - `config` - Configuration file loading and management (TOML)
//! - `error` - Common error types and Result aliases
//! - `logging` - Simple structured logging
//! - `time` - Date/time utilities and formatting
//! - `collections` - Extended collection utilities (LRU cache)
//! - `validation` - Input validation utilities
//! - `retry` - Retry logic with backoff strategies
//! - `id` - ID generation (timestamp, random, UUID-like)
//! - `env` - Environment variable helpers
//! - `fs` - Cross-platform filesystem utilities
//!
//! ## Quick Start
//!
//! ```rust
//! use commons::prelude::*;
//!
//! // Use the LRU cache
//! let mut cache = LruCache::new(100);
//! cache.insert("key", "value");
//! ```
//!
//! ## Feature Flags
//!
//! Enable only what you need:
//!
//! ```toml
//! [dependencies]
//! commons = { version = "0.0.2", default-features = false, features = ["error", "time"] }
//! ```

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all)]

// NOTE: The vendored `commons` crate is `publish = false` inside this
// workspace and never published to docs.rs, so the upstream's
// `#![cfg_attr(docsrs, feature(doc_cfg))]` + per-module
// `#[cfg_attr(docsrs, doc(cfg(feature = "…")))]` attributes were
// stripped. They require the nightly `doc_cfg` feature, which trips
// the repo-wide `rust-toolchain.toml` stable pin when the shared docs
// workflow runs `cargo doc` with `RUSTDOCFLAGS="--cfg docsrs"`.

#[cfg(feature = "config")]
pub mod config;

#[cfg(feature = "error")]
pub mod error;

#[cfg(feature = "logging")]
pub mod logging;

#[cfg(feature = "time")]
pub mod time;

#[cfg(feature = "collections")]
pub mod collections;

#[cfg(feature = "validation")]
pub mod validation;

#[cfg(feature = "retry")]
pub mod retry;

#[cfg(feature = "id")]
pub mod id;

#[cfg(feature = "env")]
pub mod env;

#[cfg(feature = "fs")]
pub mod fs;

/// Prelude module for convenient imports.
///
/// Import everything commonly needed:
///
/// ```rust
/// use commons::prelude::*;
/// ```
pub mod prelude {
    #[cfg(feature = "error")]
    pub use crate::error::{CommonError, CommonResult};

    #[cfg(feature = "config")]
    pub use crate::config::{Config, ConfigBuilder, ConfigError};

    #[cfg(feature = "logging")]
    pub use crate::logging::{LogLevel, Logger};

    #[cfg(feature = "time")]
    pub use crate::time::{format_duration, parse_duration, unix_timestamp, unix_timestamp_millis};

    #[cfg(feature = "collections")]
    pub use crate::collections::LruCache;

    #[cfg(feature = "validation")]
    pub use crate::validation::{
        Validator, is_valid_email, is_valid_url, validate_length, validate_range,
    };

    #[cfg(feature = "retry")]
    pub use crate::retry::{BackoffStrategy, RetryConfig, retry};

    #[cfg(feature = "id")]
    pub use crate::id::{IdFormat, IdGenerator, generate_id, generate_prefixed_id};

    #[cfg(feature = "env")]
    pub use crate::env::{get_env, get_env_or, is_development, is_production, require_env};

    #[cfg(feature = "fs")]
    pub use crate::fs::{ensure_dir, from_wsl_path, is_wsl, resolve_path, to_wsl_path};
}

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the crate version.
#[must_use]
pub const fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), "0.0.3");
    }
}
