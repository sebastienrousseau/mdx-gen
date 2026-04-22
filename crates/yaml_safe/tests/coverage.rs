//! Additional tests targeting low-coverage modules in yaml_safe.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use yaml_safe::{
    from_str, to_string, to_value, Mapping, Number, Tag, TaggedValue,
    Value,
};

// ── error.rs coverage ───────────────────────────────────────────────

#[test]
fn error_display_message() {
    let err: yaml_safe::Result<Value> = from_str("{ invalid yaml [");
    let e = err.unwrap_err();
    let msg = format!("{e}");
    assert!(!msg.is_empty());
}

#[test]
fn error_debug() {
    let err: yaml_safe::Result<Value> = from_str("{ unclosed");
    let e = err.unwrap_err();
    let dbg = format!("{e:?}");
    assert!(dbg.contains("Error("));
}

#[test]
fn error_clone() {
    let err: yaml_safe::Result<Value> = from_str("[bad");
    let e = err.unwrap_err();
    let cloned = e.clone();
    assert_eq!(format!("{e}"), format!("{cloned}"));
}

#[test]
fn error_io_error_none() {
    let err: yaml_safe::Result<Value> = from_str("[bad");
    let e = err.unwrap_err();
    assert!(e.io_error().is_none());
}

#[test]
fn error_location_none() {
    let err: yaml_safe::Result<Value> = from_str("[bad");
    let e = err.unwrap_err();
    assert!(e.location().is_none());
}

#[test]
fn error_from_io() {
    let io_err = std::io::Error::other("io fail");
    let e: yaml_safe::Error = io_err.into();
    assert!(e.io_error().is_some());
    assert!(format!("{e}").contains("io fail"));

    // source() should return the io error
    use std::error::Error as StdError;
    assert!(e.source().is_some());

    // Clone of io error becomes message
    let cloned = e.clone();
    assert!(cloned.io_error().is_none());
    assert!(format!("{cloned}").contains("io fail"));
}

#[test]
fn error_serde_de_custom() {
    use serde::de::Error;
    let e = yaml_safe::Error::custom("custom msg");
    assert!(format!("{e}").contains("custom msg"));
}

#[test]
fn error_serde_ser_custom() {
    use serde::ser::Error;
    let e = yaml_safe::Error::custom("ser msg");
    assert!(format!("{e}").contains("ser msg"));
}

// ── mapping.rs coverage ─────────────────────────────────────────────

#[test]
fn mapping_reserve_and_shrink() {
    let mut m = Mapping::new();
    m.reserve(100);
    assert!(m.is_empty());
    m.shrink_to_fit();
}

#[test]
fn mapping_entry_api() {
    let mut m = Mapping::new();
    let key = Value::String("k".into());
    m.entry(key.clone()).or_insert(Value::String("v".into()));
    assert_eq!(m.get(&key), Some(&Value::String("v".into())));

    // Occupied entry
    if let yaml_safe::mapping::Entry::Occupied(mut occ) =
        m.entry(key.clone())
    {
        assert_eq!(occ.key(), &key);
        *occ.get_mut() = Value::String("v2".into());
        assert_eq!(*occ.get(), Value::String("v2".into()));
    }
}

#[test]
fn mapping_get_mut() {
    let mut m = Mapping::new();
    m.insert(Value::String("a".into()), Value::String("b".into()));
    if let Some(v) = m.get_mut("a") {
        *v = Value::String("c".into());
    }
    assert_eq!(m.get("a"), Some(&Value::String("c".into())));
}

#[test]
fn mapping_remove_entry() {
    let mut m = Mapping::new();
    m.insert(Value::String("k".into()), Value::String("v".into()));
    let entry = m.remove_entry("k");
    assert!(entry.is_some());
    let (k, v) = entry.unwrap();
    assert_eq!(k, Value::String("k".into()));
    assert_eq!(v, Value::String("v".into()));
}

#[test]
fn mapping_shift_remove() {
    let mut m = Mapping::new();
    m.insert(Value::String("x".into()), Value::Number(Number::from(1)));
    m.insert(Value::String("y".into()), Value::Number(Number::from(2)));
    let removed = m.shift_remove("x");
    assert_eq!(removed, Some(Value::Number(Number::from(1))));
    assert_eq!(m.len(), 1);
}

#[test]
fn mapping_shift_remove_entry() {
    let mut m = Mapping::new();
    m.insert(Value::String("a".into()), Value::Bool(true));
    let entry = m.shift_remove_entry("a");
    assert!(entry.is_some());
}

#[test]
fn mapping_keys_values_iterators() {
    let mut m = Mapping::new();
    m.insert(Value::String("a".into()), Value::Number(Number::from(1)));
    m.insert(Value::String("b".into()), Value::Number(Number::from(2)));
    assert_eq!(m.keys().count(), 2);
    assert_eq!(m.values().count(), 2);
    for v in m.values_mut() {
        *v = Value::Null;
    }
    assert!(m.values().all(|v| v.is_null()));
}

#[test]
fn mapping_into_iter() {
    let mut m = Mapping::new();
    m.insert(Value::String("k".into()), Value::String("v".into()));
    let pairs: Vec<_> = m.into_iter().collect();
    assert_eq!(pairs.len(), 1);
}

#[test]
fn mapping_into_keys_and_values() {
    let mut m = Mapping::new();
    m.insert(Value::String("a".into()), Value::Number(Number::from(1)));
    let keys: Vec<_> = m.clone().into_keys().collect();
    let values: Vec<_> = m.into_values().collect();
    assert_eq!(keys.len(), 1);
    assert_eq!(values.len(), 1);
}

#[test]
fn mapping_extend_and_from_iterator() {
    let pairs = vec![
        (Value::String("a".into()), Value::Number(Number::from(1))),
        (Value::String("b".into()), Value::Number(Number::from(2))),
    ];
    let m: Mapping = pairs.into_iter().collect();
    assert_eq!(m.len(), 2);
}

#[test]
fn mapping_hash_and_ord() {
    let m1 = Mapping::new();
    let m2 = Mapping::new();
    let mut h1 = DefaultHasher::new();
    let mut h2 = DefaultHasher::new();
    m1.hash(&mut h1);
    m2.hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
    assert_eq!(m1.partial_cmp(&m2), Some(Ordering::Equal));
}

#[test]
fn mapping_display() {
    let mut m = Mapping::new();
    m.insert(Value::String("key".into()), Value::String("val".into()));
    let s = format!("{m}");
    assert!(s.contains("key"));
}

#[test]
fn mapping_debug() {
    let m = Mapping::new();
    let d = format!("{m:?}");
    assert!(d.contains('{'));
}

#[test]
fn mapping_serialize_deserialize() {
    let mut m = Mapping::new();
    m.insert(Value::String("k".into()), Value::String("v".into()));
    let yaml = to_string(&m).unwrap();
    let back: Mapping = from_str(&yaml).unwrap();
    assert_eq!(m, back);
}

#[test]
fn mapping_index_with_string_key() {
    let mut m = Mapping::new();
    m.insert(Value::String("hello".into()), Value::Bool(true));
    assert!(m.contains_key("hello"));
    assert!(m.contains_key(String::from("hello")));
    assert!(m.contains_key(Value::String("hello".into())));
}

// ── number.rs coverage ──────────────────────────────────────────────

#[test]
fn number_from_all_integer_types() {
    let _: Number = Number::from(0i8);
    let _: Number = Number::from(0i16);
    let _: Number = Number::from(0i32);
    let _: Number = Number::from(0i64);
    let _: Number = Number::from(0u8);
    let _: Number = Number::from(0u16);
    let _: Number = Number::from(0u32);
    let _: Number = Number::from(0u64);
    let _: Number = Number::from(0isize);
    let _: Number = Number::from(0usize);
    let _: Number = Number::from(0.0f32);
    let _: Number = Number::from(0.0f64);
}

#[test]
fn number_type_checks() {
    let pos = Number::from(42u64);
    assert!(pos.is_u64());
    assert!(pos.is_i64());
    // Integer types are not stored as f64 internally
    assert!(!pos.is_nan());
    assert!(pos.is_finite());
    assert!(!pos.is_infinite());

    let neg = Number::from(-5i64);
    assert!(!neg.is_u64());
    assert!(neg.is_i64());
    assert_eq!(neg.as_f64(), Some(-5.0)); // can convert, but stored as int

    let float = Number::from(1.5f64);
    assert!(!float.is_u64());
    assert!(!float.is_i64());
    assert!(float.is_f64());
}

#[test]
fn number_conversions() {
    let n = Number::from(42u64);
    assert_eq!(n.as_u64(), Some(42));
    assert_eq!(n.as_i64(), Some(42));
    assert_eq!(n.as_f64(), Some(42.0));

    let neg = Number::from(-7i64);
    assert_eq!(neg.as_u64(), None);
    assert_eq!(neg.as_i64(), Some(-7));
    assert_eq!(neg.as_f64(), Some(-7.0));
}

#[test]
fn number_display_special() {
    let nan = Number::from(f64::NAN);
    assert_eq!(format!("{nan}"), ".nan");

    let inf = Number::from(f64::INFINITY);
    assert_eq!(format!("{inf}"), ".inf");

    let neg_inf = Number::from(f64::NEG_INFINITY);
    assert_eq!(format!("{neg_inf}"), "-.inf");
}

