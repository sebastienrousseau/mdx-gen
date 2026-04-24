// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::error::Error;
use crate::value::Value;
use serde::{
    de::{
        value::StrDeserializer, DeserializeSeed, Deserializer,
        EnumAccess, VariantAccess, Visitor,
    },
    forward_to_deserialize_any,
    ser::{Serialize, SerializeMap, Serializer},
    Deserialize,
};
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
};

/// A YAML `!Tag`.
#[derive(Clone)]
pub struct Tag {
    /// The string representation of the tag.
    pub string: String,
}

/// A `Tag` + `Value` representing a tagged YAML value.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct TaggedValue {
    /// The tag.
    pub tag: Tag,
    /// The value.
    pub value: Value,
}

impl TaggedValue {
    /// Creates a copy of this tagged value.
    pub fn copy(&self) -> TaggedValue {
        TaggedValue {
            tag: self.tag.clone(),
            value: self.value.clone(),
        }
    }
}

impl Tag {
    /// Creates a new `Tag`.
    pub fn new(string: impl Into<String>) -> Self {
        let tag: String = string.into();
        assert!(!tag.is_empty(), "empty YAML tag not allowed");
        Tag { string: tag }
    }
}

/// Returns the portion after the leading `!`, if any.
pub fn nobang(maybe_banged: &str) -> &str {
    match maybe_banged.strip_prefix('!') {
        Some("") | None => maybe_banged,
        Some(unbanged) => unbanged,
    }
}

impl Eq for Tag {}

impl PartialEq for Tag {
    fn eq(&self, other: &Tag) -> bool {
        nobang(&self.string) == nobang(&other.string)
    }
}

impl<T> PartialEq<T> for Tag
where
    T: ?Sized + AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        nobang(&self.string) == nobang(other.as_ref())
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        nobang(&self.string).cmp(nobang(&other.string))
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        nobang(&self.string).hash(hasher);
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "!{}", nobang(&self.string))
    }
}

impl Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl Serialize for TaggedValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        struct SerializeTag<'a>(&'a Tag);

        impl Serialize for SerializeTag<'_> {
            fn serialize<S>(
                &self,
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.collect_str(self.0)
            }
        }

        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&SerializeTag(&self.tag), &self.value)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for TaggedValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TaggedValueVisitor;

        impl<'de> Visitor<'de> for TaggedValueVisitor {
            type Value = TaggedValue;

            fn expecting(
                &self,
                f: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                f.write_str("a YAML value with a !Tag")
            }

            fn visit_enum<A>(
                self,
                data: A,
            ) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, contents) =
                    data.variant_seed(TagStringVisitor)?;
                let value = contents.newtype_variant()?;
                Ok(TaggedValue { tag, value })
            }
        }

        deserializer.deserialize_any(TaggedValueVisitor)
    }
}

impl<'de> Deserializer<'de> for TaggedValue {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    fn deserialize_ignored_any<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char
        str string bytes byte_buf option unit unit_struct
        newtype_struct seq tuple tuple_struct map struct
        enum identifier
    }
}

impl<'de> EnumAccess<'de> for TaggedValue {
    type Error = Error;
    type Variant = Value;

    fn variant_seed<V>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Error>
    where
        V: DeserializeSeed<'de>,
    {
        let tag =
            StrDeserializer::<Error>::new(nobang(&self.tag.string));
        let value = seed.deserialize(tag)?;
        Ok((value, self.value))
    }
}

impl<'de> VariantAccess<'de> for Value {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        Deserialize::deserialize(self)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Sequence(v) = self {
            let de = crate::de::SeqDeserializer::new(v);
            serde::Deserializer::deserialize_any(de, visitor)
        } else {
            Err(serde::de::Error::invalid_type(
                self.unexpected(),
                &"tuple variant",
            ))
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Mapping(v) = self {
            let de = crate::de::MapDeserializer::new(v);
            serde::Deserializer::deserialize_any(de, visitor)
        } else {
            Err(serde::de::Error::invalid_type(
                self.unexpected(),
                &"struct variant",
            ))
        }
    }
}

pub(crate) struct TagStringVisitor;

impl Visitor<'_> for TagStringVisitor {
    type Value = Tag;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a YAML tag string")
    }

    fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_string(string.to_owned())
    }

    fn visit_string<E>(self, string: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if string.is_empty() {
            return Err(E::custom("empty YAML tag is not allowed"));
        }
        Ok(Tag::new(string))
    }
}

impl<'de> DeserializeSeed<'de> for TagStringVisitor {
    type Value = Tag;

    fn deserialize<D>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(self)
    }
}

/// A tagged value with an optional tag.
#[derive(Debug)]
pub enum MaybeTag<T> {
    /// The tag.
    Tag(String),
    /// The value.
    NotTag(T),
}

