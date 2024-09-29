//! Core Markdown processing functionality.
//!
//! This module handles the conversion of Markdown content into HTML,
//! with support for custom blocks, enhanced tables, and syntax highlighting.
//!

use crate::error::MarkdownError;
use crate::extensions::{
    apply_syntax_highlighting, process_custom_blocks, process_tables,
};
use comrak::{markdown_to_html, ComrakOptions};
use lazy_static::lazy_static;
use log::{debug, info, warn};
use regex::Regex;

/// Options for configuring Markdown processing behavior.
#[derive(Debug, Clone)]
pub struct MarkdownOptions<'a> {
    /// Options for the underlying Comrak Markdown parser.
    pub comrak_options: ComrakOptions<'a>,
    /// Enable or disable processing of custom blocks (e.g., note, warning, tip).
    pub enable_custom_blocks: bool,
    /// Enable or disable syntax highlighting for code blocks.
    pub enable_syntax_highlighting: bool,
    /// Enable or disable enhanced table formatting.
    pub enable_enhanced_tables: bool,
    /// Optional custom theme for syntax highlighting.
    pub syntax_theme: Option<String>,
}

impl<'a> Default for MarkdownOptions<'a> {
    /// Provides default options where custom blocks, syntax highlighting,
    /// and enhanced tables are all enabled, with default theme.
    fn default() -> Self {
        Self {
            comrak_options: ComrakOptions::default(),
            enable_custom_blocks: true,
            enable_syntax_highlighting: true,
            enable_enhanced_tables: true,
            syntax_theme: None, // Default: no custom theme
        }
    }
}

impl<'a> MarkdownOptions<'a> {
    /// Creates a new instance of `MarkdownOptions` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables or disables custom blocks.
    pub fn with_custom_blocks(mut self, enable: bool) -> Self {
        self.enable_custom_blocks = enable;
        self
    }

    /// Enables or disables syntax highlighting for code blocks.
    pub fn with_syntax_highlighting(mut self, enable: bool) -> Self {
        self.enable_syntax_highlighting = enable;
        self
    }

    /// Enables or disables enhanced table formatting.
    pub fn with_enhanced_tables(mut self, enable: bool) -> Self {
        self.enable_enhanced_tables = enable;
        self
    }

    /// Sets a custom theme for syntax highlighting.
    pub fn with_custom_theme(mut self, theme: String) -> Self {
        self.syntax_theme = Some(theme);
        self
    }

    /// Sets custom Comrak options.
    pub fn with_comrak_options(
        mut self,
        options: ComrakOptions<'a>,
    ) -> Self {
        self.comrak_options = options;
        self
    }

    /// Validates the `MarkdownOptions` to ensure they are consistent and compatible.
    pub fn validate(&self) -> Result<(), String> {
        if self.enable_enhanced_tables
            && !self.comrak_options.extension.table
        {
            return Err("Enhanced tables are enabled, but Comrak table extension is disabled.".to_string());
        }
        Ok(())
    }
}

/// Creates a new instance of `MarkdownOptions` with default values.
///
/// # Purpose
///
/// This function initializes a `MarkdownOptions` struct with default settings for processing Markdown content.
///
/// # Parameters
///
/// This function does not take any parameters.
///
/// # Return
///
/// Returns a `MarkdownOptions` instance with the following default settings:
/// - Custom blocks are enabled.
/// - Syntax highlighting is enabled.
/// - Enhanced table formatting is enabled.
/// - The Comrak Markdown parser's table extension is enabled.
///
pub fn default_markdown_options() -> MarkdownOptions<'static> {
    MarkdownOptions::new()
        .with_custom_blocks(true)
        .with_syntax_highlighting(true)
        .with_enhanced_tables(true)
        .with_comrak_options({
            let mut opts = ComrakOptions::default();
            opts.extension.table = true;
            opts
        })
}

