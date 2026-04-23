// Copyright © 2024 - 2026 MDX Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! # Docs — A documentation page with anchors, alerts, and outline
//!
//! ## What this example is
//!
//! A documentation-site scenario: header anchors with a GitHub-
//! style `user-content-` prefix, an outline sidebar built from a
//! nested [`Heading`] list, the custom-block alert family
//! (note / warning / tip / info / important / caution), enhanced
//! tables with alignment, and class-based syntax highlighting —
//! all in one rendered page.
//!
//! ## What it demonstrates
//!
//! - [`process_markdown_with_toc`] — HTML + structured outline
//!   from a single AST pass.
//! - [`MarkdownOptions::with_header_ids`] with a prefix — matches
//!   the anchor scheme GitHub uses in its own rendered Markdown.
//! - [`MarkdownOptions::with_custom_blocks`] + the
//!   [`CustomBlockType`] family — opinionated Bootstrap-shaped
//!   alert markup.
//! - [`MarkdownOptions::with_enhanced_tables`] — responsive
//!   wrapper + alignment classes.
//! - [`Heading`] rendered as a nested `<ul>` for a sidebar.
//!
//! ## When to use this pattern
//!
//! Project handbooks, API references, static-site docs, README
//! pages rendered in a CMS — anywhere the reader wants both a
//! clickable outline and rich inline callouts.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example docs
//! ```
//!
//! The assembled page is written to `target/examples/docs.html`.

use std::fs;
use std::path::PathBuf;

use mdx_gen::{
    process_markdown_with_toc, CustomBlockType, Heading,
    MarkdownOptions, Options,
};

const SAMPLE: &str = r##"# API reference

Welcome to the mdx-gen API reference. This page shows how the
library renders a realistic documentation source.

## Installation

Add the crate:

```toml
[dependencies]
mdx-gen = "0.0.3"
```

<div class="note">The default feature set includes syntax
highlighting. Disable it with <code>default-features = false</code>
if you don't need it.</div>

## Rendering

```rust
use mdx_gen::{process_markdown, MarkdownOptions};
let html = process_markdown("# hello", &MarkdownOptions::default())?;
```

<div class="tip">Call <code>process_markdown_with_toc</code>
instead when you need an outline alongside the HTML.</div>

## Options

| Option              | Default | Notes                          |
|:--------------------|:-------:|:-------------------------------|
| `custom_blocks`     |  true   | Bootstrap-shaped alerts        |
| `enhanced_tables`   |  true   | Responsive wrapper + alignment |
| `syntax_highlighting` | true  | Class-based spans              |

<div class="warning">Enhanced tables require the comrak table
extension. The builder validates this before rendering.</div>

## Errors

Every fallible call returns `Result<_, MarkdownError>`.

<div class="important">Always match on the variant — swallowing
errors with <code>.unwrap()</code> will panic on oversized input
or IO failures.</div>

### Common variants

- `InputTooLarge` — exceeds the configured cap.
- `InvalidOptionsError` — builder validation tripped.
- `IoError` — propagated from the streaming writer.

<div class="caution">`InputTooLarge` fires before parsing, so a
hostile 2 GB input cannot exhaust memory with the default 1 MiB
cap in place.</div>
"##;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Docs page pipeline");
    println!("─────────────────────");

    // ── Step 1: Configure the pipeline ────────────────────────────
    let mut comrak_options = Options::default();
    comrak_options.extension.table = true;
    comrak_options.extension.strikethrough = true;
    comrak_options.extension.autolink = true;

    let options = MarkdownOptions::new()
        .with_comrak_options(comrak_options)
        .with_custom_blocks(true)
        .with_enhanced_tables(true)
        .with_syntax_highlighting(true)
        .with_header_ids("user-content-")
        .with_unsafe_html(false);

    // ── Step 2: Render + collect outline ──────────────────────────
    let (body_html, toc) = process_markdown_with_toc(SAMPLE, &options)?;
    println!(
        "    ✅ rendered {} bytes of HTML + {} headings",
        body_html.len(),
        toc.len()
    );

    // ── Step 3: Show the alert family at a glance ─────────────────
    println!("    ℹ️  Custom block alert classes:");
    for b in [
        CustomBlockType::Note,
        CustomBlockType::Warning,
        CustomBlockType::Tip,
        CustomBlockType::Info,
        CustomBlockType::Important,
        CustomBlockType::Caution,
    ] {
        println!(
            "       {:<10} class={:<18} title={}",
            format!("{b:?}"),
            b.get_alert_class(),
            b.get_title()
        );
    }

    // ── Step 4: Sanity-check every ToC id in the HTML ─────────────
    let mut all_ok = true;
    for h in &toc {
        let needle = format!("id=\"{}\"", h.id);
        if !body_html.contains(&needle) {
            println!("    ❌ missing {needle}");
            all_ok = false;
        }
    }
    if all_ok {
        println!(
            "    ✅ every ToC id ({}) appears in the HTML",
            toc.len()
        );
    }

    // ── Step 5: Compose the final page ────────────────────────────
    let out_dir: PathBuf = PathBuf::from("target/examples");
    fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join("docs.html");
    fs::write(&out_path, render_page(&toc, &body_html))?;
    println!("    ✅ wrote {}", out_path.display());

    Ok(())
}

/// Builds a minimal documentation shell around the rendered body,
/// with the outline laid out as a nested list on the left.
fn render_page(toc: &[Heading], body: &str) -> String {
    let outline = toc
        .iter()
        .map(|h| {
            let indent =
                "  ".repeat(h.level.saturating_sub(1) as usize);
            format!(
                "{indent}<li><a href=\"#{id}\">{text}</a></li>",
                id = h.id,
                text = h.text
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>mdx-gen docs demo</title>
</head>
<body>
  <aside><strong>Outline</strong><ul>
{outline}
  </ul></aside>
  <main>
{body}
  </main>
</body>
</html>
"#
    )
}
