use comrak::ComrakOptions;
use mdx_gen::{process_markdown, MarkdownOptions};

#[test]
fn test_complex_markdown_with_all_features() {
    let markdown = r#"
# Advanced Markdown Processing Test

## Custom Blocks

<div class="note">This is a note.</div>

<div class="warning">This is a warning.</div>

<div class="tip">This is a tip.</div>

## Code Blocks with Syntax Highlighting

```rust
fn main() {
    println!("Hello, world!");
}
```

## Enhanced Tables

| Left-aligned | Center-aligned | Right-aligned |
|:-------------|:--------------:|---------------:|
| A            |       B        |              C |
| D            |       E        |              F |

## Mixed Content

Here's a paragraph with **bold** and *italic* text, followed by a list:

1. First item
2. Second item
   - Subitem 1
   - Subitem 2
3. Third item

[Link to Rust website](https://www.rust-lang.org/)

> This is a blockquote.
> It can span multiple lines.

---

"#;

    let options = MarkdownOptions::new()
        .with_custom_blocks(true)
        .with_syntax_highlighting(true)
        .with_enhanced_tables(true)
        .with_comrak_options({
            let mut opts = ComrakOptions::default();
            opts.extension.table = true;
            opts.extension.strikethrough = true;
            opts.extension.tasklist = true;
            opts.extension.autolink = true;
            opts
        });

    let result = process_markdown(markdown, &options);
    assert!(
        result.is_ok(),
        "Markdown processing failed: {:?}",
        result.err()
    );

    let html = result.unwrap();

    // Print the entire HTML output
    println!("Generated HTML:\n{}", html);

    // Check for presence of key elements
    assert!(html.contains("Advanced Markdown Processing Test"));
    assert!(html.contains(r#"<div class="alert alert-info" role="alert"><strong>Note:</strong>"#));
    assert!(html.contains(r#"<div class="alert alert-warning" role="alert"><strong>Warning:</strong>"#));
    assert!(html.contains(r#"<div class="alert alert-success" role="alert"><strong>Tip:</strong>"#));
    assert!(html.contains(r#"<pre><code class="language-rust">"#));
    assert!(html.contains("Hello, world!"));
    assert!(html.contains(
        r#"<div class="table-responsive"><table class="table">"#
    ));
    assert!(
        html.contains(r#"<td align="left" class="text-left">A</td>"#)
    );
    assert!(html
        .contains(r#"<td align="center" class="text-center">B</td>"#));
    assert!(
        html.contains(r#"<td align="right" class="text-right">C</td>"#)
    );
    assert!(html.contains("<strong>bold</strong>"));
    assert!(html.contains("<em>italic</em>"));
    assert!(html.contains("<ol>"));
    assert!(html.contains("<ul>"));
    assert!(html.contains(r#"<a href="https://www.rust-lang.org/">Link to Rust website</a>"#));
    assert!(html.contains("<blockquote>"));
    assert!(html.contains("<hr"));
}

#[test]
fn test_basic_markdown_elements() {
    let markdown = r#"
# Header 1
## Header 2
### Header 3

This is a simple paragraph with **bold text** and *italic text*.

---

Another paragraph.
"#;

    let options = MarkdownOptions::new()
        .with_custom_blocks(true)
        .with_syntax_highlighting(true)
        .with_enhanced_tables(true)
        .with_comrak_options({
            let mut opts = ComrakOptions::default();
            opts.extension.table = true;
            opts.extension.strikethrough = true;
            opts.extension.tasklist = true;
            opts.extension.autolink = true;
            opts
        });

    let result = process_markdown(markdown, &options);
    assert!(
        result.is_ok(),
        "Markdown processing failed: {:?}",
        result.err()
    );

    let html = result.unwrap();

    // Check for presence of key elements
    assert!(html.contains("<h1>Header 1</h1>"));
    assert!(html.contains("<h2>Header 2</h2>"));
    assert!(html.contains("<h3>Header 3</h3>"));
    assert!(html.contains("<p>This is a simple paragraph with <strong>bold text</strong> and <em>italic text</em>.</p>"));
    assert!(html.contains("<hr"));
    assert!(html.contains("<p>Another paragraph.</p>"));
}

#[test]
fn test_links_and_images() {
    let markdown = r#"
[Link to Rust](https://www.rust-lang.org/)

![Rust logo](https://www.rust-lang.org/static/images/rust-logo-blk.svg)
"#;

    let options = MarkdownOptions::new()
        .with_custom_blocks(true)
        .with_syntax_highlighting(true)
        .with_enhanced_tables(true)
        .with_comrak_options({
            let mut opts = ComrakOptions::default();
            opts.extension.table = true;
            opts.extension.strikethrough = true;
            opts.extension.tasklist = true;
            opts.extension.autolink = true;
            opts
        });

    let result = process_markdown(markdown, &options);
    assert!(
        result.is_ok(),
        "Markdown processing failed: {:?}",
        result.err()
    );

    let html = result.unwrap();

    // Check for links and images
    assert!(html.contains(
        r#"<a href="https://www.rust-lang.org/">Link to Rust</a>"#
    ));
    assert!(html.contains(r#"<img src="https://www.rust-lang.org/static/images/rust-logo-blk.svg" alt="Rust logo" />"#));
}

#[test]
fn test_lists_and_blockquotes() {
    let markdown = r#"
1. First ordered item
2. Second ordered item
   - Subitem 1
   - Subitem 2

> This is a blockquote.
"#;

    let options = MarkdownOptions::new()
        .with_custom_blocks(true)
        .with_syntax_highlighting(true)
        .with_enhanced_tables(true)
        .with_comrak_options({
            let mut opts = ComrakOptions::default();
            opts.extension.table = true;
            opts.extension.strikethrough = true;
            opts.extension.tasklist = true;
            opts.extension.autolink = true;
            opts
        });

    let result = process_markdown(markdown, &options);
    assert!(
        result.is_ok(),
        "Markdown processing failed: {:?}",
        result.err()
    );

    let html = result.unwrap();

    // Check for lists and blockquotes
    assert!(html.contains("<ol>"));
    assert!(html.contains("<ul>"));
    assert!(html.contains("<blockquote>"));
}

#[test]
fn test_horizontal_rules_and_inline_code() {
    let markdown = r#"
This is some inline `code`.

---

Another line followed by an HR.
"#;

    let options = MarkdownOptions::new()
        .with_custom_blocks(true)
        .with_syntax_highlighting(true)
        .with_enhanced_tables(true)
        .with_comrak_options({
            let mut opts = ComrakOptions::default();
            opts.extension.table = true;
            opts.extension.strikethrough = true;
            opts.extension.tasklist = true;
            opts.extension.autolink = true;
            opts
        });

    let result = process_markdown(markdown, &options);
    assert!(
        result.is_ok(),
        "Markdown processing failed: {:?}",
        result.err()
    );

    let html = result.unwrap();

    // Check for inline code and horizontal rule
    assert!(html.contains("<code>code</code>"));
    assert!(html.contains("<hr"));
}

#[test]
fn test_strikethrough_and_tasklist() {
    let markdown = r#"
~~Strikethrough text~~

- [x] Task 1
- [ ] Task 2
"#;

    let options = MarkdownOptions::new()
        .with_custom_blocks(true)
        .with_syntax_highlighting(true)
        .with_enhanced_tables(true)
        .with_comrak_options({
            let mut opts = ComrakOptions::default();
            opts.extension.table = true;
            opts.extension.strikethrough = true;
            opts.extension.tasklist = true;
            opts.extension.autolink = true;
            opts
        });

    let result = process_markdown(markdown, &options);
    assert!(
        result.is_ok(),
        "Markdown processing failed: {:?}",
        result.err()
    );

    let html = result.unwrap();

    // Print the HTML output to check the structure
    println!("Generated HTML:\n{}", html);

    // Check for strikethrough
    assert!(html.contains("<del>Strikethrough text</del>"));

    // Check for task list with correct HTML structure
    assert!(html.contains(r#"<li><input type="checkbox" checked="" disabled="" /> Task 1</li>"#),
        "Task list rendering failed. Actual HTML: {}", html);
    assert!(
        html.contains(
            r#"<li><input type="checkbox" disabled="" /> Task 2</li>"#
        ),
        "Task list rendering failed. Actual HTML: {}",
        html
    );
}

#[test]
fn test_autolink_urls() {
    let markdown = r#"
Here is a URL: https://www.example.com
"#;

    let options = MarkdownOptions::new()
        .with_custom_blocks(true)
        .with_syntax_highlighting(true)
        .with_enhanced_tables(true)
        .with_comrak_options({
            let mut opts = ComrakOptions::default();
            opts.extension.table = true;
            opts.extension.strikethrough = true;
            opts.extension.tasklist = true;
            opts.extension.autolink = true;
            opts
        });

    let result = process_markdown(markdown, &options);
    assert!(
        result.is_ok(),
        "Markdown processing failed: {:?}",
        result.err()
    );

    let html = result.unwrap();

    // Check for autolinked URL
    assert!(html.contains(r#"<a href="https://www.example.com">https://www.example.com</a>"#));
}

#[test]
fn test_nested_blockquotes() {
    let markdown = r#"
> This is a blockquote.
>
> > This is a nested blockquote.
"#;

    let options = MarkdownOptions::new()
        .with_custom_blocks(true)
        .with_syntax_highlighting(true)
        .with_enhanced_tables(true)
        .with_comrak_options({
            let mut opts = ComrakOptions::default();
            opts.extension.table = true;
            opts.extension.strikethrough = true;
            opts.extension.tasklist = true;
            opts.extension.autolink = true;
            opts
        });

    let result = process_markdown(markdown, &options);
    assert!(
        result.is_ok(),
        "Markdown processing failed: {:?}",
        result.err()
    );

    let html = result.unwrap();

    // Check for nested blockquotes
    assert!(html.contains("<blockquote>\n<p>This is a blockquote.</p>\n<blockquote>\n<p>This is a nested blockquote.</p>\n</blockquote>\n</blockquote>"));
}

#[test]
fn test_emphasis_in_block_elements() {
    let markdown = r#"
> **Bold text** and *italic text* inside a blockquote.
"#;

    let options = MarkdownOptions::new()
        .with_custom_blocks(true)
        .with_syntax_highlighting(true)
        .with_enhanced_tables(true)
        .with_comrak_options({
            let mut opts = ComrakOptions::default();
            opts.extension.table = true;
            opts.extension.strikethrough = true;
            opts.extension.tasklist = true;
            opts.extension.autolink = true;
            opts
        });

    let result = process_markdown(markdown, &options);
    assert!(
        result.is_ok(),
        "Markdown processing failed: {:?}",
        result.err()
    );

    let html = result.unwrap();

    // Check for bold and italic inside blockquote
    assert!(html.contains("<blockquote>\n<p><strong>Bold text</strong> and <em>italic text</em> inside a blockquote.</p>\n</blockquote>"));
}