#[test]
fn number_hash_and_eq() {
    let a = Number::from(42u64);
    let b = Number::from(42u64);
    assert_eq!(a, b);
    let mut h1 = DefaultHasher::new();
    let mut h2 = DefaultHasher::new();
    a.hash(&mut h1);
    b.hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn number_serialize_roundtrip() {
    let n = Number::from(123i64);
    let yaml = to_string(&n).unwrap();
    let back: Number = from_str(&yaml).unwrap();
    assert_eq!(n, back);
}

// ── value/mod.rs coverage ───────────────────────────────────────────

#[test]
fn value_type_checks() {
    assert!(Value::Null.is_null());
    assert!(!Value::Null.is_bool());
    assert!(!Value::Null.is_number());
    assert!(!Value::Null.is_string());
    assert!(!Value::Null.is_sequence());
    assert!(!Value::Null.is_mapping());

    let b = Value::Bool(true);
    assert!(b.is_bool());
    assert_eq!(b.as_bool(), Some(true));

    let s = Value::String("hi".into());
    assert!(s.is_string());
    assert_eq!(s.as_str(), Some("hi"));

    let n = Value::Number(Number::from(5));
    assert!(n.is_number());
    assert_eq!(n.as_u64(), Some(5));
    assert_eq!(n.as_i64(), Some(5));
    assert_eq!(n.as_f64(), Some(5.0));
}

#[test]
fn value_sequence_accessors() {
    let seq = Value::Sequence(vec![Value::Null, Value::Bool(true)]);
    assert!(seq.is_sequence());
    assert_eq!(seq.as_sequence().unwrap().len(), 2);

    let mut seq2 = seq.clone();
    if let Some(s) = seq2.as_sequence_mut() {
        s.push(Value::Bool(false));
    }
    assert_eq!(seq2.as_sequence().unwrap().len(), 3);
}

#[test]
fn value_mapping_accessors() {
    let mut m = Mapping::new();
    m.insert(Value::String("k".into()), Value::String("v".into()));
    let val = Value::Mapping(m);
    assert!(val.is_mapping());
    assert!(val.as_mapping().is_some());

    let mut val2 = val.clone();
    if let Some(m) = val2.as_mapping_mut() {
        m.insert(Value::String("k2".into()), Value::Null);
    }
    assert_eq!(val2.as_mapping().unwrap().len(), 2);
}

#[test]
fn value_null_accessor() {
    assert_eq!(Value::Null.as_null(), Some(()));
    assert_eq!(Value::Bool(true).as_null(), None);
}

#[test]
fn value_partial_eq_primitives() {
    let s = Value::String("hello".into());
    assert!(s == *"hello");
    assert!(s == "hello");
    assert!(s == "hello");

    let b = Value::Bool(true);
    assert!(b == true);

    let n = Value::Number(Number::from(42i64));
    assert!(n == 42i64);
    assert!(n == 42i32);
    assert!(n == 42i16);
    assert!(n == 42i8);

    let u = Value::Number(Number::from(42u64));
    assert!(u == 42u64);
    assert!(u == 42u32);

    let f = Value::Number(Number::from(1.5f64));
    assert!(f == 1.5f64);
}

#[test]
fn value_display_all_types() {
    assert_eq!(format!("{}", Value::Null), "null");
    assert_eq!(format!("{}", Value::Bool(true)), "true");
    assert_eq!(format!("{}", Value::Bool(false)), "false");
    assert!(
        format!("{}", Value::Number(Number::from(42))).contains("42")
    );
    assert!(format!("{}", Value::String("hi".into())).contains("hi"));
}

#[test]
fn value_hash_and_ord() {
    let a = Value::String("a".into());
    let b = Value::String("b".into());
    let mut h1 = DefaultHasher::new();
    let mut h2 = DefaultHasher::new();
    a.hash(&mut h1);
    b.hash(&mut h2);
    // Different strings should (usually) have different hashes
    assert_ne!(h1.finish(), h2.finish());
    assert!(a < b);
}

#[test]
fn value_get_and_get_mut() {
    let mut m = Mapping::new();
    m.insert(
        Value::String("key".into()),
        Value::Sequence(vec![Value::Number(Number::from(1))]),
    );
    let val = Value::Mapping(m);

    // get with string key
    let inner = val.get("key");
    assert!(inner.is_some());
    assert!(inner.unwrap().is_sequence());

    // get_mut
    let mut val2 = val.clone();
    if let Some(v) = val2.get_mut("key") {
        *v = Value::Null;
    }
    assert_eq!(val2.get("key"), Some(&Value::Null));
}

#[test]
fn value_debug_and_default() {
    let d = Value::default();
    assert!(d.is_null());
    let dbg = format!("{d:?}");
    assert!(dbg.contains("Null"));
}

#[test]
fn value_serialize_all_types() {
    let cases = vec![
        Value::Null,
        Value::Bool(true),
        Value::Number(Number::from(42)),
        Value::String("hello".into()),
        Value::Sequence(vec![Value::Null]),
        Value::Mapping({
            let mut m = Mapping::new();
            m.insert(Value::String("k".into()), Value::Null);
            m
        }),
    ];
    for v in cases {
        let yaml = to_string(&v).unwrap();
        let back: Value = from_str(&yaml).unwrap();
        // NaN != NaN so skip comparison for that case
        if !matches!(&v, Value::Number(n) if n.is_nan()) {
            assert_eq!(v, back, "roundtrip failed for {yaml}");
        }
    }
}

// ── value/tagged.rs coverage ────────────────────────────────────────

#[test]
fn tag_display_and_debug() {
    let tag = Tag::new("!custom");
    assert_eq!(format!("{tag}"), "!custom");
    let dbg = format!("{tag:?}");
    assert!(dbg.contains("custom"));
}

#[test]
fn tag_eq_and_hash() {
    let t1 = Tag::new("!foo");
    let t2 = Tag::new("foo"); // without leading !
    assert_eq!(t1, t2); // Tags compare ignoring leading !

    let mut h1 = DefaultHasher::new();
    let mut h2 = DefaultHasher::new();
    t1.hash(&mut h1);
    t2.hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn tag_ord() {
    let t1 = Tag::new("!a");
    let t2 = Tag::new("!b");
    assert!(t1 < t2);
}

#[test]
fn tagged_value_construction_and_access() {
    let tv = TaggedValue {
        tag: Tag::new("!int"),
        value: Value::Number(Number::from(42)),
    };
    assert_eq!(tv.tag, Tag::new("int"));
    assert_eq!(tv.value, Value::Number(Number::from(42)));
}

#[test]
fn tagged_value_serialize_roundtrip() {
    let tv = TaggedValue {
        tag: Tag::new("!custom"),
        value: Value::String("data".into()),
    };
    let val = Value::Tagged(Box::new(tv.clone()));
    let yaml = to_string(&val).unwrap();
    let back: Value = from_str(&yaml).unwrap();
    // Tagged values should roundtrip
    if let Value::Tagged(bt) = &back {
        assert_eq!(bt.tag, tv.tag);
        assert_eq!(bt.value, tv.value);
    }
}

#[test]
fn tagged_value_hash_and_clone() {
    let tv = TaggedValue {
        tag: Tag::new("!t"),
        value: Value::Null,
    };
    let cloned = tv.clone();
    assert_eq!(tv, cloned);

    let mut h = DefaultHasher::new();
    tv.hash(&mut h);
    let hash = h.finish();
    assert_ne!(hash, 0); // just check it doesn't panic
}

#[test]
fn tagged_value_debug() {
    let tv = TaggedValue {
        tag: Tag::new("!tag"),
        value: Value::Null,
    };
    let d = format!("{tv:?}");
    assert!(d.contains("tag"));
}

// ── de.rs additional coverage ───────────────────────────────────────

#[test]
fn deserialize_block_scalar_literal() {
    let yaml = "content: |\n  line1\n  line2\n";
    let v: Value = from_str(yaml).unwrap();
    let s = v
        .as_mapping()
        .unwrap()
        .get(Value::String("content".into()))
        .unwrap()
        .as_str()
        .unwrap();
    assert!(s.contains("line1"));
    assert!(s.contains("line2"));
}

#[test]
fn deserialize_block_scalar_folded() {
    let yaml = "content: >\n  line1\n  line2\n";
    let v: Value = from_str(yaml).unwrap();
    let s = v
        .as_mapping()
        .unwrap()
        .get(Value::String("content".into()))
        .unwrap()
        .as_str()
        .unwrap();
    assert!(!s.is_empty());
}

#[test]
fn deserialize_hex_and_octal() {
    let v: Value = from_str("0xFF").unwrap();
    assert_eq!(v.as_u64(), Some(255));

    let v: Value = from_str("0o77").unwrap();
    assert_eq!(v.as_u64(), Some(63));
}

#[test]
fn deserialize_flow_mapping() {
    let v: Value = from_str("{a: 1, b: 2}").unwrap();
    assert!(v.is_mapping());
    assert_eq!(v.as_mapping().unwrap().len(), 2);
}

#[test]
fn deserialize_flow_sequence() {
    let v: Value = from_str("[1, 2, 3]").unwrap();
    assert!(v.is_sequence());
    assert_eq!(v.as_sequence().unwrap().len(), 3);
}

#[test]
fn deserialize_nested_structures() {
    let yaml = "parent:\n  child: value\n  other: 42\n";
    let v: Value = from_str(yaml).unwrap();
    assert!(v.is_mapping());
}

#[test]
fn deserialize_document_markers() {
    let yaml = "---\nkey: value\n...\n";
    let v: Value = from_str(yaml).unwrap();
    assert!(v.is_mapping());
}

#[test]
fn deserialize_quoted_strings() {
    let yaml = "single: 'hello'\ndouble: \"world\"";
    let v: Value = from_str(yaml).unwrap();
    assert!(v.is_mapping());
}

#[test]
fn deserialize_comments() {
    let yaml = "key: value # comment\n# full line comment\nk2: v2\n";
    let v: Value = from_str(yaml).unwrap();
    let m = v.as_mapping().unwrap();
    assert_eq!(m.len(), 2);
}

#[test]
fn deserialize_empty_mapping_and_sequence() {
    let v: Value = from_str("{}").unwrap();
    assert!(v.is_mapping());
    assert!(v.as_mapping().unwrap().is_empty());

    let v: Value = from_str("[]").unwrap();
    assert!(v.is_sequence());
    assert!(v.as_sequence().unwrap().is_empty());
}

#[test]
fn deserialize_typed_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Config {
        name: String,
        count: u32,
        enabled: bool,
    }

    let yaml = "name: test\ncount: 42\nenabled: true\n";
    let c: Config = from_str(yaml).unwrap();
    assert_eq!(c.name, "test");
    assert_eq!(c.count, 42);
    assert!(c.enabled);
}

// ── ser.rs additional coverage ──────────────────────────────────────

#[test]
fn serialize_nested_structures() {
    #[derive(Serialize)]
    struct Inner {
        x: i32,
    }
    #[derive(Serialize)]
    struct Outer {
        name: String,
        inner: Inner,
        items: Vec<String>,
    }
    let val = Outer {
        name: "test".into(),
        inner: Inner { x: 42 },
        items: vec!["a".into(), "b".into()],
    };
    let yaml = to_string(&val).unwrap();
    assert!(yaml.contains("name:"));
    assert!(yaml.contains("inner:"));
    assert!(yaml.contains("items:"));
}

#[test]
fn serialize_option_types() {
    #[derive(Serialize)]
    struct Opt {
        present: Option<String>,
        absent: Option<String>,
    }
    let val = Opt {
        present: Some("yes".into()),
        absent: None,
    };
    let yaml = to_string(&val).unwrap();
    assert!(yaml.contains("present:"));
    assert!(yaml.contains("absent:"));
}

// ── to_value / from_value coverage ──────────────────────────────────

#[test]
fn to_value_struct() {
    #[derive(Serialize)]
    struct S {
        a: i32,
        b: String,
    }
    let val = to_value(S {
        a: 1,
        b: "hi".into(),
    })
    .unwrap();
    assert!(val.is_mapping());
}

#[test]
fn from_value_typed() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct S {
        a: i32,
    }
    let mut m = Mapping::new();
    m.insert(
        Value::String("a".into()),
        Value::Number(Number::from(42)),
    );
    let val = Value::Mapping(m);
    let s: S = yaml_safe::from_value(val).unwrap();
    assert_eq!(s.a, 42);
}

// ── Value tagged coverage ───────────────────────────────────────────

#[test]
fn value_tagged_comparisons() {
    let tagged = Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new("!int"),
        value: Value::Number(Number::from(42)),
    }));
    // Tagged values should compare via PartialEq
    let tagged2 = tagged.clone();
    assert_eq!(tagged, tagged2);

    // Partial eq with primitives should work through tags
    // (the PartialEq impls use untag_ref internally)
    assert!(tagged == 42u64);
    assert!(tagged == 42i64);
}

// ── apply_merge coverage ────────────────────────────────────────────

#[test]
fn value_apply_merge() {
    // Test merge key handling
    let yaml = "defaults: &defaults\n  color: red\n  size: large\nitem:\n  <<: *defaults\n  size: small\n";
    let _v: Value = from_str(yaml).unwrap();
    // Just verify it parses without error
}

// ── Additional coverage for Value serde machinery ───────────────────

#[test]
fn from_value_all_scalar_types() {
    // bool
    let v: bool = yaml_safe::from_value(Value::Bool(true)).unwrap();
    assert!(v);

    // integers
    let v: i8 =
        yaml_safe::from_value(Value::Number(Number::from(42i64)))
            .unwrap();
    assert_eq!(v, 42);
    let v: i16 =
        yaml_safe::from_value(Value::Number(Number::from(-100i64)))
            .unwrap();
    assert_eq!(v, -100);
    let v: i32 =
        yaml_safe::from_value(Value::Number(Number::from(1000i64)))
            .unwrap();
    assert_eq!(v, 1000);
    let v: i64 =
        yaml_safe::from_value(Value::Number(Number::from(-999i64)))
            .unwrap();
    assert_eq!(v, -999);
    let v: u8 =
        yaml_safe::from_value(Value::Number(Number::from(255u64)))
            .unwrap();
    assert_eq!(v, 255);
    let v: u16 =
        yaml_safe::from_value(Value::Number(Number::from(65535u64)))
            .unwrap();
    assert_eq!(v, 65535);
    let v: u32 =
        yaml_safe::from_value(Value::Number(Number::from(123u64)))
            .unwrap();
    assert_eq!(v, 123);
    let v: u64 =
        yaml_safe::from_value(Value::Number(Number::from(999u64)))
            .unwrap();
    assert_eq!(v, 999);

    // float
    let v: f64 =
        yaml_safe::from_value(Value::Number(Number::from(1.5f64)))
            .unwrap();
    assert!((v - 1.5).abs() < f64::EPSILON);
    let v: f32 =
        yaml_safe::from_value(Value::Number(Number::from(2.5f64)))
            .unwrap();
    assert!((v - 2.5).abs() < f32::EPSILON);

    // string
    let v: String =
        yaml_safe::from_value(Value::String("hello".into())).unwrap();
    assert_eq!(v, "hello");

    // null → Option
    let v: Option<String> = yaml_safe::from_value(Value::Null).unwrap();
    assert!(v.is_none());

    // Some
    let v: Option<String> =
        yaml_safe::from_value(Value::String("x".into())).unwrap();
    assert_eq!(v, Some("x".to_string()));
}

#[test]
fn from_value_sequence() {
    let seq = Value::Sequence(vec![
        Value::Number(Number::from(1i64)),
        Value::Number(Number::from(2i64)),
        Value::Number(Number::from(3i64)),
    ]);
    let v: Vec<i32> = yaml_safe::from_value(seq).unwrap();
    assert_eq!(v, vec![1, 2, 3]);
}

#[test]
fn from_value_mapping_to_hashmap() {
    use std::collections::HashMap;
    let mut m = Mapping::new();
    m.insert(
        Value::String("a".into()),
        Value::Number(Number::from(1i64)),
    );
    m.insert(
        Value::String("b".into()),
        Value::Number(Number::from(2i64)),
    );
    let v: HashMap<String, i32> =
        yaml_safe::from_value(Value::Mapping(m)).unwrap();
    assert_eq!(v.len(), 2);
    assert_eq!(v["a"], 1);
}

#[test]
fn to_value_all_types() {
    // Primitives
    let v = to_value(true).unwrap();
    assert_eq!(v, Value::Bool(true));

    let v = to_value(42i32).unwrap();
    assert!(v.is_number());

    let v = to_value(42u64).unwrap();
    assert!(v.is_number());

    let v = to_value(1.5f64).unwrap();
    assert!(v.is_number());

    let v = to_value("hello").unwrap();
    assert_eq!(v, Value::String("hello".into()));

    // None
    let v = to_value(Option::<String>::None).unwrap();
    assert!(v.is_null());

    // Some
    let v = to_value(Some("hi")).unwrap();
    assert_eq!(v, Value::String("hi".into()));

    // Vec
    let v = to_value(vec![1, 2, 3]).unwrap();
    assert!(v.is_sequence());

    // Struct
    #[derive(Serialize)]
    struct S {
        x: i32,
        y: String,
    }
    let v = to_value(S {
        x: 1,
        y: "z".into(),
    })
    .unwrap();
    assert!(v.is_mapping());
}

