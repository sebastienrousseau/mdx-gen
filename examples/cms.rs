// Copyright © 2024 - 2026 MDX Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! # CMS — Safe rendering for user-submitted content
//!
//! ## What this example is
//!
//! A walkthrough of [`SanitizerConfig`], the typed extension point
//! for the ammonia-backed sanitiser. The example prints four
//! side-by-side comparisons:
//!
//! 1. **Strict defaults** — no config, raw HTML stripped hard.
//! 2. **Extra tag** — `<main>` added to the tag allow-list.
//! 3. **Allowed class on a new tag** — per-tag class whitelist.
//! 4. **Opt back into `style`** — for trusted content only.
//!
//! ## What it demonstrates
//!
//! - [`MarkdownOptions::with_sanitizer_config`] — plugging a
//!   [`SanitizerConfig`] into the pipeline.
//! - [`SanitizerConfig::with_tag`] /
//!   [`SanitizerConfig::with_tag_attribute`] — extending the
//!   tag + attribute allow-lists.
//! - [`SanitizerConfig::with_allowed_class`] — per-tag class
//!   whitelist; swaps the tag out of permissive class mode so the
//!   rest of the defaults stay intact.
//! - [`SanitizerConfig::with_generic_attribute`] — opting a tag
//!   attribute (e.g. `style`) back in globally.
//!
//! ## Security note
//!
//! `style` was removed from the default allow-list in 0.0.3
//! because inline styles are a UI-redress / clickjacking vector
//! when the input is untrusted. Only opt back in when the input
//! is trusted (your own CMS output, a test harness, etc.).
//!
//! ## When to use this pattern
//!
//! User-generated content pipelines: forums, comment threads,
//! CMS-authored pages, any flow where the Markdown source is
//! untrusted. The default policy errs on the side of stripping;
//! extend only the pieces your template needs.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example cms
//! ```

use mdx_gen::{process_markdown, MarkdownOptions, SanitizerConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Sanitizer configuration");
    println!("──────────────────────────");

    strict_defaults()?;
    extra_tag()?;
    allowed_class_restriction()?;
    style_opt_in()?;

    println!("\n    🎉 Sanitizer demos complete");
    Ok(())
}

/// Baseline: no config, untrusted-content defaults. `<main>` is
/// not in ammonia's built-in list, so it disappears; the inner
/// text survives. `<script>` content is dropped wholesale.
fn strict_defaults() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🛡️  Strict defaults (no config)");
    let md = "<main>wrapper</main>\n\n<script>alert(1)</script>\n";
    let options = MarkdownOptions::new()
        .with_custom_blocks(false)
        .with_enhanced_tables(false)
        .with_unsafe_html(false);

    let html = process_markdown(md, &options)?;
    print_transform(md, &html);
    assert!(!html.contains("<main>"));
    assert!(!html.contains("<script>"));
    Ok(())
}

/// Adding `<main>` (and its optional `id` attribute) to the
/// allow-list. The tag survives and keeps its attribute.
fn extra_tag() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🛡️  Extra tag: <main id=\"…\">");
    let md = "<main id=\"page\">wrapper</main>\n";

    let options = MarkdownOptions::new()
        .with_custom_blocks(false)
        .with_enhanced_tables(false)
        .with_unsafe_html(false)
        .with_sanitizer_config(
            SanitizerConfig::new()
                .with_tag("main")
                .with_tag_attribute("main", "id"),
        );

    let html = process_markdown(md, &options)?;
    print_transform(md, &html);
    assert!(html.contains("<main id=\"page\">"));
    Ok(())
}

/// Restricting classes on `<span>` to a known whitelist. The
/// default policy allows any class on `<span>` (so the syntax
/// highlighter's open-ended class names survive); supplying
/// per-tag allowed classes swaps it out of permissive mode and
/// only the whitelisted values pass.
fn allowed_class_restriction() -> Result<(), Box<dyn std::error::Error>>
{
    println!("\n🛡️  Allowed class (restricted <span>)");
    let md = "<span class=\"badge\">ok</span> <span class=\"danger\">nope</span>\n";

    let options = MarkdownOptions::new()
        .with_custom_blocks(false)
        .with_enhanced_tables(false)
        .with_unsafe_html(false)
        .with_sanitizer_config(
            SanitizerConfig::new().with_allowed_class("span", "badge"),
        );

    let html = process_markdown(md, &options)?;
    print_transform(md, &html);
    assert!(html.contains("class=\"badge\""));
    assert!(!html.contains("class=\"danger\""));
    Ok(())
}

/// Opting `style` back in as a generic attribute. Only do this
/// for trusted content.
fn style_opt_in() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🛡️  Opt back into `style` (trusted content only)");
    let md = "<p style=\"color:red\">alert</p>\n";

    let options = MarkdownOptions::new()
        .with_custom_blocks(false)
        .with_enhanced_tables(false)
        .with_unsafe_html(false)
        .with_sanitizer_config(
            SanitizerConfig::new().with_generic_attribute("style"),
        );

    let html = process_markdown(md, &options)?;
    print_transform(md, &html);
    assert!(html.contains("style=\"color:red\""));
    Ok(())
}

fn print_transform(before: &str, after: &str) {
    println!("    ── before ─────────────────");
    println!("{}", before.trim_end());
    println!("    ── after  ─────────────────");
    println!("{}", after.trim_end());
}
