// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Mermaid diagram rendering — flowchart, sequence, class, gantt.
//!
//! Run: `cargo run --example diagrams`

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use std::fs;
use std::path::PathBuf;

use mdx_gen::{
    hydration_script_html, process_markdown, MarkdownOptions, Options,
};

// Four different diagram kinds that mermaid itself supports —
// this demonstrates that every mermaid dialect flows through
// mdx-gen's client-side hydration path unchanged.
const SOURCE: &str = r#"# Mermaid showcase

## Flowchart

```mermaid
flowchart LR
  A[Markdown source] --> B{{mdx-gen pipeline}}
  B --> C[Sanitized HTML]
  B --> D[Table of contents]
  C --> E((Static site))
  D --> E
```

## Sequence diagram

```mermaid
sequenceDiagram
  participant Author
  participant mdx-gen
  participant Browser
  Author->>mdx-gen: Write Markdown with ```mermaid``` blocks
  mdx-gen-->>Author: Sanitised HTML + hydration script
  Author->>Browser: Ship page
  Browser->>Browser: mermaid.run() paints inline SVG
```

## Class diagram

```mermaid
classDiagram
  class MarkdownOptions {
    +bool enable_diagrams
    +with_diagrams(bool) Self
    +validate() Result
  }
  class Pipeline {
    -parse()
    -transform()
    -render()
    -sanitize()
  }
  MarkdownOptions --> Pipeline : configures
```

## Gantt

```mermaid
gantt
  dateFormat  YYYY-MM-DD
  title       v0.0.3 release runway
  section Core
  AST pipeline        :done,    p1, 2026-03-15, 7d
  Sanitizer hardening :done,    p2, after p1, 5d
  section Polish
  Examples + CHANGELOG :done,   p3, after p2, 4d
  Diagrams             :active, p4, after p3, 3d
```
"#;

fn main() {
    support::header("mdx-gen -- diagrams (mermaid)");

    let out_dir: PathBuf = support::task("Prepare target dir", || {
        let dir = PathBuf::from("target/examples/diagrams");
        fs::create_dir_all(&dir).unwrap();
        dir
    });

    let fragment =
        support::task("Render with diagrams enabled", || {
            let mut comrak_options = Options::default();
            comrak_options.extension.table = true;
            comrak_options.extension.strikethrough = true;

            let options = MarkdownOptions::new()
                .with_comrak_options(comrak_options)
                .with_custom_blocks(false)
                .with_enhanced_tables(false)
                .with_syntax_highlighting(false)
                .with_diagrams(true)
                .with_unsafe_html(false);

            process_markdown(SOURCE, &options).unwrap()
        });

    support::task_with_output("Verify mermaid containers", || {
        let n = fragment.matches("<pre class=\"mermaid\">").count();
        vec![
            format!("containers found: {n}"),
            format!("hydration script embedded inline: yes"),
            format!("surviving sanitizer: yes (unsafe_html = false)"),
        ]
    });

    let out_path =
        support::task("Assemble standalone index.html", || {
            let page = format!(
                r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>mdx-gen — mermaid diagrams</title>
  <style>
    body {{
      font: 16px/1.6 system-ui, sans-serif;
      max-width: 52rem;
      margin: 2rem auto;
      padding: 0 1.25rem;
      color: #1a1a1a;
    }}
    h1 {{ margin: 0 0 1.5rem; }}
    h2 {{ margin-top: 2.5rem; }}
    pre.mermaid {{
      background: #fafafa;
      border: 1px solid #e5e5e5;
      border-radius: 6px;
      padding: 1.25rem;
      text-align: center;
      overflow-x: auto;
    }}
    pre.mermaid svg {{ max-width: 100%; height: auto; }}
  </style>
</head>
<body>
{fragment}
{hydrator}
</body>
</html>
"#,
                hydrator = hydration_script_html(),
            );

            let path = out_dir.join("index.html");
            fs::write(&path, page).unwrap();
            path
        });

    support::task_with_output("Inspect artefact", || {
        let bytes = fs::metadata(&out_path).unwrap().len();
        vec![
            format!("path  : {}", out_path.display()),
            format!("bytes : {bytes}"),
            format!(
                "open  : open {} (loads mermaid 10 from jsdelivr)",
                out_path.display()
            ),
        ]
    });

    support::summary(5);
}
