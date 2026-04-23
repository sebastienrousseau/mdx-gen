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
}
