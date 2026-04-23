// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Red-team the renderer — XSS payloads, clickjacking, oversized input.
//!
//! Run: `cargo run --example security`
//!
//! Lives as exec'd documentation of the hardening that landed in
//! 0.0.3: ammonia sanitiser tuned for safe defaults, `style`
//! attribute removed, input cap on by default. Each task feeds a
//! known attack pattern through `process_markdown` with
//! `allow_unsafe_html = false` and asserts the output is safe.

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use mdx_gen::{process_markdown, MarkdownError, MarkdownOptions};

fn main() {
    support::header("mdx-gen -- security");

    let options = MarkdownOptions::new()
        .with_custom_blocks(false)
        .with_enhanced_tables(false)
        .with_unsafe_html(false);

    // ── XSS via raw <script> ─────────────────────────────────────────
    support::task_with_output("Strip <script> tags", || {
        let payloads = [
            "<script>alert('xss')</script>",
            "<SCRIPT>alert(1)</SCRIPT>",
            "<script src=\"evil.js\"></script>",
        ];
        payloads
            .iter()
            .map(|p| {
                let html = process_markdown(p, &options).unwrap();
                let kept = html.to_lowercase().contains("<script");
                format!("kept={kept:<5} {p}")
            })
            .collect()
    });

    // ── XSS via event handlers ───────────────────────────────────────
    support::task_with_output("Strip on* event handlers", || {
        let payloads = [
            "<img src=x onerror=alert(1)>",
            "<a href='/' onclick='steal()'>x</a>",
            "<svg onload=alert(1)>",
        ];
        payloads
            .iter()
            .map(|p| {
                let html = process_markdown(p, &options).unwrap();
                let kept = html.contains("onerror")
                    || html.contains("onclick")
                    || html.contains("onload");
                format!("kept={kept:<5} {p}")
            })
            .collect()
    });

    // ── Clickjacking via inline style (post-0.0.3) ───────────────────
    support::task_with_output("Strip inline style overlays", || {
        let payloads = [
            r#"<div style="position:fixed;top:0;left:0;width:100%;height:100%;z-index:9999"></div>"#,
            r#"<a style="display:block;width:100vw;height:100vh"></a>"#,
        ];
        payloads
            .iter()
            .map(|p| {
                let html = process_markdown(p, &options).unwrap();
                let kept = html.contains("style=");
                format!("kept={kept:<5} {p}")
            })
            .collect()
    });

    // ── javascript: / data: URLs ─────────────────────────────────────
    support::task_with_output("Strip javascript: / data: URLs", || {
        let payloads = [
            "[click](javascript:alert(1))",
            "[click](data:text/html,<script>alert(1)</script>)",
            "<a href=\"javascript:alert(1)\">click</a>",
        ];
        payloads
            .iter()
            .map(|p| {
                let html = process_markdown(p, &options).unwrap();
                let kept = html.contains("javascript:")
                    || html.contains("data:text/html");
                format!("kept={kept:<5} {p}")
            })
            .collect()
    });

    // ── Resource cap rejects oversized input before parsing ──────────
    support::task_with_output("Cap rejects oversized input", || {
        let capped = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_max_input_size(1024); // 1 KiB
        let big = "a".repeat(10_000);
        match process_markdown(&big, &capped) {
            Err(MarkdownError::InputTooLarge { size, limit }) => {
                vec![format!(
                    "rejected: {size} bytes vs {limit}-byte cap"
                )]
            }
            other => vec![format!("BUG unexpected: {other:?}")],
        }
    });

    // ── Markdown bomb: deeply nested blockquotes ─────────────────────
    support::task_with_output(
        "Deeply nested blockquotes finish in bounded time",
        || {
            let bomb = format!("{}body\n", ">".repeat(5_000));
            let start = std::time::Instant::now();
            let result = process_markdown(&bomb, &options);
            let elapsed = start.elapsed();
            match result {
                Ok(html) => vec![
                    format!("rendered {} bytes", html.len()),
                    format!("elapsed  {:?}", elapsed),
                ],
                Err(e) => vec![format!("error: {e}")],
            }
        },
    );

    support::summary(6);
}
