// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Same code highlighted in every bundled syntect theme.
//!
//! Run: `cargo run --example gallery`
//!
//! Output: `target/examples/gallery/{<theme>.html, gallery.html}`.
//! Open `gallery.html`; it embeds one iframe per theme so each
//! stylesheet lives in its own document.

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use std::fs;
use std::path::PathBuf;

use mdx_gen::highlight::SyntectAdapter;
use mdx_gen::{process_markdown, theme_css, MarkdownOptions, Options};

const SAMPLE: &str = r#"```rust
use std::collections::HashMap;

fn count_words(text: &str) -> HashMap<&str, usize> {
    let mut counts = HashMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word).or_insert(0) += 1;
    }
    counts
}

fn main() {
    let counts = count_words("the quick brown fox the");
    println!("the appears {} times", counts["the"]);
}
```
"#;

fn main() {
    support::header("mdx-gen -- gallery");

    let out_dir: PathBuf = support::task("Prepare target dir", || {
        let dir = PathBuf::from("target/examples/gallery");
        fs::create_dir_all(&dir).unwrap();
        dir
    });

    let themes = support::task("Enumerate bundled themes", || {
        SyntectAdapter::available_themes()
    });

    let fragment = support::task("Render the sample once", || {
        let mut comrak_options = Options::default();
        comrak_options.extension.table = true;
        let options = MarkdownOptions::new()
            .with_comrak_options(comrak_options)
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_syntax_highlighting(true)
            .with_unsafe_html(false);
        process_markdown(SAMPLE, &options).unwrap()
    });

    let pages = support::task("Write one page per theme", || {
        let mut pages: Vec<(String, PathBuf)> = Vec::new();
        for name in &themes {
            let css = match theme_css(name) {
                Some(c) => c,
                None => continue,
            };
            let slug = slugify(name);
            let path = out_dir.join(format!("{slug}.html"));
            let html = format!(
                r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>{name}</title>
  <style>
    body {{
      font-family: system-ui, sans-serif;
      margin: 0;
      padding: 0.75rem 1rem;
      background: #fafafa;
    }}
    .label {{
      font-size: 0.75rem;
      color: #666;
      margin: 0 0 0.5rem;
      letter-spacing: 0.05em;
      text-transform: uppercase;
    }}
    pre {{
      margin: 0;
      padding: 0.75rem;
      border-radius: 4px;
      overflow-x: auto;
    }}
    {css}
  </style>
</head>
<body>
  <p class="label">{name}</p>
  {fragment}
</body>
</html>
"#
            );
            fs::write(&path, html).unwrap();
            pages.push((name.to_string(), path));
        }
        pages
    });

    let gallery_path = support::task("Assemble gallery.html", || {
        let frames = pages
            .iter()
            .map(|(_name, path)| {
                let file = path.file_name().unwrap().to_string_lossy();
                format!(
                    r#"      <iframe src="{file}" title="{file}" loading="lazy"></iframe>"#
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let page = format!(
            r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>mdx-gen — theme gallery</title>
  <style>
    body {{
      font-family: system-ui, sans-serif;
      margin: 0;
      padding: 1.5rem;
      background: #f0f0f0;
    }}
    h1 {{ margin: 0 0 1rem; }}
    .grid {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(420px, 1fr));
      gap: 1rem;
    }}
    iframe {{
      width: 100%;
      height: 320px;
      border: 1px solid #d0d0d0;
      border-radius: 4px;
      background: #fff;
    }}
  </style>
</head>
<body>
  <h1>mdx-gen — {n} themes</h1>
  <div class="grid">
{frames}
  </div>
</body>
</html>
"#,
            n = pages.len()
        );

        let path = out_dir.join("gallery.html");
        fs::write(&path, page).unwrap();
        path
    });

    support::task_with_output("Inspect artefacts on disk", || {
        let total_bytes: u64 = pages
            .iter()
            .map(|(_, p)| fs::metadata(p).unwrap().len())
            .sum::<u64>()
            + fs::metadata(&gallery_path).unwrap().len();
        vec![
            format!("themes written : {}", pages.len()),
            format!("gallery page   : {}", gallery_path.display()),
            format!("total bytes    : {total_bytes}"),
        ]
    });

    support::summary(6);
}

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
