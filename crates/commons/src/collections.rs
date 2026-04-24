//! Specialized data structures and collection utilities.

use std::collections::HashMap;
use std::hash::Hash;

/// A cache with a maximum capacity that evicts the least recently used items.
///
/// This implementation uses a `Vec` for order tracking, giving O(N) access
/// and insertion cost per operation. It is designed for small-capacity caches
/// (up to ~100 entries). For high-throughput or large-capacity use cases,
/// consider the [`lru`](https://crates.io/crates/lru) crate which provides
/// O(1) operations via a doubly-linked list and `HashMap`.
#[derive(Debug)]
pub struct LruCache<K, V> {
    capacity: usize,
    data: HashMap<K, V>,
    order: Vec<K>,
}

impl<K: Clone + Hash + Eq, V> LruCache<K, V> {
    /// Create a new LRU cache with the given capacity
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            data: HashMap::new(),
            order: Vec::new(),
        }
    }

    /// Insert a key-value pair into the cache
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if let Some(old_value) = self.data.insert(key.clone(), value) {
            // Key already existed, update order
            self.move_to_front(&key);
            Some(old_value)
        } else {
            // New key
            self.order.insert(0, key);
            self.evict_if_needed();
            None
        }
    }

    /// Get a value from the cache, updating its position
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if self.data.contains_key(key) {
            self.move_to_front(key);
            self.data.get(key)
        } else {
            None
        }
    }

    /// Get a value without updating its position
    #[must_use]
    pub fn peek(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }

    /// Remove a key from the cache
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(value) = self.data.remove(key) {
            self.order.retain(|k| k != key);
            Some(value)
        } else {
            None
        }
    }

    /// Get the current size of the cache
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the cache is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear all items from the cache
    pub fn clear(&mut self) {
        self.data.clear();
        self.order.clear();
    }

    fn move_to_front(&mut self, key: &K) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            let key = self.order.remove(pos);
            self.order.insert(0, key);
        }
    }

    fn evict_if_needed(&mut self) {
        while self.order.len() > self.capacity {
            if let Some(key) = self.order.pop() {
                self.data.remove(&key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_basic() {
        let mut cache = LruCache::new(2);

        cache.insert(1, "one");
        cache.insert(2, "two");

        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_lru_cache_eviction() {
        let mut cache = LruCache::new(2);

        cache.insert(1, "one");
        cache.insert(2, "two");
        cache.insert(3, "three"); // Should evict key 1

        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_insert_existing_key_updates_value_and_returns_old() {
        let mut cache = LruCache::new(3);
        assert_eq!(cache.insert("k", 1), None);
        assert_eq!(cache.insert("k", 2), Some(1));
        assert_eq!(cache.get(&"k"), Some(&2));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_peek_does_not_update_order() {
        let mut cache = LruCache::new(2);
        cache.insert(1, "a");
        cache.insert(2, "b");

        // peek should not promote key 1, so inserting a third
        // entry evicts 1.
        assert_eq!(cache.peek(&1), Some(&"a"));
        cache.insert(3, "c");
        assert_eq!(cache.peek(&1), None);
        assert_eq!(cache.peek(&2), Some(&"b"));
        assert_eq!(cache.peek(&3), Some(&"c"));
    }

    #[test]
    fn test_peek_on_missing_key_returns_none() {
        let cache: LruCache<i32, i32> = LruCache::new(2);
        assert!(cache.peek(&42).is_none());
    }

    #[test]
    fn test_remove_existing_and_missing() {
        let mut cache = LruCache::new(2);
        cache.insert(1, "a");
        cache.insert(2, "b");
        assert_eq!(cache.remove(&1), Some("a"));
        assert_eq!(cache.len(), 1);
        assert!(cache.get(&1).is_none());
        // Removing a missing key is a no-op that returns None.
        assert_eq!(cache.remove(&99), None);
    }

    #[test]
    fn test_is_empty_and_clear() {
        let mut cache = LruCache::new(2);
        assert!(cache.is_empty());
        cache.insert(1, "a");
        assert!(!cache.is_empty());
        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        // Ordering vec was also cleared — reinsert works normally.
        cache.insert(1, "a");
        cache.insert(2, "b");
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_get_promotes_mru_so_next_eviction_drops_lru() {
        let mut cache = LruCache::new(2);
        cache.insert(1, "a");
        cache.insert(2, "b");
        // Promote 1 to front.
        let _ = cache.get(&1);
        // Inserting 3 evicts the LRU key — now 2.
        cache.insert(3, "c");
        assert_eq!(cache.get(&1), Some(&"a"));
        assert!(cache.get(&2).is_none());
    }

    #[test]
    fn test_get_on_missing_key_is_none() {
        let mut cache: LruCache<i32, i32> = LruCache::new(2);
        assert!(cache.get(&0).is_none());
    }

    #[test]
    fn test_debug_format() {
        let cache: LruCache<i32, i32> = LruCache::new(2);
        let s = format!("{cache:?}");
        assert!(s.contains("LruCache"));
    }
}
