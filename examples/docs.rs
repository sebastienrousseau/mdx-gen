// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Documentation page — header ids + ToC + alert blocks + tables.
//!
//! Run: `cargo run --example docs`

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use std::fs;
use std::path::PathBuf;

use mdx_gen::{
    process_markdown_with_toc, CustomBlockType, Heading,
    MarkdownOptions, Options,
};

const SOURCE: &str = r##"# API reference

Welcome to the mdx-gen API reference. This page shows how the
library renders a realistic documentation source.

## Installation

Add the crate:

```toml
[dependencies]
mdx-gen = "0.0.3"
```

<div class="note">The default feature set covers most needs.</div>

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

<div class="important">Match on the variant — <code>.unwrap()</code>
panics on oversized input or IO failures.</div>

### Common variants

- `InputTooLarge`
- `InvalidOptionsError`
- `IoError`

<div class="caution">`InputTooLarge` fires before parsing, so a
hostile 2 GB input cannot exhaust memory with the default 1 MiB
cap in place.</div>
"##;

fn main() {
    support::header("mdx-gen -- docs");

    let options = support::task("Build docs-site options", || {
        let mut comrak_options = Options::default();
        comrak_options.extension.table = true;
        comrak_options.extension.strikethrough = true;
        comrak_options.extension.autolink = true;

        MarkdownOptions::new()
            .with_comrak_options(comrak_options)
            .with_custom_blocks(true)
            .with_enhanced_tables(true)
            .with_syntax_highlighting(true)
            .with_header_ids("user-content-")
            .with_unsafe_html(false)
    });

    let (html, toc) =
        support::task("Render body + collect ToC", || {
            process_markdown_with_toc(SOURCE, &options).unwrap()
        });

    support::task_with_output(
        "Inspect custom-block alert family",
        || {
            [
                CustomBlockType::Note,
                CustomBlockType::Warning,
                CustomBlockType::Tip,
                CustomBlockType::Info,
                CustomBlockType::Important,
                CustomBlockType::Caution,
            ]
            .iter()
            .map(|b| {
                format!(
                    "{:<10} class={:<18} title={}",
                    format!("{b:?}"),
                    b.get_alert_class(),
                    b.get_title()
                )
            })
            .collect()
        },
    );

    support::task_with_output("Inspect outline", || {
        toc.iter()
            .map(|h| {
                let indent =
                    "  ".repeat(h.level.saturating_sub(1) as usize);
                format!("{indent}H{} {} (#{})", h.level, h.text, h.id)
            })
            .collect()
    });

    support::task("Verify every ToC id is present in HTML", || {
        for h in &toc {
            let needle = format!("id=\"{}\"", h.id);
            assert!(html.contains(&needle), "missing anchor {needle}");
        }
    });

    let out_path = support::task("Write docs.html", || {
        let out_dir: PathBuf = PathBuf::from("target/examples");
        fs::create_dir_all(&out_dir).unwrap();
        let out_path = out_dir.join("docs.html");
        fs::write(&out_path, render_page(&toc, &html)).unwrap();
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
