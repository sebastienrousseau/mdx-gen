//! Retry logic with configurable backoff strategies.
//!
//! # Example
//!
//! ```rust
//! use commons::retry::{retry, RetryConfig, BackoffStrategy};
//! use std::time::Duration;
//!
//! let config = RetryConfig::new()
//!     .max_attempts(3)
//!     .backoff(BackoffStrategy::Exponential {
//!         initial: Duration::from_millis(100),
//!         max: Duration::from_secs(5),
//!         multiplier: 2.0,
//!     });
//!
//! let result = retry(config, || {
//!     // Operation that might fail
//!     Ok::<_, &str>("success")
//! });
//! ```

use std::thread;
use std::time::Duration;

/// Backoff strategy for retries.
#[derive(Debug, Clone, Copy)]
pub enum BackoffStrategy {
    /// No delay between retries.
    None,

    /// Constant delay between retries.
    Constant(Duration),

    /// Linear backoff: delay increases linearly.
    Linear {
        /// Initial delay.
        initial: Duration,
        /// Increment per attempt.
        increment: Duration,
        /// Maximum delay.
        max: Duration,
    },

    /// Exponential backoff: delay doubles each attempt.
    Exponential {
        /// Initial delay.
        initial: Duration,
        /// Maximum delay.
        max: Duration,
        /// Multiplier (typically 2.0).
        multiplier: f64,
    },
}

impl BackoffStrategy {
    /// Calculate delay for a given attempt number (0-indexed).
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        clippy::cast_possible_wrap
    )]
    pub fn delay_for_attempt(&self, attempt: usize) -> Duration {
        match self {
            Self::None => Duration::ZERO,

            Self::Constant(d) => *d,

            Self::Linear {
                initial,
                increment,
                max,
            } => {
                let delay = *initial + (*increment * attempt as u32);
                delay.min(*max)
            }

            Self::Exponential {
                initial,
                max,
                multiplier,
            } => {
                let mult = multiplier.powi(attempt as i32);
                let delay_nanos = initial.as_nanos() as f64 * mult;
                let delay = Duration::from_nanos(delay_nanos as u64);
                delay.min(*max)
            }
        }
    }
}

impl Default for BackoffStrategy {
    fn default() -> Self {
        Self::Exponential {
            initial: Duration::from_millis(100),
            max: Duration::from_secs(30),
            multiplier: 2.0,
        }
    }
}

/// Configuration for retry behavior.
#[derive(Debug, Clone, Copy)]
pub struct RetryConfig {
    /// Maximum number of attempts (including first try).
    pub max_attempts: usize,
    /// Backoff strategy between attempts.
    pub backoff: BackoffStrategy,
    /// Whether to add jitter to delays.
    pub jitter: bool,
}

impl RetryConfig {
    /// Create a new retry configuration with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum number of attempts.
    #[must_use]
    pub const fn max_attempts(mut self, n: usize) -> Self {
        if n < 1 {
            self.max_attempts = 1;
        } else {
            self.max_attempts = n;
        }
        self
    }

    /// Set backoff strategy.
    #[must_use]
    pub const fn backoff(mut self, strategy: BackoffStrategy) -> Self {
        self.backoff = strategy;
        self
    }

    /// Enable or disable jitter.
    #[must_use]
    pub const fn jitter(mut self, enabled: bool) -> Self {
        self.jitter = enabled;
        self
    }

    /// Create config for no retries.
    #[must_use]
    pub const fn no_retry() -> Self {
        Self {
            max_attempts: 1,
            backoff: BackoffStrategy::None,
            jitter: false,
        }
    }

    /// Create config with simple constant delay.
    #[must_use]
    pub const fn with_constant_delay(attempts: usize, delay: Duration) -> Self {
        Self {
            max_attempts: attempts,
            backoff: BackoffStrategy::Constant(delay),
            jitter: false,
        }
    }

    /// Create config with exponential backoff.
    #[must_use]
    pub const fn with_exponential_backoff(
        attempts: usize,
        initial: Duration,
        max: Duration,
    ) -> Self {
        Self {
            max_attempts: attempts,
            backoff: BackoffStrategy::Exponential {
                initial,
                max,
                multiplier: 2.0,
            },
            jitter: true,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff: BackoffStrategy::default(),
            jitter: true,
        }
    }
}

/// Result of a retry operation.
#[derive(Debug)]
pub struct RetryResult<T, E> {
    /// The final result.
    pub result: Result<T, E>,
    /// Number of attempts made.
    pub attempts: usize,
    /// Total time spent (including delays).
    pub total_time: Duration,
}

impl<T, E> RetryResult<T, E> {
    /// Check if the operation succeeded.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        self.result.is_ok()
    }

    /// Check if the operation failed.
    #[must_use]
    pub const fn is_err(&self) -> bool {
        self.result.is_err()
    }

    /// Unwrap the result, panicking on error.
    ///
    /// # Panics
    ///
    /// Panics if the result is an error.
    pub fn unwrap(self) -> T
    where
        E: std::fmt::Debug,
    {
        self.result.unwrap()
    }

    /// Get the result, converting error.
    ///
    /// # Errors
    ///
    /// Returns the last error if all retry attempts failed.
    pub fn into_result(self) -> Result<T, E> {
        self.result
    }
}

