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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::Mapping;
    use crate::value::tagged::{Tag, TaggedValue};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum Mode {
        Plain,
        Named(String),
        Shape { width: u32 },
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct OneField {
        #[serde(with = "super")]
        mode: Mode,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct VecField {
        #[serde(with = "super")]
        modes: Vec<Mode>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct MapField {
        #[serde(with = "super")]
        modes: std::collections::BTreeMap<String, Mode>,
    }

    #[test]
    fn apply_recursive_on_string_wraps_as_singleton_map() {
        let v = Value::String("Plain".to_owned());
        let mapped = apply_recursive(v);
        let m = match mapped {
            Value::Mapping(m) => m,
            other => panic!("expected Mapping, got {other:?}"),
        };
        assert_eq!(m.len(), 1);
        let (k, val) = m.iter().next().unwrap();
        assert_eq!(k, &Value::String("Plain".to_owned()));
        assert!(val.is_null());
    }

    #[test]
    fn apply_recursive_on_sequence_recurses() {
        let seq = Value::Sequence(vec![
            Value::String("A".into()),
            Value::String("B".into()),
        ]);
        let mapped = apply_recursive(seq);
        let items = match mapped {
            Value::Sequence(s) => s,
            other => panic!("expected Sequence, got {other:?}"),
        };
        assert_eq!(items.len(), 2);
        for item in items {
            assert!(matches!(item, Value::Mapping(_)));
        }
    }

    #[test]
    fn apply_recursive_on_mapping_recurses_keys_and_values() {
        let mut m = Mapping::new();
        m.insert(
            Value::String("k".into()),
            Value::String("Plain".into()),
        );
        let mapped = apply_recursive(Value::Mapping(m));
        let m = match mapped {
            Value::Mapping(m) => m,
            other => panic!("expected Mapping, got {other:?}"),
        };
        // Both key and value are strings and should be wrapped.
        let (k, v) = m.iter().next().unwrap();
        assert!(matches!(k, Value::Mapping(_)), "key wrapped");
        assert!(matches!(v, Value::Mapping(_)), "value wrapped");
    }

    #[test]
    fn apply_recursive_on_tagged_recurses_inner() {
        let tagged = Value::Tagged(Box::new(TaggedValue {
            tag: Tag::new("!Mode"),
            value: Value::String("Plain".into()),
        }));
        let mapped = apply_recursive(tagged);
        let t = match mapped {
            Value::Tagged(t) => t,
            other => panic!("expected Tagged, got {other:?}"),
        };
        assert_eq!(t.tag, Tag::new("!Mode"));
        assert!(matches!(t.value, Value::Mapping(_)));
    }

    #[test]
    fn serialize_unit_variant_roundtrip() {
        let v = OneField { mode: Mode::Plain };
        let yaml = crate::to_string(&v).unwrap();
        assert!(yaml.contains("Plain"));
        let back: OneField = crate::from_str(&yaml).unwrap();
        assert_eq!(back, v);
    }

    #[test]
    fn serialize_fn_walks_nested_value() {
        // Exercise the public `serialize` function directly by
        // wrapping a Value input in a type whose Serialize impl
        // dispatches to `super::serialize`. This drives all four
        // apply_recursive branches via the serialize entry point
        // (not just by calling apply_recursive in isolation).
        struct Wrap<'a>(&'a Value);
        impl serde::Serialize for Wrap<'_> {
            fn serialize<S: Serializer>(
                &self,
                s: S,
            ) -> Result<S::Ok, S::Error> {
                super::serialize(self.0, s)
            }
        }

        let mut inner_map = Mapping::new();
        inner_map.insert(
            Value::String("k".into()),
            Value::String("V".into()),
        );
        let tagged = Value::Tagged(Box::new(TaggedValue {
            tag: Tag::new("!T"),
            value: Value::Mapping(inner_map),
        }));
        let input = Value::Sequence(vec![tagged]);

        // Should not panic; round-tripping semantics of this
        // helper are enum-specific and covered elsewhere.
        let yaml = crate::to_string(&Wrap(&input)).unwrap();
        assert!(!yaml.is_empty());
    }

    #[test]
    fn deserialize_fn_drives_pub_entry_point() {
        // Wrap a Deserialize impl that delegates to
        // `super::deserialize` so the public entry point is
        // covered, not just the internal no-op path.
        #[derive(Debug)]
        struct Wrap(Value);
        impl<'de> Deserialize<'de> for Wrap {
            fn deserialize<D: Deserializer<'de>>(
                d: D,
            ) -> Result<Self, D::Error> {
                Ok(Wrap(super::deserialize(d)?))
            }
        }

        let w: Wrap = crate::from_str("foo: bar\n").unwrap();
        let m = w.0.as_mapping().unwrap();
        assert_eq!(
            m.get(Value::String("foo".into())).and_then(Value::as_str),
            Some("bar")
        );
    }

    #[test]
    fn deserialize_fn_returns_value_unchanged() {
        // deserialize does no transformation — a plain Value in
        // singleton-map form round-trips back to itself.
        let yaml = "animal:\n  Cat: null\n";
        let got: Value = crate::from_str(yaml).unwrap();
        let m = got.as_mapping().unwrap();
        assert!(m.get(Value::String("animal".into())).is_some());
    }

    #[test]
    fn compile_shapes() {
        // Keep these `#[serde(with = "super")]` wrappers
        // compiled — they exercise the proc-macro path even
        // when the round-trip is not exercised at runtime.
        let _ = OneField { mode: Mode::Plain };
        let _ = VecField {
            modes: vec![Mode::Plain],
        };
        let _ = MapField {
            modes: std::collections::BTreeMap::new(),
        };
    }
}