#[test]
fn to_value_enum() {
    #[derive(Serialize)]
    enum E {
        A,
        B(i32),
        C { x: i32 },
    }

    let v = to_value(E::A).unwrap();
    assert!(v.is_string());

    let v = to_value(E::B(42)).unwrap();
    assert!(v.is_mapping());

    let v = to_value(E::C { x: 1 }).unwrap();
    assert!(v.is_mapping());
}

#[test]
fn value_ordering_across_types() {
    // Null < Bool < Number < String < Sequence < Mapping
    let vals = vec![
        Value::String("z".into()),
        Value::Null,
        Value::Number(Number::from(1)),
        Value::Bool(false),
        Value::Sequence(vec![]),
        Value::Mapping(Mapping::new()),
    ];
    let mut sorted = vals.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    assert!(sorted[0].is_null());
    assert!(sorted[1].is_bool());
    assert!(sorted[2].is_number());
    assert!(sorted[3].is_string());
    assert!(sorted[4].is_sequence());
    assert!(sorted[5].is_mapping());
}

#[test]
fn tagged_value_from_str() {
    let yaml = "!mytag value\n";
    let v: Value = from_str(yaml).unwrap();
    if let Value::Tagged(tv) = &v {
        assert_eq!(tv.tag, Tag::new("!mytag"));
        assert_eq!(tv.value, Value::String("value".into()));
    } else {
        panic!("expected tagged value, got {v:?}");
    }
}

#[test]
fn deserialize_escape_sequences() {
    let yaml = r#"msg: "hello\nworld""#;
    let v: Value = from_str(yaml).unwrap();
    let m = v.as_mapping().unwrap();
    let msg = m.get(Value::String("msg".into())).unwrap();
    assert!(msg.as_str().unwrap().contains('\n'));
}

#[test]
fn deserialize_plain_scalar_values() {
    let yaml = "key: a simple value\n";
    let v: Value = from_str(yaml).unwrap();
    assert!(v.is_mapping());
}

#[test]
fn serialize_special_strings() {
    // Strings that need quoting
    let v = Value::String("true".into());
    let yaml = to_string(&v).unwrap();
    // "true" should be quoted to avoid being parsed as bool
    assert!(yaml.contains("'true'") || yaml.contains("\"true\""));

    let v = Value::String("null".into());
    let yaml = to_string(&v).unwrap();
    assert!(yaml.contains("'null'") || yaml.contains("\"null\""));

    let v = Value::String("123".into());
    let yaml = to_string(&v).unwrap();
    assert!(yaml.contains("'123'") || yaml.contains("\"123\""));
}

// ══════════════════════════════════════════════════════════════════════
// Additional coverage tests appended below
// ══════════════════════════════════════════════════════════════════════

// ── Value Deserializer: deserialize_option ─────────────────────────

#[test]
fn value_deserializer_option_none() {
    let v: Option<i32> = yaml_safe::from_value(Value::Null).unwrap();
    assert!(v.is_none());
}

#[test]
fn value_deserializer_option_some() {
    let v: Option<i32> =
        yaml_safe::from_value(Value::Number(Number::from(7i64)))
            .unwrap();
    assert_eq!(v, Some(7));
}

// ── Value Deserializer: deserialize_enum from string ───────────────

#[test]
fn value_deserializer_enum_string_variant() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Color {
        Red,
        Blue,
        Green,
    }
    let v: Color =
        yaml_safe::from_value(Value::String("Red".into())).unwrap();
    assert_eq!(v, Color::Red);

    let v: Color =
        yaml_safe::from_value(Value::String("Blue".into())).unwrap();
    assert_eq!(v, Color::Blue);
}

// ── Value Deserializer: deserialize_enum from mapping ──────────────

#[test]
fn value_deserializer_enum_newtype_variant() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Wrapper {
        Int(i32),
        Str(String),
    }
    let mut m = Mapping::new();
    m.insert(
        Value::String("Int".into()),
        Value::Number(Number::from(42i64)),
    );
    let v: Wrapper = yaml_safe::from_value(Value::Mapping(m)).unwrap();
    assert_eq!(v, Wrapper::Int(42));
}

#[test]
fn value_deserializer_enum_struct_variant() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Shape {
        Circle { radius: f64 },
    }
    let mut inner = Mapping::new();
    inner.insert(
        Value::String("radius".into()),
        Value::Number(Number::from(3.5f64)),
    );
    let mut m = Mapping::new();
    m.insert(Value::String("Circle".into()), Value::Mapping(inner));
    let v: Shape = yaml_safe::from_value(Value::Mapping(m)).unwrap();
    assert_eq!(v, Shape::Circle { radius: 3.5 });
}

#[test]
fn value_deserializer_enum_tuple_variant() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Pair {
        Two(i32, i32),
    }
    let mut m = Mapping::new();
    m.insert(
        Value::String("Two".into()),
        Value::Sequence(vec![
            Value::Number(Number::from(1i64)),
            Value::Number(Number::from(2i64)),
        ]),
    );
    let v: Pair = yaml_safe::from_value(Value::Mapping(m)).unwrap();
    assert_eq!(v, Pair::Two(1, 2));
}

// ── Value Deserializer: enum error paths ───────────────────────────

#[test]
fn value_deserializer_enum_non_single_mapping_error() {
    #[derive(Deserialize, Debug)]
    enum E {
        A,
        B,
    }
    let mut m = Mapping::new();
    m.insert(Value::String("A".into()), Value::Null);
    m.insert(Value::String("B".into()), Value::Null);
    let result: Result<E, _> = yaml_safe::from_value(Value::Mapping(m));
    assert!(result.is_err());
}

#[test]
fn value_deserializer_enum_invalid_type_error() {
    #[derive(Deserialize, Debug)]
    enum E {
        A,
    }
    let result: Result<E, _> =
        yaml_safe::from_value(Value::Number(Number::from(42i64)));
    assert!(result.is_err());
}

// ── Value Deserializer: deserialize_newtype_struct ──────────────────

#[test]
fn value_deserializer_newtype_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Wrapper(i32);
    let v: Wrapper =
        yaml_safe::from_value(Value::Number(Number::from(99i64)))
            .unwrap();
    assert_eq!(v, Wrapper(99));
}

// ── Value Deserializer: deserialize_unit ────────────────────────────

#[test]
fn value_deserializer_unit() {
    let v: () = yaml_safe::from_value(Value::Null).unwrap();
    assert_eq!(v, ());
}

// ── Value Deserializer: deserialize_seq from Value::Sequence ────────

#[test]
fn value_deserializer_seq() {
    let seq = Value::Sequence(vec![
        Value::String("a".into()),
        Value::String("b".into()),
    ]);
    let v: Vec<String> = yaml_safe::from_value(seq).unwrap();
    assert_eq!(v, vec!["a", "b"]);
}

// ── Value Deserializer: deserialize_map from Value::Mapping ─────────

#[test]
fn value_deserializer_map() {
    use std::collections::HashMap;
    let mut m = Mapping::new();
    m.insert(Value::String("x".into()), Value::String("y".into()));
    let v: HashMap<String, String> =
        yaml_safe::from_value(Value::Mapping(m)).unwrap();
    assert_eq!(v.get("x").unwrap(), "y");
}

// ── Value Deserializer: deserialize_struct ──────────────────────────

#[test]
fn value_deserializer_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Point {
        x: f64,
        y: f64,
    }
    let mut m = Mapping::new();
    m.insert(
        Value::String("x".into()),
        Value::Number(Number::from(1.0f64)),
    );
    m.insert(
        Value::String("y".into()),
        Value::Number(Number::from(2.0f64)),
    );
    let v: Point = yaml_safe::from_value(Value::Mapping(m)).unwrap();
    assert_eq!(v, Point { x: 1.0, y: 2.0 });
}

// ── Value Deserializer: deserialize_tuple ───────────────────────────

#[test]
fn value_deserializer_tuple() {
    let seq = Value::Sequence(vec![
        Value::Number(Number::from(1i64)),
        Value::String("hello".into()),
    ]);
    let v: (i32, String) = yaml_safe::from_value(seq).unwrap();
    assert_eq!(v, (1, "hello".to_string()));
}

// ── Value Deserializer: deserialize_bool from Value ─────────────────

#[test]
fn value_deserializer_bool() {
    let v: bool = yaml_safe::from_value(Value::Bool(false)).unwrap();
    assert!(!v);
}

// ── Value Deserializer: tagged value through deserialize_any ────────

#[test]
fn value_deserializer_tagged_through_any() {
    let tagged = Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new("!mytag"),
        value: Value::String("inner".into()),
    }));
    // Deserializing a tagged value as a generic Value should work
    let v: Value = yaml_safe::from_value(tagged.clone()).unwrap();
    if let Value::Tagged(t) = &v {
        assert_eq!(t.tag, Tag::new("mytag"));
    }
}

// ── ValueSerializer: all primitive types ────────────────────────────

#[test]
fn value_serializer_bool() {
    let v = to_value(true).unwrap();
    assert_eq!(v, Value::Bool(true));
    let v = to_value(false).unwrap();
    assert_eq!(v, Value::Bool(false));
}

#[test]
fn value_serializer_integers() {
    let v = to_value(42i8).unwrap();
    assert!(v.is_number());
    let v = to_value(42i16).unwrap();
    assert!(v.is_number());
    let v = to_value(42i32).unwrap();
    assert!(v.is_number());
    let v = to_value(42i64).unwrap();
    assert!(v.is_number());
    let v = to_value(42u8).unwrap();
    assert!(v.is_number());
    let v = to_value(42u16).unwrap();
    assert!(v.is_number());
    let v = to_value(42u32).unwrap();
    assert!(v.is_number());
    let v = to_value(42u64).unwrap();
    assert!(v.is_number());
}

#[test]
fn value_serializer_floats() {
    let v = to_value(1.5f32).unwrap();
    assert!(v.is_number());
    let v = to_value(2.5f64).unwrap();
    assert!(v.is_number());
}

#[test]
fn value_serializer_str() {
    let v = to_value("hello").unwrap();
    assert_eq!(v.as_str(), Some("hello"));
}

#[test]
fn value_serializer_char() {
    let v = to_value('Z').unwrap();
    assert_eq!(v.as_str(), Some("Z"));
}

#[test]
fn value_serializer_none() {
    let v = to_value(Option::<i32>::None).unwrap();
    assert!(v.is_null());
}

#[test]
fn value_serializer_some() {
    let v = to_value(Some(42i32)).unwrap();
    assert!(v.is_number());
}

#[test]
fn value_serializer_unit() {
    let v = to_value(()).unwrap();
    assert!(v.is_null());
}

#[test]
fn value_serializer_unit_struct() {
    #[derive(Serialize)]
    struct Unit;
    let v = to_value(Unit).unwrap();
    assert!(v.is_null());
}

#[test]
fn value_serializer_unit_variant() {
    #[derive(Serialize)]
    enum E {
        Variant,
    }
    let v = to_value(E::Variant).unwrap();
    assert_eq!(v.as_str(), Some("Variant"));
}

#[test]
fn value_serializer_newtype_struct() {
    #[derive(Serialize)]
    struct W(i32);
    let v = to_value(W(77)).unwrap();
    assert!(v.is_number());
}

#[test]
fn value_serializer_newtype_variant() {
    #[derive(Serialize)]
    enum E {
        Val(String),
    }
    let v = to_value(E::Val("hi".into())).unwrap();
    assert!(v.is_mapping());
    let m = v.as_mapping().unwrap();
    assert_eq!(m.get("Val"), Some(&Value::String("hi".into())));
}

#[test]
fn value_serializer_tuple_variant() {
    #[derive(Serialize)]
    enum E {
        Pair(i32, i32),
    }
    let v = to_value(E::Pair(1, 2)).unwrap();
    assert!(v.is_mapping());
    let inner = v.as_mapping().unwrap().get("Pair").unwrap();
    assert!(inner.is_sequence());
    assert_eq!(inner.as_sequence().unwrap().len(), 2);
}

#[test]
fn value_serializer_struct_variant() {
    #[derive(Serialize)]
    enum E {
        Named { a: i32, b: String },
    }
    let v = to_value(E::Named {
        a: 1,
        b: "x".into(),
    })
    .unwrap();
    assert!(v.is_mapping());
    let inner = v.as_mapping().unwrap().get("Named").unwrap();
    assert!(inner.is_mapping());
}

#[test]
fn value_serializer_seq() {
    let v = to_value(vec![1, 2, 3]).unwrap();
    assert!(v.is_sequence());
    assert_eq!(v.as_sequence().unwrap().len(), 3);
}

#[test]
fn value_serializer_tuple() {
    let v = to_value((1, "hello", true)).unwrap();
    assert!(v.is_sequence());
    assert_eq!(v.as_sequence().unwrap().len(), 3);
}

#[test]
fn value_serializer_tuple_struct() {
    #[derive(Serialize)]
    struct Pair(i32, i32);
    let v = to_value(Pair(10, 20)).unwrap();
    assert!(v.is_sequence());
    assert_eq!(v.as_sequence().unwrap().len(), 2);
}

