//! Regression test for GHSA-xg9p-p4jc-c46g.
//!
//! comrak 0.54 and earlier linked bare email addresses by recursing
//! once per address, so a paragraph of a few thousand of them
//! overflowed the stack. A Rust stack overflow aborts the process
//! rather than panicking, so on an affected comrak this test binary
//! dies instead of reporting a failure; either way `cargo test` fails.
//! comrak 0.55 links them in a loop.

use mdx_gen::{process_markdown, MarkdownOptions, Options};
use std::thread;

/// The worker-thread stack size the advisory measured against: a
/// server pool thread of this size crashed on about 2,000 emails.
const WORKER_STACK: usize = 512 * 1024;

/// Comfortably past that threshold.
const EMAILS: usize = 3_000;

#[test]
fn bare_email_autolinks_do_not_overflow_a_small_stack() {
    let markdown = "a@b.co ".repeat(EMAILS);

    let html = thread::Builder::new()
        .stack_size(WORKER_STACK)
        .spawn(move || {
            let mut comrak = Options::default();
            comrak.extension.autolink = true;
            let options = MarkdownOptions::new()
                .with_enhanced_tables(false)
                .with_comrak_options(comrak);
            process_markdown(&markdown, &options)
        })
        .expect("spawn the render thread")
        .join()
        .expect("the render thread panicked")
        .expect("the render failed");

    assert_eq!(html.matches("href=\"mailto:a@b.co\"").count(), EMAILS);
}
