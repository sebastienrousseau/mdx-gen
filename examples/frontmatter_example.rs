// Copyright © 2024 - 2026 MDX Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! # Frontmatter Example — Extract + parse YAML frontmatter
//!
//! ## What this example is
//!
//! A three-step dance that lifts a `---`-delimited YAML block off
//! the top of a Markdown file, parses it (both as an untyped
//! [`yaml_safe::Value`] and as a strongly-typed [`FrontMatter`]
//! struct), and then hands the body through
//! [`process_markdown`] so you see how the two halves fit
//! together.
//!
//! ## What it demonstrates
//!
//! - [`extract_frontmatter`] — splits `(yaml, body)` from a
//!   source string, returning `(None, original)` when no
//!   frontmatter is present. Strict from 0.0.3: the opening
//!   `---` must sit at byte 0.
//! - [`parse_frontmatter`] — generic parse into
//!   [`yaml_safe::Value`].
//! - [`parse_frontmatter_as::<T>`] — typed parse into any
//!   `Deserialize` struct. Use this when you know the shape up
//!   front; it gives you compile-time checks and IDE help.
//! - Pipeline composition — body feeds straight into
//!   [`process_markdown`]; frontmatter fields drive titling,
//!   metadata, and routing.
//!
//! ## Required feature
//!
//! ```toml
//! [dependencies]
//! mdx-gen = { version = "0.0.3", features = ["yaml_support"] }
//! ```
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example frontmatter_example --features yaml_support
//! ```

use mdx_gen::frontmatter::{
    extract_frontmatter, parse_frontmatter, parse_frontmatter_as,
};
use mdx_gen::{process_markdown, MarkdownOptions};
use serde::{Deserialize, Serialize};

// NB: yaml_safe is a minimal parser — nested block-sequences
// (`tags:\n  - release`) aren't supported; use inline `[…]` flow
// syntax instead. Dates are kept as strings so no type coercion
// is required.
const DOCUMENT: &str = r#"---
title: "Release notes — 0.0.3"
slug: release-notes-003
date: "2026-04-23"
tags: [release, breaking]
draft: false
---

# Release notes

Class-based syntax highlighting shipped in **0.0.3**. See the
migration notes for details.
"#;

#[derive(Debug, Deserialize, Serialize)]
struct FrontMatter {
    title: String,
    slug: String,
    date: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    draft: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 YAML frontmatter");
    println!("───────────────────");

    // ── Step 1: Split frontmatter from body ───────────────────────
    let (yaml, body) = extract_frontmatter(DOCUMENT);
    let yaml = yaml.ok_or("document is missing frontmatter")?;
    println!(
        "    ✅ extracted {} bytes of YAML + {} bytes of Markdown",
        yaml.len(),
        body.len()
    );

    // ── Step 2: Parse untyped (useful for exploration or schemaless flows) ──
    let value = parse_frontmatter(yaml)?;
    let mapping = value
        .as_mapping()
        .ok_or("frontmatter is not a YAML mapping")?;
    println!("    📋 top-level keys:");
    for (k, _) in mapping {
        if let Some(name) = k.as_str() {
            println!("       • {name}");
        }
    }

    // ── Step 3: Parse typed (production path) ─────────────────────
    let fm: FrontMatter = parse_frontmatter_as(yaml)?;
    println!(
        "    ✅ typed parse → title={:?}, slug={:?}, tags={:?}, draft={}",
        fm.title, fm.slug, fm.tags, fm.draft,
    );

    // ── Step 4: Render the body ───────────────────────────────────
    let options = MarkdownOptions::new()
        .with_custom_blocks(false)
        .with_enhanced_tables(false)
        .with_syntax_highlighting(false);
    let html = process_markdown(body, &options)?;
    println!("    ✅ rendered {} bytes of HTML", html.len());
    println!("\n    ── body → html ────────────────");
    println!("{html}");

    // ── Step 5: Strict-mode demo ──────────────────────────────────
    //
    // Leading whitespace disables frontmatter detection since 0.0.3
    // — matching Jekyll / Hugo. The extractor returns the original
    // string unchanged so the caller can pass it through as plain
    // Markdown.
    let leading_ws = "  ---\ntitle: x\n---\n# Body\n";
    let (maybe, _rest) = extract_frontmatter(leading_ws);
    assert!(maybe.is_none());
    println!(
        "\n    ✅ leading whitespace disables detection (returned None)"
    );

    Ok(())
}