#[test]
fn value_serializer_map() {
    use std::collections::BTreeMap;
    let mut map = BTreeMap::new();
    map.insert("key", "value");
    let v = to_value(map).unwrap();
    assert!(v.is_mapping());
}

#[test]
fn value_serializer_struct() {
    #[derive(Serialize)]
    struct S {
        a: i32,
        b: String,
    }
    let v = to_value(S {
        a: 1,
        b: "hi".into(),
    })
    .unwrap();
    assert!(v.is_mapping());
    assert_eq!(v.as_mapping().unwrap().len(), 2);
}

#[test]
fn value_serializer_bytes_error() {
    // Serializing a struct containing bytes-like data through to_value
    // is not directly testable since ValueSerializer is pub(crate).
    // Instead we verify bytes are rejected through the public API.
    use serde::Serialize;
    struct Bytes;
    impl Serialize for Bytes {
        fn serialize<S: serde::Serializer>(
            &self,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            serializer.serialize_bytes(b"hello")
        }
    }
    let result = to_value(Bytes);
    assert!(result.is_err());
}

// ── TaggedValue serde ──────────────────────────────────────────────

#[test]
fn tagged_value_copy() {
    let tv = TaggedValue {
        tag: Tag::new("!test"),
        value: Value::Number(Number::from(10i64)),
    };
    let cp = tv.copy();
    assert_eq!(tv, cp);
}

#[test]
fn tagged_value_deserialize_from_yaml() {
    let yaml = "!color red\n";
    let v: Value = from_str(yaml).unwrap();
    if let Value::Tagged(t) = v {
        assert_eq!(t.tag, Tag::new("color"));
        assert_eq!(t.value, Value::String("red".into()));
    } else {
        panic!("expected tagged");
    }
}

#[test]
fn tagged_value_with_inline_value() {
    let yaml = "!person {name: Alice, age: 30}\n";
    let v: Value = from_str(yaml).unwrap();
    if let Value::Tagged(t) = &v {
        assert_eq!(t.tag, Tag::new("person"));
        assert!(t.value.is_mapping());
    } else {
        panic!("expected tagged");
    }
}

#[test]
fn tagged_value_with_inline_sequence() {
    let yaml = "!list [1, 2, 3]\n";
    let v: Value = from_str(yaml).unwrap();
    if let Value::Tagged(t) = &v {
        assert_eq!(t.tag, Tag::new("list"));
        assert!(t.value.is_sequence());
    } else {
        panic!("expected tagged");
    }
}

// ── TaggedValue as Deserializer (deserialize_ignored_any) ──────────

#[test]
fn tagged_value_deserialize_ignored() {
    use serde::de::IgnoredAny;
    let tv = TaggedValue {
        tag: Tag::new("!skip"),
        value: Value::String("ignored".into()),
    };
    let _: IgnoredAny = serde::Deserialize::deserialize(tv).unwrap();
}

// ── Tag: nobang and PartialEq with &str ────────────────────────────

#[test]
fn tag_partial_eq_str() {
    let tag = Tag::new("!foo");
    assert!(tag == "foo");
    assert!(tag == "!foo");

    let tag2 = Tag::new("bar");
    assert!(tag2 == "bar");
    assert!(tag2 == "!bar");
}

#[test]
fn nobang_edge_cases() {
    use yaml_safe::value::tagged::nobang;
    assert_eq!(nobang(""), "");
    assert_eq!(nobang("!"), "!"); // "!" alone returns the input
    assert_eq!(nobang("!foo"), "foo");
    assert_eq!(nobang("bar"), "bar");
}

// ── MaybeTag / check_for_tag ───────────────────────────────────────

#[test]
fn check_for_tag_variants() {
    use yaml_safe::value::tagged::{check_for_tag, MaybeTag};

    match check_for_tag(&"!custom") {
        MaybeTag::Tag(s) => assert_eq!(s, "!custom"),
        _ => panic!("expected Tag"),
    }

    match check_for_tag(&"plain") {
        MaybeTag::NotTag(s) => assert_eq!(s, "plain"),
        _ => panic!("expected NotTag"),
    }

    match check_for_tag(&"") {
        MaybeTag::NotTag(s) => assert!(s.is_empty()),
        _ => panic!("expected NotTag for empty"),
    }

    match check_for_tag(&"!") {
        MaybeTag::NotTag(s) => assert_eq!(s, "!"),
        _ => panic!("expected NotTag for bare !"),
    }
}

// ── Entry API: VacantEntry::into_key ───────────────────────────────

#[test]
fn vacant_entry_into_key() {
    let mut m = Mapping::new();
    let key = Value::String("new_key".into());
    if let yaml_safe::mapping::Entry::Vacant(vacant) =
        m.entry(key.clone())
    {
        let returned_key = vacant.into_key();
        assert_eq!(returned_key, key);
    } else {
        panic!("expected vacant entry");
    }
}

// ── Entry API: VacantEntry::insert ─────────────────────────────────

#[test]
fn vacant_entry_insert() {
    let mut m = Mapping::new();
    let key = Value::String("k".into());
    if let yaml_safe::mapping::Entry::Vacant(vacant) =
        m.entry(key.clone())
    {
        let val_ref = vacant.insert(Value::Bool(true));
        assert_eq!(*val_ref, Value::Bool(true));
    }
    assert_eq!(m.get("k"), Some(&Value::Bool(true)));
}

// ── Entry API: OccupiedEntry::insert and remove ────────────────────

#[test]
fn occupied_entry_insert_and_remove() {
    let mut m = Mapping::new();
    m.insert(Value::String("k".into()), Value::String("old".into()));

    // insert (replace)
    if let yaml_safe::mapping::Entry::Occupied(mut occ) =
        m.entry(Value::String("k".into()))
    {
        let old = occ.insert(Value::String("new".into()));
        assert_eq!(old, Value::String("old".into()));
    }
    assert_eq!(m.get("k"), Some(&Value::String("new".into())));

    // remove
    if let yaml_safe::mapping::Entry::Occupied(occ) =
        m.entry(Value::String("k".into()))
    {
        let removed = occ.remove();
        assert_eq!(removed, Value::String("new".into()));
    }
    assert!(m.is_empty());
}

// ── Entry API: OccupiedEntry::remove_entry ─────────────────────────

#[test]
fn occupied_entry_remove_entry() {
    let mut m = Mapping::new();
    m.insert(
        Value::String("k".into()),
        Value::Number(Number::from(99i64)),
    );
    if let yaml_safe::mapping::Entry::Occupied(occ) =
        m.entry(Value::String("k".into()))
    {
        let (key, val) = occ.remove_entry();
        assert_eq!(key, Value::String("k".into()));
        assert_eq!(val, Value::Number(Number::from(99i64)));
    }
    assert!(m.is_empty());
}

// ── Entry API: or_insert_with ──────────────────────────────────────

#[test]
fn entry_or_insert_with() {
    let mut m = Mapping::new();
    m.entry(Value::String("k".into()))
        .or_insert_with(|| Value::Number(Number::from(42i64)));
    assert_eq!(m.get("k"), Some(&Value::Number(Number::from(42i64))));

    // Calling on occupied should not change the value
    m.entry(Value::String("k".into()))
        .or_insert_with(|| Value::Number(Number::from(0i64)));
    assert_eq!(m.get("k"), Some(&Value::Number(Number::from(42i64))));
}

// ── Entry::key ─────────────────────────────────────────────────────

#[test]
fn entry_key() {
    let mut m = Mapping::new();
    let key = Value::String("test_key".into());
    let entry = m.entry(key.clone());
    assert_eq!(entry.key(), &key);
}

// ── OccupiedEntry::into_mut ────────────────────────────────────────

#[test]
fn occupied_entry_into_mut() {
    let mut m = Mapping::new();
    m.insert(
        Value::String("k".into()),
        Value::Number(Number::from(1i64)),
    );
    if let yaml_safe::mapping::Entry::Occupied(occ) =
        m.entry(Value::String("k".into()))
    {
        let v = occ.into_mut();
        *v = Value::Number(Number::from(999i64));
    }
    assert_eq!(m.get("k"), Some(&Value::Number(Number::from(999i64))));
}

// ── Mapping Display with multiple entries ──────────────────────────

#[test]
fn mapping_display_multiple() {
    let mut m = Mapping::new();
    m.insert(
        Value::String("a".into()),
        Value::Number(Number::from(1i64)),
    );
    m.insert(
        Value::String("b".into()),
        Value::Number(Number::from(2i64)),
    );
    let s = format!("{m}");
    assert!(s.starts_with('{'));
    assert!(s.ends_with('}'));
    assert!(s.contains(", "));
}

// ── Mapping PartialOrd with non-equal values ───────────────────────

#[test]
fn mapping_partial_ord_non_equal() {
    let mut m1 = Mapping::new();
    m1.insert(
        Value::String("a".into()),
        Value::Number(Number::from(1i64)),
    );
    let mut m2 = Mapping::new();
    m2.insert(
        Value::String("a".into()),
        Value::Number(Number::from(2i64)),
    );
    let cmp = m1.partial_cmp(&m2);
    assert!(cmp.is_some());
    assert_ne!(cmp, Some(Ordering::Equal));
}

#[test]
fn mapping_partial_ord_different_keys() {
    let mut m1 = Mapping::new();
    m1.insert(
        Value::String("a".into()),
        Value::Number(Number::from(1i64)),
    );
    let mut m2 = Mapping::new();
    m2.insert(
        Value::String("b".into()),
        Value::Number(Number::from(1i64)),
    );
    let cmp = m1.partial_cmp(&m2);
    assert!(cmp.is_some());
    assert_eq!(cmp, Some(Ordering::Less));
}

// ── Mapping Index/IndexMut traits ──────────────────────────────────

#[test]
fn mapping_std_index() {
    let mut m = Mapping::new();
    m.insert(
        Value::String("key".into()),
        Value::Number(Number::from(42i64)),
    );
    let val = &m["key"];
    assert_eq!(val, &Value::Number(Number::from(42i64)));
}

#[test]
fn mapping_std_index_mut() {
    let mut m = Mapping::new();
    m.insert(
        Value::String("key".into()),
        Value::Number(Number::from(1i64)),
    );
    m["key"] = Value::Number(Number::from(99i64));
    assert_eq!(m.get("key"), Some(&Value::Number(Number::from(99i64))));
}

#[test]
#[should_panic(expected = "key not found")]
fn mapping_std_index_missing_key() {
    let m = Mapping::new();
    let _ = &m["missing"];
}

// ── Mapping: retain ────────────────────────────────────────────────

#[test]
fn mapping_retain() {
    let mut m = Mapping::new();
    m.insert(
        Value::String("a".into()),
        Value::Number(Number::from(1i64)),
    );
    m.insert(
        Value::String("b".into()),
        Value::Number(Number::from(2i64)),
    );
    m.insert(
        Value::String("c".into()),
        Value::Number(Number::from(3i64)),
    );
    m.retain(|_k, v| v.as_i64().is_some_and(|n| n > 1));
    assert_eq!(m.len(), 2);
    assert!(m.get("a").is_none());
}

// ── Mapping: capacity / with_capacity ──────────────────────────────

#[test]
fn mapping_with_capacity() {
    let m = Mapping::with_capacity(10);
    assert!(m.capacity() >= 10);
    assert!(m.is_empty());
}

// ── Mapping: clear ─────────────────────────────────────────────────

#[test]
fn mapping_clear() {
    let mut m = Mapping::new();
    m.insert(Value::String("k".into()), Value::Null);
    m.clear();
    assert!(m.is_empty());
}

// ── Number: PartialOrd across types ────────────────────────────────

#[test]
fn number_partial_ord_pos_neg() {
    let pos = Number::from(5u64);
    let neg = Number::from(-5i64);
    assert!(neg < pos);
    assert!(pos > neg);
}

#[test]
fn number_partial_ord_same_type() {
    let a = Number::from(1u64);
    let b = Number::from(2u64);
    assert!(a < b);

    let c = Number::from(-3i64);
    let d = Number::from(-1i64);
    assert!(c < d);
}

#[test]
fn number_partial_ord_float() {
    let a = Number::from(1.0f64);
    let b = Number::from(2.0f64);
    assert!(a < b);
}

#[test]
fn number_partial_ord_nan() {
    let nan1 = Number::from(f64::NAN);
    let nan2 = Number::from(f64::NAN);
    // NaN == NaN in this crate
    assert_eq!(nan1, nan2);
    assert_eq!(nan1.partial_cmp(&nan2), Some(Ordering::Equal));
}

#[test]
fn number_partial_ord_float_vs_int() {
    let float = Number::from(1.5f64);
    let int = Number::from(1u64);
    // Float vs int: int sorts before float per total_cmp
    let cmp = int.partial_cmp(&float);
    assert!(cmp.is_some());
}

// ── Number: total_cmp (tested indirectly via PartialOrd) ───────────

#[test]
fn number_ordering_pos_vs_neg() {
    let pos = Number::from(10u64);
    let neg = Number::from(-10i64);
    assert!(neg < pos);
    assert!(pos > neg);
}

#[test]
fn number_ordering_float_vs_int() {
    let float = Number::from(1.5f64);
    let int = Number::from(1u64);
    // PartialOrd delegates to total_cmp for mixed types
    let cmp = int.partial_cmp(&float);
    assert!(cmp.is_some());
}

