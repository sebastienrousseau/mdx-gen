//! Validation primitives used by [`MarkdownOptions::validate`].
//!
//! Inlined from the formerly-vendored `euxis-commons` crate so
//! `mdx-gen` has no path-only workspace dependencies at publish time.
//! The surface is trimmed to what the crate actually uses:
//! [`ValidationError`](crate::validation::ValidationError),
//! [`Validator`](crate::validation::Validator), and a
//! [`ValidationResult`](crate::validation::ValidationResult) alias.
//!
//! [`MarkdownOptions::validate`]: crate::MarkdownOptions::validate

/// Validation error types produced by [`Validator`] checks.
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
                write!(
                    f,
                    "Value too short: minimum {min}, got {actual}"
                )
            }
            Self::TooLong { max, actual } => {
                write!(f, "Value too long: maximum {max}, got {actual}")
            }
            Self::BelowMin { min, actual } => {
                write!(
                    f,
                    "Value below minimum: min {min}, got {actual}"
                )
            }
            Self::AboveMax { max, actual } => {
                write!(
                    f,
                    "Value above maximum: max {max}, got {actual}"
                )
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

/// Builder for composing multiple validations into a single pass.
///
/// Each [`check`](Self::check) captures a `(field, ValidationError)`
/// pair on failure without short-circuiting, so callers receive every
/// problem at once.
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

    /// Run a validation closure and record any error under `field`.
    pub fn check<F>(&mut self, field: &str, validation: F) -> &mut Self
    where
        F: FnOnce() -> Result<(), ValidationError>,
    {
        if let Err(e) = validation() {
            self.errors.push((field.to_string(), e));
        }
        self
    }

    /// Returns `true` if no check has failed so far.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns a slice of all accumulated errors.
    #[must_use]
    pub fn errors(&self) -> &[(String, ValidationError)] {
        &self.errors
    }

    /// Consume the validator and return `Ok(())` on success, or the
    /// full list of `(field, error)` pairs otherwise.
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
    fn validation_error_display_covers_every_variant() {
        assert_eq!(
            ValidationError::Empty.to_string(),
            "Value cannot be empty"
        );
        assert_eq!(
            ValidationError::TooShort { min: 3, actual: 1 }.to_string(),
            "Value too short: minimum 3, got 1"
        );
        assert_eq!(
            ValidationError::TooLong { max: 5, actual: 9 }.to_string(),
            "Value too long: maximum 5, got 9"
        );
        assert_eq!(
            ValidationError::BelowMin {
                min: "1".into(),
                actual: "0".into()
            }
            .to_string(),
            "Value below minimum: min 1, got 0"
        );
        assert_eq!(
            ValidationError::AboveMax {
                max: "10".into(),
                actual: "11".into()
            }
            .to_string(),
            "Value above maximum: max 10, got 11"
        );
        assert_eq!(
            ValidationError::InvalidPattern {
                pattern: "email".into()
            }
            .to_string(),
            "Value doesn't match pattern: email"
        );
        assert!(ValidationError::NotInSet {
            allowed: vec!["a".into(), "b".into()]
        }
        .to_string()
        .starts_with("Value not in allowed set:"));
        assert_eq!(
            ValidationError::Custom("x".into()).to_string(),
            "x"
        );
    }

    #[test]
    fn validation_error_implements_std_error() {
        let err = ValidationError::Empty;
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn validator_accumulates_every_failure() {
        let mut v = Validator::new();
        v.check("name", || Err(ValidationError::Empty));
        v.check("pattern", || {
            Err(ValidationError::InvalidPattern {
                pattern: "email".into(),
            })
        });
        assert!(!v.is_valid());
        assert_eq!(v.errors().len(), 2);
        let errs = v.finish().unwrap_err();
        assert_eq!(errs.len(), 2);
        assert_eq!(errs[0].0, "name");
        assert_eq!(errs[1].0, "pattern");
    }

    #[test]
    fn validator_finish_ok_when_no_checks_failed() {
        let v = Validator::new();
        assert!(v.finish().is_ok());
    }

    #[test]
    fn validator_check_skips_recording_on_ok() {
        let mut v = Validator::new();
        v.check("field", || Ok(()));
        assert!(v.is_valid());
        assert!(v.errors().is_empty());
    }
}
