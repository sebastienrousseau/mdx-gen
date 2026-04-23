// Copyright © 2024 - 2026 MDX Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! # Theme CSS Example — Generate a stylesheet for the highlighter
//!
//! ## What this example is
//!
//! Since 0.0.3 the syntax highlighter emits **class-based** spans
//! (`<span class="source rust">…</span>`) instead of inline
//! `style="color:#…"`. That means your rendered HTML needs a
//! stylesheet to show colour — this example generates one file
//! per built-in theme using [`theme_css`] so you can pick one (or
//! ship several and switch at runtime with a `<link>` tag).
//!
//! ## What it demonstrates
//!
//! - [`theme_css`] — returns the CSS for a named syntect theme,
//!   or `None` if the theme isn't bundled.
//! - [`SyntectAdapter::available_themes`] — enumerates every
//!   built-in theme name, useful for the picker UI.
//! - Pairing with rendered output — the example also runs
//!   [`process_markdown`] over a small code block and writes both
//!   artefacts into `target/examples/` so you can open the HTML
//!   with the CSS next to it and see the result.
//!
//! ## When to use this pattern
//!
//! Build-time: call this once during your site generator's build
//! step and serve the resulting `.css` file alongside your HTML.
//! Runtime: call it lazily the first time a page needs colouring
//! and cache the result.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example theme_css_example
//! ```
//!
//! Output lands in `target/examples/theme_css_example/`.
//!
//! [`SyntectAdapter::available_themes`]: mdx_gen::highlight::SyntectAdapter::available_themes

use std::fs;
use std::path::PathBuf;

use mdx_gen::highlight::SyntectAdapter;
use mdx_gen::{process_markdown, theme_css, MarkdownOptions, Options};

const DEMO_MARKDOWN: &str = r#"# Theme demo

```rust
fn main() {
    println!("Hello, mdx-gen!");
}
```

```python
def greet(name: str) -> str:
    return f"Hello, {name}!"
```
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Syntect theme → CSS");
    println!("──────────────────────");

    // ── Step 1: Pick an output directory ──────────────────────────
    let out_dir: PathBuf =
        PathBuf::from("target/examples/theme_css_example");
    fs::create_dir_all(&out_dir)?;

    // ── Step 2: List every built-in theme ─────────────────────────
    let themes = SyntectAdapter::available_themes();
    println!("    🎨 {} bundled themes", themes.len());
    for name in &themes {
        println!("       • {name}");
    }

    // ── Step 3: Write one .css file per theme ─────────────────────
    let mut written = 0usize;
    for name in &themes {
        if let Some(css) = theme_css(name) {
            let path = out_dir.join(format!("{}.css", slugify(name)));
            fs::write(&path, css)?;
            written += 1;
        }
    }
    println!(
        "    ✅ wrote {written} stylesheet(s) to {}",
        out_dir.display()
    );

    // ── Step 4: Write an HTML sample using the default theme ──────
    let mut comrak_options = Options::default();
    comrak_options.extension.table = true;
    let options = MarkdownOptions::new()
        .with_comrak_options(comrak_options)
        .with_custom_blocks(false)
        .with_enhanced_tables(false)
        .with_syntax_highlighting(true)
        .with_unsafe_html(false);

    let rendered_fragment = process_markdown(DEMO_MARKDOWN, &options)?;
    let default_theme =
        SyntectAdapter::new(None).theme_name().to_owned();

    let css_href = format!("{}.css", slugify(&default_theme));
    let html = format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>mdx-gen theme_css demo</title>
  <link rel="stylesheet" href="{css_href}">
  <style>body {{ font-family: system-ui, sans-serif; max-width: 40rem; margin: 2rem auto; padding: 0 1rem; }}</style>
</head>
<body>
{rendered_fragment}
</body>
</html>
"#
    );

    let html_path = out_dir.join("index.html");
    fs::write(&html_path, html)?;
    println!(
        "    ✅ wrote demo HTML → {} (paired with {default_theme}.css)",
        html_path.display()
    );
    println!(
        "    ↪ open it with `open {}` to see the highlighter in action",
        html_path.display()
    );

    Ok(())
}

/// Converts a theme name like `"Solarized (dark)"` to a filesystem-
/// safe token like `"solarized-dark"`.
fn slugify(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_dash = false;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.extend(c.to_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_owned()
}