/// Processes the input Markdown content and converts it into HTML.
/// Applies custom blocks, syntax highlighting, and enhanced tables based on the provided options.
///
/// # Arguments
///
/// * `content` - The input Markdown string to be processed.
/// * `options` - The configuration options for Markdown processing.
///
/// # Returns
///
/// A result containing the generated HTML string or an error if processing fails.
///
/// # Example
///
/// ```
/// use mdx_gen::MarkdownOptions;
/// use mdx_gen::process_markdown;
/// use mdx_gen::ComrakOptions;
/// use mdx_gen::markdown::default_markdown_options;
///
/// let markdown = "# Title\n\nSome text.";
/// let options = default_markdown_options();
/// let html = process_markdown(markdown, &options).unwrap();
/// println!("{}", html);
/// ```
pub fn process_markdown(
    content: &str,
    options: &MarkdownOptions,
) -> Result<String, MarkdownError> {
    info!("Starting markdown processing");
    debug!("Markdown options: {:?}", options);

    // Validate options
    if let Err(msg) = options.validate() {
        warn!("Invalid MarkdownOptions: {}", msg);
        return Err(MarkdownError::ConversionError(msg));
    }

    // Clone Comrak options and enable unsafe rendering
    let mut comrak_opts = options.comrak_options.clone();
    comrak_opts.render.unsafe_ = true;

    // Convert Markdown to initial HTML
    debug!("Converting markdown to HTML using Comrak");
    let mut html = markdown_to_html(content, &comrak_opts);

    // Apply syntax highlighting if enabled
    if options.enable_syntax_highlighting {
        debug!("Applying syntax highlighting");
        html = highlight_code_blocks(&html)?;
    }

    // Process enhanced tables if enabled
    if options.enable_enhanced_tables {
        debug!("Processing enhanced tables");
        html = process_tables(&html);
    }

    // Process custom blocks (e.g., note, warning, tip) if enabled
    if options.enable_custom_blocks {
        debug!("Processing custom blocks");
        html = process_custom_blocks(&html);
    }

    info!("Markdown processing completed successfully");
    Ok(html)
}

