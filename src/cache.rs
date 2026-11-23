//! Opt-in full-document render cache (`cache` feature).
//!
//! [`CachingProcessor`] owns one [`MarkdownOptions`] and memoises the
//! output of [`process_markdown`](crate::process_markdown) and
//! [`process_markdown_with_toc`](crate::process_markdown_with_toc)
//! per input document. Because the options are fixed when the
//! processor is built, the cache key only has to identify the
//! content: build one processor per option set.
//!
//! # Keying and correctness
//!
//! Entries are looked up by a 64-bit hash of the input, then the
//! stored input is compared byte for byte before a hit is returned.
//! A hash collision therefore costs a re-render, never a wrong
//! answer. The cache holds a copy of every cached input alongside
//! its HTML, so memory is roughly `capacity x (input + output)`.
//!
//! # Eviction
//!
//! The cache is bounded by entry count ([`DEFAULT_CAPACITY`] unless
//! set with [`CachingProcessor::with_capacity`]). When full, the
//! least recently used entry is evicted. Eviction scans the entries,
//! which is negligible next to a Markdown render at the default
//! bound.
//!
//! # Concurrency
//!
//! `CachingProcessor` is `Send + Sync` and can be shared behind an
//! `Arc`. The map sits behind one [`std::sync::Mutex`] held only for
//! lookup and insertion; rendering happens outside the lock, so
//! threads rendering different documents do not serialise on each
//! other. Two threads that miss on the same document at the same
//! time both render it and the second insert wins, which is
//! harmless because the outputs are identical. A poisoned lock is
//! recovered rather than propagated: the map holds only finished
//! renders, so a panic elsewhere cannot leave it half-updated.
//!
//! Hits and misses are reported through the `log` crate at `debug`
//! level, and counted in [`CacheStats`].

use crate::error::MarkdownError;
use crate::extensions::Heading;
use crate::markdown::{
    process_markdown, process_markdown_with_toc, MarkdownOptions,
};
use log::debug;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, MutexGuard};

/// Default number of documents a [`CachingProcessor`] keeps.
pub const DEFAULT_CAPACITY: usize = 1000;

/// Hit and miss counters for a [`CachingProcessor`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CacheStats {
    /// Lookups answered from the cache.
    pub hits: u64,
    /// Lookups that had to render the document.
    pub misses: u64,
}

/// Which entry point produced a cached value; the HTML-only and the
/// table-of-contents variants are cached separately.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Variant {
    Html,
    Toc,
}

struct Entry {
    content: String,
    html: String,
    toc: Vec<Heading>,
    last_used: u64,
}

#[derive(Default)]
struct Inner {
    entries: HashMap<(Variant, u64), Entry>,
    tick: u64,
    stats: CacheStats,
}

impl Inner {
    fn lookup(
        &mut self,
        key: (Variant, u64),
        content: &str,
    ) -> Option<(String, Vec<Heading>)> {
        self.tick += 1;
        let tick = self.tick;
        let found = match self.entries.get_mut(&key) {
            Some(e) if e.content == content => {
                e.last_used = tick;
                Some((e.html.clone(), e.toc.clone()))
            }
            _ => None,
        };
        if found.is_some() {
            self.stats.hits += 1;
        } else {
            self.stats.misses += 1;
        }
        found
    }

    fn insert(
        &mut self,
        key: (Variant, u64),
        entry: Entry,
        cap: usize,
    ) {
        if !self.entries.contains_key(&key) && self.entries.len() >= cap
        {
            self.evict_lru();
        }
        self.entries.insert(key, entry);
    }

    fn evict_lru(&mut self) {
        let oldest = self
            .entries
            .iter()
            .min_by_key(|(_, e)| e.last_used)
            .map(|(k, _)| *k);
        if let Some(k) = oldest {
            let _ = self.entries.remove(&k);
        }
    }
}

/// A Markdown processor that caches rendered output per document.
///
/// # Example
///
/// ```
/// use mdx_gen::{CachingProcessor, MarkdownOptions};
///
/// let processor = CachingProcessor::new(MarkdownOptions::default());
/// let first = processor.process("# Hello").unwrap();   // miss: renders
/// let second = processor.process("# Hello").unwrap();  // hit: cached
/// assert_eq!(first, second);
///
/// let stats = processor.stats();
/// assert_eq!((stats.hits, stats.misses), (1, 1));
/// ```
pub struct CachingProcessor<'a> {
    options: MarkdownOptions<'a>,
    capacity: usize,
    inner: Mutex<Inner>,
}

impl<'a> CachingProcessor<'a> {
    /// Creates a processor with [`DEFAULT_CAPACITY`] entries.
    pub fn new(options: MarkdownOptions<'a>) -> Self {
        Self::with_capacity(options, DEFAULT_CAPACITY)
    }

