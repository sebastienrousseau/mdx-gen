// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! MarkdownError catalogue — live + constructed variants.
//!
//! Run: `cargo run --example errors`

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use mdx_gen::{
    process_markdown, process_markdown_to_writer, MarkdownError,
    MarkdownOptions, Options,
};

fn main() {
    support::header("mdx-gen -- errors");

    support::task_with_output("InputTooLarge (live)", || {
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_max_input_size(16);
        match process_markdown(&"a".repeat(64), &options) {
            Err(MarkdownError::InputTooLarge { size, limit }) => {
                vec![format!(
                    "{size} bytes rejected against {limit}-byte cap"
                )]
            }
            other => vec![format!("BUG unexpected: {other:?}")],
        }
    });

    support::task_with_output("InvalidOptionsError (live)", || {
        let mut comrak_options = Options::default();
        comrak_options.extension.table = false;
        let options = MarkdownOptions::new()
            .with_enhanced_tables(true)
            .with_comrak_options(comrak_options);
        match process_markdown("# hi", &options) {
            Err(MarkdownError::InvalidOptionsError(msg)) => {
                vec![format!("validation tripped: {msg}")]
            }
            other => vec![format!("BUG unexpected: {other:?}")],
        }
    });

    support::task_with_output(
        "IoError via always-failing writer",
        || {
            struct AlwaysFails;
            impl std::io::Write for AlwaysFails {
                fn write(
                    &mut self,
                    _: &[u8],
                ) -> std::io::Result<usize> {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        "simulated pipe",
                    ))
                }
                fn flush(&mut self) -> std::io::Result<()> {
                    Ok(())
                }
            }

            let options = MarkdownOptions::new()
                .with_custom_blocks(false)
                .with_enhanced_tables(false);
            match process_markdown_to_writer(
                "# hi",
                &mut AlwaysFails,
                &options,
            ) {
                Err(MarkdownError::IoError(e)) => {
                    vec![format!("writer failure surfaced: {e}")]
                }
                other => vec![format!("BUG unexpected: {other:?}")],
            }
        },
    );

    support::task_with_output(
        "Constructed variants (Display form)",
        || {
            let variants = [
                MarkdownError::ParseError(
                    "unterminated code fence".into(),
                ),
                MarkdownError::ConversionError(
                    "comrak write failed".into(),
                ),
                MarkdownError::CustomBlockError(
                    "unknown block: frobnicate".into(),
                ),
                MarkdownError::SyntaxHighlightError(
                    "no syntax for <klingon>".into(),
                ),
                MarkdownError::SyntaxSetError(
                    "failed to load defaults".into(),
                ),
                MarkdownError::RenderError("non-UTF-8 output".into()),
            ];
            variants.iter().map(|e| format!("{e}")).collect()
        },
    );

    support::summary(4);
}
