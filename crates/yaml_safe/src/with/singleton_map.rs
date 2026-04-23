// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Serialize/deserialize enums as single-key YAML mappings.
//!
//! By default, serde serializes externally-tagged enums as
//! `variant_name` (unit) or `{variant: value}` (other). This
//! module forces *all* variants — including unit variants — to
//! use the single-key mapping form:
//!
//! ```yaml
//! variant_name: null   # unit variant
//! variant_name: value  # newtype/tuple/struct variant
//! ```
//!
//! # Usage
//!
//! ```ignore
//! #[derive(Serialize, Deserialize)]
//! struct Config {
//!     #[serde(with = "yaml_safe::with::singleton_map")]
//!     mode: Mode,
//! }
//! ```

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::Error;
use crate::mapping::Mapping;
use crate::value::Value;

/// Serialize an enum as a single-key mapping.
pub fn serialize<T, S>(
    value: &T,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    // Serialize to Value first, then convert unit variants
    let v = value
        .serialize(crate::value::ValueSerializer)
        .map_err(serde::ser::Error::custom)?;
    let mapped = to_singleton_map(v);
    mapped.serialize(serializer)
}

/// Deserialize an enum from a single-key mapping.
pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    let v = Value::deserialize(deserializer)?;
    let converted = from_singleton_map(v);
    T::deserialize(converted).map_err(serde::de::Error::custom)
}

/// Convert a Value representing an enum to singleton-map form.
pub(crate) fn to_singleton_map(v: Value) -> Value {
    match v {
        // Unit variant: "VariantName" → {VariantName: null}
        Value::String(s) => {
            let mut m = Mapping::new();
            m.insert(Value::String(s), Value::Null);
            Value::Mapping(m)
        }
        // Already a mapping (newtype/tuple/struct variant)
        other => other,
    }
}

/// Convert a singleton-map Value back to enum form.
pub(crate) fn from_singleton_map(v: Value) -> Value {
    match v {
        Value::Mapping(ref m) if m.len() == 1 => {
            let (_, val) = m.iter().next().expect("len == 1");
            if val.is_null() {
                // Could be a unit variant: {Name: null} → "Name"
                // But it might also be a legitimate newtype with
                // null value. We leave it as mapping and let serde
                // decide — the enum deserializer handles both.
                v
            } else {
                v
            }
        }
        other => other,
    }
}

/// Applies singleton_map transformation to a Value, suitable
/// for use in `apply_to_value` patterns.
pub fn apply_to_value(v: &mut Value) -> Result<(), Error> {
    *v = to_singleton_map(std::mem::take(v));
    Ok(())
}