/// Highlights code blocks in the generated HTML using the specified syntax highlighter.
///
/// # Arguments
///
/// * `html` - The input HTML string that contains code blocks.
/// * `options` - The configuration options, including the theme for syntax highlighting.
///
/// # Returns
///
/// A result containing the HTML with highlighted code or an error if highlighting fails.
///
/// # Example
///
/// ```
/// use mdx_gen::ComrakOptions;
/// use mdx_gen::{MarkdownOptions, process_markdown};
/// use mdx_gen::markdown::default_markdown_options;
///
/// let markdown = "```rust\nfn main() { println!(\"Hello, world!\"); }\n```";
/// let options = default_markdown_options();
///
/// // Process the markdown to HTML
/// let highlighted_html = process_markdown(markdown, &options).unwrap();
/// println!("{}", highlighted_html);
/// ```
fn highlight_code_blocks(html: &str) -> Result<String, MarkdownError> {
    debug!("Highlighting code blocks");

    lazy_static! {
        static ref CODE_BLOCK_RE: Regex = Regex::new(
            r#"(?s)<pre><code class="language-(.*?)">(.*?)</code></pre>"#
        ).unwrap();
    }

    let mut highlighted_html = String::new();
    let mut last_end = 0;

    // Iterate over captured code blocks and apply syntax highlighting
    for cap in CODE_BLOCK_RE.captures_iter(html) {
        let before = &html[last_end..cap.get(0).unwrap().start()];
        highlighted_html.push_str(before);

        let lang = &cap[1];
        let code = html_escape::decode_html_entities(&cap[2]);

        debug!(
            "Attempting to highlight code block with language: {}",
            lang
        );
        let highlighted_code = apply_syntax_highlighting(&code, lang)
            .map_err(|e| {
            MarkdownError::ConversionError(format!(
                "Failed to highlight code block in language '{}': {}",
                lang, e
            ))
        })?;
        debug!("Highlighted code: {}", highlighted_code);

        highlighted_html.push_str(&format!(
            "<pre><code class=\"language-{}\">{}</code></pre>",
            lang, highlighted_code
        ));
        last_end = cap.get(0).unwrap().end();
    }

    // Append the remaining portion of the HTML
    highlighted_html.push_str(&html[last_end..]);
    Ok(highlighted_html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_markdown_with_all_features() {
        let markdown = r#"
# Test Markdown

Here's a table:

| Left | Center | Right |
|:-----|:------:|------:|
| 1    |   2    |     3 |

```rust
fn main() {
    println!("Hello, world!");
}
```

<div class="note">This is a note.</div>

<div class="warning">This is a warning.</div>

<div class="tip">This is a tip.</div>
"#;

        let options = default_markdown_options();

        let result = process_markdown(markdown, &options);
        assert!(result.is_ok(), "Markdown processing failed");

        let html = result.unwrap();

        assert!(
            html.contains(
                r#"<div class="table-responsive"><table class="table">"#
            ),
            "Table not processed correctly"
        );
        assert!(
            html.contains(r#"<pre><code class="language-rust">"#),
            "Syntax highlighting not applied"
        );
        assert!(html.contains(r#"<div class="alert alert-info" role="alert"><strong>Note:</strong>"#), "Note block not processed");
        assert!(html.contains(r#"<div class="alert alert-warning" role="alert"><strong>Warning:</strong>"#), "Warning block not processed");
        assert!(html.contains(r#"<div class="alert alert-success" role="alert"><strong>Tip:</strong>"#), "Tip block not processed");
    }

    #[test]
    fn test_process_markdown_without_custom_blocks() {
        let markdown = "# Test Markdown\n<div class=\"note\">This is a note.</div>";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_comrak_options({
                let mut opts = ComrakOptions::default();
                opts.extension.table = true; // Enable table extension if you have tables
                opts
            });

        let result = process_markdown(markdown, &options);
        assert!(
            result.is_ok(),
            "Markdown processing failed: {:?}",
            result
        );
        let html = result.unwrap();
        assert!(
            html.contains("<div class=\"note\">This is a note.</div>"),
            "Custom block should not be processed when disabled"
        );
    }

    #[test]
    fn test_process_markdown_without_enhanced_tables() {
        let markdown = r#"
# Test Markdown

| Left | Center | Right |
|:-----|:------:|------:|
| 1    |   2    |     3 |
"#;

        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_comrak_options({
                let mut opts = ComrakOptions::default();
                opts.extension.table = true;
                opts
            });

        let result = process_markdown(markdown, &options);
        assert!(result.is_ok(), "Markdown processing failed");

        let html = result.unwrap();
        assert!(
            !html.contains(
                r#"<div class="table-responsive"><table class="table">"#
            ),
            "Enhanced table processing applied when disabled"
        );
        assert!(
            html.contains("<table>"),
            "Basic table should still be present"
        );
    }

    #[test]
    fn test_markdown_options_validation() {
        let options = MarkdownOptions::new()
            .with_enhanced_tables(true)
            .with_comrak_options({
                let mut opts = ComrakOptions::default();
                opts.extension.table = false;
                opts
            });

        assert!(options.validate().is_err(), "Validation should fail when enhanced tables are enabled but Comrak table extension is disabled");

        let options = MarkdownOptions::new()
            .with_enhanced_tables(true)
            .with_comrak_options({
                let mut opts = ComrakOptions::default();
                opts.extension.table = true;
                opts
            });

        assert!(
            options.validate().is_ok(),
            "Validation should pass when options are consistent"
        );
    }

    #[test]
    fn test_markdown_options_builder() {
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_syntax_highlighting(true)
            .with_enhanced_tables(false);

        assert!(!options.enable_custom_blocks);
        assert!(options.enable_syntax_highlighting);
        assert!(!options.enable_enhanced_tables);
    }

    #[test]
    fn test_process_markdown_with_invalid_options() {
        let markdown = "# Test\n\n| Column 1 | Column 2 |\n| -------- | -------- |\n| Value 1  | Value 2  |";

        let options = MarkdownOptions::new()
            .with_enhanced_tables(true)
            .with_comrak_options({
                let mut opts = ComrakOptions::default();
                opts.extension.table = false;
                opts
            });

        let result = process_markdown(markdown, &options);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(MarkdownError::ConversionError(_))
        ));
    }

    #[test]
    fn test_process_markdown_with_empty_content() {
        let markdown = "";
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false) // No need for enhanced tables in an empty document
            .with_comrak_options({
                let mut opts = ComrakOptions::default();
                opts.extension.table = false; // Disable table extension
                opts
            });

        let result = process_markdown(markdown, &options);
        assert!(
            result.is_ok(),
            "Markdown processing failed for empty content: {:?}",
            result
        );
        assert_eq!(result.unwrap().trim(), "");
    }

    #[test]
    fn test_process_markdown_with_only_custom_blocks() {
        let markdown = "<div class=\"note\">This is a note.</div>";
        let options = MarkdownOptions::new()
            .with_custom_blocks(true)
            .with_enhanced_tables(false) // Disable enhanced tables since they're not used here
            .with_comrak_options({
                let mut opts = ComrakOptions::default();
                opts.extension.table = false; // Ensure table extension is disabled
                opts
            });

        let result = process_markdown(markdown, &options);
        assert!(
            result.is_ok(),
            "Markdown processing failed for custom blocks: {:?}",
            result
        );
        let html = result.unwrap();
        assert!(html.contains(r#"<div class="alert alert-info" role="alert"><strong>Note:</strong>"#), "Custom block not processed correctly");
    }

    #[test]
    fn test_apply_syntax_highlighting() {
        let code = r#"fn main() { println!("Hello, world!"); }"#;
        let result = apply_syntax_highlighting(code, "rust");

        assert!(result.is_ok(), "Syntax highlighting failed");
        let highlighted = result.unwrap();
        assert!(
            highlighted.contains("<span"),
            "Highlighted code is missing expected HTML"
        );
    }

    /// Test Markdown processing with empty options (all disabled)
    #[test]
    fn test_process_markdown_with_no_features_enabled() {
        let markdown = r#"# Title

    Some plain text. "#;
        let options = MarkdownOptions::new()
            .with_syntax_highlighting(false)
            .with_custom_blocks(false)
            .with_enhanced_tables(false);

        let result = process_markdown(markdown, &options);
        assert!(result.is_ok(), "Markdown processing failed");

        let html = result.unwrap();
        assert!(
            html.contains("<h1>Title</h1>"),
            "Heading not processed correctly"
        );
        assert!(
            html.contains("Some plain text."),
            "Plain text not processed correctly"
        );
    }
}
