#[cfg(test)]
mod tests {
    use anyhow::Result;
    use mdx_gen::{error::parse_markdown_with_context, MarkdownError};

    /// Test the MarkdownError::ParseError variant.
    #[test]
    fn test_markdown_error_parse_error() {
        let error =
            MarkdownError::ParseError("Failed to parse".to_string());
        assert_eq!(
            format!("{}", error),
            "Failed to parse Markdown: Failed to parse"
        );
    }

    /// Test the MarkdownError::ConversionError variant.
    #[test]
    fn test_markdown_error_conversion_error() {
        let error = MarkdownError::ConversionError(
            "Conversion failed".to_string(),
        );
        assert_eq!(
            format!("{}", error),
            "Failed to convert Markdown to HTML: Conversion failed"
        );
    }

    /// Test the MarkdownError::CustomBlockError variant.
    #[test]
    fn test_markdown_error_custom_block_error() {
        let error = MarkdownError::CustomBlockError(
            "Custom block failed".to_string(),
        );
        assert_eq!(
            format!("{}", error),
            "Failed to process custom block: Custom block failed"
        );
    }

    /// Test the MarkdownError::SyntaxHighlightError variant.
    #[test]
    fn test_markdown_error_syntax_highlight_error() {
        let error = MarkdownError::SyntaxHighlightError(
            "Highlighting failed".to_string(),
        );
        assert_eq!(
            format!("{}", error),
            "Syntax highlighting error: Highlighting failed"
        );
    }

    /// Test the MarkdownError::InvalidOptionsError variant.
    #[test]
    fn test_markdown_error_invalid_options_error() {
        let error = MarkdownError::InvalidOptionsError(
            "Invalid options".to_string(),
        );
        assert_eq!(
            format!("{}", error),
            "Invalid Markdown options: Invalid options"
        );
    }

    /// Test the MarkdownError::SyntaxSetError variant.
    #[test]
    fn test_markdown_error_syntax_set_error() {
        let error = MarkdownError::SyntaxSetError(
            "Failed to load syntax set".to_string(),
        );
        assert_eq!(
            format!("{}", error),
            "Failed to load syntax set: Failed to load syntax set"
        );
    }

    /// Test for parsing valid markdown content.
    #[test]
    fn test_parse_markdown_with_context_success() -> Result<()> {
        let result =
            parse_markdown_with_context("valid markdown content")?;
        assert_eq!(result, "Parsed markdown content");
        Ok(())
    }

    /// Test for parsing invalid markdown content (empty input).
    #[test]
    fn test_parse_markdown_with_context_failure() {
        let result = parse_markdown_with_context("");
        assert!(result.is_err());

        let error = result.unwrap_err();

        assert!(format!("{}", error)
            .contains("Failed while parsing markdown content"));

        if let Some(source) = error.source() {
            assert_eq!(
                source.to_string(),
                "Failed to parse Markdown: Input is empty"
            );
        } else {
            panic!("Expected an underlying error");
        }
    }

    /// Test handling of empty markdown input.
    #[test]
    fn test_empty_markdown_input() {
        let result = parse_markdown_with_context("");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(format!("{}", error)
            .contains("Failed while parsing markdown content"));
    }
}
