// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `CachingProcessor` (issue #31, `cache` feature).

#![cfg(feature = "cache")]

use mdx_gen::{
    process_markdown, process_markdown_with_toc, CacheStats,
    CachingProcessor, MarkdownOptions,
};
use std::sync::Arc;

const DOC: &str = "# Title\n\n## Part\n\nText with **bold**.\n";

#[test]
fn miss_then_hit_returns_identical_html() {
    let options = MarkdownOptions::default().with_header_ids("");
    let expected = process_markdown(DOC, &options).unwrap();
    let p = CachingProcessor::new(options);

    assert_eq!(p.process(DOC).unwrap(), expected);
    assert_eq!(p.stats(), CacheStats { hits: 0, misses: 1 });
    assert_eq!(p.process(DOC).unwrap(), expected);
    assert_eq!(p.stats(), CacheStats { hits: 1, misses: 1 });
}

#[test]
fn toc_variant_is_cached_separately_and_matches_uncached() {
    let options = MarkdownOptions::default().with_header_ids("");
    let expected = process_markdown_with_toc(DOC, &options).unwrap();
    let p = CachingProcessor::new(options);

    p.process(DOC).unwrap();
    assert_eq!(p.process_with_toc(DOC).unwrap(), expected); // miss
    assert_eq!(p.process_with_toc(DOC).unwrap(), expected); // hit
    assert_eq!(p.stats(), CacheStats { hits: 1, misses: 2 });
    assert_eq!(p.len(), 2);
}

#[test]
fn changed_content_is_a_miss() {
    let p = CachingProcessor::new(MarkdownOptions::default());
    let a = p.process("# A").unwrap();
    let b = p.process("# B").unwrap();
    assert_ne!(a, b);
    assert_eq!(p.stats().misses, 2);
}

#[test]
fn errors_are_returned_and_not_cached() {
    let options = MarkdownOptions::default().with_max_input_size(4);
    let p = CachingProcessor::new(options);
    assert!(p.process("too long").is_err());
    assert!(p.process("too long").is_err());
    assert!(p.is_empty());
    assert_eq!(p.stats().misses, 2);
}

#[test]
fn clear_empties_and_resets() {
    let p = CachingProcessor::new(MarkdownOptions::default());
    p.process(DOC).unwrap();
    p.clear();
    assert!(p.is_empty());
    assert_eq!(p.stats(), CacheStats::default());
    assert_eq!(p.capacity(), mdx_gen::cache::DEFAULT_CAPACITY);
}

#[test]
fn shared_across_threads() {
    let p = Arc::new(CachingProcessor::new(MarkdownOptions::default()));
    let expected = process_markdown(DOC, p.options()).unwrap();
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let p = Arc::clone(&p);
            std::thread::spawn(move || {
                (0..20)
                    .map(|_| p.process(DOC).unwrap())
                    .collect::<Vec<_>>()
            })
        })
        .collect();
    for h in handles {
        for html in h.join().unwrap() {
            assert_eq!(html, expected);
        }
    }
    let stats = p.stats();
    assert_eq!(stats.hits + stats.misses, 160);
    assert!(
        stats.hits >= 152,
        "at most one miss per thread: {stats:?}"
    );
    assert_eq!(p.len(), 1);
}
