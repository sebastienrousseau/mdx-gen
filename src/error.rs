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
}
