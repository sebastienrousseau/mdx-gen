// Copyright © 2024 - 2026 MDX Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! # Extensions Example — Custom blocks, enhanced tables, inline highlighter
//!
//! ## What this example is
//!
//! A focused tour of the three extension points exposed under
//! [`mdx_gen::extensions`] plus the standalone syntax highlighter
//! API. Each section prints a before/after snippet so you can see
//! exactly what each piece transforms.
//!
//! ## What it demonstrates
//!
//! - [`process_custom_blocks`] — rewrites `<div class="note">` style
//!   divs into Bootstrap-shaped alert markup (string-level API, kept
//!   around for callers that post-process already-rendered HTML).
//! - [`process_tables`] — the legacy responsive-wrapper + alignment
//!   pass. The AST-level equivalent runs automatically inside
//!   [`process_markdown`] when `enable_enhanced_tables` is on.
//! - [`apply_syntax_highlighting`] — highlights a code string
//!   outside the Markdown pipeline, returning class-based
//!   `<span class="…">` tokens ready for a paired stylesheet.
//! - [`ColumnAlignment`] and [`CustomBlockType`] — the enums behind
//!   the two extension points, useful when you're writing your own
//!   renderer on top of the AST.
//!
//! ## When to use this pattern
//!
//! Reach for the extensions module directly when you're integrating
//! mdx-gen into a larger pipeline and want to call a single stage in
//! isolation — for example, highlighting a code snippet pulled from
//! a database, or re-running the custom-block pass on HTML produced
//! by a different renderer.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example extensions_example
//! ```

use mdx_gen::apply_syntax_highlighting;
use mdx_gen::extensions::{
    process_custom_blocks, process_tables, ColumnAlignment,
    CustomBlockType,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Extensions walkthrough");
    println!("─────────────────────────");

    custom_blocks_section()?;
    tables_section()?;
    highlighter_section()?;
    enum_reference_section();

    println!("\n    🎉 Extension demos complete");
    Ok(())
}

fn custom_blocks_section() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📦 Custom blocks");
    let before = r#"<div class="note">Remember to back up your data.</div>
<div class="warning">This action is irreversible.</div>
<div class="tip">Press ⌘K to search.</div>"#;
    let after = process_custom_blocks(before);
    println!("    ── before ─────────────────");
    println!("{before}");
    println!("    ── after  ─────────────────");
    println!("{after}");
    Ok(())
}

fn tables_section() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Tables (legacy string-level pass)");
    let before = r#"<table>
<tr><td align="left">L</td><td align="center">C</td><td align="right">R</td></tr>
</table>"#;
    let after = process_tables(before);
    println!("    ── before ─────────────────");
    println!("{before}");
    println!("    ── after  ─────────────────");
    println!("{after}");
    Ok(())
}

fn highlighter_section() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎨 Standalone syntax highlighter");
    let code = "fn main() { println!(\"hello\"); }";
    let html = apply_syntax_highlighting(code, "rust")?;
    println!("    input:  {code}");
    println!("    output: {html}");
    println!(
        "    ↪ pair with `mdx_gen::theme_css(\"<theme>\")` to get a stylesheet"
    );
    Ok(())
}

fn enum_reference_section() {
    println!("\n📋 Enum reference");

    print!("    ColumnAlignment →");
    for a in [
        ColumnAlignment::Left,
        ColumnAlignment::Center,
        ColumnAlignment::Right,
    ] {
        print!(" {a:?}");
    }
    println!();

    println!("    CustomBlockType");
    for b in [
        CustomBlockType::Note,
        CustomBlockType::Warning,
        CustomBlockType::Tip,
        CustomBlockType::Info,
        CustomBlockType::Important,
        CustomBlockType::Caution,
    ] {
        println!(
            "        {:<10} class={:<18} title={}",
            format!("{b:?}"),
            b.get_alert_class(),
            b.get_title()
        );
    }
}