    /// Creates a processor that keeps at most `capacity` documents.
    /// A capacity of `0` disables caching: every call renders.
    pub fn with_capacity(
        options: MarkdownOptions<'a>,
        capacity: usize,
    ) -> Self {
        Self {
            options,
            capacity,
            inner: Mutex::new(Inner::default()),
        }
    }

    /// The options every render uses.
    pub fn options(&self) -> &MarkdownOptions<'a> {
        &self.options
    }

    /// The maximum number of cached documents.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Number of documents currently cached (both variants).
    pub fn len(&self) -> usize {
        self.lock().entries.len()
    }

    /// Returns `true` when nothing is cached.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Hit and miss counts since creation or the last [`clear`].
    ///
    /// [`clear`]: CachingProcessor::clear
    pub fn stats(&self) -> CacheStats {
        self.lock().stats
    }

    /// Drops every cached document and resets the counters.
    pub fn clear(&self) {
        *self.lock() = Inner::default();
    }

    /// Cached equivalent of [`process_markdown`].
    ///
    /// # Errors
    ///
    /// Mirrors [`process_markdown`]. Errors are not cached.
    pub fn process(
        &self,
        content: &str,
    ) -> Result<String, MarkdownError> {
        self.cached(Variant::Html, content, |c, o| {
            process_markdown(c, o).map(|html| (html, Vec::new()))
        })
        .map(|(html, _)| html)
    }

    /// Cached equivalent of [`process_markdown_with_toc`].
    ///
    /// # Errors
    ///
    /// Mirrors [`process_markdown_with_toc`]. Errors are not cached.
    pub fn process_with_toc(
        &self,
        content: &str,
    ) -> Result<(String, Vec<Heading>), MarkdownError> {
        self.cached(Variant::Toc, content, process_markdown_with_toc)
    }

    fn cached<F>(
        &self,
        variant: Variant,
        content: &str,
        render: F,
    ) -> Result<(String, Vec<Heading>), MarkdownError>
    where
        F: FnOnce(
            &str,
            &MarkdownOptions<'a>,
        )
            -> Result<(String, Vec<Heading>), MarkdownError>,
    {
        let key = (variant, hash_content(content));
        if let Some(hit) = self.lock().lookup(key, content) {
            debug!("mdx-gen cache hit ({} bytes)", content.len());
            return Ok(hit);
        }
        debug!("mdx-gen cache miss ({} bytes)", content.len());
        let (html, toc) = render(content, &self.options)?;
        if self.capacity > 0 {
            let (content, cached_html, cached_toc) =
                (content.to_owned(), html.clone(), toc.clone());
            let mut inner = self.lock();
            let entry = Entry {
                content,
                html: cached_html,
                toc: cached_toc,
                last_used: inner.tick,
            };
            inner.insert(key, entry, self.capacity);
        }
        Ok((html, toc))
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl std::fmt::Debug for CachingProcessor<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachingProcessor")
            .field("options", &self.options)
            .field("capacity", &self.capacity)
            .field("len", &self.len())
            .field("stats", &self.stats())
            .finish()
    }
}

fn hash_content(content: &str) -> u64 {
    let mut h = DefaultHasher::new();
    content.hash(&mut h);
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_bound_evicts_least_recently_used() {
        let p = CachingProcessor::with_capacity(
            MarkdownOptions::default(),
            2,
        );
        p.process("a").unwrap();
        p.process("b").unwrap();
        p.process("a").unwrap(); // touch a, so b is the LRU entry
        p.process("c").unwrap(); // evicts b
        assert_eq!(p.len(), 2);
        let before = p.stats();
        p.process("a").unwrap();
        assert_eq!(p.stats().hits, before.hits + 1, "a survived");
        p.process("b").unwrap();
        assert_eq!(
            p.stats().misses,
            before.misses + 1,
            "b was evicted"
        );
    }

    #[test]
    fn colliding_key_with_different_content_is_a_miss() {
        let p = CachingProcessor::new(MarkdownOptions::default());
        let key = (Variant::Html, hash_content("# one"));
        p.lock().insert(
            key,
            Entry {
                content: "# other".into(),
                html: "<p>stale</p>".into(),
                toc: Vec::new(),
                last_used: 0,
            },
            10,
        );
        assert!(p.lock().lookup(key, "# one").is_none());
    }

    #[test]
    fn zero_capacity_never_stores() {
        let p = CachingProcessor::with_capacity(
            MarkdownOptions::default(),
            0,
        );
        p.process("# x").unwrap();
        p.process("# x").unwrap();
        assert!(p.is_empty());
        assert_eq!(p.stats(), CacheStats { hits: 0, misses: 2 });
    }
}
