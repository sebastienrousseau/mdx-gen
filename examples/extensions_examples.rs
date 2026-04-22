// Copyright © 2024 MDX Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # MDX Gen Extensions Examples
//!
//! This example demonstrates the usage of extension functionality in the MDX Gen library.
//! It covers syntax highlighting, table formatting, and custom block handling.

#![allow(missing_docs)]

use mdx_gen::apply_syntax_highlighting;
use mdx_gen::extensions::{
    process_custom_blocks, process_tables, ColumnAlignment,
    CustomBlockType,
};

/// Entry point for the MDX Gen extensions examples.
///
/// This function runs various examples demonstrating the extension functionality
/// of the MDX Gen library.
///
/// # Errors
///
/// Returns an error if any of the example functions fail.
pub(crate) fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 MDX Gen Extensions Examples\n");

    syntax_highlighting_example()?;
    table_processing_example()?;
    custom_block_example()?;
    column_alignment_example()?;
    custom_block_type_example()?;

    println!("\n🎉  All extensions examples completed successfully!");

    Ok(())
}

/// Demonstrates the syntax highlighting functionality.
///
/// This function shows how to apply syntax highlighting to code blocks.
///
/// # Errors
///
/// Returns an error if syntax highlighting fails.
fn syntax_highlighting_example(
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Syntax Highlighting Example");
    println!("---------------------------------------------");

    let code = r#"fn main() {
    println!("Hello, world!");
}"#;
    let language = "rust";

    let highlighted = apply_syntax_highlighting(code, language)?;
    println!("    ✅  Highlighted Rust code:\n{}", highlighted);

    Ok(())
}

/// Demonstrates the enhanced table processing functionality.
///
/// This function shows how tables are processed and formatted.
///
/// # Errors
///
/// Returns an error if table processing fails.
fn table_processing_example() -> Result<(), Box<dyn std::error::Error>>
{
    println!("\n🦀 Table Processing Example");
    println!("---------------------------------------------");

    let table_html = r#"<table>
    <tr><td align="left">Left</td><td align="center">Center</td><td align="right">Right</td></tr>
</table>"#;

    let processed_table = process_tables(table_html);
    println!("    ✅  Processed table HTML:\n{}", processed_table);

    Ok(())
}

/// Demonstrates the custom block processing functionality.
///
/// This function shows how custom blocks are processed and transformed.
///
/// # Errors
///
/// Returns an error if custom block processing fails.
fn custom_block_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🦀 Custom Block Example");
    println!("---------------------------------------------");

    let content = r#"<div class="note">This is a note.</div>
<div class="warning">This is a warning.</div>
<div class="tip">This is a tip.</div>"#;

    let processed_content = process_custom_blocks(content);
    println!("    ✅  Processed custom blocks:\n{}", processed_content);

    Ok(())
}

/// Demonstrates the usage of column alignment functionality.
///
/// This function shows how column alignments are represented and used.
///
/// # Errors
///
/// Returns an error if the example fails.
fn column_alignment_example() -> Result<(), Box<dyn std::error::Error>>
{
    println!("\n🦀 Column Alignment Example");
    println!("---------------------------------------------");

    let alignments = vec![
        ColumnAlignment::Left,
        ColumnAlignment::Center,
        ColumnAlignment::Right,
    ];

    for alignment in alignments {
        println!("    ✅  Column alignment: {:?}", alignment);
    }

    Ok(())
}

/// Demonstrates the usage of custom block types.
///
/// This function shows how different custom block types are represented and used.
///
/// # Errors
///
/// Returns an error if the example fails.
fn custom_block_type_example() -> Result<(), Box<dyn std::error::Error>>
{
    println!("\n🦀 Custom Block Type Example");
    println!("---------------------------------------------");

    let block_types = vec![
        CustomBlockType::Note,
        CustomBlockType::Warning,
        CustomBlockType::Tip,
        CustomBlockType::Info,
        CustomBlockType::Important,
        CustomBlockType::Caution,
    ];

    for block_type in block_types {
        println!(
            "    ✅  Block type: {:?}, Alert class: {}, Title: {}",
            block_type,
            block_type.get_alert_class(),
            block_type.get_title()
        );
    }

    Ok(())
}
