//! Common error types and handling utilities.
//!
//! This module provides a unified error type for use across ecosystem projects,
//! along with convenient Result type aliases.
//!
//! # Example
//!
//! ```rust
//! use commons::error::{CommonError, CommonResult};
//!
//! fn process_data(input: &str) -> CommonResult<String> {
//!     if input.is_empty() {
//!         return Err(CommonError::InvalidInput("Input cannot be empty".into()));
//!     }
//!     Ok(input.to_uppercase())
//! }
//! ```

use thiserror::Error;

/// Common error type for ecosystem projects.
///
/// This enum covers the most common error cases encountered across projects.
/// For project-specific errors, consider wrapping this or creating derived types.
#[derive(Error, Debug)]
pub enum CommonError {
    /// Invalid input provided to a function.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// IO operation failed.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Parse error for various formats.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Resource not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Operation not permitted.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Operation timed out.
    #[error("Timeout: {0}")]
    Timeout(String),

    /// External service error.
    #[error("External error: {0}")]
    External(String),

    /// Generic error with custom message.
    #[error("{0}")]
    Custom(String),
}

/// Result type alias using [`CommonError`].
pub type CommonResult<T> = Result<T, CommonError>;

impl CommonError {
    /// Create a new invalid input error.
    #[must_use]
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Create a new configuration error.
    #[must_use]
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a new parse error.
    #[must_use]
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Parse(msg.into())
    }

    /// Create a new not found error.
    #[must_use]
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create a new custom error.
    #[must_use]
    pub fn custom(msg: impl Into<String>) -> Self {
        Self::Custom(msg.into())
    }

    /// Check if this is an input validation error.
    #[must_use]
    pub const fn is_input_error(&self) -> bool {
        matches!(self, Self::InvalidInput(_) | Self::Parse(_))
    }

    /// Check if this is a recoverable error.
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(self, Self::Timeout(_) | Self::External(_))
    }
}

/// Extension trait for Result types.
pub trait ResultExt<T> {
    /// Convert any error to a `CommonError` with context.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying result is an error.
    fn with_context(self, context: &str) -> CommonResult<T>;
}

impl<T, E: std::error::Error> ResultExt<T> for Result<T, E> {
    fn with_context(self, context: &str) -> CommonResult<T> {
        self.map_err(|e| CommonError::Custom(format!("{context}: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = CommonError::invalid_input("test");
        assert!(err.is_input_error());
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_error_display() {
        let err = CommonError::NotFound("file.txt".into());
        assert_eq!(err.to_string(), "Not found: file.txt");
    }

    #[test]
    fn test_result_ext() {
        let result: Result<(), std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        let common_result = result.with_context("Reading file");
        assert!(common_result.is_err());
    }
}
