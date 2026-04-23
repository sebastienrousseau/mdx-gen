// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Blog post pipeline — frontmatter + ToC + highlighting + sanitize.
//!
//! Run: `cargo run --example blog`

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use std::fs;
use std::path::PathBuf;

use mdx_gen::frontmatter::{extract_frontmatter, parse_frontmatter_as};
use mdx_gen::{
    process_markdown_with_toc, Heading, MarkdownOptions, Options,
};
use serde::{Deserialize, Serialize};

// yaml_safe is minimal — inline `[…]` sequences only; no block
// `- tags` under a mapping key.
const SOURCE: &str = r#"---
title: "Shipping 0.0.3"
slug: "shipping-003"
date: "2026-04-23"
author: "Sebastien Rousseau"
tags: [release, notes]
---

# Shipping 0.0.3

0.0.3 switches the syntax highlighter to class-based output.

## What changed

The highlighter now emits `<span class="…">` tokens instead of
inline `style="color:#…"`.

## How to migrate

Generate a stylesheet once at build time and serve it alongside
your HTML.

## Anything else?

That's it.
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

fn main() {
    support::header("mdx-gen -- blog");

    let (yaml, body) =
        support::task("Split frontmatter from body", || {
            let (yaml, body) = extract_frontmatter(SOURCE);
            (
                yaml.expect("missing frontmatter").to_owned(),
                body.to_owned(),
            )
        });

    let post: Post = support::task("Parse frontmatter (typed)", || {
        parse_frontmatter_as(&yaml).unwrap()
    });

    support::task_with_output("Inspect post metadata", || {
        vec![
            format!("title : {}", post.title),
            format!("slug  : {}", post.slug),
            format!("date  : {}", post.date),
            format!("author: {}", post.author),
            format!("tags  : {:?}", post.tags),
        ]
    });

    let (html, toc) =
        support::task("Render body + collect ToC", || {
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

            process_markdown_with_toc(&body, &options).unwrap()
        });

    support::task_with_output("Inspect outline", || {
        toc.iter()
            .map(|h| {
                let indent =
                    "  ".repeat(h.level.saturating_sub(1) as usize);
                format!(
                    "{indent}H{lvl} {text} (#{id})",
                    lvl = h.level,
                    text = h.text,
                    id = h.id
                )
            })
            .collect()
    });

    let out_path = support::task("Assemble page + write file", || {
        let out_dir: PathBuf = PathBuf::from("target/examples");
        fs::create_dir_all(&out_dir).unwrap();
        let out_path = out_dir.join("blog.html");
        fs::write(&out_path, render_page(&post, &toc, &html)).unwrap();
        out_path
    });

    support::task_with_output("Verify artefact on disk", || {
        let bytes = fs::metadata(&out_path).unwrap().len();
        vec![
            format!("path  : {}", out_path.display()),
            format!("bytes : {bytes}"),
        ]
    });

    support::summary(6);
}

/// Wraps the rendered body in a minimal HTML document with `<head>`
/// populated from `post` metadata and a sidebar outline built from
/// the returned [`Heading`]s.
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
