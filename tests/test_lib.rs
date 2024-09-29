#[cfg(test)]
mod tests {
    use comrak::ComrakOptions;
    use mdx_gen::{process_markdown, MarkdownError, MarkdownOptions};

    #[test]
    fn test_markdown_error_access() {
        // Ensure that the MarkdownError enum is accessible and works
        let error =
            MarkdownError::ConversionError("Test error".to_string());
        assert_eq!(
            format!("{}", error),
            "Failed to convert Markdown to HTML: Test error"
        );
    }

    #[test]
    fn test_process_markdown_with_default_options() {
        let markdown = "# Heading\n\nThis is a **bold** text.";
        let options = MarkdownOptions::new().with_comrak_options({
            let mut opts = ComrakOptions::default();
            opts.extension.table = true; // Ensure table extension is disabled for default test
            opts
        });
        let result = process_markdown(markdown, &options)
            .expect("Failed to process markdown");
        assert!(
            result.contains("<h1>Heading</h1>"),
            "Heading was not processed correctly"
        );
        assert!(
            result.contains("<strong>bold</strong>"),
            "Bold text was not processed correctly"
        );
    }

    #[test]
    fn test_process_markdown_with_custom_blocks() {
        let markdown = r#"<div class="note">This is a note.</div>"#;
        let options = MarkdownOptions::new()
            .with_custom_blocks(true)
            .with_comrak_options({
                let mut opts = ComrakOptions::default();
                opts.extension.table = true;
                opts
            });
        let result = process_markdown(markdown, &options)
            .expect("Failed to process markdown with custom blocks");
        assert!(result.contains(r#"<div class="alert alert-info" role="alert"><strong>Note:</strong>"#), "Custom block was not processed correctly");
    }

    #[test]
    fn test_process_markdown_with_syntax_highlighting() {
        let markdown = "```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```";
        let options = MarkdownOptions::new()
            .with_syntax_highlighting(true)
            .with_comrak_options({
                let mut opts = ComrakOptions::default();
                opts.extension.table = true; // Ensure table extension is enabled to avoid conflicts
                opts
            });
        let result = process_markdown(markdown, &options).expect(
            "Failed to process markdown with syntax highlighting",
        );
        println!(
            "Processed Markdown (Syntax Highlighting):\n{}",
            result
        );

        // Check for syntax highlighting in the generated HTML
        assert!(result.contains("<code class=\"language-rust\">"), "Code block was not processed with the expected syntax highlighting class");

        // Check that the general structure of the code content is present (using Hello, world!)
        assert!(
            result.contains("Hello, world!"),
            "Code block content was not processed correctly"
        );

        // Check that syntax highlighting styles are applied
        assert!(result.contains("color:#a3be8c;"), "Syntax highlighting style for string was not applied correctly");
    }

    #[test]
    fn test_process_markdown_with_enhanced_tables() {
        let markdown = "| Header 1 | Header 2 |\n| --- | --- |\n| Cell 1 | Cell 2 |";
        let options = MarkdownOptions::new()
            .with_enhanced_tables(true)
            .with_comrak_options({
                let mut opts = ComrakOptions::default();
                opts.extension.table = true; // Enable table extension
                opts
            });
        let result = process_markdown(markdown, &options)
            .expect("Failed to process markdown with enhanced tables");
        println!("Processed Markdown (Enhanced Tables):\n{}", result);

        // Check for table elements with enhanced formatting
        assert!(
            result.contains("<div class=\"table-responsive\">"),
            "Table wrapper was not processed correctly"
        );
        assert!(
            result.contains("<table class=\"table\">"),
            "Table class was not applied correctly"
        );
        assert!(
            result.contains("<td class=\"text-left\">Cell 1</td>"),
            "Cell 1 was not processed correctly"
        );
        assert!(
            result.contains("<td class=\"text-left\">Cell 2</td>"),
            "Cell 2 was not processed correctly"
        );
    }

    #[test]
    fn test_markdown_options_customization() {
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_syntax_highlighting(false)
            .with_enhanced_tables(false);
        assert!(
            !options.enable_custom_blocks,
            "Custom blocks should be disabled"
        );
        assert!(
            !options.enable_syntax_highlighting,
            "Syntax highlighting should be disabled"
        );
        assert!(
            !options.enable_enhanced_tables,
            "Enhanced tables should be disabled"
        );
    }
}
