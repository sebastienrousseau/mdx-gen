// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Custom-block alert overrides — re-skin the built-in callouts.
//!
//! Run: `cargo run --example alerts`

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use mdx_gen::{
    process_markdown, CustomBlockConfig, CustomBlockType,
    MarkdownOptions,
};

const SOURCE: &str = r#"<div class="note">A note for the reader.</div>
<div class="warning">Mind the gap.</div>
<div class="tip">Press ⌘K to search.</div>
<div class="info">Build status is green.</div>
<div class="important">Read the migration guide first.</div>
<div class="caution">Backups are advised.</div>
"#;

fn main() {
    support::header("mdx-gen -- alerts");

    // ── Default Bootstrap-shaped output ─────────────────────────────
    let default_html =
        support::task("Render with default config", || {
            let options = MarkdownOptions::new()
                .with_custom_blocks(true)
                .with_enhanced_tables(false);
            process_markdown(SOURCE, &options).unwrap()
        });
    support::task_with_output("Inspect default class names", || {
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
                "{:<10} {:<18} title={}",
                format!("{b:?}"),
                b.get_alert_class(),
                b.get_title()
            )
        })
        .collect()
    });

    // ── Tailwind-style override (callout-* + custom titles) ─────────
    let tailwind_html =
        support::task("Render with Tailwind overrides", || {
            let config = CustomBlockConfig::new()
                .with_class(CustomBlockType::Note, "callout-info")
                .with_class(CustomBlockType::Warning, "callout-warn")
                .with_class(CustomBlockType::Tip, "callout-tip")
                .with_class(CustomBlockType::Info, "callout-info")
                .with_class(
                    CustomBlockType::Important,
                    "callout-danger",
                )
                .with_class(CustomBlockType::Caution, "callout-danger")
                .with_title(CustomBlockType::Note, "Did you know?")
                .with_title(CustomBlockType::Warning, "Heads up")
                .with_title(CustomBlockType::Tip, "Pro tip")
                .with_title(CustomBlockType::Info, "Status")
                .with_title(CustomBlockType::Important, "Required")
                .with_title(CustomBlockType::Caution, "Be careful");

            let options = MarkdownOptions::new()
                .with_custom_blocks(true)
                .with_enhanced_tables(false)
                .with_custom_block_config(config);
            process_markdown(SOURCE, &options).unwrap()
        });

    support::task_with_output("Verify default vs override", || {
        vec![
            format!(
                "default contains 'alert-info'  : {}",
                default_html.contains("alert-info")
            ),
            format!(
                "default contains 'callout-info': {}",
                default_html.contains("callout-info")
            ),
            format!(
                "tailwind contains 'callout-warn': {}",
                tailwind_html.contains("callout-warn")
            ),
            format!(
                "tailwind contains 'Did you know?': {}",
                tailwind_html.contains("Did you know?")
            ),
            format!(
                "tailwind contains 'Heads up'    : {}",
                tailwind_html.contains("Heads up")
            ),
        ]
    });

    // ── Per-block-type lookup helpers ────────────────────────────────
    support::task_with_output(
        "Lookup helpers respect the config",
        || {
            let config = CustomBlockConfig::new()
                .with_class(CustomBlockType::Note, "my-special-note");
            vec![
                format!(
                    "Note default class               : {}",
                    CustomBlockType::Note.get_alert_class()
                ),
                format!(
                    "Note class via alert_class_with  : {}",
                    CustomBlockType::Note.alert_class_with(&config)
                ),
            ]
        },
    );

    support::summary(5);
}
