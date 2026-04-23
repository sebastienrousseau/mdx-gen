// Copyright © 2024 - 2026 MDX Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! # Quickstart — Every feature wired together
//!
//! ## What this example is
//!
//! A single document that exercises the full mdx-gen pipeline:
//! CommonMark + GFM extensions, custom block alerts, enhanced tables,
//! class-based syntax highlighting, header ids, and the sanitiser.
//! Useful as a **visual integration test** — if this runs clean you
//! know every moving part is talking to the others.
//!
//! ## What it demonstrates
//!
//! - **Builder composition** — chaining [`MarkdownOptions`] toggles
//!   for each feature.
//! - **Enhanced tables** — responsive wrapper + alignment classes
//!   applied at the AST level.
//! - **Custom blocks** — the `<div class="note">` / `warning` / `tip`
//!   shorthand transformed into Bootstrap-shaped alert markup.
//! - **Syntax highlighting** — class-based spans ready to be paired
//!   with a stylesheet (see the `styling` example).
//! - **Header ids** — anchor ids so headings become jump targets.
//! - **Sanitiser** — dangerous raw HTML stripped while the safe
//!   structural tags we emit survive.
//!
//! ## How this differs from `basic`
//!
//! `basic` is the minimum-wiring starter — one call, defaults,
//! prints the HTML. `quickstart` turns **every** feature on at once
//! so you can see how they compose; use it as a template when you
//! want the kitchen-sink configuration.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example quickstart
//! ```

use mdx_gen::{process_markdown, MarkdownOptions, Options};

const SAMPLE: &str = r#"# Release notes

## Highlights

- GFM tables
- **bold** and *italic*
- Inline `code` and ~~strikethrough~~

<div class="note">Custom blocks render as Bootstrap alerts.</div>
<div class="warning">Malicious raw HTML is stripped by the sanitiser.</div>

## Metrics

| Feature           | Status |
|:------------------|:------:|
| Tables            |   ✓    |
| Custom blocks     |   ✓    |
| Header ids        |   ✓    |
| Class-based spans |   ✓    |

## Code

```rust
fn main() {
    println!("Hello, mdx-gen!");
}
```

<script>alert('xss')</script>
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Full pipeline walkthrough");
    println!("────────────────────────────");

    // ── Step 1: Wire comrak extensions ────────────────────────────
    let mut comrak_options = Options::default();
    comrak_options.extension.strikethrough = true;
    comrak_options.extension.table = true;
    comrak_options.extension.tasklist = true;
    comrak_options.extension.autolink = true;

    // ── Step 2: Compose MarkdownOptions ───────────────────────────
    let options = MarkdownOptions::new()
        .with_comrak_options(comrak_options)
        .with_custom_blocks(true)
        .with_enhanced_tables(true)
        .with_syntax_highlighting(true)
        .with_header_ids("user-content-")
        .with_unsafe_html(false);

    // ── Step 3: Render ────────────────────────────────────────────
    let html = process_markdown(SAMPLE, &options)?;

    // ── Step 4: Quick sanity checks ───────────────────────────────
    let checks: Vec<(&str, bool)> = vec![
        (
            "responsive table wrapper",
            html.contains("table-responsive"),
        ),
        (
            "alert markup from custom block",
            html.contains("alert alert-info"),
        ),
        (
            "header id with prefix",
            html.contains("id=\"user-content-highlights\""),
        ),
        (
            "class-based highlighter span",
            html.contains("<span class=\""),
        ),
        ("<script> stripped by sanitizer", !html.contains("<script>")),
    ];

    for (label, ok) in &checks {
        let tick = if *ok { "✅" } else { "❌" };
        println!("    {tick} {label}");
    }

    println!("\n    📄 HTML ({} bytes):\n", html.len());
    println!("{html}");

    let all_ok = checks.iter().all(|(_, ok)| *ok);
    if all_ok {
        println!("\n    🎉 Pipeline produced the expected shape");
        Ok(())
    } else {
        Err("one or more pipeline invariants failed".into())
    }
}
