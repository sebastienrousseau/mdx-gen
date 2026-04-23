// Copyright © 2024 - 2026 MDX Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! # Basic Example — Minimal Markdown → HTML conversion
//!
//! ## What this example is
//!
//! The smallest useful snippet: a handful of Markdown lines, default
//! options, one call to [`process_markdown`], and the HTML printed to
//! stdout. Use this file as the starting point when you just want to
//! see whether mdx-gen fits your project.
//!
//! ## What it demonstrates
//!
//! - **One call, one String** — [`process_markdown`] takes `&str` and
//!   returns HTML.
//! - **Comrak extensions wired through** — tables, strikethrough, and
//!   autolinks enabled via [`mdx_gen::Options`].
//! - **No features you don't need** — syntax highlighting, custom
//!   blocks, and table enhancement disabled so the output is plain
//!   GFM HTML.
//!
//! ## When to use this pattern
//!
//! When you're evaluating the crate or writing a one-off converter
//! and want the simplest possible wiring. Reach for the other
//! examples (`pipeline_example`, `extensions_example`, `toc_example`)
//! once you need the extras.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example basic_example
//! ```

use mdx_gen::{process_markdown, MarkdownOptions, Options};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Basic Markdown → HTML conversion");
    println!("───────────────────────────────────");

    let markdown = r#"# Welcome to MDX Gen

This is **bold**, this is *italic*, and this is ~~strikethrough~~.

## Features

- CommonMark core
- GFM tables
- Autolinks — https://docs.rs/mdx-gen

[Docs](https://docs.rs/mdx-gen) for details.
"#;

    let mut comrak_options = Options::default();
    comrak_options.extension.strikethrough = true;
    comrak_options.extension.table = true;
    comrak_options.extension.autolink = true;

    let options = MarkdownOptions::new()
        .with_custom_blocks(false)
        .with_enhanced_tables(false)
        .with_syntax_highlighting(false)
        .with_comrak_options(comrak_options);

    let html = process_markdown(markdown, &options)?;
    println!("    ✅ Converted {} bytes of Markdown", markdown.len());
    println!("    📄 Output:\n");
    println!("{html}");

    Ok(())
}