/// Check if a value is a YAML tag.
pub fn check_for_tag<T>(value: &T) -> MaybeTag<String>
where
    T: ?Sized + Display,
{
    let s = format!("{}", value);
    match s.as_str() {
        "" => MaybeTag::NotTag(String::new()),
        "!" => MaybeTag::NotTag("!".to_owned()),
        tag if tag.starts_with('!') => MaybeTag::Tag(tag.to_owned()),
        _ => MaybeTag::NotTag(s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::Mapping;

    #[test]
    fn nobang_strips_single_leading_bang() {
        assert_eq!(nobang("!foo"), "foo");
        assert_eq!(nobang("foo"), "foo");
        assert_eq!(nobang(""), "");
        // A bare "!" is preserved — strip_prefix returns Some("")
        // which the match treats as "no tag" and returns the
        // whole string.
        assert_eq!(nobang("!"), "!");
    }

    #[test]
    #[should_panic(expected = "empty YAML tag not allowed")]
    fn tag_new_empty_panics() {
        let _ = Tag::new("");
    }

    #[test]
    fn tag_eq_ignores_leading_bang() {
        let a = Tag::new("!foo");
        let b = Tag::new("foo");
        assert_eq!(a, b);
        assert_eq!(a, "foo");
        assert_eq!(a, "!foo");
    }

    #[test]
    fn tag_ord_and_hash() {
        let a = Tag::new("!a");
        let b = Tag::new("!b");
        assert!(a < b);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));

        use std::collections::hash_map::DefaultHasher;
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        Tag::new("!x").hash(&mut h1);
        Tag::new("x").hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn tag_display_and_debug() {
        let a = Tag::new("!foo");
        assert_eq!(format!("{a}"), "!foo");
        assert_eq!(format!("{a:?}"), "!foo");
    }

    #[test]
    fn tagged_value_copy_clones_inner() {
        let tv = TaggedValue {
            tag: Tag::new("!T"),
            value: Value::String("x".into()),
        };
        let dup = tv.copy();
        assert_eq!(dup.tag, tv.tag);
        assert_eq!(dup.value, tv.value);
    }

    #[test]
    fn tagged_value_roundtrip_via_yaml() {
        // YAML literal `!Foo hello` parses into a TaggedValue
        // with tag "!Foo" and value String("hello"). This
        // exercises the Deserialize impl and the tag visitor
        // happy path.
        let tv: TaggedValue = crate::from_str("!Foo hello\n").unwrap();
        assert_eq!(tv.tag, Tag::new("!Foo"));
        assert_eq!(tv.value, Value::String("hello".into()));

        // Serialize back and round-trip a second time.
        let yaml = crate::to_string(&tv).unwrap();
        assert!(yaml.contains("Foo"));
    }

    #[test]
    fn check_for_tag_variants() {
        let t = check_for_tag("!Foo");
        assert!(matches!(t, MaybeTag::Tag(ref s) if s == "!Foo"));
        let t = check_for_tag("");
        assert!(matches!(t, MaybeTag::NotTag(ref s) if s.is_empty()));
        let t = check_for_tag("!");
        assert!(matches!(t, MaybeTag::NotTag(ref s) if s == "!"));
        let t = check_for_tag("plain");
        assert!(matches!(t, MaybeTag::NotTag(ref s) if s == "plain"));
    }

    #[test]
    fn variant_access_unit_and_newtype() {
        use serde::Deserialize;

        #[derive(Debug, PartialEq, Deserialize)]
        enum E {
            Unit,
            Name(String),
            Point { x: i32, y: i32 },
        }

        // Build a TaggedValue manually and deserialize it
        // through the enum path — exercises EnumAccess and
        // VariantAccess::{unit_variant, newtype_variant_seed}.
        let tv = TaggedValue {
            tag: Tag::new("!Unit"),
            value: Value::Null,
        };
        let got: E = E::deserialize(tv).unwrap();
        assert_eq!(got, E::Unit);

        let tv = TaggedValue {
            tag: Tag::new("!Name"),
            value: Value::String("alice".into()),
        };
        let got: E = E::deserialize(tv).unwrap();
        assert_eq!(got, E::Name("alice".into()));

        // Struct variant — exercises struct_variant on Value.
        let mut m = Mapping::new();
        m.insert(Value::String("x".into()), Value::Number(1.into()));
        m.insert(Value::String("y".into()), Value::Number(2.into()));
        let tv = TaggedValue {
            tag: Tag::new("!Point"),
            value: Value::Mapping(m),
        };
        let got: E = E::deserialize(tv).unwrap();
        assert_eq!(got, E::Point { x: 1, y: 2 });
    }

    #[test]
    fn variant_access_tuple_variant_via_sequence() {
        use serde::Deserialize;

        #[derive(Debug, PartialEq, Deserialize)]
        enum E {
            Pair(i32, i32),
        }

        let tv = TaggedValue {
            tag: Tag::new("!Pair"),
            value: Value::Sequence(vec![
                Value::Number(3.into()),
                Value::Number(4.into()),
            ]),
        };
        let got: E = E::deserialize(tv).unwrap();
        assert_eq!(got, E::Pair(3, 4));
    }

    #[test]
    fn variant_access_tuple_mismatch_returns_error() {
        use serde::Deserialize;

        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        enum E {
            Pair(i32, i32),
        }

        // Non-sequence value for a tuple variant must error.
        let tv = TaggedValue {
            tag: Tag::new("!Pair"),
            value: Value::String("oops".into()),
        };
        assert!(E::deserialize(tv).is_err());
    }

    #[test]
    fn variant_access_struct_mismatch_returns_error() {
        use serde::Deserialize;

        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        enum E {
            Thing { x: i32 },
        }

        let tv = TaggedValue {
            tag: Tag::new("!Thing"),
            value: Value::String("oops".into()),
        };
        assert!(E::deserialize(tv).is_err());
    }

    #[test]
    fn deserialize_ignored_any_on_tagged_value() {
        use serde::de::IgnoredAny;
        use serde::Deserialize;

        let tv = TaggedValue {
            tag: Tag::new("!Whatever"),
            value: Value::String("x".into()),
        };
        let _: IgnoredAny = IgnoredAny::deserialize(tv).unwrap();
    }
}