/// Execute an operation with retries.
///
/// # Arguments
///
/// * `config` - Retry configuration
/// * `operation` - The operation to retry
///
/// # Returns
///
/// The result of the operation, or the last error if all retries failed.
///
/// # Panics
///
/// Panics if `max_attempts` is somehow zero after internal clamping
/// (should be unreachable).
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn retry<T, E, F>(config: RetryConfig, mut operation: F) -> RetryResult<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let start = std::time::Instant::now();
    let mut last_error: Option<E> = None;
    // Defensively clamp: public fields can bypass the builder's min-1 guard.
    let max_attempts = config.max_attempts.max(1);

    for attempt in 0..max_attempts {
        match operation() {
            Ok(value) => {
                return RetryResult {
                    result: Ok(value),
                    attempts: attempt + 1,
                    total_time: start.elapsed(),
                };
            }
            Err(e) => {
                last_error = Some(e);

                // Don't sleep after the last attempt
                if attempt + 1 < max_attempts {
                    let mut delay = config.backoff.delay_for_attempt(attempt);

                    // Add jitter (0-25% of delay)
                    if config.jitter && delay > Duration::ZERO {
                        let jitter_factor = simple_random() * 0.25;
                        let jitter =
                            Duration::from_nanos((delay.as_nanos() as f64 * jitter_factor) as u64);
                        delay += jitter;
                    }

                    if delay > Duration::ZERO {
                        thread::sleep(delay);
                    }
                }
            }
        }
    }

    RetryResult {
        result: Err(last_error.expect("At least one attempt should have been made")),
        attempts: max_attempts,
        total_time: start.elapsed(),
    }
}

/// Execute an operation with retries, with access to attempt number.
///
/// # Panics
///
/// Panics if `max_attempts` is somehow zero after internal clamping
/// (should be unreachable).
pub fn retry_with_context<T, E, F>(config: RetryConfig, mut operation: F) -> RetryResult<T, E>
where
    F: FnMut(usize) -> Result<T, E>,
{
    let start = std::time::Instant::now();
    let mut last_error: Option<E> = None;
    let max_attempts = config.max_attempts.max(1);

    for attempt in 0..max_attempts {
        match operation(attempt) {
            Ok(value) => {
                return RetryResult {
                    result: Ok(value),
                    attempts: attempt + 1,
                    total_time: start.elapsed(),
                };
            }
            Err(e) => {
                last_error = Some(e);

                if attempt + 1 < max_attempts {
                    let delay = config.backoff.delay_for_attempt(attempt);
                    if delay > Duration::ZERO {
                        thread::sleep(delay);
                    }
                }
            }
        }
    }

    RetryResult {
        result: Err(last_error.expect("At least one attempt should have been made")),
        attempts: max_attempts,
        total_time: start.elapsed(),
    }
}

/// Simple pseudo-random number generator (0.0 to 1.0).
fn simple_random() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    f64::from(nanos % 1000) / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn test_retry_succeeds_first_try() {
        let config = RetryConfig::new().max_attempts(3);
        let result = retry(config, || Ok::<_, &str>("success"));

        assert!(result.is_ok());
        assert_eq!(result.attempts, 1);
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_retry_succeeds_after_failures() {
        let attempts = Cell::new(0);
        let config = RetryConfig::new()
            .max_attempts(3)
            .backoff(BackoffStrategy::None);

        let result = retry(config, || {
            let n = attempts.get();
            attempts.set(n + 1);
            if n < 2 { Err("not yet") } else { Ok("success") }
        });

        assert!(result.is_ok());
        assert_eq!(result.attempts, 3);
    }

    #[test]
    fn test_retry_exhausted() {
        let config = RetryConfig::new()
            .max_attempts(3)
            .backoff(BackoffStrategy::None);

        let result = retry(config, || Err::<(), _>("always fails"));

        assert!(result.is_err());
        assert_eq!(result.attempts, 3);
    }

    #[test]
    fn test_backoff_constant() {
        let strategy = BackoffStrategy::Constant(Duration::from_millis(100));
        assert_eq!(strategy.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(strategy.delay_for_attempt(5), Duration::from_millis(100));
    }

    #[test]
    fn test_backoff_exponential() {
        let strategy = BackoffStrategy::Exponential {
            initial: Duration::from_millis(100),
            max: Duration::from_secs(10),
            multiplier: 2.0,
        };

        assert_eq!(strategy.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(strategy.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(strategy.delay_for_attempt(2), Duration::from_millis(400));
        assert_eq!(strategy.delay_for_attempt(3), Duration::from_millis(800));
    }

    #[test]
    fn test_backoff_max_cap() {
        let strategy = BackoffStrategy::Exponential {
            initial: Duration::from_secs(1),
            max: Duration::from_secs(5),
            multiplier: 2.0,
        };

        // 1s * 2^10 = 1024s, but capped at 5s
        assert_eq!(strategy.delay_for_attempt(10), Duration::from_secs(5));
    }

    #[test]
    fn test_no_retry_config() {
        let config = RetryConfig::no_retry();
        assert_eq!(config.max_attempts, 1);
    }

    #[test]
    fn test_zero_max_attempts_does_not_panic() {
        // Bypass the builder by constructing directly with max_attempts = 0.
        let config = RetryConfig {
            max_attempts: 0,
            backoff: BackoffStrategy::None,
            jitter: false,
        };
        let result = retry(config, || Err::<(), _>("fail"));
        // Should clamp to 1 attempt instead of panicking.
        assert!(result.is_err());
        assert_eq!(result.attempts, 1);
    }

    #[test]
    fn test_zero_max_attempts_with_context() {
        let config = RetryConfig {
            max_attempts: 0,
            backoff: BackoffStrategy::None,
            jitter: false,
        };
        let result = retry_with_context(config, |_| Err::<(), _>("fail"));
        assert!(result.is_err());
        assert_eq!(result.attempts, 1);
    }
}
