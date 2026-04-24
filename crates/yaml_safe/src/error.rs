// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{de, ser};
use std::{
    error::Error as StdError,
    fmt::{self, Debug, Display},
    io, result,
};

/// An error that occurred during YAML processing.
pub struct Error(Box<ErrorImpl>);

/// Alias for a `Result` with error type `serde_yml::Error`.
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
enum ErrorImpl {
    Message(String),
    MessageAt(String, Location),
    Io(io::Error),
}

/// The input location where an error occurred.
#[derive(Clone, Copy, Debug)]
pub struct Location {
    index: usize,
    line: usize,
    column: usize,
}

impl Location {
    /// Returns the byte index where the error occurred.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns the line number where the error occurred.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the column number where the error occurred.
    pub fn column(&self) -> usize {
        self.column
    }
}

impl Error {
    /// Returns the I/O error that caused this, if any.
    pub fn io_error(&self) -> Option<&io::Error> {
        if let ErrorImpl::Io(err) = &*self.0 {
            Some(err)
        } else {
            None
        }
    }

    /// Returns the location where the error occurred.
    pub fn location(&self) -> Option<Location> {
        match &*self.0 {
            ErrorImpl::MessageAt(_, loc) => Some(*loc),
            _ => None,
        }
    }

    pub(crate) fn msg(s: impl Display) -> Self {
        Error(Box::new(ErrorImpl::Message(s.to_string())))
    }

    pub(crate) fn msg_at(s: impl Display, loc: Location) -> Self {
        Error(Box::new(ErrorImpl::MessageAt(s.to_string(), loc)))
    }
}

impl Location {
    /// Creates a new `Location`.
    pub fn new(index: usize, line: usize, column: usize) -> Self {
        Location {
            index,
            line,
            column,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self.0 {
            ErrorImpl::Message(msg) => f.write_str(msg),
            ErrorImpl::MessageAt(msg, loc) => {
                write!(
                    f,
                    "{} at line {} column {}",
                    msg, loc.line, loc.column
                )
            }
            ErrorImpl::Io(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error({:?})", self.to_string())
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &*self.0 {
            ErrorImpl::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl Clone for Error {
    fn clone(&self) -> Self {
        match &*self.0 {
            ErrorImpl::Message(msg) => {
                Error(Box::new(ErrorImpl::Message(msg.clone())))
            }
            ErrorImpl::MessageAt(msg, loc) => {
                Error(Box::new(ErrorImpl::MessageAt(msg.clone(), *loc)))
            }
            ErrorImpl::Io(err) => {
                Error(Box::new(ErrorImpl::Message(err.to_string())))
            }
        }
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error(Box::new(ErrorImpl::Message(msg.to_string())))
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error(Box::new(ErrorImpl::Message(msg.to_string())))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error(Box::new(ErrorImpl::Io(err)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::Error as DeError;
    use serde::ser::Error as SerError;

    #[test]
    fn location_is_none_for_message_without_location() {
        let err = Error::msg("no loc");
        assert!(err.location().is_none());
    }

    #[test]
    fn source_is_none_for_non_io_variant() {
        let err = Error::msg("plain");
        assert!(err.source().is_none());

        let loc = Location::new(0, 1, 1);
        let err_at = Error::msg_at("at", loc);
        assert!(err_at.source().is_none());
    }

    #[test]
    fn clone_on_message_variant_preserves_text() {
        let err = Error::msg("hello");
        let cloned = err.clone();
        assert_eq!(format!("{cloned}"), "hello");
    }

    #[test]
    fn ser_error_custom_builds_message_variant() {
        let err: Error = <Error as SerError>::custom("boom");
        assert_eq!(format!("{err}"), "boom");
    }

    #[test]
    fn de_error_custom_builds_message_variant() {
        let err: Error = <Error as DeError>::custom("de");
        assert_eq!(format!("{err}"), "de");
    }

    #[test]
    fn display_io_variant_prefixes() {
        let io = io::Error::other("disk");
        let err: Error = io.into();
        let s = format!("{err}");
        assert!(s.starts_with("I/O error:"));
        assert!(s.contains("disk"));
    }

    #[test]
    fn location_accessors() {
        let loc = Location::new(42, 3, 7);
        assert_eq!(loc.index(), 42);
        assert_eq!(loc.line(), 3);
        assert_eq!(loc.column(), 7);
    }
}
