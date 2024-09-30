// Copyright © 2024 MDX Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # MDX Gen Error Handling Examples
//!
//! This example demonstrates the usage of the error types and error handling
//! functionality in the MDX Gen library. It covers various error scenarios,
//! error conversion, and error handling for Markdown processing.

#![allow(missing_docs)]

use comrak::ComrakOptions;
use mdx_gen::{process_markdown, MarkdownError, MarkdownOptions};

/// Entry point for the MDX Gen error handling examples.
///
/// This function runs various examples demonstrating error creation, conversion,
/// and handling for different scenarios in the MDX Gen library.
///
/// # Errors
///
/// Returns an error if any of the example functions fail.
#[tokio::main]
pub(crate) async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 MDX Gen Error Handling Examples\n");

    parse_error_example()?;
    conversion_error_example()?;
    custom_block_error_example()?;
    syntax_highlight_error_example()?;
    invalid_options_error_example()?;
    syntax_set_error_example()?;

    println!(
        "\n🎉  All error handling examples completed successfully!"
    );

    Ok(())
}

/// Demonstrates handling of Markdown parsing errors.
///
/// This function attempts to process invalid Markdown content and shows
/// how MDX Gen handles parsing errors.
///
/// # Errors
///
/// Returns an error if the Markdown processing fails (which is expected in this example).
fn parse_error_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Parse Error Example");
    println!("---------------------------------------------");

    let invalid_markdown =
        "# Heading\n\n```invalid-language\ncode\n```";
    let options = MarkdownOptions::default();

    match process_markdown(invalid_markdown, &options) {
        Ok(_) => println!(
            "    ❌  Unexpected success in parsing invalid Markdown"
        ),
        Err(e) => match e {
            MarkdownError::ParseError(msg) => println!(
                "    ✅  Successfully caught parse error: {}",
                msg
            ),
            _ => println!("    ❌  Unexpected error type: {:?}", e),
        },
    }

    Ok(())
}

/// Demonstrates handling of Markdown to HTML conversion errors.
///
/// This function simulates a conversion error and shows how it's represented
/// in the MDX Gen error system.
///
/// # Errors
///
/// Returns a Result to demonstrate error handling, but always returns Ok in this example.
fn conversion_error_example() -> Result<(), Box<dyn std::error::Error>>
{
    println!("\n🦀  Conversion Error Example");
    println!("---------------------------------------------");

    let conversion_error = MarkdownError::ConversionError(
        "Failed to convert code block".to_string(),
    );
    println!(
        "    ✅  Created Conversion Error: {:?}",
        conversion_error
    );

    // Simulate error handling
    match Err::<(), MarkdownError>(conversion_error) {
        Ok(_) => println!(
            "    ❌  Unexpected success in conversion error handling"
        ),
        Err(e) => println!(
            "    ✅  Successfully handled conversion error: {}",
            e
        ),
    }

    Ok(())
}

/// Demonstrates handling of custom block processing errors.
///
/// This function creates a custom block error and shows how it's handled
/// in the MDX Gen library.
///
/// # Errors
///
/// Returns a Result to demonstrate error handling, but always returns Ok in this example.
fn custom_block_error_example() -> Result<(), Box<dyn std::error::Error>>
{
    println!("\n🦀  Custom Block Error Example");
    println!("---------------------------------------------");

    let custom_block_error = MarkdownError::CustomBlockError(
        "Invalid custom block type".to_string(),
    );
    println!(
        "    ✅  Created Custom Block Error: {:?}",
        custom_block_error
    );

    // Demonstrate custom block error handling
    let markdown = "<div class=\"invalid-type\">Content</div>";
    let options = MarkdownOptions::new().with_custom_blocks(true);

    match process_markdown(markdown, &options) {
        Ok(_) => println!("    ❌  Unexpected success in processing invalid custom block"),
        Err(e) => println!("    ✅  Successfully caught custom block error: {}", e),
    }

    Ok(())
}

/// Demonstrates handling of syntax highlighting errors.
///
/// This function simulates a syntax highlighting error and shows how it's
/// represented and handled in the MDX Gen library.
///
/// # Errors
///
/// Returns a Result to demonstrate error handling, but always returns Ok in this example.
fn syntax_highlight_error_example(
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🦀  Syntax Highlight Error Example");
    println!("---------------------------------------------");

    let syntax_highlight_error = MarkdownError::SyntaxHighlightError(
        "Unknown language: unknown".to_string(),
    );
    println!(
        "    ✅  Created Syntax Highlight Error: {:?}",
        syntax_highlight_error
    );

    // Simulate syntax highlighting error
    let markdown = "```unknown\n++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.>+.+++++++..+++.>++.<<+++++++++++++++.>.+++.------.--------.>+.>.\n```";
    let options = MarkdownOptions::new().with_syntax_highlighting(true);

    match process_markdown(markdown, &options) {
        Ok(_) => println!("    ❌  Unexpected success in highlighting unknown language"),
        Err(e) => println!("    ✅  Successfully caught syntax highlight error: {}", e),
    }

    Ok(())
}

/// Demonstrates handling of invalid Markdown options errors.
///
/// This function creates an invalid options scenario and shows how
/// MDX Gen handles and reports these errors.
///
/// # Errors
///
/// Returns a Result to demonstrate error handling, but always returns Ok in this example.
fn invalid_options_error_example(
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🦀  Invalid Options Error Example");
    println!("---------------------------------------------");

    let invalid_options_error = MarkdownError::InvalidOptionsError("Enhanced tables enabled but Comrak table extension is disabled".to_string());
    println!(
        "    ✅  Created Invalid Options Error: {:?}",
        invalid_options_error
    );

    // Demonstrate invalid options scenario
    let markdown = "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |";
    let mut comrak_options = ComrakOptions::default();
    comrak_options.extension.table = false;
    let options = MarkdownOptions::new()
        .with_enhanced_tables(true)
        .with_comrak_options(comrak_options);

    match process_markdown(markdown, &options) {
        Ok(_) => {
            println!("    ❌  Unexpected success with invalid options")
        }
        Err(e) => println!(
            "    ✅  Successfully caught invalid options error: {}",
            e
        ),
    }

    Ok(())
}

/// Demonstrates handling of syntax set loading errors.
///
/// This function simulates a syntax set loading error and shows how
/// it's represented in the MDX Gen error system.
///
/// # Errors
///
/// Returns a Result to demonstrate error handling, but always returns Ok in this example.
fn syntax_set_error_example() -> Result<(), Box<dyn std::error::Error>>
{
    println!("\n🦀  Syntax Set Error Example");
    println!("---------------------------------------------");

    let syntax_set_error = MarkdownError::SyntaxSetError(
        "Failed to load syntax definition for Rust".to_string(),
    );
    println!(
        "    ✅  Created Syntax Set Error: {:?}",
        syntax_set_error
    );

    // In a real scenario, this error might occur when initializing the syntax highlighter
    // For this example, we'll just simulate the error handling
    match Err::<(), MarkdownError>(syntax_set_error) {
        Ok(_) => println!(
            "    ❌  Unexpected success in syntax set error handling"
        ),
        Err(e) => println!(
            "    ✅  Successfully handled syntax set error: {}",
            e
        ),
    }

    Ok(())
}
