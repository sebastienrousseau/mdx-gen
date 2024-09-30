//! # Basic Markdown to HTML Conversion Example
//!
//! This example demonstrates how to use the `mdx-gen` crate to convert Markdown content
//! into HTML using the `comrak` library. It shows how to configure various Markdown extensions
//! (e.g., strikethrough, tables, and autolinks) and then process the Markdown content to generate HTML.
//!
//! ## Usage
//!
//! Simply run the example, and it will print the converted HTML to the console. You can customize
//! the Markdown content and options to see how different configurations affect the output.
//!
//! ## Features Demonstrated
//!
//! - **Basic Markdown Conversion**: Simple conversion of text with basic Markdown formatting like bold and italics.
//! - **Custom Extensions**: Enabling of common Markdown extensions such as strikethrough, tables, and autolinking.

use comrak::ComrakOptions;
use mdx_gen::{process_markdown, MarkdownOptions};

/// Entry point for the basic Markdown to HTML conversion example.
///
/// This function demonstrates how to use the `mdx-gen` crate along with the `comrak` library
/// to convert a block of Markdown into HTML. It enables several extensions such as strikethrough,
/// tables, and autolinks.
///
/// # Errors
///
/// Returns an error if the Markdown content fails to process.
pub(crate) fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Basic Markdown to HTML Conversion Example");
    println!("---------------------------------------------");

    // Markdown content to be converted
    let markdown = r#"
# Welcome to MDX Gen

This is a **bold** statement and this is *italic*.

## Features

- Easy to use
- Extensible
- Fast

Check out [our website](https://example.com) for more information.
    "#;

    // Initialize MarkdownOptions with default Comrak options
    let mut comrak_options = ComrakOptions::default();
    comrak_options.extension.strikethrough = true; // Enable strikethrough
    comrak_options.extension.table = true; // Enable tables
    comrak_options.extension.autolink = true; // Enable automatic links

    let options = MarkdownOptions::new()
        .with_comrak_options(comrak_options)
        .with_syntax_highlighting(false); // Syntax highlighting is disabled in this example

    // Process the markdown content to HTML
    match process_markdown(markdown, &options) {
        Ok(html) => {
            println!(
                "    ✅  Converted Markdown to HTML successfully:"
            );
            println!("\n{}", html);
        }
        Err(e) => {
            eprintln!("    ❌  Error in Markdown conversion: {}", e);
        }
    }

    Ok(())
}