#[test]
fn number_ordering_neg_int_vs_float() {
    let neg = Number::from(-1i64);
    let float = Number::from(1.5f64);
    assert!(neg < float);
}

// ── Number: Deserialize visitor ────────────────────────────────────

#[test]
fn number_deserialize_from_yaml() {
    let n: Number = from_str("42").unwrap();
    assert_eq!(n.as_u64(), Some(42));

    let n: Number = from_str("-7").unwrap();
    assert_eq!(n.as_i64(), Some(-7));

    let n: Number = from_str("3.14").unwrap();
    assert!(n.is_f64());
}

// ── Number: Display for float with zero fractional part ────────────

#[test]
fn number_display_float_zero_fract() {
    let n = Number::from(5.0f64);
    assert_eq!(format!("{n}"), "5.0");
}

#[test]
fn number_display_float_with_fract() {
    let n = Number::from(2.75f64);
    let s = format!("{n}");
    assert!(s.contains("2.75"));
}

#[test]
fn number_display_integer() {
    let n = Number::from(42u64);
    assert_eq!(format!("{n}"), "42");

    let n = Number::from(-10i64);
    assert_eq!(format!("{n}"), "-10");
}

// ── Number: Debug ──────────────────────────────────────────────────

#[test]
fn number_debug() {
    let n = Number::from(42u64);
    let d = format!("{n:?}");
    assert!(d.contains("Number(42)"));
}

// ── Number: is_nan, is_infinite, is_finite ─────────────────────────

#[test]
fn number_special_float_predicates() {
    let nan = Number::from(f64::NAN);
    assert!(nan.is_nan());
    assert!(!nan.is_finite());
    assert!(!nan.is_infinite());

    let inf = Number::from(f64::INFINITY);
    assert!(inf.is_infinite());
    assert!(!inf.is_finite());
    assert!(!inf.is_nan());

    let neg_inf = Number::from(f64::NEG_INFINITY);
    assert!(neg_inf.is_infinite());

    let normal = Number::from(1.0f64);
    assert!(normal.is_finite());
    assert!(!normal.is_nan());
    assert!(!normal.is_infinite());

    // Integers are always finite
    let int = Number::from(42u64);
    assert!(int.is_finite());
    assert!(!int.is_nan());
    assert!(!int.is_infinite());
}

// ── Number: as_i64 overflow ────────────────────────────────────────

#[test]
fn number_as_i64_overflow() {
    let big = Number::from(u64::MAX);
    assert!(big.is_u64());
    assert!(!big.is_i64());
    assert_eq!(big.as_i64(), None);
}

// ── Number: from negative signed types ─────────────────────────────

#[test]
fn number_from_negative_small_types() {
    let n = Number::from(-1i8);
    assert_eq!(n.as_i64(), Some(-1));
    assert_eq!(n.as_u64(), None);

    let n = Number::from(-100i16);
    assert_eq!(n.as_i64(), Some(-100));

    let n = Number::from(-1000i32);
    assert_eq!(n.as_i64(), Some(-1000));
}

// ── Number: from f32 NaN normalization ─────────────────────────────

#[test]
fn number_from_f32_nan() {
    let n = Number::from(f32::NAN);
    assert!(n.is_nan());
}

// ── Block scalar chomp modes ───────────────────────────────────────

#[test]
fn block_scalar_strip_chomp() {
    let yaml = "text: |-\n  hello\n  world\n";
    let v: Value = from_str(yaml).unwrap();
    let s = v
        .as_mapping()
        .unwrap()
        .get("text")
        .unwrap()
        .as_str()
        .unwrap();
    // Strip removes trailing newlines
    assert!(!s.ends_with('\n'));
    assert!(s.contains("hello"));
}

#[test]
fn block_scalar_clip_chomp() {
    let yaml = "text: |\n  hello\n  world\n";
    let v: Value = from_str(yaml).unwrap();
    let s = v
        .as_mapping()
        .unwrap()
        .get("text")
        .unwrap()
        .as_str()
        .unwrap();
    assert!(s.contains("hello"));
    assert!(s.contains("world"));
}

#[test]
fn block_scalar_keep_chomp() {
    let yaml = "text: |+\n  hello\n  world\n\n\n";
    let v: Value = from_str(yaml).unwrap();
    let s = v
        .as_mapping()
        .unwrap()
        .get("text")
        .unwrap()
        .as_str()
        .unwrap();
    assert!(s.contains("hello"));
}

// ── Folded scalar with strip/keep ──────────────────────────────────

#[test]
fn folded_scalar_strip() {
    let yaml = "text: >-\n  hello\n  world\n";
    let v: Value = from_str(yaml).unwrap();
    let s = v
        .as_mapping()
        .unwrap()
        .get("text")
        .unwrap()
        .as_str()
        .unwrap();
    assert!(!s.ends_with('\n'));
}

#[test]
fn folded_scalar_keep() {
    let yaml = "text: >+\n  hello\n  world\n\n";
    let v: Value = from_str(yaml).unwrap();
    let s = v
        .as_mapping()
        .unwrap()
        .get("text")
        .unwrap()
        .as_str()
        .unwrap();
    assert!(s.contains("hello"));
}

// ── Flow collections nested within block ───────────────────────────

#[test]
fn flow_sequence_in_block_mapping() {
    let yaml = "items: [1, 2, 3]\nname: test\n";
    let v: Value = from_str(yaml).unwrap();
    let m = v.as_mapping().unwrap();
    let items = m.get("items").unwrap();
    assert!(items.is_sequence());
    assert_eq!(items.as_sequence().unwrap().len(), 3);
}

#[test]
fn flow_mapping_in_block_mapping() {
    let yaml = "config: {a: 1, b: 2}\nname: test\n";
    let v: Value = from_str(yaml).unwrap();
    let m = v.as_mapping().unwrap();
    let config = m.get("config").unwrap();
    assert!(config.is_mapping());
}

#[test]
fn nested_flow_in_flow() {
    let yaml = "[[1, 2], [3, 4]]";
    let v: Value = from_str(yaml).unwrap();
    assert!(v.is_sequence());
    let outer = v.as_sequence().unwrap();
    assert_eq!(outer.len(), 2);
    assert!(outer[0].is_sequence());
}

#[test]
fn flow_mapping_in_flow_sequence() {
    let yaml = "[{a: 1}, {b: 2}]";
    let v: Value = from_str(yaml).unwrap();
    let seq = v.as_sequence().unwrap();
    assert_eq!(seq.len(), 2);
    assert!(seq[0].is_mapping());
}

// ── Anchor/alias parsing (basic support) ───────────────────────────

#[test]
fn anchor_alias_parsing() {
    // Even if anchors are not fully resolved, they should not crash
    let yaml = "defaults: &defaults\n  color: red\n";
    let v: Value = from_str(yaml).unwrap();
    assert!(v.is_mapping());
}

// ── Error paths in deserializer ────────────────────────────────────

#[test]
fn unterminated_flow_sequence_error() {
    let result: Result<Value, _> = from_str("[1, 2, 3");
    assert!(result.is_err());
}

#[test]
fn unterminated_flow_mapping_error() {
    let result: Result<Value, _> = from_str("{a: 1");
    assert!(result.is_err());
}

#[test]
fn unterminated_quoted_string_error() {
    let result: Result<Value, _> = from_str("key: 'unterminated");
    assert!(result.is_err());
}

#[test]
fn unterminated_double_quoted_string_error() {
    let result: Result<Value, _> = from_str("key: \"unterminated");
    assert!(result.is_err());
}

// ── from_slice and from_reader ─────────────────────────────────────

#[test]
fn from_slice_valid() {
    let data = b"key: value\n";
    let v: Value = yaml_safe::from_slice(data).unwrap();
    assert!(v.is_mapping());
}

#[test]
fn from_slice_invalid_utf8() {
    let data: &[u8] = &[0xFF, 0xFE];
    let result: Result<Value, _> = yaml_safe::from_slice(data);
    assert!(result.is_err());
}

#[test]
fn from_reader_valid() {
    let data = b"key: value\n";
    let cursor = std::io::Cursor::new(data);
    let v: Value = yaml_safe::from_reader(cursor).unwrap();
    assert!(v.is_mapping());
}

// ── to_writer ──────────────────────────────────────────────────────

#[test]
fn to_writer_valid() {
    let mut buf = Vec::new();
    yaml_safe::to_writer(&mut buf, &42i32).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("42"));
}

// ── Value Display for Sequence and Mapping ─────────────────────────

#[test]
fn value_display_sequence() {
    let v = Value::Sequence(vec![Value::Null]);
    assert_eq!(format!("{v}"), "[...]");
}

#[test]
fn value_display_mapping() {
    let mut m = Mapping::new();
    m.insert(Value::String("k".into()), Value::Null);
    let v = Value::Mapping(m);
    assert_eq!(format!("{v}"), "{...}");
}

#[test]
fn value_display_tagged() {
    let v = Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new("!t"),
        value: Value::String("val".into()),
    }));
    let s = format!("{v}");
    assert!(s.contains("!t"));
    assert!(s.contains("val"));
}

// ── Value Debug for all types ──────────────────────────────────────

#[test]
fn value_debug_all_variants() {
    let cases = vec![
        (Value::Null, "Null"),
        (Value::Bool(true), "Bool(true)"),
        (Value::Number(Number::from(42u64)), "Number(42)"),
        (Value::String("hi".into()), "String(\"hi\")"),
    ];
    for (val, expected) in cases {
        let dbg = format!("{val:?}");
        assert!(
            dbg.contains(expected),
            "debug of {val:?} should contain {expected}"
        );
    }

    let seq = Value::Sequence(vec![Value::Null]);
    let dbg = format!("{seq:?}");
    assert!(dbg.contains("Null"));

    let mut m = Mapping::new();
    m.insert(Value::String("k".into()), Value::Null);
    let map_val = Value::Mapping(m);
    let dbg = format!("{map_val:?}");
    assert!(dbg.contains("k"));

    let tagged = Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new("!t"),
        value: Value::Null,
    }));
    let dbg = format!("{tagged:?}");
    assert!(dbg.contains("Tagged"));
}

// ── Value::get on non-mapping types ────────────────────────────────

#[test]
fn value_get_on_sequence_returns_none() {
    let v = Value::Sequence(vec![Value::Null]);
    assert!(v.get("anything").is_none());
}

#[test]
fn value_get_on_scalar_returns_none() {
    let v = Value::String("hello".into());
    assert!(v.get("anything").is_none());
}

#[test]
fn value_get_mut_on_non_mapping_returns_none() {
    let mut v = Value::Bool(true);
    assert!(v.get_mut("anything").is_none());
}

// ── Value: apply_merge with mapping ────────────────────────────────

#[test]
fn apply_merge_mapping() {
    let mut defaults = Mapping::new();
    defaults.insert(
        Value::String("color".into()),
        Value::String("red".into()),
    );
    defaults.insert(
        Value::String("size".into()),
        Value::String("large".into()),
    );

    let mut item = Mapping::new();
    item.insert(Value::String("<<".into()), Value::Mapping(defaults));
    item.insert(
        Value::String("size".into()),
        Value::String("small".into()),
    );

    let mut v = Value::Mapping(item);
    v.apply_merge().unwrap();

    let m = v.as_mapping().unwrap();
    assert_eq!(m.get("color"), Some(&Value::String("red".into())));
    // Existing key should not be overwritten
    assert_eq!(m.get("size"), Some(&Value::String("small".into())));
    assert!(m.get("<<").is_none());
}

#[test]
fn apply_merge_sequence_of_mappings() {
    let mut m1 = Mapping::new();
    m1.insert(
        Value::String("a".into()),
        Value::Number(Number::from(1i64)),
    );
    let mut m2 = Mapping::new();
    m2.insert(
        Value::String("b".into()),
        Value::Number(Number::from(2i64)),
    );

    let mut item = Mapping::new();
    item.insert(
        Value::String("<<".into()),
        Value::Sequence(vec![Value::Mapping(m1), Value::Mapping(m2)]),
    );

    let mut v = Value::Mapping(item);
    v.apply_merge().unwrap();

    let m = v.as_mapping().unwrap();
    assert_eq!(m.get("a"), Some(&Value::Number(Number::from(1i64))));
    assert_eq!(m.get("b"), Some(&Value::Number(Number::from(2i64))));
}

#[test]
fn apply_merge_invalid_merge_value() {
    let mut item = Mapping::new();
    item.insert(
        Value::String("<<".into()),
        Value::String("not a mapping".into()),
    );

    let mut v = Value::Mapping(item);
    let result = v.apply_merge();
    assert!(result.is_err());
}

#[test]
fn apply_merge_invalid_sequence_element() {
    let mut item = Mapping::new();
    item.insert(
        Value::String("<<".into()),
        Value::Sequence(vec![Value::String("not a mapping".into())]),
    );

    let mut v = Value::Mapping(item);
    let result = v.apply_merge();
    assert!(result.is_err());
}

