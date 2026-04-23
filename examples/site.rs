// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Full pipeline → deployable site directory.
//!
//! Run: `cargo run --example site`
//!
//! Output: `target/examples/site/{index.html, syntax.css}`. Serve
//! the directory with any static file server (`python -m http.server`,
//! `caddy file-server`, etc).

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use std::fs;
use std::path::PathBuf;

use mdx_gen::frontmatter::{extract_frontmatter, parse_frontmatter_as};
use mdx_gen::highlight::SyntectAdapter;
use mdx_gen::{
    process_markdown_with_toc, theme_css, CustomBlockConfig,
    CustomBlockType, Heading, MarkdownOptions, Options,
};
use serde::{Deserialize, Serialize};

// yaml_safe is minimal — inline `[…]` sequences only.
const SOURCE: &str = r##"---
title: "Shipping 0.0.3"
slug: "shipping-003"
date: "2026-04-23"
author: "Sebastien Rousseau"
tags: [release, notes]
theme: "base16-ocean.dark"
---

# Shipping 0.0.3

The headline change is the **class-based syntax highlighter**.
Code blocks now emit `<span class="…">` tokens instead of inline
`style="color:#…"` strings.

<div class="note">Pair the rendered HTML with the generated
<code>syntax.css</code> stylesheet (this page already does).</div>

## Installation

```toml
[dependencies]
mdx-gen = "0.0.3"
```

## Migration

Generate a stylesheet once at build time:

```rust
let css = mdx_gen::theme_css("base16-ocean.dark").unwrap();
std::fs::write("syntax.css", css)?;
```

<div class="warning">If you depended on the previous inline-style
output, update your CSS selectors before upgrading.</div>

## Try it

| Surface       | Method                          |
|:--------------|:--------------------------------|
| One-shot      | `process_markdown`              |
| Streaming     | `process_markdown_to_writer`    |
| With ToC      | `process_markdown_with_toc`     |

<div class="tip">All three accept the same `MarkdownOptions`.</div>
"##;

#[derive(Debug, Deserialize, Serialize)]
struct Post {
    title: String,
    slug: String,
    date: String,
    author: String,
    #[serde(default)]
    tags: Vec<String>,
    /// Optional theme override. Falls back to mdx-gen's default
    /// when absent or unknown.
    #[serde(default)]
    theme: Option<String>,
}

fn main() {
    support::header("mdx-gen -- site");

    let out_dir: PathBuf =
        support::task("Prepare output directory", || {
            let dir = PathBuf::from("target/examples/site");
            fs::create_dir_all(&dir).unwrap();
            dir
        });

    let (yaml, body) =
        support::task("Split frontmatter from body", || {
            let (yaml, body) = extract_frontmatter(SOURCE);
            (
                yaml.expect("missing frontmatter").to_owned(),
                body.to_owned(),
            )
        });

    let post: Post = support::task("Parse Post struct", || {
        parse_frontmatter_as(&yaml).unwrap()
    });

    let theme_name: String = support::task("Resolve theme", || {
        let requested = post.theme.as_deref();
        SyntectAdapter::new(requested).theme_name().to_owned()
    });

    let css_bytes = support::task("Generate syntax.css", || {
        let css = theme_css(&theme_name).expect("theme exists");
        let path = out_dir.join("syntax.css");
        fs::write(&path, &css).unwrap();
        css.len()
    });

    let (body_html, toc) =
        support::task("Render body + collect ToC", || {
            let mut comrak_options = Options::default();
            comrak_options.extension.table = true;
            comrak_options.extension.strikethrough = true;
            comrak_options.extension.autolink = true;

            // Custom block titles match the Post tone.
            let block_config = CustomBlockConfig::new()
                .with_title(CustomBlockType::Note, "Note")
                .with_title(CustomBlockType::Warning, "Heads up")
                .with_title(CustomBlockType::Tip, "Tip");

            let options = MarkdownOptions::new()
                .with_comrak_options(comrak_options)
                .with_custom_blocks(true)
                .with_custom_block_config(block_config)
                .with_enhanced_tables(true)
                .with_syntax_highlighting(true)
                .with_header_ids("")
                .with_unsafe_html(false);

            process_markdown_with_toc(&body, &options).unwrap()
        });

    let html_bytes = support::task("Assemble index.html", || {
        let page = render_page(&post, &toc, &body_html);
        let path = out_dir.join("index.html");
        fs::write(&path, &page).unwrap();
        page.len()
    });

    support::task_with_output("Inspect deployable directory", || {
        let entries: Vec<_> = fs::read_dir(&out_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                let bytes = e.metadata().unwrap().len();
                format!("{name:<14} {bytes} bytes")
            })
            .collect();
        let mut lines = vec![format!("dir: {}", out_dir.display())];
        lines.extend(entries);
        lines.push(format!("theme  : {theme_name}"));
        lines.push(format!("headings: {}", toc.len()));
        lines.push(format!("css     : {css_bytes} bytes (written)"));
        lines.push(format!("html    : {html_bytes} bytes (written)"));
        lines
    });

    support::summary(8);
}

