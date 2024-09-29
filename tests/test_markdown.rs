#[cfg(test)]
mod tests {
    use comrak::ComrakOptions;
    use mdx_gen::{process_markdown, MarkdownOptions};

    #[test]
    fn test_markdown_options_default() {
        let options = MarkdownOptions::default();
        assert!(options.enable_custom_blocks);
        assert!(options.enable_syntax_highlighting);
        assert!(options.enable_enhanced_tables);
    }

    #[test]
    fn test_markdown_options_customization() {
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_syntax_highlighting(false)
            .with_enhanced_tables(false);
        assert!(!options.enable_custom_blocks);
        assert!(!options.enable_syntax_highlighting);
        assert!(!options.enable_enhanced_tables);
    }

    #[test]
    fn test_process_markdown_with_default_options() {
        let markdown = "# Heading\n\nThis is a **bold** text.";
        let options = MarkdownOptions::new().with_comrak_options({
            let mut opts = ComrakOptions::default();
            opts.extension.table = true; // Enable table extension
            opts
        });
        let result = process_markdown(markdown, &options)
            .expect("Failed to process markdown");
        assert!(result.contains("<h1>Heading</h1>"));
        assert!(result.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_process_markdown_with_enhanced_tables() {
        let markdown = "| Header 1 | Header 2 |\n| --- | --- |\n| Cell 1 | Cell 2 |";
        let options = MarkdownOptions::new()
            .with_enhanced_tables(true)
            .with_comrak_options({
                let mut opts = ComrakOptions::default();
                opts.extension.table = true; // Ensure the Comrak table extension is enabled
                opts
            });
        let result = process_markdown(markdown, &options)
            .expect("Failed to process markdown with tables");
        println!("Processed Markdown (Enhanced Tables):\n{}", result);

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
    fn test_process_markdown_with_syntax_highlighting() {
        let markdown = "```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```";
        let options = MarkdownOptions::new()
            .with_syntax_highlighting(true)
            .with_comrak_options({
                let mut opts = ComrakOptions::default();
                opts.extension.table = true; // Ensure the table extension is enabled to avoid conflicts
                opts
            });
        let result = process_markdown(markdown, &options).expect(
            "Failed to process markdown with syntax highlighting",
        );
        println!(
            "Processed Markdown (Syntax Highlighting):\n{}",
            result
        );

        // Check that the syntax highlighting and code block are processed
        assert!(result.contains("<code class=\"language-rust\">"), "Code block was not processed with the expected syntax highlighting class");
        assert!(
            result.contains("Hello, world!"),
            "Code block content was not processed correctly"
        );
        assert!(result.contains("color:#a3be8c;"), "Syntax highlighting style for string was not applied correctly");
    }

    #[test]
    fn test_process_markdown_with_html_tags() {
        let markdown =
            "<div>This is Markdown with <b>HTML tags</b></div>";
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false) // Disable enhanced tables
            .with_comrak_options({
                let mut opts = ComrakOptions::default();
                opts.extension.table = false;
                opts
            });
        let result = process_markdown(markdown, &options);
        assert!(result.is_ok(), "Processing Markdown with HTML tags should not result in an error. Error: {:?}", result.err());
        let html = result.unwrap();
        assert!(
            html.contains("<div>"),
            "HTML tags should be preserved in the output"
        );
        assert!(
            html.contains("<b>"),
            "Nested HTML tags should be preserved in the output"
        );
    }

    #[test]
    fn test_process_markdown_with_invalid_markdown() {
        let markdown = "This is not really invalid Markdown";
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false) // Disable enhanced tables
            .with_comrak_options({
                let mut opts = ComrakOptions::default();
                opts.extension.table = false;
                opts
            });
        let result = process_markdown(markdown, &options);
        assert!(result.is_ok(), "Processing invalid Markdown should not result in an error. Error: {:?}", result.err());
        let html = result.unwrap();
        assert!(!html.is_empty(), "Resulting HTML should not be empty");
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
}
