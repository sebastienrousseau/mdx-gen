// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Load `MarkdownOptions` from a TOML config file, then render with it.
//!
//! Run: `cargo run --example config_file --features serde`

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use mdx_gen::{process_markdown, MarkdownOptions};

/// What a site generator might keep next to its own settings in
/// `mdx-gen.toml`. Every key is optional; missing keys keep their
/// defaults.
const CONFIG: &str = r#"
enable-custom-blocks = true
enable-syntax-highlighting = false
enable-lazy-images = true
header-ids = "user-content-"
max-input-size = 524288

[custom-block-config.class-overrides]
note = "callout-note"

[custom-block-config.title-overrides]
note = "Heads up"
warning = "Careful"

[sanitizer-config]
extra-tags = ["mark"]

[sanitizer-config.extra-allowed-classes]
div = ["callout-note"]
"#;

const SOURCE: &str = "# Release notes

<div class=\"note\">Configured from TOML.</div>

Some <mark>highlighted</mark> text and an image:

![Architecture](arch.png \"Pipeline\")
";

fn main() {
    support::header("mdx-gen -- config_file");

    let path = std::env::temp_dir().join("mdx-gen-example.toml");
    support::task("Write mdx-gen.toml", || {
        std::fs::write(&path, CONFIG).unwrap();
    });

    let options: MarkdownOptions =
        support::task("Load options", || {
            let text = std::fs::read_to_string(&path).unwrap();
            toml::from_str(&text).expect("valid mdx-gen config")
        });

    let html = support::task("Render Markdown to HTML", || {
        process_markdown(SOURCE, &options).unwrap()
    });

    support::task_with_output("Inspect output", || {
        vec![
            format!("heading id:   {}", html.contains("user-content-")),
            format!("note title:   {}", html.contains("Heads up:")),
            format!("note class:   {}", html.contains("callout-note")),
            format!("<mark> kept:  {}", html.contains("<mark>")),
            format!(
                "lazy image:   {}",
                html.contains("loading=\"lazy\"")
            ),
        ]
    });

    let _ = std::fs::remove_file(&path);
    support::summary(4);
}
