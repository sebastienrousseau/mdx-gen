#[cfg(test)]
mod tests {
    use anyhow::Result;
    use mdx_gen::{error::parse_markdown_with_context, MarkdownError};

    #[test]
    fn test_markdown_error_parse_error() {
        let error =
            MarkdownError::ParseError("Failed to parse".to_string());
        assert_eq!(
            format!("{}", error),
            "Failed to parse Markdown: Failed to parse"
        );
    }

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

    #[test]
    fn test_markdown_error_syntax_highlighting_error() {
        let error = MarkdownError::SyntaxHighlightingError(
            "Failed to load syntax set".to_string(),
        );
        assert_eq!(
            format!("{}", error),
            "Failed to load syntax set: Failed to load syntax set"
        );
    }

    #[test]
    fn test_parse_markdown_with_context_success() -> Result<()> {
        let result =
            parse_markdown_with_context("valid markdown content")?;
        assert_eq!(result, "Parsed markdown content");
        Ok(())
    }

    #[test]
    fn test_parse_markdown_with_context_failure() {
        let result = parse_markdown_with_context("");
        assert!(result.is_err());

        // Extract the original error from the context
        let error = result.unwrap_err();

        // Ensure the error contains the "Failed while parsing markdown content"
        assert!(format!("{}", error)
            .contains("Failed while parsing markdown content"));

        // Check the source of the error to see if it's the "Input is empty" error
        if let Some(source) = error.source() {
            assert_eq!(
                source.to_string(),
                "Failed to parse Markdown: Input is empty"
            );
        } else {
            panic!("Expected an underlying error");
        }
    }
}
