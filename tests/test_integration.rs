use mdx_gen::{process_markdown, MarkdownOptions};

#[test]
fn test_basic_markdown_conversion() {
    let markdown = "# Hello, world!";
    let options = MarkdownOptions::new()
        .with_enhanced_tables(false)
        .with_custom_blocks(false)
        .with_comrak_options({
            let mut opts = comrak::Options::default();
            opts.extension.table = false;
            opts
        });

    let result = process_markdown(markdown, &options).unwrap();
    assert_eq!(result.trim(), "<h1>Hello, world!</h1>");
}

#[test]
fn test_markdown_with_extensions() {
    let markdown = "This is a ~~strikethrough~~ test.";
    let options = MarkdownOptions::new()
        .with_enhanced_tables(false)
        .with_custom_blocks(false)
        .with_comrak_options({
            let mut opts = comrak::Options::default();
            opts.extension.strikethrough = true;
            opts.extension.table = false;
            opts
        });

    let result = process_markdown(markdown, &options).unwrap();
    assert_eq!(
        result.trim(),
        "<p>This is a <del>strikethrough</del> test.</p>"
    );
}

#[test]
fn test_markdown_with_links() {
    let markdown = "[MDX Generator](https://mdxgen.com/)";
    let options = MarkdownOptions::new()
        .with_enhanced_tables(false)
        .with_custom_blocks(false)
        .with_comrak_options({
            let mut opts = comrak::Options::default();
            opts.extension.table = false;
            opts
        })
        .with_unsafe_html(true);

    let result = process_markdown(markdown, &options).unwrap();
    assert_eq!(
        result.trim(),
        r#"<p><a href="https://mdxgen.com/">MDX Generator</a></p>"#
    );
}
