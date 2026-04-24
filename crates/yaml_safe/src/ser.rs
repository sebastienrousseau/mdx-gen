// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! YAML serialization: `to_string` and `to_writer`.

use crate::{
    error::Error,
    value::{self, Value},
};
use serde::Serialize;
use std::io::Write;

type Result<T> = std::result::Result<T, Error>;

/// Serialize the given value as a YAML string.
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let v = value.serialize(value::ValueSerializer)?;
    let mut out = String::new();
    emit_value(&v, &mut out, 0, false);
    Ok(out)
}

/// Serialize the given value as YAML into a writer.
pub fn to_writer<W, T>(mut writer: W, value: &T) -> Result<()>
where
    W: Write,
    T: ?Sized + Serialize,
{
    let s = to_string(value)?;
    writer
        .write_all(s.as_bytes())
        .map_err(|e| Error::msg(e.to_string()))
}

/// The state of the YAML serializer (for API compat).
#[derive(Debug)]
pub enum State {
    /// Nothing in particular.
    NothingInParticular,
}

/// A YAML serializer (for API compat).
#[derive(Debug)]
pub struct Serializer {
    _private: (),
}

fn emit_value(
    v: &Value,
    out: &mut String,
    indent: usize,
    inline: bool,
) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => {
            out.push_str(if *b { "true" } else { "false" });
        }
        Value::Number(n) => {
            out.push_str(&n.to_string());
        }
        Value::String(s) => {
            emit_string(s, out);
        }
        Value::Sequence(seq) => {
            if seq.is_empty() {
                out.push_str("[]");
            } else if inline {
                // For inline, use flow style
                emit_flow_sequence(seq, out);
            } else {
                emit_block_sequence(seq, out, indent);
            }
        }
        Value::Mapping(m) => {
            if m.is_empty() {
                out.push_str("{}");
            } else if inline {
                emit_flow_mapping(m, out);
            } else {
                emit_block_mapping(m, out, indent);
            }
        }
        Value::Tagged(t) => {
            out.push_str(&format!("{} ", t.tag));
            emit_value(&t.value, out, indent, inline);
        }
    }
}

fn emit_string(s: &str, out: &mut String) {
    if s.is_empty() {
        out.push_str("''");
        return;
    }
    // Check if the string needs quoting
    if !needs_quoting(s) {
        out.push_str(s);
        return;
    }
    // If the string contains control chars or special
    // Unicode escapes, use double-quoted style
    if needs_double_quoting(s) {
        out.push('"');
        for ch in s.chars() {
            match ch {
                '\0' => out.push_str("\\0"),
                '\x07' => out.push_str("\\a"),
                '\x08' => out.push_str("\\b"),
                '\t' => out.push_str("\\t"),
                '\n' => out.push_str("\\n"),
                '\x0B' => out.push_str("\\v"),
                '\x0C' => out.push_str("\\f"),
                '\r' => out.push_str("\\r"),
                '\x1B' => out.push_str("\\e"),
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\u{0085}' => out.push_str("\\N"),
                '\u{00A0}' => out.push_str("\\_"),
                '\u{2028}' => out.push_str("\\L"),
                '\u{2029}' => out.push_str("\\P"),
                c if c.is_control() => {
                    let code = c as u32;
                    if code <= 0xFF {
                        out.push_str(&format!("\\x{:02x}", code));
                    } else if code <= 0xFFFF {
                        out.push_str(&format!("\\u{:04x}", code));
                    } else {
                        out.push_str(&format!("\\U{:08x}", code));
                    }
                }
                c => out.push(c),
            }
        }
        out.push('"');
    } else {
        // Single-quoted style
        out.push('\'');
        for ch in s.chars() {
            if ch == '\'' {
                out.push_str("''");
            } else {
                out.push(ch);
            }
        }
        out.push('\'');
    }
}

fn needs_double_quoting(s: &str) -> bool {
    s.chars().any(|ch| {
        ch.is_control()
            || ch == '\u{0085}'
            || ch == '\u{00A0}'
            || ch == '\u{2028}'
            || ch == '\u{2029}'
    })
}

