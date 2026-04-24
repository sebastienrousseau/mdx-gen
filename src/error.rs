//! Error handling for the MDX Gen library.

use crate::validation::ValidationError;

/// Represents all the errors that can occur during Markdown processing.
#[derive(thiserror::Error, Debug)]
pub enum MarkdownError {
    /// An error occurred while parsing the Markdown content.
    #[error("Failed to parse Markdown: {0}")]
    ParseError(String),

    /// An error occurred while converting Markdown to HTML.
    #[error("Failed to convert Markdown to HTML: {0}")]
    ConversionError(String),

    /// An error occurred while processing a custom block.
    #[error("Failed to process custom block: {0}")]
    CustomBlockError(String),

    /// An error occurred while applying syntax highlighting.
    #[error("Syntax highlighting error: {0}")]
    SyntaxHighlightError(String),

    /// An error occurred due to invalid options.
    #[error("Invalid Markdown options: {0}")]
    InvalidOptionsError(String),

    /// An error occurred while loading a syntax set.
    #[error("Failed to load syntax set: {0}")]
    SyntaxSetError(String),

    /// The input exceeds the configured maximum size.
    #[error(
        "Input too large: {size} bytes exceeds limit of {limit} bytes"
    )]
    InputTooLarge {
        /// Actual input size in bytes.
        size: usize,
        /// Configured maximum in bytes.
        limit: usize,
    },

    /// An error occurred while rendering HTML.
    #[error("HTML rendering error: {0}")]
    RenderError(String),

    /// An error occurred while writing output to a `Write` sink.
    #[error("Output write error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Map a single [`ValidationError`] into a domain
/// [`MarkdownError::InvalidOptionsError`].
impl From<ValidationError> for MarkdownError {
    fn from(err: ValidationError) -> Self {
        MarkdownError::InvalidOptionsError(err.to_string())
    }
}

/// Map the multi-error form produced by
/// [`Validator::finish`](crate::validation::Validator::finish) into a
/// domain [`MarkdownError::InvalidOptionsError`]. Every failing check
/// is joined into a single human-readable message with the field name
/// preserved.
///
/// This is what
/// [`MarkdownOptions::validate`](crate::MarkdownOptions::validate)
/// returns; the pipeline converts via `?`.
impl From<Vec<(String, ValidationError)>> for MarkdownError {
    fn from(errors: Vec<(String, ValidationError)>) -> Self {
        let msg = errors
            .iter()
            .map(|(field, err)| format!("{field}: {err}"))
            .collect::<Vec<_>>()
            .join("; ");
        MarkdownError::InvalidOptionsError(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let cases: Vec<(MarkdownError, &str)> = vec![
            (
                MarkdownError::ParseError("bad input".into()),
                "Failed to parse Markdown: bad input",
            ),
            (
                MarkdownError::ConversionError("failed".into()),
                "Failed to convert Markdown to HTML: failed",
            ),
            (
                MarkdownError::InputTooLarge {
                    size: 2_000_000,
                    limit: 1_000_000,
                },
                "Input too large: 2000000 bytes exceeds limit of 1000000 bytes",
            ),
            (
                MarkdownError::RenderError("fmt".into()),
                "HTML rendering error: fmt",
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(format!("{error}"), expected);
        }
    }

    #[test]
    fn test_from_validation_error() {
        // Exact-string assertion on Display (which `thiserror` derives
        // from the `#[error(...)]` attribute on `InvalidOptionsError`)
        // implicitly proves the variant is `InvalidOptionsError` — no
        // pattern-match branch whose no-match arm would be
        // uncoverable.
        let err: MarkdownError = ValidationError::Empty.into();
        assert_eq!(
            err.to_string(),
            "Invalid Markdown options: Value cannot be empty"
        );
    }

    #[test]
    fn test_from_validation_error_vec_joins_fields() {
        let errs = vec![
            ("name".into(), ValidationError::Empty),
            (
                "pattern".into(),
                ValidationError::InvalidPattern {
                    pattern: "email".into(),
                },
            ),
        ];
        let err: MarkdownError = errs.into();
        let msg = err.to_string();
        assert!(
            msg.starts_with("Invalid Markdown options: "),
            "expected InvalidOptionsError Display prefix, got: {msg}"
        );
        assert!(msg.contains("name: Value cannot be empty"));
        assert!(
            msg.contains("pattern: Value doesn't match pattern: email")
        );
        assert!(msg.contains("; "));
    }
}
