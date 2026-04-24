#[cfg(test)]
mod tests {
    use mdx_gen::MarkdownError;

    #[test]
    fn test_markdown_error_parse_error() {
        let error =
            MarkdownError::ParseError("Failed to parse".to_string());
        assert_eq!(
            format!("{error}"),
            "Failed to parse Markdown: Failed to parse"
        );
    }

    #[test]
    fn test_markdown_error_conversion_error() {
        let error = MarkdownError::ConversionError(
            "Conversion failed".to_string(),
        );
        assert_eq!(
            format!("{error}"),
            "Failed to convert Markdown to HTML: Conversion failed"
        );
    }

    #[test]
    fn test_markdown_error_custom_block_error() {
        let error = MarkdownError::CustomBlockError(
            "Custom block failed".to_string(),
        );
        assert_eq!(
            format!("{error}"),
            "Failed to process custom block: Custom block failed"
        );
    }

    #[test]
    fn test_markdown_error_syntax_highlight_error() {
        let error = MarkdownError::SyntaxHighlightError(
            "Highlighting failed".to_string(),
        );
        assert_eq!(
            format!("{error}"),
            "Syntax highlighting error: Highlighting failed"
        );
    }

    #[test]
    fn test_markdown_error_invalid_options_error() {
        let error = MarkdownError::InvalidOptionsError(
            "Invalid options".to_string(),
        );
        assert_eq!(
            format!("{error}"),
            "Invalid Markdown options: Invalid options"
        );
    }

    #[test]
    fn test_markdown_error_syntax_set_error() {
        let error = MarkdownError::SyntaxSetError(
            "Failed to load syntax set".to_string(),
        );
        assert_eq!(
            format!("{error}"),
            "Failed to load syntax set: Failed to load syntax set"
        );
    }

    #[test]
    fn test_markdown_error_input_too_large() {
        let error = MarkdownError::InputTooLarge {
            size: 2_000_000,
            limit: 1_000_000,
        };
        assert!(format!("{error}").contains("2000000"));
        assert!(format!("{error}").contains("1000000"));
    }

    #[test]
    fn test_markdown_error_render_error() {
        let error =
            MarkdownError::RenderError("fmt failed".to_string());
        assert_eq!(
            format!("{error}"),
            "HTML rendering error: fmt failed"
        );
    }
}
