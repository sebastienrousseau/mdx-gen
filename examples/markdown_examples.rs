// Copyright © 2024 MDX Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # MDX Gen Markdown Processing Examples
//!
//! This example demonstrates the core Markdown processing functionality
//! in the MDX Gen library. It covers various Markdown processing scenarios,
//! configuration options, and advanced features.

#![allow(missing_docs)]

use mdx_gen::{process_markdown, ComrakOptions, MarkdownOptions};

/// Entry point for the MDX Gen Markdown processing examples.
///
/// This function runs various examples demonstrating the Markdown processing
/// capabilities of the MDX Gen library.
///
/// # Errors
///
/// Returns an error if any of the example functions fail.
#[tokio::main]
pub(crate) async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 MDX Gen Markdown Processing Examples\n");

    basic_markdown_conversion()?;
    custom_options_example()?;
    syntax_highlighting_example()?;
    custom_blocks_example()?;
    enhanced_tables_example()?;
    advanced_configuration_example()?;

    println!(
        "\n🎉  All Markdown processing examples completed successfully!"
    );

    Ok(())
}

/// Demonstrates basic Markdown to HTML conversion.
///
/// This function shows how to convert simple Markdown to HTML using default options.
///
/// # Errors
///
/// Returns an error if Markdown processing fails.
fn basic_markdown_conversion() -> Result<(), Box<dyn std::error::Error>>
{
    println!("🦀 Basic Markdown Conversion Example");
    println!("---------------------------------------------");

    let markdown = "# Hello, world!\n\nThis is a **bold** statement.";
    let options = MarkdownOptions::new()
        .with_enhanced_tables(true) // Use enhanced tables
        .with_comrak_options({
            let mut opts = ComrakOptions::default();
            opts.extension.table = true; // Enable table extension if you have tables
            opts
        });

    match process_markdown(markdown, &options) {
        Ok(html) => {
            println!("    ✅  Converted Markdown to HTML:\n{}", html)
        }
        Err(e) => {
            eprintln!("    ❌  Error in Markdown conversion: {}", e)
        }
    }

    Ok(())
}

/// Demonstrates Markdown processing with custom options.
///
/// This function shows how to use custom MarkdownOptions for processing.
///
/// # Errors
///
/// Returns an error if Markdown processing fails.
fn custom_options_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🦀 Custom Options Example");
    println!("---------------------------------------------");

    let markdown = "# Custom Options\n\nThis uses *custom* options.";

    let options = MarkdownOptions::new()
        .with_custom_blocks(false)
        .with_syntax_highlighting(false)
        .with_comrak_options({
            let mut opts = ComrakOptions::default();
            opts.extension.table = true; // Enable table extension if you have tables
            opts
        });

    match process_markdown(markdown, &options) {
        Ok(html) => println!("    ✅  Processed Markdown with custom options:\n{}", html),
        Err(e) => eprintln!("    ❌  Error in Markdown processing with custom options: {}", e),
    }

    Ok(())
}

/// Demonstrates Markdown processing with syntax highlighting.
///
/// This function shows how code blocks are syntax highlighted.
///
/// # Errors
///
/// Returns an error if Markdown processing or syntax highlighting fails.
fn syntax_highlighting_example(
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🦀 Syntax Highlighting Example");
    println!("---------------------------------------------");

    let markdown = r#"
# Rust Code Example

```rust
fn main() {
    println!("Hello, world!");
}
```
"#;
    let options = MarkdownOptions::new()
        .with_syntax_highlighting(true)
        .with_comrak_options({
            let mut opts = ComrakOptions::default();
            opts.extension.table = true; // Enable table extension if you have tables
            opts
        });

    let html = process_markdown(markdown, &options)?;
    println!(
        "    ✅  Processed Markdown with syntax highlighting:\n{}",
        html
    );

    Ok(())
}

/// Demonstrates processing of custom blocks in Markdown.
///
/// This function shows how custom blocks are handled and transformed.
///
/// # Errors
///
/// Returns an error if Markdown processing or custom block handling fails.
fn custom_blocks_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🦀 Custom Blocks Example");
    println!("---------------------------------------------");

    let markdown = r#"
# Custom Blocks

<div class="note">This is a note.</div>

<div class="warning">This is a warning.</div>
"#;
    let options = MarkdownOptions::new()
        .with_custom_blocks(true)
        .with_comrak_options({
            let mut opts = ComrakOptions::default();
            opts.extension.table = true; // Enable table extension if you have tables
            opts
        });

    let html = process_markdown(markdown, &options)?;
    println!(
        "    ✅  Processed Markdown with custom blocks:\n{}",
        html
    );

    Ok(())
}

/// Demonstrates enhanced table processing in Markdown.
///
/// This function shows how tables are processed with enhanced formatting.
///
/// # Errors
///
/// Returns an error if Markdown processing or table enhancement fails.
fn enhanced_tables_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🦀 Enhanced Tables Example");
    println!("---------------------------------------------");

    let markdown = r#"
# Enhanced Tables

| Header 1 | Header 2 | Header 3 |
|:---------|:--------:|---------:|
| Left     | Center   | Right    |
"#;
    let mut comrak_options = ComrakOptions::default();
    comrak_options.extension.table = true;
    let options = MarkdownOptions::new()
        .with_enhanced_tables(true)
        .with_comrak_options(comrak_options);

    let html = process_markdown(markdown, &options)?;
    println!(
        "    ✅  Processed Markdown with enhanced tables:\n{}",
        html
    );

    Ok(())
}

/// Demonstrates advanced configuration for Markdown processing.
///
/// This function shows how to use a combination of features and options.
///
/// # Errors
///
/// Returns an error if Markdown processing with advanced configuration fails.
fn advanced_configuration_example(
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🦀 Advanced Configuration Example");
    println!("---------------------------------------------");

    let markdown = r#"
# Advanced MDX Gen Usage

<div class="info">This is an informational block.</div>

```python
def greet(name):
    print(f"Hello, {name}!")
```

| Feature | Status |
|---------|--------|
| Tables  | ✅     |
| Syntax  | ✅     |
| Blocks  | ✅     |
"#;
    let mut comrak_options = ComrakOptions::default();
    comrak_options.extension.table = true;
    comrak_options.extension.strikethrough = true;
    comrak_options.extension.tasklist = true;
    let options = MarkdownOptions::new()
        .with_custom_blocks(true)
        .with_syntax_highlighting(true)
        .with_enhanced_tables(true)
        .with_comrak_options(comrak_options);

    let html = process_markdown(markdown, &options)?;
    println!(
        "    ✅  Processed Markdown with advanced configuration:\n{}",
        html
    );

    Ok(())
}