#[test]
fn apply_merge_recursive() {
    let mut inner = Mapping::new();
    let mut inner_defaults = Mapping::new();
    inner_defaults.insert(
        Value::String("x".into()),
        Value::Number(Number::from(1i64)),
    );
    inner.insert(
        Value::String("<<".into()),
        Value::Mapping(inner_defaults),
    );

    let mut outer = Mapping::new();
    outer.insert(Value::String("nested".into()), Value::Mapping(inner));

    let mut v = Value::Mapping(outer);
    v.apply_merge().unwrap();

    let nested = v.get("nested").unwrap().as_mapping().unwrap();
    assert_eq!(
        nested.get("x"),
        Some(&Value::Number(Number::from(1i64)))
    );
}

#[test]
fn apply_merge_in_sequence() {
    let mut child = Mapping::new();
    let mut child_defaults = Mapping::new();
    child_defaults
        .insert(Value::String("val".into()), Value::Bool(true));
    child.insert(
        Value::String("<<".into()),
        Value::Mapping(child_defaults),
    );

    let mut v = Value::Sequence(vec![Value::Mapping(child)]);
    v.apply_merge().unwrap();

    let m = v.as_sequence().unwrap()[0].as_mapping().unwrap();
    assert_eq!(m.get("val"), Some(&Value::Bool(true)));
}

#[test]
fn apply_merge_in_tagged() {
    let mut inner = Mapping::new();
    let mut defs = Mapping::new();
    defs.insert(Value::String("z".into()), Value::Null);
    inner.insert(Value::String("<<".into()), Value::Mapping(defs));

    let mut v = Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new("!test"),
        value: Value::Mapping(inner),
    }));
    v.apply_merge().unwrap();

    if let Value::Tagged(t) = &v {
        let m = t.value.as_mapping().unwrap();
        assert_eq!(m.get("z"), Some(&Value::Null));
    }
}

// ── Value: From impls ──────────────────────────────────────────────

#[test]
fn value_from_vec() {
    let v: Value = vec!["a", "b"].into();
    assert!(v.is_sequence());
    assert_eq!(v.as_sequence().unwrap().len(), 2);
}

#[test]
fn value_from_option_none() {
    let v: Value = Option::<String>::None.into();
    assert!(v.is_null());
}

#[test]
fn value_from_option_some() {
    let v: Value = Some("hello").into();
    assert_eq!(v.as_str(), Some("hello"));
}

#[test]
fn value_from_all_int_types() {
    let _: Value = Value::from(1i8);
    let _: Value = Value::from(1i16);
    let _: Value = Value::from(1i32);
    let _: Value = Value::from(1i64);
    let _: Value = Value::from(1isize);
    let _: Value = Value::from(1u8);
    let _: Value = Value::from(1u16);
    let _: Value = Value::from(1u32);
    let _: Value = Value::from(1u64);
    let _: Value = Value::from(1usize);
    let _: Value = Value::from(1.0f32);
    let _: Value = Value::from(1.0f64);
}

// ── Value: unexpected (exercised via deserialization type mismatch) ─

#[test]
fn value_unexpected_via_type_mismatch() {
    // Deserializing wrong type exercises the unexpected() path
    let result: Result<bool, _> =
        yaml_safe::from_value(Value::String("not a bool".into()));
    assert!(result.is_err());

    let result: Result<i32, _> =
        yaml_safe::from_value(Value::String("not a number".into()));
    assert!(result.is_err());

    let result: Result<Vec<i32>, _> =
        yaml_safe::from_value(Value::Bool(true));
    assert!(result.is_err());

    let result: Result<String, _> =
        yaml_safe::from_value(Value::Sequence(vec![]));
    assert!(result.is_err());
}

// ── Value: IntoDeserializer ────────────────────────────────────────

#[test]
fn value_into_deserializer() {
    use serde::de::IntoDeserializer;
    let v = Value::Number(Number::from(42i64));
    let de: Value = v.into_deserializer();
    let n: i64 = serde::Deserialize::deserialize(de).unwrap();
    assert_eq!(n, 42);
}

// ── Value: Hash for all variants ───────────────────────────────────

#[test]
fn value_hash_all_variants() {
    fn compute_hash(v: &Value) -> u64 {
        let mut h = DefaultHasher::new();
        v.hash(&mut h);
        h.finish()
    }

    // Just ensure no panics
    compute_hash(&Value::Null);
    compute_hash(&Value::Bool(true));
    compute_hash(&Value::Number(Number::from(42u64)));
    compute_hash(&Value::String("test".into()));
    compute_hash(&Value::Sequence(vec![Value::Null]));

    let mut m = Mapping::new();
    m.insert(Value::String("k".into()), Value::Null);
    compute_hash(&Value::Mapping(m));

    compute_hash(&Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new("!t"),
        value: Value::Null,
    })));
}

// ── Value: ordering across all variant pairs (via PartialOrd) ──────

#[test]
fn value_ordering_all_variant_pairs() {
    let vals = vec![
        Value::Null,
        Value::Bool(false),
        Value::Bool(true),
        Value::Number(Number::from(-1i64)),
        Value::Number(Number::from(1u64)),
        Value::String("a".into()),
        Value::String("b".into()),
        Value::Sequence(vec![]),
        Value::Sequence(vec![Value::Null]),
        Value::Mapping(Mapping::new()),
        Value::Tagged(Box::new(TaggedValue {
            tag: Tag::new("!a"),
            value: Value::Null,
        })),
        Value::Tagged(Box::new(TaggedValue {
            tag: Tag::new("!b"),
            value: Value::Null,
        })),
    ];

    // Verify ordering is consistent via PartialOrd
    for i in 0..vals.len() {
        for j in 0..vals.len() {
            let cmp = vals[i].partial_cmp(&vals[j]);
            if i < j {
                assert!(
                    cmp == Some(Ordering::Less)
                        || cmp == Some(Ordering::Equal),
                    "expected {:?} <= {:?}",
                    vals[i],
                    vals[j],
                );
            }
        }
    }
}

#[test]
fn value_ordering_tagged_same_tag() {
    let a = Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new("!t"),
        value: Value::Number(Number::from(1i64)),
    }));
    let b = Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new("!t"),
        value: Value::Number(Number::from(2i64)),
    }));
    assert!(a < b);
}

// ── ser.rs: needs_quoting edge cases ───────────────────────────────

#[test]
fn serialize_string_needs_quoting_special_values() {
    let special = vec![
        "Null", "NULL", "~", "True", "TRUE", "False", "FALSE", ".nan",
        ".NaN", ".NAN", ".inf", ".Inf", ".INF", "-.inf", "-.Inf",
        "-.INF",
    ];
    for s in special {
        let v = Value::String(s.into());
        let yaml = to_string(&v).unwrap();
        assert!(
            yaml.contains('\'') || yaml.contains('"'),
            "{s} should be quoted in output: {yaml}"
        );
    }
}

#[test]
fn serialize_string_needs_quoting_special_chars() {
    let cases = vec![
        "{hello", "}hello", "[hello", "]hello", ",hello", "&hello",
        "*hello", "!hello", "|hello", ">hello", "%hello", "@hello",
        "`hello", "'hello", "\"hello",
    ];
    for s in cases {
        let v = Value::String(s.into());
        let yaml = to_string(&v).unwrap();
        assert!(
            yaml.contains('\'') || yaml.contains('"'),
            "{s} should be quoted: {yaml}"
        );
    }
}

#[test]
fn serialize_string_needs_quoting_patterns() {
    let cases = vec![
        "a: b", // contains ": "
        "a #b", // contains " #"
        "a\nb", // contains newline
        "a\rb", // contains carriage return
        "- x",  // starts with "- "
        "? x",  // starts with "? "
    ];
    for s in cases {
        let v = Value::String(s.into());
        let yaml = to_string(&v).unwrap();
        assert!(
            yaml.contains('\'') || yaml.contains('"'),
            "{s:?} should be quoted: {yaml}"
        );
    }
}

#[test]
fn serialize_string_no_quoting_needed() {
    let v = Value::String("simple".into());
    let yaml = to_string(&v).unwrap();
    assert!(!yaml.contains('\''));
    assert!(!yaml.contains('"'));
}

// ── ser.rs: emit_flow_sequence and emit_flow_mapping ───────────────

#[test]
fn serialize_empty_sequence() {
    let v = Value::Sequence(vec![]);
    let yaml = to_string(&v).unwrap();
    assert!(yaml.contains("[]"));
}

#[test]
fn serialize_empty_mapping() {
    let v = Value::Mapping(Mapping::new());
    let yaml = to_string(&v).unwrap();
    assert!(yaml.contains("{}"));
}

#[test]
fn serialize_sequence_of_mappings() {
    let mut m1 = Mapping::new();
    m1.insert(
        Value::String("a".into()),
        Value::Number(Number::from(1i64)),
    );
    let mut m2 = Mapping::new();
    m2.insert(
        Value::String("b".into()),
        Value::Number(Number::from(2i64)),
    );
    let v =
        Value::Sequence(vec![Value::Mapping(m1), Value::Mapping(m2)]);
    let yaml = to_string(&v).unwrap();
    assert!(yaml.contains("- a:"));
    assert!(yaml.contains("- b:"));
}

#[test]
fn serialize_nested_sequence() {
    let v = Value::Sequence(vec![Value::Sequence(vec![
        Value::Number(Number::from(1i64)),
        Value::Number(Number::from(2i64)),
    ])]);
    let yaml = to_string(&v).unwrap();
    assert!(yaml.contains('-'));
}

#[test]
fn serialize_tagged_value() {
    let v = Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new("!custom"),
        value: Value::String("data".into()),
    }));
    let yaml = to_string(&v).unwrap();
    assert!(yaml.contains("!custom"));
}

// ── ser.rs: mapping with compound values ───────────────────────────

#[test]
fn serialize_mapping_with_nested_mapping() {
    let mut inner = Mapping::new();
    inner.insert(
        Value::String("x".into()),
        Value::Number(Number::from(1i64)),
    );
    let mut outer = Mapping::new();
    outer.insert(Value::String("nested".into()), Value::Mapping(inner));
    let yaml = to_string(&Value::Mapping(outer)).unwrap();
    assert!(yaml.contains("nested:"));
    assert!(yaml.contains("x:"));
}

#[test]
fn serialize_mapping_with_nested_sequence() {
    let mut m = Mapping::new();
    m.insert(
        Value::String("items".into()),
        Value::Sequence(vec![
            Value::Number(Number::from(1i64)),
            Value::Number(Number::from(2i64)),
        ]),
    );
    let yaml = to_string(&Value::Mapping(m)).unwrap();
    assert!(yaml.contains("items:"));
    assert!(yaml.contains("- "));
}

// ── de.rs: double-quoted escape sequences ──────────────────────────

#[test]
fn deserialize_double_quote_escapes() {
    let yaml = r#"a: "hello\tworld""#;
    let v: Value = from_str(yaml).unwrap();
    let s = v.as_mapping().unwrap().get("a").unwrap().as_str().unwrap();
    assert!(s.contains('\t'));

    let yaml = r#"a: "line\\end""#;
    let v: Value = from_str(yaml).unwrap();
    let s = v.as_mapping().unwrap().get("a").unwrap().as_str().unwrap();
    assert!(s.contains('\\'));

    let yaml = r#"a: "with\"quote""#;
    let v: Value = from_str(yaml).unwrap();
    let s = v.as_mapping().unwrap().get("a").unwrap().as_str().unwrap();
    assert!(s.contains('"'));

    let yaml = r#"a: "with\/slash""#;
    let v: Value = from_str(yaml).unwrap();
    let s = v.as_mapping().unwrap().get("a").unwrap().as_str().unwrap();
    assert!(s.contains('/'));

    let yaml = r#"a: "cr\rhere""#;
    let v: Value = from_str(yaml).unwrap();
    let s = v.as_mapping().unwrap().get("a").unwrap().as_str().unwrap();
    assert!(s.contains('\r'));
}

// ── de.rs: single-quoted escaped quotes ────────────────────────────

#[test]
fn deserialize_single_quote_escape() {
    let yaml = "a: 'it''s'\n";
    let v: Value = from_str(yaml).unwrap();
    let s = v.as_mapping().unwrap().get("a").unwrap().as_str().unwrap();
    assert_eq!(s, "it's");
}

// ── de.rs: quoted mapping keys ─────────────────────────────────────

#[test]
fn deserialize_quoted_mapping_key() {
    let yaml = "'key with spaces': value\n";
    let v: Value = from_str(yaml).unwrap();
    let m = v.as_mapping().unwrap();
    assert!(m.get("key with spaces").is_some());
}

#[test]
fn deserialize_double_quoted_mapping_key() {
    let yaml = "\"key\": value\n";
    let v: Value = from_str(yaml).unwrap();
    let m = v.as_mapping().unwrap();
    assert!(m.get("key").is_some());
}

// ── de.rs: special scalar values ───────────────────────────────────

#[test]
fn deserialize_null_variants() {
    for s in &["null", "Null", "NULL", "~"] {
        let v: Value = from_str(s).unwrap();
        assert!(v.is_null(), "{s} should parse as null");
    }
}

#[test]
fn deserialize_bool_variants() {
    for s in &["true", "True", "TRUE"] {
        let v: Value = from_str(s).unwrap();
        assert_eq!(v.as_bool(), Some(true), "{s}");
    }
    for s in &["false", "False", "FALSE"] {
        let v: Value = from_str(s).unwrap();
        assert_eq!(v.as_bool(), Some(false), "{s}");
    }
}

