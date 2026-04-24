// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Safe rendering of user-submitted content — SanitizerConfig demo.
//!
//! Run: `cargo run --example cms`

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use mdx_gen::{process_markdown, MarkdownOptions, SanitizerConfig};

fn main() {
    support::header("mdx-gen -- cms");

    // ── Strict defaults ──────────────────────────────────────────────
    support::task_with_output(
        "Strict defaults strip raw <main> + <script>",
        || {
            let md =
                "<main>wrapper</main>\n\n<script>alert(1)</script>\n";
            let options = base_options();
            let html = process_markdown(md, &options).unwrap();
            vec![
                format!("<main> kept   : {}", html.contains("<main>")),
                format!(
                    "<script> kept : {}",
                    html.contains("<script>")
                ),
            ]
        },
    );

    // ── Add an extra tag ─────────────────────────────────────────────
    support::task_with_output(
        "Allow <main id=\"…\"> via extra tag + attr",
        || {
            let md = "<main id=\"page\">wrapper</main>\n";
            let options = base_options().with_sanitizer_config(
                SanitizerConfig::new()
                    .with_tag("main")
                    .with_tag_attribute("main", "id"),
            );
            let html = process_markdown(md, &options).unwrap();
            vec![format!(
                "<main id> kept: {}",
                html.contains("<main id=\"page\">")
            )]
        },
    );

    // ── Restrict span classes ────────────────────────────────────────
    support::task_with_output(
        "Whitelist span classes (drop everything else)",
        || {
            let md = "<span class=\"badge\">ok</span> <span class=\"danger\">nope</span>\n";
            let options = base_options().with_sanitizer_config(
                SanitizerConfig::new()
                    .with_allowed_class("span", "badge"),
            );
            let html = process_markdown(md, &options).unwrap();
            vec![
                format!(
                    "badge kept    : {}",
                    html.contains("class=\"badge\"")
                ),
                format!(
                    "danger kept   : {}",
                    html.contains("class=\"danger\"")
                ),
            ]
        },
    );

    // ── Opt back into style (trusted content only) ───────────────────
    support::task_with_output(
        "Opt style=\"…\" back in (trusted content only)",
        || {
            let md = "<p style=\"color:red\">alert</p>\n";
            let options = base_options().with_sanitizer_config(
                SanitizerConfig::new().with_generic_attribute("style"),
            );
            let html = process_markdown(md, &options).unwrap();
            vec![format!(
                "style kept    : {}",
                html.contains("style=\"color:red\"")
            )]
        },
    );

    support::summary(4);
}

fn base_options<'a>() -> MarkdownOptions<'a> {
    MarkdownOptions::new()
        .with_custom_blocks(false)
        .with_enhanced_tables(false)
        .with_unsafe_html(false)
}
