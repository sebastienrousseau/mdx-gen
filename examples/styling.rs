// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Highlighter stylesheet generation + standalone highlighter.
//!
//! Run: `cargo run --example styling`

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use std::fs;
use std::path::PathBuf;

use mdx_gen::highlight::SyntectAdapter;
use mdx_gen::{
    apply_syntax_highlighting, process_markdown, theme_css,
    MarkdownOptions, Options,
};

const DEMO: &str = r#"# Theme demo

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

fn main() {
    support::header("mdx-gen -- styling");

    let out_dir: PathBuf = support::task("Prepare target dir", || {
        let dir = PathBuf::from("target/examples/styling");
        fs::create_dir_all(&dir).unwrap();
        dir
    });

    let themes = support::task("Enumerate bundled themes", || {
        SyntectAdapter::available_themes()
    });

    support::task_with_output("Inspect theme list", || {
        themes.iter().map(|t| format!("• {t}")).collect()
    });

    let written =
        support::task("Write one stylesheet per theme", || {
            let mut count = 0usize;
            for name in &themes {
                if let Some(css) = theme_css(name) {
                    let path =
                        out_dir.join(format!("{}.css", slugify(name)));
                    fs::write(&path, css).unwrap();
                    count += 1;
                }
            }
            count
        });

    let default_theme = support::task("Resolve default theme", || {
        SyntectAdapter::new(None).theme_name().to_owned()
    });

    let html_path = support::task(
        "Render demo HTML paired with CSS",
        || {
            let mut comrak_options = Options::default();
            comrak_options.extension.table = true;
            let options = MarkdownOptions::new()
                .with_comrak_options(comrak_options)
                .with_custom_blocks(false)
                .with_enhanced_tables(false)
                .with_syntax_highlighting(true)
                .with_unsafe_html(false);
            let fragment = process_markdown(DEMO, &options).unwrap();
            let css_href = format!("{}.css", slugify(&default_theme));
            let page = format!(
                r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>mdx-gen styling demo</title>
  <link rel="stylesheet" href="{css_href}">
  <style>body {{ font-family: system-ui, sans-serif; max-width: 40rem; margin: 2rem auto; padding: 0 1rem; }}</style>
</head>
<body>
{fragment}
</body>
</html>
"#
            );
            let path = out_dir.join("index.html");
            fs::write(&path, page).unwrap();
            path
        },
    );

    support::task_with_output("Inspect artefacts on disk", || {
        vec![
            format!("themes written: {written}"),
            format!("default theme : {default_theme}"),
            format!("demo page     : {}", html_path.display()),
        ]
    });

    let highlighted =
        support::task("Highlight a snippet standalone", || {
            apply_syntax_highlighting("let x: u32 = 42;", "rust")
                .unwrap()
        });

    support::task_with_output("Inspect standalone output", || {
        vec![
            format!("bytes : {}", highlighted.len()),
            format!("has class= : {}", highlighted.contains("class=")),
            format!(
                "has style= : {}",
                highlighted.contains(" style=\"")
            ),
        ]
    });

    support::summary(8);
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
