//! Structured logging and telemetry utilities.

use std::fmt;

/// Log levels for structured logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Trace level - very verbose debugging
    Trace = 0,
    /// Debug level - debugging information
    Debug = 1,
    /// Info level - general information
    Info = 2,
    /// Warn level - warning messages
    Warn = 3,
    /// Error level - error messages
    Error = 4,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Trace => write!(f, "TRACE"),
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Warn => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// Simple structured logger.
///
/// Lightweight wrapper that prints timestamped, level-filtered messages to
/// stdout. Each `Logger` owns its module name as a `String` — creating one
/// allocates, so prefer storing it rather than constructing per-call.
///
/// For high-throughput or production logging, consider pairing this with the
/// [`log`](https://crates.io/crates/log) crate facade.
#[derive(Debug)]
pub struct Logger {
    level: LogLevel,
    module: String,
}

impl Logger {
    /// Create a new logger for a module
    #[must_use]
    pub fn new(module: &str) -> Self {
        Self {
            level: LogLevel::Info,
            module: module.to_string(),
        }
    }

    /// Set the minimum log level
    pub const fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    /// Log a message at the given level
    pub fn log(&self, level: LogLevel, message: &str) {
        if level >= self.level {
            let timestamp = Self::timestamp();
            println!("[{timestamp}] {level} [{}] {message}", self.module);
        }
    }

    /// Get the current Unix timestamp in seconds.
    ///
    /// Uses `crate::time::unix_timestamp()` when the `time` feature is
    /// enabled (which the `logging` feature implies). Falls back to raw
    /// `SystemTime` arithmetic otherwise, so logging still compiles even
    /// if the dependency chain is manually overridden.
    #[cfg(feature = "time")]
    fn timestamp() -> u64 {
        crate::time::unix_timestamp()
    }

    /// Fallback timestamp when the `time` feature is absent.
    #[cfg(not(feature = "time"))]
    fn timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Log a trace message
    pub fn trace(&self, message: &str) {
        self.log(LogLevel::Trace, message);
    }

    /// Log a debug message
    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }

    /// Log an info message
    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    /// Log a warning message
    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message);
    }

    /// Log an error message
    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }
}

/// Create a logger for the current module.
///
/// Returns a [`Logger`] whose module name is set to the caller's
/// [`module_path!()`]. This is the recommended way to obtain a logger
/// without hard-coding module strings.
#[cfg(feature = "logging")]
#[macro_export]
macro_rules! logger {
    () => {
        $crate::logging::Logger::new(module_path!())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_basic() {
        let logger = Logger::new("test_module");
        // Exercises the crate::time::unix_timestamp() path — will panic
        // if the `time` feature is not correctly pulled in by `logging`.
        logger.info("basic log test");
    }

    #[test]
    fn test_logger_level_filtering() {
        let mut logger = Logger::new("filter_test");
        logger.set_level(LogLevel::Warn);

        // These should not panic — they are simply filtered out.
        logger.trace("should be filtered");
        logger.debug("should be filtered");
        logger.info("should be filtered");

        // These should print (level >= Warn).
        logger.warn("visible warning");
        logger.error("visible error");
    }
}
