// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Recursively apply singleton-map enum representation to
//! all nested enum values within a structure.
//!
//! This is useful when you have deeply nested structures
//! containing enums that should all use the single-key
//! mapping form.
//!
//! # Usage
//!
//! ```ignore
//! #[derive(Serialize, Deserialize)]
//! struct Config {
//!     #[serde(with = "yaml_safe::with::singleton_map_recursive")]
//!     nested: NestedWithEnums,
//! }
//! ```

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::singleton_map;
use crate::value::Value;

/// Serialize a value, recursively converting all enum
/// representations to singleton-map form.
pub fn serialize<T, S>(
    value: &T,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    let v = value
        .serialize(crate::value::ValueSerializer)
        .map_err(serde::ser::Error::custom)?;
    let mapped = apply_recursive(v);
    mapped.serialize(serializer)
}

/// Deserialize a value, accepting singleton-map enum
/// representations recursively.
pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    let v = Value::deserialize(deserializer)?;
    // No transformation needed on deserialize — the
    // singleton-map form is already what serde expects
    // for externally-tagged enums as mappings.
    T::deserialize(v).map_err(serde::de::Error::custom)
}

/// Recursively apply singleton-map to all enum-like values.
fn apply_recursive(v: Value) -> Value {
    match v {
        // String that looks like a unit variant gets
        // wrapped as singleton_map at the point of use,
        // not recursively (we can't tell strings from
        // enum variants at the Value level without
        // schema context).
        Value::Sequence(seq) => Value::Sequence(
            seq.into_iter().map(apply_recursive).collect(),
        ),
        Value::Mapping(m) => {
            let mut new_m = crate::mapping::Mapping::new();
            for (k, val) in m {
                new_m.insert(apply_recursive(k), apply_recursive(val));
            }
            Value::Mapping(new_m)
        }
        Value::Tagged(t) => {
            Value::Tagged(Box::new(crate::value::tagged::TaggedValue {
                tag: t.tag,
                value: apply_recursive(t.value),
            }))
        }
        other => singleton_map::to_singleton_map(other),
    }
}
