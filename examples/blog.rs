// Copyright © 2024 - 2026 MDX Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! # Blog — A blog post with frontmatter, ToC, and highlighting
//!
//! ## What this example is
//!
//! The end-to-end shape of a realistic blog pipeline:
//!
//! 1. Peel YAML frontmatter off the top of the source document.
//! 2. Parse it into a typed [`Post`] struct for metadata
//!    (title, slug, date, tags).
//! 3. Render the body with a ToC, class-based syntax highlighting,
//!    and the sanitiser on.
//! 4. Assemble a standalone HTML page with the frontmatter fields
//!    wired into `<title>`, `<meta>`, and an outline sidebar.
//!
//! ## What it demonstrates
//!
//! - [`extract_frontmatter`] + [`parse_frontmatter_as`] — split
//!   the YAML header from the Markdown body and deserialise it
//!   into a `#[derive(Deserialize)]` struct in one step.
//! - [`process_markdown_with_toc`] — body + ToC from a single
//!   AST pass.
//! - [`MarkdownOptions::with_header_ids`] — anchor ids that
//!   match [`Heading::id`] so the outline links land on the
//!   rendered headings.
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
//! cargo run --example blog --features yaml_support
//! ```
//!
//! The full blog post is written to `target/examples/blog.html`.

use std::fs;
use std::path::PathBuf;

use mdx_gen::frontmatter::{extract_frontmatter, parse_frontmatter_as};
use mdx_gen::{
    process_markdown_with_toc, Heading, MarkdownOptions, Options,
};
use serde::{Deserialize, Serialize};

// Inline sequences only — yaml_safe is a minimal parser that does
// not accept block-style `- tags` under a mapping key.
const SOURCE: &str = r#"---
title: "Shipping 0.0.3"
slug: "shipping-003"
date: "2026-04-23"
author: "Sebastien Rousseau"
tags: [release, notes]
---

# Shipping 0.0.3

0.0.3 switches the syntax highlighter to class-based output. This
post walks through **what changed** and *why*.

## What changed

The highlighter now emits `<span class="…">` tokens instead of
inline `style="color:#…"`.

## How to migrate

Generate a stylesheet once at build time:

```rust
let css = mdx_gen::theme_css("base16-ocean.dark").unwrap();
std::fs::write("syntax.css", css)?;
```

## Anything else?

Nope — just drop the CSS alongside your HTML.
"#;

#[derive(Debug, Deserialize, Serialize)]
struct Post {
    title: String,
    slug: String,
    date: String,
    author: String,
    #[serde(default)]
    tags: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Blog post pipeline");
    println!("─────────────────────");

    // ── Step 1: Split YAML from Markdown ──────────────────────────
    let (yaml, body) = extract_frontmatter(SOURCE);
    let yaml = yaml.ok_or("missing frontmatter")?;
    let post: Post = parse_frontmatter_as(yaml)?;
    println!(
        "    ✅ {title} — {date} by {author}",
        title = post.title,
        date = post.date,
        author = post.author
    );
    println!("    🏷️  tags: {:?}", post.tags);

    // ── Step 2: Render body + ToC ─────────────────────────────────
    let mut comrak_options = Options::default();
    comrak_options.extension.table = true;
    comrak_options.extension.strikethrough = true;

    let options = MarkdownOptions::new()
        .with_comrak_options(comrak_options)
        .with_custom_blocks(false)
        .with_enhanced_tables(true)
        .with_syntax_highlighting(true)
        .with_header_ids("")
        .with_unsafe_html(false);

    let (body_html, toc) = process_markdown_with_toc(body, &options)?;
    println!(
        "    ✅ rendered {} bytes of HTML + {} headings",
        body_html.len(),
        toc.len()
    );

    // ── Step 3: Compose the final page ────────────────────────────
    let out_dir: PathBuf = PathBuf::from("target/examples");
    fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join("blog.html");

    let page = render_page(&post, &toc, &body_html);
    fs::write(&out_path, page)?;
    println!("    ✅ wrote {}", out_path.display());

    Ok(())
}

/// Wraps the rendered body in a minimal HTML document with a
/// `<head>` populated from `post` metadata and a sidebar outline
/// built from the returned [`Heading`]s.
fn render_page(post: &Post, toc: &[Heading], body: &str) -> String {
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
  <title>{title}</title>
  <meta name="author" content="{author}">
  <meta name="description" content="{title}">
  <meta property="article:published_time" content="{date}">
</head>
<body>
  <nav><strong>Outline</strong><ol>
{outline}
  </ol></nav>
  <article>
    <header>
      <h1>{title}</h1>
      <p><time datetime="{date}">{date}</time> · {author}</p>
    </header>
{body}
  </article>
</body>
</html>
"#,
        title = post.title,
        author = post.author,
        date = post.date,
    )
}