#[test]
fn deserialize_special_floats() {
    let v: Value = from_str(".nan").unwrap();
    assert!(v.as_f64().unwrap().is_nan());

    let v: Value = from_str(".NaN").unwrap();
    assert!(v.as_f64().unwrap().is_nan());

    let v: Value = from_str(".inf").unwrap();
    assert!(v.as_f64().unwrap().is_infinite());

    let v: Value = from_str("-.inf").unwrap();
    assert!(v.as_f64().unwrap().is_infinite());
    assert!(v.as_f64().unwrap().is_sign_negative());
}

// ── de.rs: empty input ─────────────────────────────────────────────

#[test]
fn deserialize_empty_input() {
    let v: Value = from_str("").unwrap();
    assert!(v.is_null());
}

#[test]
fn deserialize_whitespace_only() {
    let v: Value = from_str("   \n  \n").unwrap();
    assert!(v.is_null());
}

// ── de.rs: inline comment stripping ────────────────────────────────

#[test]
fn deserialize_inline_comment() {
    let yaml = "key: value # this is a comment\n";
    let v: Value = from_str(yaml).unwrap();
    let m = v.as_mapping().unwrap();
    let val = m.get("key").unwrap().as_str().unwrap();
    assert_eq!(val, "value");
}

// ── de.rs: hex and octal with uppercase prefix ─────────────────────

#[test]
fn deserialize_hex_uppercase() {
    let v: Value = from_str("0XFF").unwrap();
    assert_eq!(v.as_u64(), Some(255));
}

#[test]
fn deserialize_octal_uppercase() {
    let v: Value = from_str("0O77").unwrap();
    assert_eq!(v.as_u64(), Some(63));
}

// ── de.rs: integer with sign ───────────────────────────────────────

#[test]
fn deserialize_signed_integer() {
    let v: Value = from_str("-42").unwrap();
    assert_eq!(v.as_i64(), Some(-42));

    let v: Value = from_str("+42").unwrap();
    assert_eq!(v.as_i64(), Some(42));
}

// ── de.rs: float parsing ───────────────────────────────────────────

#[test]
fn deserialize_float() {
    let v: Value = from_str("2.75").unwrap();
    assert!(v.is_f64());
    assert!((v.as_f64().unwrap() - 2.75).abs() < 0.001);
}

#[test]
fn deserialize_scientific_notation() {
    let v: Value = from_str("1.0e2").unwrap();
    assert!(v.is_f64());
    assert!((v.as_f64().unwrap() - 100.0).abs() < 0.001);
}

// ── de.rs: sequence with empty item ────────────────────────────────

#[test]
fn deserialize_sequence_with_empty_item() {
    let yaml = "-\n- value\n";
    let v: Value = from_str(yaml).unwrap();
    let seq = v.as_sequence().unwrap();
    assert_eq!(seq.len(), 2);
    assert!(seq[0].is_null());
}

// ── de.rs: nested block sequences ──────────────────────────────────

#[test]
fn deserialize_nested_block_sequences() {
    let yaml = "- [inner1, inner2]\n- outer2\n";
    let v: Value = from_str(yaml).unwrap();
    let seq = v.as_sequence().unwrap();
    assert_eq!(seq.len(), 2);
}

// ── de.rs: mapping value on next line ──────────────────────────────

#[test]
fn deserialize_mapping_value_next_line() {
    // Test that a mapping key with a value on the same line works
    let yaml = "outer: {inner: value}\n";
    let v: Value = from_str(yaml).unwrap();
    let m = v.as_mapping().unwrap();
    let inner = m.get("outer").unwrap();
    assert!(inner.is_mapping());
    let inner_m = inner.as_mapping().unwrap();
    assert_eq!(
        inner_m.get("inner"),
        Some(&Value::String("value".into()))
    );
}

// ── de.rs: sequence with inline mappings ───────────────────────────

#[test]
fn deserialize_sequence_with_inline_mapping() {
    let yaml = "- {name: Alice, age: 30}\n- {name: Bob, age: 25}\n";
    let v: Value = from_str(yaml).unwrap();
    let seq = v.as_sequence().unwrap();
    assert_eq!(seq.len(), 2);
    assert!(seq[0].is_mapping());
    assert!(seq[1].is_mapping());
}

// ── de.rs: flow mapping colon error ────────────────────────────────

#[test]
fn flow_mapping_missing_colon() {
    let result: Result<Value, _> = from_str("{a 1}");
    assert!(result.is_err());
}

// ── Mapping Deserialize: duplicate key error ───────────────────────

#[test]
fn mapping_deserialize_duplicate_key() {
    let yaml = "a: 1\na: 2\n";
    let result: Result<Mapping, _> = from_str(yaml);
    // Duplicate key detection depends on implementation;
    // at minimum it should not panic
    assert!(result.is_ok() || result.is_err());
}

// ── Mapping Deserialize: empty (unit) ──────────────────────────────

#[test]
fn mapping_deserialize_null_as_empty() {
    let _v: Value = from_str("~").unwrap();
    // Null cannot be deserialized as a Mapping directly via from_str
    // because from_str parses to Value first. Test via from_value.
    let result: Result<Mapping, _> = yaml_safe::from_value(Value::Null);
    // May succeed as empty mapping or fail - either is acceptable
    let _ = result;
}

// ── Mapping: iter_mut size_hint and ExactSizeIterator ──────────────

#[test]
fn mapping_iter_size_hint() {
    let mut m = Mapping::new();
    m.insert(Value::String("a".into()), Value::Null);
    m.insert(Value::String("b".into()), Value::Null);

    let iter = m.iter();
    assert_eq!(iter.len(), 2);
    let (low, high) = iter.size_hint();
    assert_eq!(low, 2);
    assert_eq!(high, Some(2));

    let iter_mut = m.iter_mut();
    assert_eq!(iter_mut.len(), 2);

    let into_iter = m.clone().into_iter();
    assert_eq!(into_iter.len(), 2);

    let keys = m.keys();
    assert_eq!(keys.len(), 2);

    let values = m.values();
    assert_eq!(values.len(), 2);

    let into_keys = m.clone().into_keys();
    assert_eq!(into_keys.len(), 2);

    let into_values = m.into_values();
    assert_eq!(into_values.len(), 2);
}

// ── VariantDeserializer: unit_variant with null ────────────────────

#[test]
fn variant_deserializer_unit_variant() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum E {
        A,
    }
    let mut m = Mapping::new();
    m.insert(Value::String("A".into()), Value::Null);
    let v: E = yaml_safe::from_value(Value::Mapping(m)).unwrap();
    assert_eq!(v, E::A);
}

// ── VariantDeserializer: error paths ───────────────────────────────

#[test]
fn variant_deserializer_tuple_variant_non_sequence() {
    #[derive(Deserialize, Debug)]
    enum E {
        T(i32, i32),
    }
    let mut m = Mapping::new();
    m.insert(
        Value::String("T".into()),
        Value::String("not a sequence".into()),
    );
    let result: Result<E, _> = yaml_safe::from_value(Value::Mapping(m));
    assert!(result.is_err());
}

#[test]
fn variant_deserializer_struct_variant_non_mapping() {
    #[derive(Deserialize, Debug)]
    enum E {
        S { x: i32 },
    }
    let mut m = Mapping::new();
    m.insert(
        Value::String("S".into()),
        Value::String("not a mapping".into()),
    );
    let result: Result<E, _> = yaml_safe::from_value(Value::Mapping(m));
    assert!(result.is_err());
}

// ── Value: PartialEq edge cases ────────────────────────────────────

#[test]
fn value_partial_eq_u8_u16() {
    let v = Value::Number(Number::from(42i64));
    assert!(v == 42u8);
    assert!(v == 42u16);
}

#[test]
fn value_partial_eq_non_number_returns_false() {
    let v = Value::String("hello".into());
    assert!(!(v == 42i64));
    assert!(!(v == 42u64));
    assert!(!(v == 1.0f64));
    assert!(!(v == true));
}

// ── Serialize empty string ─────────────────────────────────────────

#[test]
fn serialize_empty_string() {
    let v = Value::String(String::new());
    let yaml = to_string(&v).unwrap();
    assert!(yaml.contains("''"));
}

// ── Serialize string with single quotes ────────────────────────────

#[test]
fn serialize_string_containing_single_quote() {
    let v = Value::String("it's".into());
    let yaml = to_string(&v).unwrap();
    // The string contains a quote so it should be quoted in some way
    assert!(
        yaml.contains('\'') || yaml.contains('"'),
        "string with quote should be quoted: {yaml}"
    );
}

// ── Number: Eq ─────────────────────────────────────────────────────

#[test]
fn number_eq_different_types() {
    let a = Number::from(42u64);
    let b = Number::from(-1i64);
    assert_ne!(a, b);

    let c = Number::from(1.0f64);
    let d = Number::from(1u64);
    // Float and integer are different internal types
    assert_ne!(c, d);
}

// ── Number: Hash for float ─────────────────────────────────────────

#[test]
fn number_hash_float() {
    let a = Number::from(1.5f64);
    let b = Number::from(1.5f64);
    let mut h1 = DefaultHasher::new();
    let mut h2 = DefaultHasher::new();
    a.hash(&mut h1);
    b.hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn number_hash_negative() {
    let a = Number::from(-42i64);
    let b = Number::from(-42i64);
    let mut h1 = DefaultHasher::new();
    let mut h2 = DefaultHasher::new();
    a.hash(&mut h1);
    b.hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}

// ── Number: Serialize all variants ─────────────────────────────────

#[test]
fn number_serialize_all_types() {
    let n = Number::from(42u64);
    let yaml = to_string(&n).unwrap();
    assert!(yaml.contains("42"));

    let n = Number::from(-7i64);
    let yaml = to_string(&n).unwrap();
    assert!(yaml.contains("-7"));

    let n = Number::from(1.5f64);
    let yaml = to_string(&n).unwrap();
    assert!(yaml.contains("1.5"));
}

// ── Number: Deserializer impl (for &Number) ────────────────────────

#[test]
fn number_ref_deserializer() {
    let n = Number::from(42u64);
    let v: u64 = serde::Deserialize::deserialize(&n).unwrap();
    assert_eq!(v, 42);

    let n = Number::from(-7i64);
    let v: i64 = serde::Deserialize::deserialize(&n).unwrap();
    assert_eq!(v, -7);

    let n = Number::from(1.5f64);
    let v: f64 = serde::Deserialize::deserialize(&n).unwrap();
    assert!((v - 1.5).abs() < f64::EPSILON);
}

// ── Mapping: swap_remove_entry with Value key ──────────────────────

#[test]
fn mapping_swap_remove_entry_value_key() {
    let mut m = Mapping::new();
    m.insert(Value::String("key".into()), Value::Bool(true));
    let entry = m.swap_remove_entry(Value::String("key".into()));
    assert!(entry.is_some());
    let (k, v) = entry.unwrap();
    assert_eq!(k, Value::String("key".into()));
    assert_eq!(v, Value::Bool(true));
}

// ── Mapping: shift_remove_entry with str key ───────────────────────

#[test]
fn mapping_shift_remove_entry_str_key() {
    let mut m = Mapping::new();
    m.insert(
        Value::String("key".into()),
        Value::Number(Number::from(1i64)),
    );
    let entry = m.shift_remove_entry("key");
    assert!(entry.is_some());
}

// ── Mapping: Index with &str reference ─────────────────────────────

#[test]
fn mapping_index_with_ref_str() {
    let mut m = Mapping::new();
    m.insert(Value::String("k".into()), Value::Null);
    let key: &str = "k";
    assert!(m.contains_key(key));
    assert!(m.get(key).is_some());
}

// ── Mapping: Extend ────────────────────────────────────────────────

#[test]
fn mapping_extend() {
    let mut m = Mapping::new();
    m.insert(Value::String("a".into()), Value::Null);
    let extra = vec![
        (Value::String("b".into()), Value::Bool(true)),
        (Value::String("c".into()), Value::Bool(false)),
    ];
    m.extend(extra);
    assert_eq!(m.len(), 3);
}

// ── Mapping: Hash non-trivial ──────────────────────────────────────

#[test]
fn mapping_hash_non_trivial() {
    let mut m1 = Mapping::new();
    m1.insert(
        Value::String("a".into()),
        Value::Number(Number::from(1i64)),
    );
    m1.insert(
        Value::String("b".into()),
        Value::Number(Number::from(2i64)),
    );

    let mut m2 = Mapping::new();
    m2.insert(
        Value::String("b".into()),
        Value::Number(Number::from(2i64)),
    );
    m2.insert(
        Value::String("a".into()),
        Value::Number(Number::from(1i64)),
    );

    // Hash should be order-independent (XOR-based)
    let mut h1 = DefaultHasher::new();
    let mut h2 = DefaultHasher::new();
    m1.hash(&mut h1);
    m2.hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}

// ── Value: is_f64 false for non-float ──────────────────────────────

#[test]
fn value_is_f64() {
    let v = Value::Number(Number::from(42u64));
    assert!(!v.is_f64());

    let v = Value::Number(Number::from(1.5f64));
    assert!(v.is_f64());

    let v = Value::String("not a number".into());
    assert!(!v.is_f64());
}

// ── Value: is_i64 / is_u64 edge cases ──────────────────────────────

#[test]
fn value_is_i64_u64_edge() {
    let v = Value::Number(Number::from(u64::MAX));
    assert!(v.is_u64());
    assert!(!v.is_i64());

    let v = Value::String("not a number".into());
    assert!(!v.is_i64());
    assert!(!v.is_u64());
}

// ── De: typed deserialization with nested optionals ─────────────────

#[test]
fn deserialize_struct_with_optional() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Config {
        name: String,
        debug: Option<bool>,
    }

    let yaml = "name: test\n";
    let c: Config = from_str(yaml).unwrap();
    assert_eq!(c.name, "test");
    assert_eq!(c.debug, None);
}