/// Wraps the rendered body in a styled HTML document with a
/// sidebar outline + reference to the generated `syntax.css`.
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

    let tags = post
        .tags
        .iter()
        .map(|t| format!(r#"<li class="tag">{t}</li>"#))
        .collect::<Vec<_>>()
        .join("");

    format!(
        r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title}</title>
  <meta name="author" content="{author}">
  <meta name="description" content="{title}">
  <meta property="article:published_time" content="{date}">
  <link rel="stylesheet" href="syntax.css">
  <style>
    :root {{
      --fg: #1a1a1a;
      --muted: #666;
      --bg: #ffffff;
      --border: #e5e5e5;
      --accent: #0066cc;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      font: 16px/1.6 system-ui, sans-serif;
      color: var(--fg);
      background: var(--bg);
    }}
    .layout {{
      display: grid;
      grid-template-columns: minmax(220px, 280px) 1fr;
      gap: 2.5rem;
      max-width: 80rem;
      margin: 0 auto;
      padding: 2rem 1.5rem;
    }}
    aside {{
      position: sticky;
      top: 2rem;
      align-self: start;
      font-size: 0.9rem;
      color: var(--muted);
    }}
    aside ol {{ list-style: none; padding: 0; margin: 0.5rem 0 0; }}
    aside a {{ color: inherit; text-decoration: none; }}
    aside a:hover {{ color: var(--accent); }}
    article header h1 {{ margin: 0; }}
    article header .meta {{
      color: var(--muted);
      margin: 0.25rem 0 1.5rem;
      font-size: 0.9rem;
    }}
    .tags {{ list-style: none; padding: 0; display: inline; }}
    .tag {{
      display: inline-block;
      padding: 0.05rem 0.5rem;
      margin-right: 0.25rem;
      background: #f0f0f0;
      border-radius: 3px;
      font-size: 0.75rem;
    }}
    pre {{
      padding: 0.75rem 1rem;
      border-radius: 4px;
      overflow-x: auto;
      background: #2b303b;
    }}
    code {{ font-size: 0.9em; }}
    .alert {{
      padding: 0.75rem 1rem;
      margin: 1rem 0;
      border-left: 3px solid var(--accent);
      background: #f6f8fa;
      border-radius: 3px;
    }}
    .table-responsive {{ overflow-x: auto; }}
    table {{ border-collapse: collapse; }}
    th, td {{ padding: 0.4rem 0.75rem; border-bottom: 1px solid var(--border); }}
  </style>
</head>
<body>
  <div class="layout">
    <aside>
      <strong>Outline</strong>
      <ol>
{outline}
      </ol>
    </aside>
    <article>
      <header>
        <h1>{title}</h1>
        <p class="meta">
          <time datetime="{date}">{date}</time> · {author}
          · <ul class="tags">{tags}</ul>
        </p>
      </header>
{body}
    </article>
  </div>
</body>
</html>
"##,
        title = post.title,
        author = post.author,
        date = post.date,
    )
}
