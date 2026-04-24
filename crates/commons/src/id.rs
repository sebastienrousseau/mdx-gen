//! ID generation utilities.
//!
//! Provides various ID generation strategies including timestamp-based,
//! random, and sortable IDs.
//!
//! # Example
//!
//! ```rust
//! use commons::id::{generate_id, IdFormat};
//!
//! let id = generate_id(IdFormat::Timestamp);
//! println!("Generated ID: {}", id);
//! ```

use std::fmt::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Counter for timestamp-based ID uniqueness within the same millisecond.
static TIMESTAMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Counter for entropy seeding in random byte generation.
static ENTROPY_COUNTER: AtomicU64 = AtomicU64::new(0);

/// ID format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdFormat {
    /// Timestamp-based ID (sortable, 20 chars).
    Timestamp,
    /// Random hex ID (32 chars).
    RandomHex,
    /// Short random ID (12 chars, base62).
    Short,
    /// Prefixed ID with custom prefix.
    Prefixed,
}

/// Generate a unique ID.
///
/// # Arguments
///
/// * `format` - The ID format to use
///
/// # Returns
///
/// A unique string ID.
#[must_use]
pub fn generate_id(format: IdFormat) -> String {
    match format {
        IdFormat::RandomHex => generate_random_hex(),
        IdFormat::Short => generate_short_id(),
        IdFormat::Timestamp | IdFormat::Prefixed => generate_timestamp_id(),
    }
}

/// Generate a prefixed ID.
///
/// # Arguments
///
/// * `prefix` - Prefix string (e.g., "usr", "ord")
///
/// # Returns
///
/// A prefixed unique ID like `usr_abc123`.
#[must_use]
pub fn generate_prefixed_id(prefix: &str) -> String {
    format!("{prefix}_{}", generate_short_id())
}

/// Generate a timestamp-based sortable ID.
///
/// Format: 13 digits timestamp + 7 digits counter = 20 chars
/// IDs generated in the same millisecond are still unique and sortable.
#[must_use]
pub fn generate_timestamp_id() -> String {
    let timestamp = current_timestamp_millis();
    let counter = TIMESTAMP_COUNTER.fetch_add(1, Ordering::SeqCst) % 10_000_000;
    format!("{timestamp:013}{counter:07}")
}

/// Generate a random hexadecimal ID (32 characters).
#[must_use]
pub fn generate_random_hex() -> String {
    let mut bytes = [0u8; 16];
    fill_random_bytes(&mut bytes);
    bytes.iter().fold(String::with_capacity(32), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// Generate a short random ID (12 characters, base62).
#[must_use]
pub fn generate_short_id() -> String {
    const CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut bytes = [0u8; 12];
    fill_random_bytes(&mut bytes);

    bytes
        .iter()
        .map(|b| CHARS[(*b as usize) % CHARS.len()] as char)
        .collect()
}

/// Generate a UUID v4-like string.
///
/// Note: This is not a cryptographically secure UUID.
/// For production use, consider the `uuid` crate.
#[must_use]
pub fn generate_uuid_like() -> String {
    let mut bytes = [0u8; 16];
    fill_random_bytes(&mut bytes);

    // Set version (4) and variant bits
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

/// Get current timestamp in milliseconds.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn current_timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Fill a byte slice with pseudo-random values.
fn fill_random_bytes(bytes: &mut [u8]) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let counter = ENTROPY_COUNTER.fetch_add(1, Ordering::SeqCst);
    let timestamp = current_timestamp_millis();

    // Use multiple sources of entropy
    let mut hasher = DefaultHasher::new();
    timestamp.hash(&mut hasher);
    counter.hash(&mut hasher);
    std::process::id().hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);

    let mut seed = hasher.finish();

    // Simple xorshift for pseudo-randomness
    for byte in bytes.iter_mut() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        *byte = (seed & 0xff) as u8;
        // Mix in counter for each byte
        seed = seed.wrapping_add(counter);
    }
}

/// ID generator with configuration.
#[derive(Debug, Clone)]
pub struct IdGenerator {
    prefix: Option<String>,
    format: IdFormat,
}

impl IdGenerator {
    /// Create a new ID generator.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a prefix for generated IDs.
    #[must_use]
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Set the ID format.
    #[must_use]
    pub const fn with_format(mut self, format: IdFormat) -> Self {
        self.format = format;
        self
    }

    /// Generate an ID.
    #[must_use]
    pub fn generate(&self) -> String {
        let id = generate_id(self.format);
        match &self.prefix {
            Some(p) => format!("{p}_{id}"),
            None => id,
        }
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self {
            prefix: None,
            format: IdFormat::Short,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_timestamp_id_format() {
        let id = generate_timestamp_id();
        assert_eq!(id.len(), 20);
        assert!(id.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_timestamp_ids_are_sortable() {
        let id1 = generate_timestamp_id();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let id2 = generate_timestamp_id();
        assert!(id1 < id2);
    }

    #[test]
    fn test_timestamp_ids_unique_in_same_ms() {
        let ids: Vec<String> = (0..100).map(|_| generate_timestamp_id()).collect();
        let unique: HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len());
    }

    #[test]
    fn test_random_hex_format() {
        let id = generate_random_hex();
        assert_eq!(id.len(), 32);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_short_id_format() {
        let id = generate_short_id();
        assert_eq!(id.len(), 12);
        assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_uuid_like_format() {
        let id = generate_uuid_like();
        assert_eq!(id.len(), 36);
        assert_eq!(id.chars().filter(|&c| c == '-').count(), 4);
    }

    #[test]
    fn test_prefixed_id() {
        let id = generate_prefixed_id("usr");
        assert!(id.starts_with("usr_"));
        assert_eq!(id.len(), 4 + 12); // "usr_" + 12 char short id
    }

    #[test]
    fn test_id_generator() {
        let generator = IdGenerator::new()
            .with_prefix("order")
            .with_format(IdFormat::Short);

        let id = generator.generate();
        assert!(id.starts_with("order_"));
    }

    #[test]
    fn test_uniqueness() {
        let ids: HashSet<String> = (0..1000).map(|_| generate_short_id()).collect();
        assert_eq!(ids.len(), 1000);
    }

    #[test]
    fn test_counter_isolation() {
        // Heavy random ID generation should not exhaust the timestamp counter domain.
        let _random_ids: Vec<String> = (0..1000).map(|_| generate_random_hex()).collect();

        // Timestamp IDs should still be unique after heavy random generation.
        let ts_ids: HashSet<String> = (0..100).map(|_| generate_timestamp_id()).collect();
        assert_eq!(ts_ids.len(), 100);
    }
}
