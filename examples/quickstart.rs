// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Every feature wired together — integration smoke test.
//!
//! Run: `cargo run --example quickstart`

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use mdx_gen::{process_markdown, MarkdownOptions, Options};

const SOURCE: &str = r#"# Release notes

## Highlights

- GFM tables
- **bold** and *italic*
- Inline `code` and ~~strikethrough~~

<div class="note">Custom blocks render as Bootstrap alerts.</div>
<div class="warning">Malicious raw HTML is stripped by the sanitiser.</div>

## Metrics

| Feature           | Status |
|:------------------|:------:|
| Tables            |   ok   |
| Custom blocks     |   ok   |
| Header ids        |   ok   |
| Class-based spans |   ok   |

## Code

```rust
fn main() {
    println!("Hello, mdx-gen!");
}
```

<script>alert('xss')</script>
"#;

fn main() {
    support::header("mdx-gen -- quickstart");

    let options = support::task("Build kitchen-sink options", || {
        let mut comrak_options = Options::default();
        comrak_options.extension.strikethrough = true;
        comrak_options.extension.table = true;
        comrak_options.extension.tasklist = true;
        comrak_options.extension.autolink = true;

        MarkdownOptions::new()
            .with_comrak_options(comrak_options)
            .with_custom_blocks(true)
            .with_enhanced_tables(true)
            .with_syntax_highlighting(true)
            .with_header_ids("user-content-")
            .with_unsafe_html(false)
    });

    let html = support::task("Render pipeline", || {
        process_markdown(SOURCE, &options).unwrap()
    });

    support::task_with_output("Verify pipeline invariants", || {
        vec![
            format!(
                "responsive table wrapper    : {}",
                html.contains("table-responsive")
            ),
            format!(
                "alert div from custom block : {}",
                html.contains("alert alert-info")
            ),
            format!(
                "prefixed header id          : {}",
                html.contains("id=\"user-content-highlights\"")
            ),
            format!(
                "class-based highlighter span: {}",
                html.contains("<span class=\"")
            ),
            format!(
                "<script> stripped           : {}",
                !html.contains("<script>")
            ),
        ]
    });

    support::summary(3);
}
