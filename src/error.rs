//! Error handling for the MDX Gen library.

use anyhow::{Context, Result};

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
}

/// A helper function that adds context to errors occurring during Markdown processing.
pub fn parse_markdown_with_context(input: &str) -> Result<String> {
    // Add context without overriding the original error message
    let parsed_content = some_markdown_parsing_function(input)
        .with_context(|| "Failed while parsing markdown content")?;

    Ok(parsed_content)
}

// Placeholder for the actual markdown parsing function
fn some_markdown_parsing_function(input: &str) -> Result<String> {
    // Simulate success or failure
    if input.is_empty() {
        return Err(MarkdownError::ParseError(
            "Input is empty".to_string(),
        )
        .into());
    }
    Ok("Parsed markdown content".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_input() {
        let result = some_markdown_parsing_function("");
        assert!(result.is_err());

        if let Err(err) = result {
            assert_eq!(
                format!("{}", err),
                "Failed to parse Markdown: Input is empty"
            );
        }
    }

    #[test]
    fn test_successful_parse() {
        let result =
            some_markdown_parsing_function("Some markdown content");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Parsed markdown content");
    }

    #[test]
    fn test_parse_markdown_with_context() {
        let result = parse_markdown_with_context("");
        assert!(result.is_err());

        if let Err(err) = result {
            let err_msg = format!("{:?}", err);
            assert!(err_msg
                .contains("Failed while parsing markdown content"));
        }
    }
}
