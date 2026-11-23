// SPDX-License-Identifier: Apache-2.0 OR MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

#![allow(missing_docs)]

use comrak::Options;
use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use mdx_gen::{process_markdown, MarkdownOptions};
use std::hint::black_box;
use std::time::Duration;

/// One section of the synthetic document used by
/// [`large_document_benchmark`]. It mixes the constructs the
/// pipeline treats specially (headings, a table, a fenced code block,
/// a custom block, inline markup, a link and an image) so every
/// stage is exercised in proportion.
fn synthetic_section(i: usize) -> String {
    format!(
        "## Section {i}: pipeline notes\n\n\
         Paragraph {i} with **bold**, *italic*, `inline code` and a \
         [link](https://example.com/{i}). The renderer walks the \
         AST once, then the sanitizer cleans the result.\n\n\
         - first item {i}\n- second item with `code`\n- third item\n\n\
         | Name | Value | Notes |\n|:-----|:-----:|------:|\n\
         | a{i} | 1 | left |\n| b{i} | 2 | right |\n\n\
         ```rust\nfn section_{i}() -> usize {{\n    let v = vec![1, 2, 3];\n    v.iter().sum()\n}}\n```\n\n\
         <div class=\"note\">Note for section {i}.</div>\n\n\
         ![diagram {i}](https://example.com/img/{i}.png \"Figure {i}\")\n\n"
    )
}

/// Builds a document of at least `target` bytes from
/// [`synthetic_section`]s, so the 1 MiB case stays inside the
/// default `max_input_size` it is meant to approach.
fn synthetic_document(target: usize) -> String {
    let mut doc = String::from("# Large document benchmark\n\n");
    let mut i = 0;
    while doc.len() < target {
        doc.push_str(&synthetic_section(i));
        i += 1;
    }
    doc
}

/// Create a valid MarkdownOptions configuration
fn create_valid_options(
    syntax_highlighting: bool,
    custom_blocks: bool,
    enhanced_tables: bool,
    enable_comrak_tables: bool,
) -> MarkdownOptions<'static> {
    let mut comrak_options = Options::default();
    comrak_options.extension.table = enable_comrak_tables;

    MarkdownOptions::new()
        .with_syntax_highlighting(syntax_highlighting)
        .with_custom_blocks(custom_blocks)
        .with_enhanced_tables(enhanced_tables)
        .with_comrak_options(comrak_options)
}

/// Benchmark the Markdown to HTML conversion process with various configurations.
fn markdown_benchmark(c: &mut Criterion) {
    let small_markdown = r#"
# Welcome to MDX Gen
This is a **bold** statement and this is *italic*.
## Features
- Easy to use
- Extensible
- Fast
Check out [our website](https://example.com) for more information.
    "#;

    let large_markdown = include_str!("../README.md");

    let markdown_sizes =
        [("small", small_markdown), ("large", large_markdown)];

    let mut group = c.benchmark_group("Markdown to HTML Conversion");

    for (size, markdown) in markdown_sizes.iter() {
        group.throughput(Throughput::Bytes(markdown.len() as u64));

        // Basic conversion (no enhanced tables)
        let basic_options =
            create_valid_options(false, false, false, false);
        group.bench_with_input(
            BenchmarkId::new("basic", size),
            markdown,
            |b, markdown| {
                b.iter(|| {
                    let _ = process_markdown(
                        black_box(markdown),
                        black_box(&basic_options),
                    )
                    .expect("Basic conversion should not fail");
                });
            },
        );

        // Full-featured conversion
        let full_options = create_valid_options(true, true, true, true);
        group.bench_with_input(
            BenchmarkId::new("full", size),
            markdown,
            |b, markdown| {
                b.iter(|| {
                    let _ = process_markdown(
                        black_box(markdown),
                        black_box(&full_options),
                    )
                    .expect("Full-featured conversion should not fail");
                });
            },
        );

        // Custom configuration
        let custom_options =
            create_valid_options(true, false, true, true);
        group.bench_with_input(
            BenchmarkId::new("custom", size),
            markdown,
            |b, markdown| {
                b.iter(|| {
                    let _ = process_markdown(
                        black_box(markdown),
                        black_box(&custom_options),
                    )
                    .expect("Custom conversion should not fail");
                });
            },
        );
    }

    group.finish();
}

/// Parameterised large-input benchmark (issue #29): 10 KB, 100 KB,
/// 500 KB and just under the 1 MiB default input cap, rendered with
/// the default (all features on, sanitizer on) options.
fn large_document_benchmark(c: &mut Criterion) {
    let sizes: [(&str, usize); 4] = [
        ("10KB", 10 * 1024),
        ("100KB", 100 * 1024),
        ("500KB", 500 * 1024),
        ("1MiB", 1_040_000),
    ];
    let options = MarkdownOptions::default();

    let mut group = c.benchmark_group("Large documents");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));

    for (label, target) in sizes {
        let doc = synthetic_document(target);
        group.throughput(Throughput::Bytes(doc.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("default", label),
            &doc,
            |b, doc| {
                b.iter(|| {
                    process_markdown(
                        black_box(doc),
                        black_box(&options),
                    )
                    .expect("synthetic document renders")
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, markdown_benchmark, large_document_benchmark);
criterion_main!(benches);