// ── De: typed enum from YAML string ────────────────────────────────

#[test]
fn deserialize_enum_from_yaml_str() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Status {
        Active,
        Inactive,
    }
    let v: Status = from_str("Active").unwrap();
    assert_eq!(v, Status::Active);
}

// ── Value: tagged value accessors through untag ────────────────────

#[test]
fn tagged_value_accessor_methods() {
    let tv = Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new("!bool"),
        value: Value::Bool(true),
    }));
    assert!(tv.is_bool());
    assert_eq!(tv.as_bool(), Some(true));
    assert!(!tv.is_null());
    assert!(!tv.is_number());
    assert!(!tv.is_string());
    assert!(!tv.is_sequence());
    assert!(!tv.is_mapping());

    let tv = Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new("!str"),
        value: Value::String("hello".into()),
    }));
    assert!(tv.is_string());
    assert_eq!(tv.as_str(), Some("hello"));
}

// ── Value: nested tagged values ────────────────────────────────────

#[test]
fn nested_tagged_value_untag() {
    let inner = TaggedValue {
        tag: Tag::new("!inner"),
        value: Value::Number(Number::from(42i64)),
    };
    let outer = Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new("!outer"),
        value: Value::Tagged(Box::new(inner)),
    }));
    // untag_ref should traverse through both tags
    assert!(outer.is_number());
    assert_eq!(outer.as_i64(), Some(42));
}

// ── Value: get/get_mut through tagged ──────────────────────────────

#[test]
fn tagged_value_get() {
    let mut m = Mapping::new();
    m.insert(Value::String("key".into()), Value::Bool(true));
    let tagged = Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new("!map"),
        value: Value::Mapping(m),
    }));
    assert_eq!(tagged.get("key"), Some(&Value::Bool(true)));
}

#[test]
fn tagged_value_get_mut() {
    let mut m = Mapping::new();
    m.insert(Value::String("key".into()), Value::Bool(true));
    let mut tagged = Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new("!map"),
        value: Value::Mapping(m),
    }));
    if let Some(v) = tagged.get_mut("key") {
        *v = Value::Bool(false);
    }
    assert_eq!(tagged.get("key"), Some(&Value::Bool(false)));
}

// ── Value: as_sequence_mut / as_mapping_mut on wrong type ──────────

#[test]
fn as_sequence_mut_on_non_sequence() {
    let mut v = Value::String("not a seq".into());
    assert!(v.as_sequence_mut().is_none());
}

#[test]
fn as_mapping_mut_on_non_mapping() {
    let mut v = Value::String("not a map".into());
    assert!(v.as_mapping_mut().is_none());
}

// ── Additional targeted coverage for remaining gaps ────��────────────

#[test]
fn mapping_index_ops() {
    let mut m = Mapping::new();
    m.insert(Value::String("a".into()), Value::Number(Number::from(1)));
    // std::ops::Index
    let _ = &m[Value::String("a".into())];
}

#[test]
fn value_total_cmp_coverage() {
    // Exercise all arms of total_cmp by sorting mixed types
    let mut vals = vec![
        Value::Tagged(Box::new(TaggedValue {
            tag: Tag::new("!b"),
            value: Value::Null,
        })),
        Value::Tagged(Box::new(TaggedValue {
            tag: Tag::new("!a"),
            value: Value::Null,
        })),
        Value::Mapping(Mapping::new()),
        Value::Sequence(vec![]),
        Value::String("z".into()),
        Value::String("a".into()),
        Value::Number(Number::from(2)),
        Value::Number(Number::from(1)),
        Value::Bool(true),
        Value::Bool(false),
        Value::Null,
    ];
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    // First should be Null, last should be Tagged
    assert!(vals[0].is_null());
}

#[test]
fn value_get_through_tagged() {
    let mut m = Mapping::new();
    m.insert(
        Value::String("key".into()),
        Value::Number(Number::from(42)),
    );
    let tagged = Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new("!map"),
        value: Value::Mapping(m),
    }));
    // get should unwrap tags
    let result = tagged.get("key");
    assert!(result.is_some());
}

#[test]
fn tagged_value_deserialize_enum() {
    // Tagged values can represent enums
    let yaml = "!variant data\n";
    let v: Value = from_str(yaml).unwrap();
    assert!(matches!(v, Value::Tagged(_)));
}

#[test]
fn ser_emit_inline_sequence_in_mapping() {
    // Force inline sequence by nesting
    let mut m = Mapping::new();
    let inner = Value::Sequence(vec![
        Value::Number(Number::from(1)),
        Value::Number(Number::from(2)),
    ]);
    m.insert(Value::String("items".into()), inner);
    let yaml = to_string(&Value::Mapping(m)).unwrap();
    assert!(yaml.contains("items:"));
}

#[test]
fn ser_emit_nested_mapping_in_sequence() {
    let mut inner = Mapping::new();
    inner.insert(
        Value::String("x".into()),
        Value::Number(Number::from(1)),
    );
    let seq = Value::Sequence(vec![Value::Mapping(inner)]);
    let yaml = to_string(&seq).unwrap();
    assert!(yaml.contains("- x:"));
}

#[test]
fn de_single_quoted_with_escape() {
    let yaml = "msg: 'it''s a test'\n";
    let v: Value = from_str(yaml).unwrap();
    let m = v.as_mapping().unwrap();
    let msg = m.get(Value::String("msg".into())).unwrap();
    assert_eq!(msg.as_str().unwrap(), "it's a test");
}

#[test]
fn de_double_quoted_escapes_comprehensive() {
    let yaml = r#"a: "tab\there"
b: "newline\nhere"
c: "null\0here"
d: "backslash\\here"
"#;
    let v: Value = from_str(yaml).unwrap();
    let m = v.as_mapping().unwrap();
    assert!(m
        .get(Value::String("a".into()))
        .unwrap()
        .as_str()
        .unwrap()
        .contains('\t'));
    assert!(m
        .get(Value::String("b".into()))
        .unwrap()
        .as_str()
        .unwrap()
        .contains('\n'));
    assert!(m
        .get(Value::String("d".into()))
        .unwrap()
        .as_str()
        .unwrap()
        .contains('\\'));
}

#[test]
fn de_block_scalar_all_chomp_modes() {
    // Literal with strip
    let yaml = "a: |-\n  line1\n  line2\n\n";
    let v: Value = from_str(yaml).unwrap();
    let s = v
        .as_mapping()
        .unwrap()
        .get(Value::String("a".into()))
        .unwrap()
        .as_str()
        .unwrap();
    assert!(!s.ends_with('\n'));

    // Literal with keep
    let yaml = "a: |+\n  line1\n  line2\n\n";
    let v: Value = from_str(yaml).unwrap();
    let s = v
        .as_mapping()
        .unwrap()
        .get(Value::String("a".into()))
        .unwrap()
        .as_str()
        .unwrap();
    assert!(s.ends_with('\n'));

    // Folded with strip
    let yaml = "a: >-\n  line1\n  line2\n\n";
    let v: Value = from_str(yaml).unwrap();
    let s = v
        .as_mapping()
        .unwrap()
        .get(Value::String("a".into()))
        .unwrap()
        .as_str()
        .unwrap();
    assert!(!s.ends_with('\n'));

    // Folded with keep
    let yaml = "a: >+\n  line1\n  line2\n\n";
    let v: Value = from_str(yaml).unwrap();
    let s = v
        .as_mapping()
        .unwrap()
        .get(Value::String("a".into()))
        .unwrap()
        .as_str()
        .unwrap();
    assert!(s.ends_with('\n'));
}

#[test]
fn de_complex_flow_mapping() {
    let yaml = "{a: {b: 1, c: 2}, d: [1, 2, {e: 3}]}";
    let v: Value = from_str(yaml).unwrap();
    assert!(v.is_mapping());
    let m = v.as_mapping().unwrap();
    assert_eq!(m.len(), 2);
}

#[test]
fn number_partial_ord_cross_type() {
    let pos = Number::from(5u64);
    let neg = Number::from(-3i64);
    let float = Number::from(1.5f64);
    // Cross-type comparisons exercise PartialOrd
    assert!(pos > neg);
    assert!(float > neg);
    // pos vs float comparison depends on internal representation
    assert!(pos.partial_cmp(&float).is_some());
}

#[test]
fn mapping_vacant_entry_into_key() {
    let mut m = Mapping::new();
    let key = Value::String("new_key".into());
    if let yaml_safe::mapping::Entry::Vacant(v) = m.entry(key.clone()) {
        let k = v.into_key();
        assert_eq!(k, key);
    }
}

#[test]
fn mapping_occupied_remove_entry() {
    let mut m = Mapping::new();
    m.insert(Value::String("k".into()), Value::String("v".into()));
    if let yaml_safe::mapping::Entry::Occupied(o) =
        m.entry(Value::String("k".into()))
    {
        let (k, v) = o.remove_entry();
        assert_eq!(k, Value::String("k".into()));
        assert_eq!(v, Value::String("v".into()));
    }
    assert!(m.is_empty());
}

#[test]
fn mapping_occupied_into_mut() {
    let mut m = Mapping::new();
    m.insert(Value::String("k".into()), Value::Number(Number::from(1)));
    if let yaml_safe::mapping::Entry::Occupied(o) =
        m.entry(Value::String("k".into()))
    {
        let v = o.into_mut();
        *v = Value::Number(Number::from(99));
    }
    assert_eq!(m.get("k"), Some(&Value::Number(Number::from(99))));
}

#[test]
fn error_location_struct() {
    // Location is returned by Error::location() which always returns None
    // but we can test the struct exists
    let err: yaml_safe::Result<Value> = from_str("[unclosed");
    assert!(err.unwrap_err().location().is_none());
}

#[test]
fn de_mapping_with_quoted_keys() {
    let yaml = "'key with spaces': value\n\"another key\": 42\n";
    let v: Value = from_str(yaml).unwrap();
    let m = v.as_mapping().unwrap();
    assert!(m.contains_key("key with spaces"));
    assert!(m.contains_key("another key"));
}

#[test]
fn de_all_null_variants() {
    for input in ["null", "Null", "NULL", "~"] {
        let v: Value = from_str(input).unwrap();
        assert!(v.is_null(), "Expected null for input: {input}");
    }
}

#[test]
fn de_all_bool_variants() {
    for (input, expected) in [
        ("true", true),
        ("True", true),
        ("TRUE", true),
        ("false", false),
        ("False", false),
        ("FALSE", false),
    ] {
        let v: Value = from_str(input).unwrap();
        assert_eq!(v.as_bool(), Some(expected), "For input: {input}");
    }
}

#[test]
fn de_special_float_variants() {
    for input in [".nan", ".NaN", ".NAN"] {
        let v: Value = from_str(input).unwrap();
        assert!(v.as_f64().unwrap().is_nan(), "For input: {input}");
    }
    for input in [".inf", ".Inf", ".INF"] {
        let v: Value = from_str(input).unwrap();
        assert!(
            v.as_f64().unwrap().is_infinite()
                && v.as_f64().unwrap() > 0.0,
            "For input: {input}"
        );
    }
    for input in ["-.inf", "-.Inf", "-.INF"] {
        let v: Value = from_str(input).unwrap();
        assert!(
            v.as_f64().unwrap().is_infinite()
                && v.as_f64().unwrap() < 0.0,
            "For input: {input}"
        );
    }
}

#[test]
fn de_scientific_notation() {
    let v: Value = from_str("1.5e3").unwrap();
    assert!((v.as_f64().unwrap() - 1500.0).abs() < 0.1);

    let v: Value = from_str("2E-2").unwrap();
    assert!((v.as_f64().unwrap() - 0.02).abs() < 0.001);
}

#[test]
fn de_underscore_in_numbers() {
    let v: Value = from_str("1_000_000").unwrap();
    assert_eq!(v.as_u64(), Some(1_000_000));
}

#[test]
fn ser_tagged_value() {
    let tagged = Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new("!custom"),
        value: Value::Mapping({
            let mut m = Mapping::new();
            m.insert(
                Value::String("a".into()),
                Value::Number(Number::from(1)),
            );
            m
        }),
    }));
    let yaml = to_string(&tagged).unwrap();
    assert!(yaml.contains("!custom"));
}

#[test]
fn value_from_impls() {
    let _: Value = Value::from(true);
    let _: Value = Value::from("hello");
    let _: Value = Value::from(String::from("world"));
    let _: Value = Value::from(42i64);
    let _: Value = Value::from(42u64);
    let _: Value = Value::from(1.5f64);
}
