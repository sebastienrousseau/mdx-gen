// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Render 1000 small documents in one process — throughput demo.
//!
//! Run: `cargo run --release --example bulk`
//!
//! Use `--release` for the realistic figure; the dev build keeps
//! debug assertions and inhibits inlining, so the per-doc number
//! is dominated by overhead unrelated to the renderer itself.

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use std::time::Instant;

use mdx_gen::{process_markdown, MarkdownOptions, Options};

const N: usize = 1_000;

const TEMPLATE: &str = r#"# Item {i}

Body for item **{i}** with a tiny `code` snippet, a list:

- one
- two
- three

| Col | Val |
|:----|----:|
| n   | {i} |
"#;

fn main() {
    support::header("mdx-gen -- bulk");

    let docs: Vec<String> =
        support::task("Synthesize input corpus", || {
            (0..N)
                .map(|i| TEMPLATE.replace("{i}", &i.to_string()))
                .collect()
        });

    let total_input: usize = docs.iter().map(|s| s.len()).sum();

    let options = support::task("Build options once", || {
        let mut comrak_options = Options::default();
        comrak_options.extension.table = true;
        MarkdownOptions::new()
            .with_comrak_options(comrak_options)
            .with_custom_blocks(false)
            .with_enhanced_tables(true)
            .with_syntax_highlighting(false)
            .with_unsafe_html(false)
    });

    // Render every doc through the same options instance — exercises
    // the LazyLock'd ammonia builder + cached SyntaxSet on the hot
    // path. The first call pays the initialisation cost; the rest
    // amortise it.
    let (total_output, elapsed) =
        support::task("Render the corpus", || {
            let start = Instant::now();
            let mut total = 0usize;
            for src in &docs {
                let html = process_markdown(src, &options).unwrap();
                total += html.len();
            }
            (total, start.elapsed())
        });

    let secs = elapsed.as_secs_f64();
    let docs_per_s = N as f64 / secs;
    let mb_per_s = (total_output as f64 / 1024.0 / 1024.0) / secs;

    support::task_with_output("Throughput summary", || {
        vec![
            format!("docs rendered : {N}"),
            format!("input bytes   : {total_input}"),
            format!("output bytes  : {total_output}"),
            format!("elapsed       : {:.3} s", secs),
            format!("rate          : {:.0} docs/s", docs_per_s),
            format!("output rate   : {:.2} MB/s", mb_per_s),
        ]
    });

    // For parallel batch jobs add `rayon` and replace the inner
    // loop with `docs.par_iter().for_each(|src| { … })`. The
    // sanitizer + syntax sets are `Sync`, so no extra plumbing
    // needed.
    support::summary(4);
}