fn needs_quoting(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    // Values that would be interpreted as non-string
    match s {
        "null" | "Null" | "NULL" | "~" | "true" | "True" | "TRUE"
        | "false" | "False" | "FALSE" | ".nan" | ".NaN" | ".NAN"
        | ".inf" | ".Inf" | ".INF" | "-.inf" | "-.Inf" | "-.INF" => {
            return true
        }
        _ => {}
    }
    let first = s.as_bytes()[0];
    // Starts with special char
    if matches!(
        first,
        b'{' | b'}'
            | b'['
            | b']'
            | b','
            | b'&'
            | b'*'
            | b'!'
            | b'|'
            | b'>'
            | b'%'
            | b'@'
            | b'`'
            | b'\''
            | b'"'
    ) {
        return true;
    }
    // Contains problematic chars
    if s.contains(": ")
        || s.contains(" #")
        || s.contains('\n')
        || s.contains('\r')
        || s.starts_with("- ")
        || s.starts_with("? ")
    {
        return true;
    }
    // Looks like a number
    if s.parse::<i64>().is_ok() || s.parse::<f64>().is_ok() {
        return true;
    }
    // Contains control chars or special Unicode that
    // would be mangled by plain scalar parsing
    if needs_double_quoting(s) {
        return true;
    }
    false
}

fn emit_block_sequence(seq: &[Value], out: &mut String, indent: usize) {
    for (i, item) in seq.iter().enumerate() {
        if (i > 0 || !out.is_empty()) && !out.ends_with('\n') {
            out.push('\n');
        }
        emit_indent(out, indent);
        out.push_str("- ");
        match item {
            Value::Mapping(m) if !m.is_empty() => {
                // First entry inline after "-", rest
                // indented
                let mut first = true;
                for (k, v) in m {
                    if first {
                        first = false;
                    } else {
                        out.push('\n');
                        emit_indent(out, indent + 2);
                    }
                    emit_value(k, out, indent + 2, true);
                    out.push_str(": ");
                    if is_compound(v) {
                        out.push('\n');
                        emit_value(v, out, indent + 4, false);
                    } else {
                        emit_value(v, out, indent + 4, true);
                    }
                }
            }
            Value::Sequence(s) if !s.is_empty() => {
                out.push('\n');
                emit_block_sequence(s, out, indent + 2);
            }
            _ => {
                emit_value(item, out, indent + 2, true);
            }
        }
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
}

fn emit_block_mapping(
    m: &crate::mapping::Mapping,
    out: &mut String,
    indent: usize,
) {
    for (i, (k, v)) in m.iter().enumerate() {
        if (i > 0 || !out.is_empty()) && !out.ends_with('\n') {
            out.push('\n');
        }
        emit_indent(out, indent);
        emit_value(k, out, indent, true);
        out.push_str(": ");
        if is_compound(v) {
            out.push('\n');
            emit_value(v, out, indent + 2, false);
        } else {
            emit_value(v, out, indent + 2, true);
            out.push('\n');
        }
    }
}

fn emit_flow_sequence(seq: &[Value], out: &mut String) {
    out.push('[');
    for (i, item) in seq.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        emit_value(item, out, 0, true);
    }
    out.push(']');
}

fn emit_flow_mapping(m: &crate::mapping::Mapping, out: &mut String) {
    out.push('{');
    for (i, (k, v)) in m.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        emit_value(k, out, 0, true);
        out.push_str(": ");
        emit_value(v, out, 0, true);
    }
    out.push('}');
}

fn emit_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

