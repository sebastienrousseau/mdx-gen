// Copyright © 2024 - 2026 MDX Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! # ToC Example — Build a table of contents alongside the HTML
//!
//! ## What this example is
//!
//! Shows [`process_markdown_with_toc`] returning both the rendered
//! HTML and a document-order `Vec<`[`Heading`]`>`. The example
//! prints the headings as an indented outline, demonstrates that
//! the anchor ids match what the rendered HTML emits, and also
//! shows the streaming variant
//! [`process_markdown_with_toc_to_writer`] for callers that want
//! the ToC alongside a streamed render.
//!
//! ## What it demonstrates
//!
//! - [`process_markdown_with_toc`] — returns
//!   `(String, Vec<Heading>)`.
//! - [`process_markdown_with_toc_to_writer`] — returns
//!   `Vec<Heading>` and streams the HTML through a writer.
//! - [`MarkdownOptions::with_header_ids`] — wiring the header-id
//!   prefix so the rendered `id="…"` attributes match
//!   [`Heading::id`].
//! - [`Heading`] fields (`level`, `text`, `id`) — enough to build
//!   a sidebar, anchor map, or JSON index.
//! - Deduplication behaviour — repeated headings get `-1`, `-2`
//!   suffixes via comrak's [`Anchorizer`].
//!
//! ## When to use this pattern
//!
//! Documentation sites, long-form posts, or any UI that wants a
//! clickable outline sidebar. The returned `Vec<Heading>` is
//! ready to serialise to JSON, feed into a template, or post to a
//! front-end search index.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example toc_example
//! ```
//!
//! [`Anchorizer`]: https://docs.rs/comrak/latest/comrak/struct.Anchorizer.html

use mdx_gen::{
    process_markdown_with_toc, process_markdown_with_toc_to_writer,
    Heading, MarkdownOptions,
};

const SAMPLE: &str = r#"# Release notes

## Highlights

Some body text.

## Breaking changes

### Syntax highlighter

### Sanitiser

## Migration

### Syntax highlighter

### Sanitiser

# Full changelog

## 0.0.3
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Table of contents");
    println!("────────────────────");

    string_variant()?;
    writer_variant()?;
    dedup_behaviour()?;

    println!("\n    🎉 ToC demos complete");
    Ok(())
}

fn string_variant() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📑 process_markdown_with_toc");
    let options = MarkdownOptions::new()
        .with_custom_blocks(false)
        .with_enhanced_tables(false)
        .with_header_ids("");

    let (html, toc) = process_markdown_with_toc(SAMPLE, &options)?;

    println!("    ── outline ────────────────");
    print_outline(&toc);

    println!("\n    ── every ToC id appears in the HTML ──");
    for h in &toc {
        let needle = format!("id=\"{}\"", h.id);
        let ok = html.contains(&needle);
        let tick = if ok { "✅" } else { "❌" };
        println!("      {tick} {needle}");
    }
    Ok(())
}

fn writer_variant() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📑 process_markdown_with_toc_to_writer");
    let options = MarkdownOptions::new()
        .with_custom_blocks(false)
        .with_enhanced_tables(false)
        .with_header_ids("user-content-");

    let mut buf: Vec<u8> = Vec::new();
    let toc = process_markdown_with_toc_to_writer(
        SAMPLE, &mut buf, &options,
    )?;

    println!(
        "    ✅ streamed {} bytes of HTML and returned {} headings",
        buf.len(),
        toc.len()
    );
    if let Some(first) = toc.first() {
        println!("    ↳ first id (note the prefix): {:?}", first.id);
    }
    Ok(())
}

fn dedup_behaviour() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📑 Dedup with repeated heading text");
    let md = "# Notes\n## Notes\n### Notes\n";
    let options = MarkdownOptions::new()
        .with_custom_blocks(false)
        .with_enhanced_tables(false)
        .with_header_ids("");

    let (_html, toc) = process_markdown_with_toc(md, &options)?;
    for h in &toc {
        println!(
            "      H{lvl:?}  text={text:?}  id={id:?}",
            lvl = h.level,
            text = h.text,
            id = h.id
        );
    }
    Ok(())
}

/// Renders the flat `Vec<Heading>` as an indented outline. Uses the
/// level directly as the indent depth, so a document whose first
/// heading is `##` will indent by two levels — the output mirrors
/// the *structure* of the document rather than normalising it.
fn print_outline(toc: &[Heading]) {
    for h in toc {
        let indent = "  ".repeat(h.level.saturating_sub(1) as usize);
        println!(
            "      {indent}• {text} (#{id})",
            text = h.text,
            id = h.id
        );
    }
}
