// SPDX-License-Identifier: Apache-2.0 OR MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! Repeated-content workload for `CachingProcessor` (issue #31):
//! the same document rendered again and again, uncached versus
//! cached. Run with `cargo bench --features cache --bench cache_benchmark`.

#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use mdx_gen::{process_markdown, CachingProcessor, MarkdownOptions};
use std::hint::black_box;

fn cache_benchmark(c: &mut Criterion) {
    let doc = include_str!("../README.md");
    let options = MarkdownOptions::default();
    let processor = CachingProcessor::new(options.clone());
    processor.process(doc).expect("README renders");

    let mut group = c.benchmark_group("Repeated content");
    group.bench_function("uncached", |b| {
        b.iter(|| process_markdown(black_box(doc), &options).unwrap())
    });
    group.bench_function("cached", |b| {
        b.iter(|| processor.process(black_box(doc)).unwrap())
    });
    group.finish();
}

criterion_group!(benches, cache_benchmark);
criterion_main!(benches);
