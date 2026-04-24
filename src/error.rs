//! Error handling for the MDX Gen library.

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

    /// An error occurred while parsing YAML frontmatter.
    #[error("Frontmatter error: {0}")]
    FrontmatterError(String),

    /// An error occurred while writing output to a `Write` sink.
    #[error("Output write error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Map a shared [`commons::error::CommonError`] into a domain
/// [`MarkdownError`] so callers upstream in the EUXIS ecosystem can
/// propagate failures with `?` into mdx-gen's `Result` types.
///
/// The mapping keeps domain-specific variants intact:
///
/// * `InvalidInput` / `Parse` → [`MarkdownError::ParseError`]
/// * `Io` → [`MarkdownError::IoError`]
/// * everything else → [`MarkdownError::ConversionError`] with the
///   original `Display` form preserved (no information loss).
impl From<commons::error::CommonError> for MarkdownError {
    fn from(err: commons::error::CommonError) -> Self {
        use commons::error::CommonError;
        match err {
            CommonError::InvalidInput(msg)
            | CommonError::Parse(msg) => MarkdownError::ParseError(msg),
            CommonError::Io(e) => MarkdownError::IoError(e),
            other => MarkdownError::ConversionError(other.to_string()),
        }
    }
}

/// Map a shared [`commons::validation::ValidationError`] into a
/// domain [`MarkdownError::InvalidOptionsError`].
///
/// This is the bridge that lets ecosystem-wide single-shot
/// validation errors feed into the mdx-gen error pipeline via `?`.
impl From<commons::validation::ValidationError> for MarkdownError {
    fn from(err: commons::validation::ValidationError) -> Self {
        MarkdownError::InvalidOptionsError(err.to_string())
    }
}

/// Map the multi-error form produced by
/// [`commons::validation::Validator::finish`] into a domain
/// [`MarkdownError::InvalidOptionsError`]. Every failing check is
/// joined into a single human-readable message with the field name
/// preserved.
///
/// This is what
/// [`MarkdownOptions::validate`](crate::MarkdownOptions::validate)
/// returns; the pipeline converts via `?`.
impl From<Vec<(String, commons::validation::ValidationError)>>
    for MarkdownError
{
    fn from(
        errors: Vec<(String, commons::validation::ValidationError)>,
    ) -> Self {
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
            (
                MarkdownError::FrontmatterError("invalid yaml".into()),
                "Frontmatter error: invalid yaml",
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(format!("{error}"), expected);
        }
    }

    #[test]
    fn test_from_common_error_parse() {
        let err: MarkdownError =
            commons::error::CommonError::InvalidInput("bad".into())
                .into();
        assert!(matches!(err, MarkdownError::ParseError(_)));

        let err: MarkdownError =
            commons::error::CommonError::Parse("syntax".into()).into();
        assert!(matches!(err, MarkdownError::ParseError(_)));
    }

    #[test]
    fn test_from_common_error_io() {
        let source =
            std::io::Error::new(std::io::ErrorKind::BrokenPipe, "nope");
        let err: MarkdownError =
            commons::error::CommonError::Io(source).into();
        assert!(matches!(err, MarkdownError::IoError(_)));
    }

    #[test]
    fn test_from_common_error_other_preserves_display() {
        let err: MarkdownError =
            commons::error::CommonError::NotFound("x".into()).into();
        match err {
            MarkdownError::ConversionError(msg) => {
                assert!(msg.contains("Not found"));
                assert!(msg.contains('x'));
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn test_from_validation_error() {
        let err: MarkdownError =
            commons::validation::ValidationError::Empty.into();
        assert!(matches!(err, MarkdownError::InvalidOptionsError(_)));
    }
}
