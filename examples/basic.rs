// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Minimal Markdown → HTML conversion.
//!
//! Run: `cargo run --example basic`

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use mdx_gen::{process_markdown, MarkdownOptions, Options};

const SOURCE: &str = "# Welcome to MDX Gen

This is **bold**, this is *italic*, and this is ~~strikethrough~~.

- CommonMark core
- GFM tables
- Autolinks — https://docs.rs/mdx-gen
";

fn main() {
    support::header("mdx-gen -- basic");

    let options = support::task("Build MarkdownOptions", || {
        let mut comrak_options = Options::default();
        comrak_options.extension.strikethrough = true;
        comrak_options.extension.table = true;
        comrak_options.extension.autolink = true;
        MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_syntax_highlighting(false)
            .with_comrak_options(comrak_options)
    });

    let html = support::task("Render Markdown to HTML", || {
        process_markdown(SOURCE, &options).unwrap()
    });

    support::task_with_output("Inspect output", || {
        vec![
            format!("source: {} bytes", SOURCE.len()),
            format!("html:   {} bytes", html.len()),
            format!("<strong> present: {}", html.contains("<strong>")),
            format!("<em> present: {}", html.contains("<em>")),
            format!("<del> present: {}", html.contains("<del>")),
        ]
    });

    support::summary(3);
}
