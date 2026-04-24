// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Serialize/deserialize `Option<Enum>` as an optional
//! single-key YAML mapping.
//!
//! Combines `Option` handling with `singleton_map`.
//! `None` serializes as YAML `null`, `Some(variant)`
//! serializes as a single-key mapping.
//!
//! # Usage
//!
//! ```ignore
//! #[derive(Serialize, Deserialize)]
//! struct Config {
//!     #[serde(with = "yaml_safe::with::singleton_map_optional")]
//!     mode: Option<Mode>,
//! }
//! ```

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::singleton_map;
use crate::value::Value;

/// Serialize an `Option<T>` where `T` is an enum, using
/// singleton-map representation for `Some(variant)`.
pub fn serialize<T, S>(
    value: &Option<T>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    match value {
        None => serializer.serialize_none(),
        Some(inner) => singleton_map::serialize(inner, serializer),
    }
}

/// Deserialize an `Option<T>` where `T` is an enum, from a
/// singleton-map or null.
pub fn deserialize<'de, T, D>(
    deserializer: D,
) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    let v = Value::deserialize(deserializer)?;
    match v {
        Value::Null => Ok(None),
        other => {
            let converted = singleton_map::from_singleton_map(other);
            T::deserialize(converted)
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
    }
}