fn is_compound(v: &Value) -> bool {
    matches!(
        v,
        Value::Mapping(m) if !m.is_empty()
    ) || matches!(
        v,
        Value::Sequence(s) if !s.is_empty()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::Mapping;
    use crate::value::tagged::{Tag, TaggedValue};

    #[test]
    fn emit_tagged_value_top_level() {
        let tv = TaggedValue {
            tag: Tag::new("!T"),
            value: Value::String("x".into()),
        };
        let yaml = to_string(&Value::Tagged(Box::new(tv))).unwrap();
        assert!(yaml.contains("!T"));
        assert!(yaml.contains("x"));
    }

    #[test]
    fn emit_empty_sequence_and_mapping() {
        assert_eq!(
            to_string(&Value::Sequence(Vec::new())).unwrap().trim(),
            "[]"
        );
        assert_eq!(
            to_string(&Value::Mapping(Mapping::new())).unwrap().trim(),
            "{}"
        );
    }

    #[test]
    fn emit_string_with_control_chars_uses_double_quotes() {
        let s = "line1\nline2\tend\0and\"quote\\back";
        let yaml = to_string(&Value::String(s.to_string())).unwrap();
        assert!(yaml.starts_with('"'));
        assert!(yaml.contains("\\n"));
        assert!(yaml.contains("\\t"));
        assert!(yaml.contains("\\0"));
        assert!(yaml.contains("\\\""));
        assert!(yaml.contains("\\\\"));
    }

    #[test]
    fn emit_string_with_unicode_line_separators() {
        let s = "a\u{2028}b\u{2029}c\u{0085}d\u{00A0}e";
        let yaml = to_string(&Value::String(s.to_string())).unwrap();
        assert!(yaml.contains("\\L"));
        assert!(yaml.contains("\\P"));
        assert!(yaml.contains("\\N"));
        assert!(yaml.contains("\\_"));
    }

    #[test]
    fn emit_string_with_rare_control_chars_use_hex_escape() {
        // Characters that fall into the fallback is_control
        // branch: \x01, \x0B, \x0C, \x07, \x1B are handled by
        // named escapes, but e.g. \x06 needs \x06 hex form.
        let s = "\x06";
        let yaml = to_string(&Value::String(s.to_string())).unwrap();
        assert!(yaml.contains("\\x06"));
    }

    #[test]
    fn emit_string_with_named_control_chars() {
        // \x07 BEL, \x08 BS, \x0B VT, \x0C FF, \x1B ESC
        for (raw, escape) in [
            ('\x07', "\\a"),
            ('\x08', "\\b"),
            ('\x0B', "\\v"),
            ('\x0C', "\\f"),
            ('\x1B', "\\e"),
            ('\r', "\\r"),
        ] {
            let s = raw.to_string();
            let yaml = to_string(&Value::String(s)).unwrap();
            assert!(
                yaml.contains(escape),
                "expected {escape} in {yaml:?}"
            );
        }
    }

    #[test]
    fn emit_empty_string_as_single_quoted() {
        let yaml = to_string(&Value::String(String::new())).unwrap();
        assert!(yaml.trim().starts_with("''"));
    }

    #[test]
    fn emit_string_with_single_quote_gets_escaped() {
        // Leading `!` forces quoting but not double-quoting, so
        // the single-quote escape branch runs.
        let yaml =
            to_string(&Value::String("!it's fine".into())).unwrap();
        assert!(yaml.starts_with('\''));
        assert!(yaml.contains("it''s"));
    }

    #[test]
    fn needs_quoting_detects_special_starts() {
        // Starts with special chars.
        for s in [
            "{x", "}x", "[x", "]x", ",x", "&x", "*x", "!x", "|x", ">x",
            "%x", "@x", "`x", "'x", "\"x",
        ] {
            assert!(needs_quoting(s), "expected {s:?} to need quoting");
        }
    }

    #[test]
    fn needs_quoting_detects_special_substrings_and_prefixes() {
        assert!(needs_quoting("key: value"));
        assert!(needs_quoting("a #comment"));
        assert!(needs_quoting("line\nbreak"));
        assert!(needs_quoting("- item"));
        assert!(needs_quoting("? mark"));
    }

    #[test]
    fn needs_quoting_detects_reserved_keywords_and_numbers() {
        for s in [
            "null", "NULL", "~", "true", "False", ".nan", ".inf",
            "-.inf",
        ] {
            assert!(needs_quoting(s), "{s} should require quoting");
        }
        assert!(needs_quoting("42"));
        assert!(needs_quoting("3.14"));
    }

    #[test]
    fn emit_flow_sequence_inside_mapping() {
        // A mapping whose value is a sequence that gets inlined
        // via the flow path: force inline=true by nesting inside
        // a one-element sequence value of a tagged node.
        let seq = Value::Sequence(vec![
            Value::Number(1.into()),
            Value::Number(2.into()),
        ]);
        let tv = Value::Tagged(Box::new(TaggedValue {
            tag: Tag::new("!Flow"),
            value: seq,
        }));
        // Wrap in an outer seq so tagged value serialization
        // emits inline flow form.
        let outer = Value::Sequence(vec![tv]);
        let yaml = to_string(&outer).unwrap();
        // The inner sequence is emitted either inline or block,
        // but the tag marker should appear.
        assert!(yaml.contains("!Flow"));
    }

    #[test]
    fn emit_flow_mapping_inside_sequence() {
        let mut inner = Mapping::new();
        inner.insert(
            Value::String("k".into()),
            Value::String("v".into()),
        );
        let outer = Value::Sequence(vec![Value::Tagged(Box::new(
            TaggedValue {
                tag: Tag::new("!Map"),
                value: Value::Mapping(inner),
            },
        ))]);
        let yaml = to_string(&outer).unwrap();
        assert!(yaml.contains("!Map"));
        assert!(yaml.contains('k'));
    }

    #[test]
    fn emit_nested_sequence_in_sequence() {
        let inner = Value::Sequence(vec![
            Value::Number(1.into()),
            Value::Number(2.into()),
        ]);
        let outer = Value::Sequence(vec![inner]);
        let yaml = to_string(&outer).unwrap();
        assert!(yaml.contains('-'));
    }

    #[test]
    fn to_writer_writes_bytes() {
        let mut buf = Vec::new();
        to_writer(&mut buf, &Value::String("hello".into())).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("hello"));
    }

    #[test]
    fn emit_flow_sequence_direct() {
        // The `Value::Tagged` serialize path converts to a
        // mapping, so the emit_value flow branches aren't
        // reachable from the public API with the current
        // Serialize impl. Call the private helpers directly.
        let seq = vec![
            Value::Number(1.into()),
            Value::Number(2.into()),
            Value::Number(3.into()),
        ];
        let mut out = String::new();
        emit_flow_sequence(&seq, &mut out);
        assert_eq!(out, "[1, 2, 3]");
    }

    #[test]
    fn emit_flow_mapping_direct() {
        let mut m = Mapping::new();
        m.insert(Value::String("a".into()), Value::Number(1.into()));
        m.insert(Value::String("b".into()), Value::Number(2.into()));
        let mut out = String::new();
        emit_flow_mapping(&m, &mut out);
        assert!(out.starts_with('{'));
        assert!(out.ends_with('}'));
        assert!(out.contains("a: 1"));
        assert!(out.contains(", b: 2"));
    }

    #[test]
    fn emit_value_with_tagged_branch_direct() {
        // emit_value is private; call it directly to exercise
        // the `Value::Tagged` arm (lines 87-90). The public
        // Serialize for TaggedValue reroutes through a mapping,
        // so this branch is otherwise dead code on top-level
        // serialization.
        let tv = Value::Tagged(Box::new(TaggedValue {
            tag: Tag::new("!X"),
            value: Value::String("plain".into()),
        }));
        let mut out = String::new();
        emit_value(&tv, &mut out, 0, false);
        assert!(out.starts_with("!X "));
        assert!(out.contains("plain"));
    }

    #[test]
    fn emit_value_inline_non_empty_sequence_uses_flow() {
        // Also private-helper-only path: Value::Sequence with
        // inline=true and non-empty content triggers
        // emit_flow_sequence via emit_value.
        let seq = Value::Sequence(vec![
            Value::Number(1.into()),
            Value::Number(2.into()),
        ]);
        let mut out = String::new();
        emit_value(&seq, &mut out, 0, true);
        assert_eq!(out, "[1, 2]");
    }

    #[test]
    fn needs_quoting_empty_string_returns_true() {
        assert!(needs_quoting(""));
    }

    #[test]
    fn to_writer_propagates_io_error() {
        struct Failing;
        impl Write for Failing {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("no"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let err =
            to_writer(Failing, &Value::String("x".into())).unwrap_err();
        assert!(format!("{err}").contains("no"));
    }
}
