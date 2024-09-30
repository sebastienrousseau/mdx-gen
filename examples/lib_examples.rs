//! # MDX Gen Library Examples
//!
//! This file demonstrates various usage examples for the MDX Gen library, including
//! how to process Markdown into HTML, apply syntax highlighting, handle custom blocks,
//! and use enhanced table formatting.

use mdx_gen::extensions::CustomBlockType;
use mdx_gen::{
    apply_syntax_highlighting, process_markdown, ComrakOptions,
    MarkdownOptions,
};

/// Example of processing a basic Markdown string into HTML.
///
/// This example shows how to configure basic options and convert Markdown content
/// into HTML using default and custom configurations.
///
/// # Example
/// ```
/// use mdx_gen::{process_markdown, MarkdownOptions, ComrakOptions};
///
/// let markdown_input = "# Hello, World!\n\nThis is an **example**.";
/// let mut comrak_options = ComrakOptions::default();
/// comrak_options.extension.strikethrough = true; // Enable strikethrough
///
/// let options = MarkdownOptions::default().with_comrak_options(comrak_options);
///
/// let html_output = process_markdown(markdown_input, &options).expect("Failed to process markdown");
/// println!("{}", html_output);
/// ```
///
/// # Errors
///
/// Returns an error if Markdown processing fails.
pub fn example_basic_markdown_conversion(
) -> Result<(), Box<dyn std::error::Error>> {
    let markdown_input =
        "# Welcome to MDX Gen\n\nThis is a **bold** statement.";

    let mut comrak_options = ComrakOptions::default();
    comrak_options.extension.strikethrough = true; // Enable strikethrough

    let options =
        MarkdownOptions::default().with_comrak_options(comrak_options);

    let html_output = process_markdown(markdown_input, &options)?;
    println!("Converted Markdown:\n{}", html_output);

    Ok(())
}

/// Example of syntax highlighting for code blocks.
///
/// This example demonstrates how to use the syntax highlighting functionality in MDX Gen
/// by applying highlighting to a code block.
///
/// # Example
/// ```
/// use mdx_gen::apply_syntax_highlighting;
///
/// let code = r#"
/// fn main() {
///     println!("Hello, world!");
/// }
/// "#;
///
/// let highlighted = apply_syntax_highlighting(code, "rust").expect("Syntax highlighting failed");
/// println!("{}", highlighted);
/// ```
///
/// # Errors
///
/// Returns an error if the syntax highlighting fails.
pub fn example_syntax_highlighting(
) -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
    fn main() {
        println!("Hello, world!");
    }
    "#;

    let highlighted_code = apply_syntax_highlighting(code, "rust")?;
    println!("Highlighted Code:\n{}", highlighted_code);

    Ok(())
}

/// Example of using custom blocks in Markdown processing.
///
/// This example shows how custom blocks, such as notes or warnings, can be processed
/// and rendered as styled HTML elements.
///
/// # Example
/// ```
/// use mdx_gen::extensions::CustomBlockType;
///
/// let block_type = CustomBlockType::Note;
/// let block_html = block_type.get_alert_class(); // Returns the corresponding class for the block
/// assert_eq!(block_html, "alert-info");
/// ```
///
/// # Errors
///
/// Returns an error if the custom block type is unknown.
pub fn example_custom_blocks() -> Result<(), Box<dyn std::error::Error>>
{
    let block_content = "<div class=\"note\">This is a note.</div>";

    let custom_block_type = CustomBlockType::Note;
    println!(
        "Processed Custom Block (Type: {}):\n{}",
        custom_block_type.get_title(),
        block_content
    );

    Ok(())
}

/// Example of enhanced table processing.
///
/// This example demonstrates how tables in Markdown can be processed and enhanced with
/// responsive design and alignment formatting.
///
/// # Example
/// ```
/// use mdx_gen::ColumnAlignment;
///
/// let alignment = ColumnAlignment::Left;
/// assert_eq!(alignment, ColumnAlignment::Left);
/// ```
///
/// # Errors
///
/// Returns an error if table processing fails.
pub fn example_enhanced_tables(
) -> Result<(), Box<dyn std::error::Error>> {
    let markdown_table = r#"
    | Header 1 | Header 2 | Header 3 |
    |:---------|:--------:|---------:|
    | Left     | Center   | Right    |
    "#;

    let mut comrak_options = ComrakOptions::default();
    comrak_options.extension.table = true; // Enable table support

    let options =
        MarkdownOptions::default().with_comrak_options(comrak_options);

    let html_table = process_markdown(markdown_table, &options)?;
    println!("Processed Table:\n{}", html_table);

    Ok(())
}

/// Entry point for the MDX Gen library examples.
///
/// This function runs various examples demonstrating Markdown processing, syntax highlighting,
/// custom block handling, and enhanced table formatting.
///
/// # Errors
///
/// Returns an error if any of the example functions fail.
#[tokio::main]
pub(crate) async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 MDX Gen Library Examples\n");

    example_basic_markdown_conversion()?;
    example_syntax_highlighting()?;
    example_custom_blocks()?;
    example_enhanced_tables()?;

    println!("\n🎉  All MDX Gen examples completed successfully!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_basic_markdown_conversion() {
        example_basic_markdown_conversion().unwrap();
    }

    #[test]
    fn test_example_syntax_highlighting() {
        example_syntax_highlighting().unwrap();
    }

    #[test]
    fn test_example_custom_blocks() {
        example_custom_blocks().unwrap();
    }

    #[test]
    fn test_example_enhanced_tables() {
        example_enhanced_tables().unwrap();
    }
}
