// Copyright © 2024 - 2026 MDX Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! # Errors — Every [`MarkdownError`] variant
//!
//! ## What this example is
//!
//! A catalogue of the error types mdx-gen can return, paired with
//! the shortest input or configuration that provokes each one. Lets
//! you see the exact `Debug` and `Display` shapes before you wire
//! the library into your own error plumbing.
//!
//! ## What it demonstrates
//!
//! - [`MarkdownError::InputTooLarge`] — the resource cap at work.
//! - [`MarkdownError::InvalidOptionsError`] — consistency check
//!   fired by [`MarkdownOptions::validate`].
//! - [`MarkdownError::IoError`] — propagated from the writer when
//!   using [`process_markdown_to_writer`].
//! - Constructor-level `Debug` / `Display` for [`ParseError`],
//!   [`ConversionError`], [`CustomBlockError`],
//!   [`SyntaxHighlightError`], [`SyntaxSetError`],
//!   [`RenderError`], [`FrontmatterError`]. These do not currently
//!   have a user-triggerable path in the public pipeline, so the
//!   example constructs them directly and prints how they look.
//!
//! ## When to use this pattern
//!
//! Copy this file when you're writing an integration that needs to
//! branch on specific error variants — match arms here line up with
//! the variants you'll see in production.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example errors
//! ```
//!
//! [`ParseError`]: mdx_gen::MarkdownError::ParseError
//! [`ConversionError`]: mdx_gen::MarkdownError::ConversionError
//! [`CustomBlockError`]: mdx_gen::MarkdownError::CustomBlockError
//! [`SyntaxHighlightError`]: mdx_gen::MarkdownError::SyntaxHighlightError
//! [`SyntaxSetError`]: mdx_gen::MarkdownError::SyntaxSetError
//! [`RenderError`]: mdx_gen::MarkdownError::RenderError
//! [`FrontmatterError`]: mdx_gen::MarkdownError::FrontmatterError

use mdx_gen::{
    process_markdown, process_markdown_to_writer, MarkdownError,
    MarkdownOptions, Options,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 MarkdownError catalogue");
    println!("──────────────────────────");

    input_too_large_section();
    invalid_options_section();
    io_error_section();
    constructed_variants_section();

    println!("\n    🎉 Error catalogue complete");
    Ok(())
}

fn input_too_large_section() {
    println!("\n🧱 InputTooLarge");
    let options = MarkdownOptions::new()
        .with_custom_blocks(false)
        .with_enhanced_tables(false)
        .with_max_input_size(16);

    match process_markdown(&"a".repeat(64), &options) {
        Err(MarkdownError::InputTooLarge { size, limit }) => {
            println!(
                "    ✅ {size} bytes rejected against {limit}-byte cap"
            );
        }
        other => println!("    ❌ unexpected: {other:?}"),
    }
}

fn invalid_options_section() {
    println!("\n🧱 InvalidOptionsError");
    // enable_enhanced_tables requires comrak's table extension.
    let mut comrak_options = Options::default();
    comrak_options.extension.table = false;
    let options = MarkdownOptions::new()
        .with_enhanced_tables(true)
        .with_comrak_options(comrak_options);

    match process_markdown("# hi", &options) {
        Err(MarkdownError::InvalidOptionsError(msg)) => {
            println!("    ✅ validation tripped: {msg}");
        }
        other => println!("    ❌ unexpected: {other:?}"),
    }
}

fn io_error_section() {
    println!("\n🧱 IoError");
    struct AlwaysFails;
    impl std::io::Write for AlwaysFails {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
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

    match process_markdown_to_writer("# hi", &mut AlwaysFails, &options)
    {
        Err(MarkdownError::IoError(e)) => {
            println!("    ✅ writer failure surfaced: {e}");
        }
        other => println!("    ❌ unexpected: {other:?}"),
    }
}

fn constructed_variants_section() {
    println!("\n🧱 Constructed (Display form)");
    let variants = [
        MarkdownError::ParseError("unterminated code fence".into()),
        MarkdownError::ConversionError("comrak write failed".into()),
        MarkdownError::CustomBlockError(
            "unknown block: frobnicate".into(),
        ),
        MarkdownError::SyntaxHighlightError(
            "no syntax for <klingon>".into(),
        ),
        MarkdownError::SyntaxSetError("failed to load defaults".into()),
        MarkdownError::RenderError("non-UTF-8 output".into()),
        MarkdownError::FrontmatterError(
            "unexpected node in mapping".into(),
        ),
    ];

    for err in &variants {
        println!("    ↳ {err}");
    }
}
