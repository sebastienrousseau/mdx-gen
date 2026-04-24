// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! YAML deserialization: parser and `from_str`.

use crate::{
    error::Error,
    mapping::Mapping,
    number::Number,
    value::{tagged::TaggedValue, Value},
};
use serde::{
    de::{DeserializeOwned, MapAccess, SeqAccess, Visitor},
    forward_to_deserialize_any,
};
use std::collections::HashMap;
use std::io::Read;

type Result<T> = std::result::Result<T, Error>;

// ---- Public API ----

/// Deserialize an instance of type `T` from a YAML string.
///
/// Uses a streaming deserializer that drives the parser
/// directly via serde's visitor pattern, avoiding a full
/// intermediate `Value` tree for known types.
pub fn from_str<T>(s: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let mut parser = Parser::new(s);
    parser.skip_blanks_and_comments();
    // Skip document start marker
    if parser.rest().starts_with("---") {
        parser.advance_by(3);
        let rest_of_line = parser.current_line();
        if rest_of_line.trim().is_empty() {
            parser.skip_to_eol();
        } else {
            parser.skip_inline_spaces();
        }
    }
    let deser = StreamDeserializer::new(&mut parser, 0);
    T::deserialize(deser)
}

/// Deserialize from a byte slice.
pub fn from_slice<T>(v: &[u8]) -> Result<T>
where
    T: DeserializeOwned,
{
    let s = std::str::from_utf8(v)
        .map_err(|e| Error::msg(e.to_string()))?;
    from_str(s)
}

/// Deserialize from a reader.
pub fn from_reader<R, T>(mut rdr: R) -> Result<T>
where
    R: Read,
    T: DeserializeOwned,
{
    let mut s = String::new();
    rdr.read_to_string(&mut s)
        .map_err(|e| Error::msg(e.to_string()))?;
    from_str(&s)
}

/// A YAML deserializer supporting multi-document streams.
///
/// Use [`Deserializer::from_str`] to create an instance, then
/// iterate over documents with [`into_iter`](Deserializer::into_iter)
/// or call [`next_document`](Deserializer::next_document).
#[derive(Debug)]
pub struct Deserializer<'a> {
    input: &'a str,
    pos: usize,
    finished: bool,
}

impl<'a> Deserializer<'a> {
    /// Creates a new `Deserializer` from a YAML string.
    pub fn new(input: &'a str) -> Self {
        Deserializer {
            input,
            pos: 0,
            finished: false,
        }
    }

    /// Creates a new `Deserializer` from a YAML string.
    ///
    /// Alias for [`new`](Deserializer::new) for API
    /// compatibility.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(input: &'a str) -> Self {
        Self::new(input)
    }

    /// Deserializes the next document in the stream, or
    /// `None` if no documents remain.
    pub fn next_document<T>(&mut self) -> Option<Result<T>>
    where
        T: DeserializeOwned,
    {
        if self.finished {
            return None;
        }
        let remaining = &self.input[self.pos..];
        if remaining.trim().is_empty() {
            self.finished = true;
            return None;
        }
        match parse_one_document(remaining) {
            Ok((value, consumed)) => {
                self.pos += consumed;
                Some(T::deserialize(value))
            }
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}

/// Iterator over YAML documents in a stream.
#[derive(Debug)]
pub struct DocumentIter<'a> {
    deser: Deserializer<'a>,
}

impl<'a> IntoIterator for Deserializer<'a> {
    type Item = Result<Value>;
    type IntoIter = DocumentIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        DocumentIter { deser: self }
    }
}

impl Iterator for DocumentIter<'_> {
    type Item = Result<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        self.deser.next_document()
    }
}

// ---- YAML Parser ----

/// Parse one YAML document from the input, returning the
/// parsed value and the number of bytes consumed.
fn parse_one_document(input: &str) -> Result<(Value, usize)> {
    let mut parser = Parser::new(input);
    parser.skip_blanks_and_comments();

    if parser.is_eof() {
        return Ok((Value::Null, parser.pos));
    }

    // Skip document start marker
    if parser.rest().starts_with("---") {
        parser.advance_by(3);
        // If "---" is followed by content on same line
        // (not just whitespace/newline), skip to next line
        let rest_of_line = parser.current_line();
        if rest_of_line.trim().is_empty() {
            parser.skip_to_eol();
        } else {
            // Content after --- on same line (e.g., "--- tag")
            parser.skip_inline_spaces();
        }
    }

    let value = parser.parse_value(0)?;

    // Skip trailing whitespace and document end marker
    parser.skip_blanks_and_comments();
    if parser.rest().starts_with("...") {
        parser.advance_by(3);
        parser.skip_blanks_and_comments();
    }
    // If the next thing is "---", don't consume it —
    // leave it for the next document
    if !parser.rest().starts_with("---") {
        parser.skip_blanks_and_comments();
    }

    Ok((value, parser.pos))
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
    anchors: HashMap<String, Value>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser {
            input,
            pos: 0,
            anchors: HashMap::new(),
        }
    }

    fn rest(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn location(&self) -> crate::error::Location {
        let consumed = &self.input[..self.pos];
        let line = consumed.chars().filter(|&c| c == '\n').count() + 1;
        let column = match consumed.rfind('\n') {
            Some(nl) => self.pos - nl,
            None => self.pos + 1,
        };
        crate::error::Location::new(self.pos, line, column)
    }

    fn error(&self, msg: impl std::fmt::Display) -> Error {
        Error::msg_at(msg, self.location())
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn advance_by(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.input.len());
    }

    fn skip_to_eol(&mut self) {
        if let Some(idx) = self.rest().find('\n') {
            self.advance_by(idx + 1);
        } else {
            self.pos = self.input.len();
        }
    }

    fn skip_blanks_and_comments(&mut self) {
        loop {
            // Skip newlines at current position
            while self.peek() == Some('\n') || self.peek() == Some('\r')
            {
                if self.peek() == Some('\r') {
                    self.advance_by(1);
                }
                if self.peek() == Some('\n') {
                    self.advance_by(1);
                }
            }
            if self.is_eof() {
                break;
            }
            // Look at this line: if it's blank or a
            // comment, skip it; otherwise stop and
            // preserve its leading spaces for indent
            // detection
            let rest = self.rest();
            let line_end = rest.find('\n').unwrap_or(rest.len());
            let line = &rest[..line_end];
            let trimmed = line.trim_start();
            if trimmed.is_empty() {
                self.advance_by(line_end);
            } else if trimmed.starts_with('#') {
                self.skip_to_eol();
            } else {
                break;
            }
        }
    }

    fn peek_line_indent(&self) -> usize {
        let rest = self.rest();
        rest.len() - rest.trim_start_matches(' ').len()
    }

    fn current_line(&self) -> &'a str {
        let rest = self.rest();
        let end = rest.find('\n').unwrap_or(rest.len());
        &rest[..end]
    }

    fn at_document_marker(&self) -> bool {
        let rest = self.rest();
        let trimmed = rest.trim_start_matches(' ');
        // Document markers must be at column 0
        let indent = rest.len() - trimmed.len();
        if indent != 0 {
            return false;
        }
        (trimmed.starts_with("---") || trimmed.starts_with("..."))
            && (trimmed.len() == 3
                || trimmed.as_bytes().get(3) == Some(&b' ')
                || trimmed.as_bytes().get(3) == Some(&b'\n')
                || trimmed.as_bytes().get(3) == Some(&b'\r')
                || trimmed.as_bytes().get(3) == Some(&b'\t'))
    }

    fn parse_value(&mut self, min_indent: usize) -> Result<Value> {
        self.skip_blanks_and_comments();
        if self.is_eof() {
            return Ok(Value::Null);
        }

        // Stop at document markers (--- or ...)
        if self.at_document_marker() {
            return Ok(Value::Null);
        }

        let indent = self.peek_line_indent();
        if indent < min_indent {
            return Ok(Value::Null);
        }

        // Move past leading spaces
        let rest = self.rest().trim_start_matches(' ');
        let first_char = match rest.chars().next() {
            Some(c) => c,
            None => return Ok(Value::Null),
        };

        // Handle alias (*name)
        if first_char == '*' {
            self.advance_to_content();
            return self.parse_alias();
        }

        // Handle anchor (&name) — parse the anchor name,
        // then parse the attached value and store it
        let anchor_name = if first_char == '&' {
            self.advance_to_content();
            Some(self.parse_anchor_name()?)
        } else {
            None
        };

        // Re-evaluate first char after anchor
        let rest = self.rest().trim_start_matches(' ');
        let first_char = match rest.chars().next() {
            Some(c) => c,
            None => {
                if let Some(name) = anchor_name {
                    self.anchors.insert(name, Value::Null);
                }
                return Ok(Value::Null);
            }
        };

        let indent = self.peek_line_indent();

        let value = match first_char {
            // Flow sequence
            '[' => {
                self.advance_to_content();
                self.parse_flow_sequence()?
            }
            // Flow mapping
            '{' => {
                self.advance_to_content();
                self.parse_flow_mapping()?
            }
            // Block sequence
            '-' if self.is_sequence_dash(indent) => {
                self.parse_block_sequence(indent)?
            }
            // Tag
            '!' => {
                self.advance_to_content();
                self.parse_tagged_value(indent)?
            }
            // Quoted string
            '\'' | '"' => {
                self.advance_to_content();
                let s = self.parse_quoted_string(first_char)?;
                // Check if this is a mapping key
                self.skip_inline_spaces();
                if self.peek() == Some(':') && self.is_mapping_colon() {
                    self.parse_mapping_from_first_key(
                        Value::String(s),
                        indent,
                    )?
                } else {
                    Value::String(s)
                }
            }
            // Block scalar
            '|' | '>' => {
                self.advance_to_content();
                self.parse_block_scalar(first_char, indent)?
            }
            // Mapping or plain scalar
            _ => {
                if self.line_has_mapping_colon(indent) {
                    self.parse_block_mapping(indent)?
                } else {
                    self.advance_to_content();
                    self.parse_plain_scalar()?
                }
            }
        };

        if let Some(name) = anchor_name {
            self.anchors.insert(name, value.clone());
        }

        Ok(value)
    }

    fn parse_anchor_name(&mut self) -> Result<String> {
        // Skip '&'
        self.advance_by(1);
        let rest = self.rest();
        let end = rest
            .find(|c: char| {
                c == ' '
                    || c == '\t'
                    || c == '\n'
                    || c == '\r'
                    || c == ','
                    || c == ']'
                    || c == '}'
                    || c == ':'
            })
            .unwrap_or(rest.len());
        if end == 0 {
            return Err(self.error("empty anchor name"));
        }
        let name = rest[..end].to_owned();
        self.advance_by(end);
        self.skip_inline_spaces();
        Ok(name)
    }

    fn parse_alias(&mut self) -> Result<Value> {
        // Skip '*'
        self.advance_by(1);
        let rest = self.rest();
        let end = rest
            .find(|c: char| {
                c == ' '
                    || c == '\t'
                    || c == '\n'
                    || c == '\r'
                    || c == ','
                    || c == ']'
                    || c == '}'
                    || c == ':'
            })
            .unwrap_or(rest.len());
        if end == 0 {
            return Err(self.error("empty alias name"));
        }
        let name = &rest[..end];
        let value =
            self.anchors.get(name).cloned().ok_or_else(|| {
                self.error(format!("unknown alias: *{}", name))
            })?;
        self.advance_by(end);
        self.skip_inline_spaces();
        // Skip past rest of line if at newline
        if self.peek() == Some('\n') || self.peek() == Some('\r') {
            // don't consume — let parent handle
        }
        Ok(value)
    }

    fn advance_to_content(&mut self) {
        let indent = self.peek_line_indent();
        self.advance_by(indent);
    }

    fn skip_inline_spaces(&mut self) {
        while self.peek() == Some(' ') || self.peek() == Some('\t') {
            self.advance_by(1);
        }
    }

    fn is_sequence_dash(&self, expected_indent: usize) -> bool {
        let rest = self.rest();
        let indent = rest.len() - rest.trim_start_matches(' ').len();
        if indent != expected_indent {
            return false;
        }
        let trimmed = rest.trim_start_matches(' ');
        trimmed.starts_with("- ")
            || trimmed == "-"
            || trimmed.starts_with("-\n")
            || trimmed.starts_with("-\r")
    }

    fn is_mapping_colon(&self) -> bool {
        let rest = self.rest();
        rest.starts_with(": ")
            || rest == ":"
            || rest.starts_with(":\n")
            || rest.starts_with(":\r")
    }

    fn line_has_mapping_colon(&self, expected_indent: usize) -> bool {
        let rest = self.rest();
        let indent = rest.len() - rest.trim_start_matches(' ').len();
        if indent != expected_indent {
            return false;
        }
        let trimmed = rest.trim_start_matches(' ');
        let line = match trimmed.find('\n') {
            Some(i) => &trimmed[..i],
            None => trimmed,
        };
        // Check for key: value pattern (not inside quotes)
        self.find_mapping_colon(line).is_some()
    }

    fn find_mapping_colon(&self, line: &str) -> Option<usize> {
        let mut in_single = false;
        let mut in_double = false;
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'\'' if !in_double => {
                    in_single = !in_single;
                }
                b'"' if !in_single => {
                    in_double = !in_double;
                }
                b':' if !in_single
                    && !in_double
                    && (i + 1 >= bytes.len()
                        || bytes[i + 1] == b' '
                        || bytes[i + 1] == b'\t') =>
                {
                    return Some(i);
                }
                b'#' if !in_single && !in_double => {
                    // Rest is comment
                    return None;
                }
                _ => {}
            }
            i += 1;
        }
        None
    }

    fn parse_block_mapping(&mut self, indent: usize) -> Result<Value> {
        let mut mapping = Mapping::new();
        loop {
            self.skip_blanks_and_comments();
            if self.is_eof() {
                break;
            }
            let cur_indent = self.peek_line_indent();
            if cur_indent != indent {
                break;
            }
            // Check for document end
            let rest_trimmed = self.rest().trim_start_matches(' ');
            if rest_trimmed.starts_with("---")
                || rest_trimmed.starts_with("...")
            {
                break;
            }

            self.advance_by(indent);
            let (key, value) = self.parse_mapping_entry(indent)?;
            mapping.insert(key, value);
        }
        Ok(Value::Mapping(mapping))
    }

    fn parse_mapping_entry(
        &mut self,
        indent: usize,
    ) -> Result<(Value, Value)> {
        let key = self.parse_mapping_key()?;
        // Skip ':'
        if self.peek() == Some(':') {
            self.advance_by(1);
        }
        self.skip_inline_spaces();

        let value = if self.peek() == Some('\n')
            || self.peek() == Some('\r')
            || self.peek() == Some('#')
            || self.is_eof()
        {
            // Skip comment
            if self.peek() == Some('#') {
                self.skip_to_eol();
            }
            // Value on next line(s)
            self.parse_value(indent + 1)?
        } else {
            self.parse_inline_value(indent)?
        };

        Ok((key, value))
    }

    fn parse_mapping_key(&mut self) -> Result<Value> {
        match self.peek() {
            Some('\'') | Some('"') => {
                let q = self
                    .peek()
                    .ok_or_else(|| self.error("unexpected EOF"))?;
                let s = self.parse_quoted_string(q)?;
                Ok(Value::String(s))
            }
            _ => {
                // Read until ':'
                let rest = self.rest();
                let line = match rest.find('\n') {
                    Some(i) => &rest[..i],
                    None => rest,
                };
                let colon_pos =
                    self.find_mapping_colon(line).ok_or_else(|| {
                        self.error(format!(
                            "expected ':' in mapping, \
                             got: {:?}",
                            &line[..line.len().min(40)]
                        ))
                    })?;
                let key_str = rest[..colon_pos].trim_end();
                let key = interpret_scalar(key_str);
                self.advance_by(colon_pos);
                Ok(key)
            }
        }
    }

    fn parse_mapping_from_first_key(
        &mut self,
        first_key: Value,
        indent: usize,
    ) -> Result<Value> {
        // We already have the key, now skip ':'
        if self.peek() == Some(':') {
            self.advance_by(1);
        }
        self.skip_inline_spaces();

        let first_value = if self.peek() == Some('\n')
            || self.peek() == Some('\r')
            || self.peek() == Some('#')
            || self.is_eof()
        {
            if self.peek() == Some('#') {
                self.skip_to_eol();
            }
            self.parse_value(indent + 1)?
        } else {
            self.parse_inline_value(indent)?
        };

        let mut mapping = Mapping::new();
        mapping.insert(first_key, first_value);

        // Continue parsing remaining entries
        loop {
            self.skip_blanks_and_comments();
            if self.is_eof() {
                break;
            }
            let cur_indent = self.peek_line_indent();
            if cur_indent != indent {
                break;
            }
            let rest_trimmed = self.rest().trim_start_matches(' ');
            if rest_trimmed.starts_with("---")
                || rest_trimmed.starts_with("...")
            {
                break;
            }
            self.advance_by(indent);
            let (key, value) = self.parse_mapping_entry(indent)?;
            mapping.insert(key, value);
        }
        Ok(Value::Mapping(mapping))
    }

    fn parse_inline_value(
        &mut self,
        parent_indent: usize,
    ) -> Result<Value> {
        match self.peek() {
            Some('[') => self.parse_flow_sequence(),
            Some('{') => self.parse_flow_mapping(),
            Some('*') => self.parse_alias(),
            Some('&') => {
                let name = self.parse_anchor_name()?;
                // If the value is on the next line, delegate
                // to parse_value for block content
                let value = if self.peek() == Some('\n')
                    || self.peek() == Some('\r')
                    || self.peek() == Some('#')
                    || self.is_eof()
                {
                    if self.peek() == Some('#') {
                        self.skip_to_eol();
                    }
                    self.parse_value(parent_indent + 1)?
                } else {
                    self.parse_inline_value(parent_indent)?
                };
                self.anchors.insert(name, value.clone());
                Ok(value)
            }
            Some('\'') | Some('"') => {
                let q = self
                    .peek()
                    .ok_or_else(|| self.error("unexpected EOF"))?;
                let s = self.parse_quoted_string(q)?;
                Ok(Value::String(s))
            }
            Some('|') | Some('>') => {
                let ch = self
                    .peek()
                    .ok_or_else(|| self.error("unexpected EOF"))?;
                self.parse_block_scalar(ch, parent_indent)
            }
            Some('!') => self.parse_tagged_value(parent_indent),
            _ => self.parse_plain_scalar(),
        }
    }

    fn parse_block_sequence(&mut self, indent: usize) -> Result<Value> {
        let mut items = Vec::new();
        loop {
            self.skip_blanks_and_comments();
            if self.is_eof() {
                break;
            }
            let cur_indent = self.peek_line_indent();
            if cur_indent != indent {
                break;
            }
            let rest_trimmed = self.rest().trim_start_matches(' ');
            if !rest_trimmed.starts_with("- ")
                && rest_trimmed != "-"
                && !rest_trimmed.starts_with("-\n")
                && !rest_trimmed.starts_with("-\r")
            {
                break;
            }
            // Skip indent + "- "
            self.advance_by(indent);
            self.advance_by(1); // '-'
            if self.peek() == Some(' ') {
                self.advance_by(1);
            }

            self.skip_inline_spaces();
            // Check if rest of line is empty (value on
            // next line)
            if self.peek() == Some('\n')
                || self.peek() == Some('\r')
                || self.peek() == Some('#')
                || self.is_eof()
            {
                if self.peek() == Some('#') {
                    self.skip_to_eol();
                }
                let item = self.parse_value(indent + 2)?;
                items.push(item);
            } else {
                // Inline value after "- "
                // Check if it's a mapping
                let line = self.current_line();
                if self.find_mapping_colon(line).is_some() {
                    // It's a mapping starting on this
                    // line. The indent for this mapping
                    // is indent + 2.
                    let item_indent = indent + 2;
                    let (key, value) =
                        self.parse_mapping_entry(item_indent)?;
                    let mut m = Mapping::new();
                    m.insert(key, value);
                    // Continue reading more mapping
                    // entries at item_indent
                    loop {
                        self.skip_blanks_and_comments();
                        if self.is_eof() {
                            break;
                        }
                        let ci = self.peek_line_indent();
                        if ci != item_indent {
                            break;
                        }
                        let rt = self.rest().trim_start_matches(' ');
                        if self
                            .find_mapping_colon(match rt.find('\n') {
                                Some(i) => &rt[..i],
                                None => rt,
                            })
                            .is_none()
                        {
                            break;
                        }
                        self.advance_by(item_indent);
                        let (k, v) =
                            self.parse_mapping_entry(item_indent)?;
                        m.insert(k, v);
                    }
                    items.push(Value::Mapping(m));
                } else {
                    let item = self.parse_inline_value(indent)?;
                    items.push(item);
                }
            }
        }
        Ok(Value::Sequence(items))
    }

    fn parse_flow_sequence(&mut self) -> Result<Value> {
        // Skip '['
        self.advance_by(1);
        let mut items = Vec::new();
        loop {
            self.skip_flow_whitespace();
            match self.peek() {
                None => {
                    return Err(self.error("unterminated flow sequence"))
                }
                Some(']') => {
                    self.advance_by(1);
                    break;
                }
                Some(',') => {
                    self.advance_by(1);
                    continue;
                }
                _ => {}
            }
            let item = self.parse_flow_value()?;
            items.push(item);
        }
        Ok(Value::Sequence(items))
    }

    fn parse_flow_mapping(&mut self) -> Result<Value> {
        // Skip '{'
        self.advance_by(1);
        let mut mapping = Mapping::new();
        loop {
            self.skip_flow_whitespace();
            match self.peek() {
                None => {
                    return Err(self.error("unterminated flow mapping"))
                }
                Some('}') => {
                    self.advance_by(1);
                    break;
                }
                Some(',') => {
                    self.advance_by(1);
                    continue;
                }
                _ => {}
            }
            // Parse key
            let key = self.parse_flow_value()?;
            self.skip_flow_whitespace();
            if self.peek() != Some(':') {
                return Err(self.error("expected ':' in flow mapping"));
            }
            self.advance_by(1);
            self.skip_flow_whitespace();
            // Parse value
            let value = self.parse_flow_value()?;
            mapping.insert(key, value);
        }
        Ok(Value::Mapping(mapping))
    }

    fn parse_flow_value(&mut self) -> Result<Value> {
        self.skip_flow_whitespace();
        match self.peek() {
            Some('[') => self.parse_flow_sequence(),
            Some('{') => self.parse_flow_mapping(),
            Some('*') => self.parse_alias(),
            Some('&') => {
                let name = self.parse_anchor_name()?;
                let value = self.parse_flow_value()?;
                self.anchors.insert(name, value.clone());
                Ok(value)
            }
            Some('\'') | Some('"') => {
                let q = self
                    .peek()
                    .ok_or_else(|| self.error("unexpected EOF"))?;
                let s = self.parse_quoted_string(q)?;
                Ok(Value::String(s))
            }
            _ => {
                // Read until , ] } : or newline
                let rest = self.rest();
                let mut end = rest.len();
                for (i, ch) in rest.char_indices() {
                    if ch == ',' || ch == ']' || ch == '}' || ch == ':'
                    {
                        end = i;
                        break;
                    }
                }
                let token = rest[..end].trim_end();
                if token.is_empty() {
                    return Ok(Value::Null);
                }
                let value = interpret_scalar(token);
                self.advance_by(end);
                Ok(value)
            }
        }
    }

    fn skip_flow_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
                self.advance_by(ch.len_utf8());
            } else if ch == '#' {
                self.skip_to_eol();
            } else {
                break;
            }
        }
    }

    fn parse_quoted_string(&mut self, quote: char) -> Result<String> {
        // Skip opening quote
        self.advance_by(1);
        let mut result = String::new();
        loop {
            match self.peek() {
                None => {
                    return Err(self.error("unterminated quoted string"))
                }
                Some(ch) if ch == quote => {
                    self.advance_by(1);
                    // For single quotes, check for ''
                    if quote == '\'' && self.peek() == Some('\'') {
                        result.push('\'');
                        self.advance_by(1);
                        continue;
                    }
                    break;
                }
                Some('\\') if quote == '"' => {
                    self.advance_by(1);
                    match self.peek() {
                        Some('0') => {
                            result.push('\0');
                            self.advance_by(1);
                        }
                        Some('a') => {
                            result.push('\x07');
                            self.advance_by(1);
                        }
                        Some('b') => {
                            result.push('\x08');
                            self.advance_by(1);
                        }
                        Some('t') | Some('\t') => {
                            result.push('\t');
                            self.advance_by(1);
                        }
                        Some('n') => {
                            result.push('\n');
                            self.advance_by(1);
                        }
                        Some('v') => {
                            result.push('\x0B');
                            self.advance_by(1);
                        }
                        Some('f') => {
                            result.push('\x0C');
                            self.advance_by(1);
                        }
                        Some('r') => {
                            result.push('\r');
                            self.advance_by(1);
                        }
                        Some('e') => {
                            result.push('\x1B');
                            self.advance_by(1);
                        }
                        Some(' ') => {
                            result.push(' ');
                            self.advance_by(1);
                        }
                        Some('"') => {
                            result.push('"');
                            self.advance_by(1);
                        }
                        Some('/') => {
                            result.push('/');
                            self.advance_by(1);
                        }
                        Some('\\') => {
                            result.push('\\');
                            self.advance_by(1);
                        }
                        Some('N') => {
                            // Next line (U+0085)
                            result.push('\u{0085}');
                            self.advance_by(1);
                        }
                        Some('_') => {
                            // Non-breaking space (U+00A0)
                            result.push('\u{00A0}');
                            self.advance_by(1);
                        }
                        Some('L') => {
                            // Line separator (U+2028)
                            result.push('\u{2028}');
                            self.advance_by(1);
                        }
                        Some('P') => {
                            // Paragraph separator (U+2029)
                            result.push('\u{2029}');
                            self.advance_by(1);
                        }
                        Some('x') => {
                            self.advance_by(1);
                            let ch = self.parse_hex_escape(2)?;
                            result.push(ch);
                        }
                        Some('u') => {
                            self.advance_by(1);
                            let ch = self.parse_hex_escape(4)?;
                            result.push(ch);
                        }
                        Some('U') => {
                            self.advance_by(1);
                            let ch = self.parse_hex_escape(8)?;
                            result.push(ch);
                        }
                        Some('\n') | Some('\r') => {
                            // Escaped line break — skip the
                            // break and all trailing whitespace
                            // (line folding escape)
                            self.skip_line_break();
                            while self.peek() == Some(' ')
                                || self.peek() == Some('\t')
                            {
                                self.advance_by(1);
                            }
                        }
                        Some(c) => {
                            result.push('\\');
                            result.push(c);
                            self.advance_by(c.len_utf8());
                        }
                        None => {
                            result.push('\\');
                        }
                    }
                }
                // Multi-line double-quoted: fold line breaks
                Some('\n') | Some('\r') if quote == '"' => {
                    self.skip_line_break();
                    // Trim trailing whitespace from current
                    // content before the fold
                    let trimmed_len = result.trim_end().len();
                    result.truncate(trimmed_len);
                    // Count consecutive empty lines
                    let mut empty_lines = 0;
                    loop {
                        // Skip leading whitespace on new line
                        while self.peek() == Some(' ')
                            || self.peek() == Some('\t')
                        {
                            self.advance_by(1);
                        }
                        if self.peek() == Some('\n')
                            || self.peek() == Some('\r')
                        {
                            empty_lines += 1;
                            self.skip_line_break();
                        } else {
                            break;
                        }
                    }
                    if empty_lines > 0 {
                        for _ in 0..empty_lines {
                            result.push('\n');
                        }
                    } else {
                        result.push(' ');
                    }
                }
                Some(ch) => {
                    result.push(ch);
                    self.advance_by(ch.len_utf8());
                }
            }
        }
        Ok(result)
    }

    fn parse_hex_escape(&mut self, digits: usize) -> Result<char> {
        let rest = self.rest();
        if rest.len() < digits {
            return Err(self.error(format!(
                "expected {} hex digits in escape, got EOF",
                digits
            )));
        }
        let hex = &rest[..digits];
        let code = u32::from_str_radix(hex, 16).map_err(|_| {
            self.error(format!("invalid hex escape: {:?}", hex))
        })?;
        let ch = char::from_u32(code).ok_or_else(|| {
            self.error(format!(
                "invalid Unicode code point: U+{:04X}",
                code
            ))
        })?;
        self.advance_by(digits);
        Ok(ch)
    }

    fn skip_line_break(&mut self) {
        if self.peek() == Some('\r') {
            self.advance_by(1);
        }
        if self.peek() == Some('\n') {
            self.advance_by(1);
        }
    }

    fn parse_block_scalar(
        &mut self,
        style: char,
        _parent_indent: usize,
    ) -> Result<Value> {
        // Skip '|' or '>'
        self.advance_by(1);

        // Parse optional indentation indicator and
        // chomping indicator in either order:
        // |2, |+, |2+, |+2, >-, >1-, >-1, etc.
        let mut chomp = None;
        let mut explicit_indent: Option<usize> = None;

        for _ in 0..2 {
            match self.peek() {
                Some('-') if chomp.is_none() => {
                    chomp = Some(Chomp::Strip);
                    self.advance_by(1);
                }
                Some('+') if chomp.is_none() => {
                    chomp = Some(Chomp::Keep);
                    self.advance_by(1);
                }
                Some(c @ '1'..='9') if explicit_indent.is_none() => {
                    explicit_indent = Some((c as u8 - b'0') as usize);
                    self.advance_by(1);
                }
                _ => break,
            }
        }

        let chomp = chomp.unwrap_or(Chomp::Clip);

        // Skip rest of indicator line
        self.skip_to_eol();

        // Determine content indent: use explicit if
        // given, otherwise detect from first non-empty
        // line
        let content_indent = if let Some(ei) = explicit_indent {
            // Explicit indent is relative — find the
            // parent block's indent from the current
            // position's first non-empty line
            let base = {
                let mut look = self.pos;
                loop {
                    if look >= self.input.len() {
                        break 0;
                    }
                    let rem = &self.input[look..];
                    let line = match rem.find('\n') {
                        Some(i) => &rem[..i],
                        None => rem,
                    };
                    if line.trim().is_empty() {
                        look += line.len() + 1;
                        continue;
                    }
                    let detected =
                        line.len() - line.trim_start_matches(' ').len();
                    break detected;
                }
            };
            // If a first content line exists, use that;
            // otherwise use explicit as absolute
            if base > 0 {
                base
            } else {
                ei
            }
        } else {
            let mut look = self.pos;
            loop {
                if look >= self.input.len() {
                    break 0;
                }
                let rem = &self.input[look..];
                let line = match rem.find('\n') {
                    Some(i) => &rem[..i],
                    None => rem,
                };
                if line.trim().is_empty() {
                    look += line.len() + 1;
                    continue;
                }
                break line.len() - line.trim_start_matches(' ').len();
            }
        };

        if content_indent == 0 {
            return Ok(Value::String(String::new()));
        }

        let mut lines = Vec::new();
        loop {
            if self.is_eof() {
                break;
            }
            let line_rest = self.rest();
            let line = match line_rest.find('\n') {
                Some(i) => &line_rest[..i],
                None => line_rest,
            };

            if line.trim().is_empty() {
                lines.push(String::new());
                self.advance_by(line.len());
                if self.peek() == Some('\n') {
                    self.advance_by(1);
                }
                continue;
            }

            let line_indent =
                line.len() - line.trim_start_matches(' ').len();
            if line_indent < content_indent {
                break;
            }

            lines.push(line[content_indent..].to_owned());
            self.advance_by(line.len());
            if self.peek() == Some('\n') {
                self.advance_by(1);
            }
        }

        // Remove trailing empty lines for processing
        let trailing_empties =
            lines.iter().rev().take_while(|l| l.is_empty()).count();

        let content = if style == '|' {
            // Literal: preserve newlines
            lines.join("\n")
        } else {
            // Folded: join with spaces (preserve double
            // newlines)
            let mut result = String::new();
            for (i, line) in lines.iter().enumerate() {
                if i > 0 {
                    if line.is_empty() || lines[i - 1].is_empty() {
                        result.push('\n');
                    } else {
                        result.push(' ');
                    }
                }
                result.push_str(line);
            }
            result
        };

        let content = match chomp {
            Chomp::Strip => content.trim_end_matches('\n').to_owned(),
            Chomp::Clip => {
                let trimmed = content.trim_end_matches('\n').to_owned();
                if trailing_empties > 0 || !content.ends_with('\n') {
                    trimmed + "\n"
                } else {
                    trimmed
                }
            }
            Chomp::Keep => content,
        };

        Ok(Value::String(content))
    }

    fn parse_plain_scalar(&mut self) -> Result<Value> {
        let rest = self.rest();
        let line = match rest.find('\n') {
            Some(i) => &rest[..i],
            None => rest,
        };
        // Strip inline comment
        let effective = strip_inline_comment(line).trim_end();
        if effective.is_empty() {
            self.skip_to_eol();
            return Ok(Value::Null);
        }
        let value = interpret_scalar(effective);
        self.advance_by(line.len());
        if self.peek() == Some('\n') {
            self.advance_by(1);
        }
        Ok(value)
    }

    fn parse_tagged_value(&mut self, indent: usize) -> Result<Value> {
        // Skip '!'
        self.advance_by(1);
        // Read tag name
        let rest = self.rest();
        let end = rest.find([' ', '\n', '\r']).unwrap_or(rest.len());
        let tag_name = rest[..end].to_owned();
        self.advance_by(end);
        self.skip_inline_spaces();

        let tag =
            crate::value::tagged::Tag::new(format!("!{}", tag_name));

        // Parse the tagged value
        let value = if self.peek() == Some('\n')
            || self.peek() == Some('\r')
            || self.is_eof()
        {
            self.parse_value(indent + 1)?
        } else {
            self.parse_inline_value(indent)?
        };

        Ok(Value::Tagged(Box::new(TaggedValue { tag, value })))
    }
}

#[derive(Clone, Copy)]
enum Chomp {
    Strip,
    Clip,
    Keep,
}

fn strip_inline_comment(line: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\'' if !in_double => {
                in_single = !in_single;
            }
            b'"' if !in_single => {
                in_double = !in_double;
            }
            b' ' if !in_single
                && !in_double
                && i + 1 < bytes.len()
                && bytes[i + 1] == b'#' =>
            {
                return &line[..i];
            }
            _ => {}
        }
        i += 1;
    }
    line
}

/// Interpret a plain (unquoted) scalar string as the
/// appropriate YAML type.
fn interpret_scalar(s: &str) -> Value {
    match s {
        "" | "null" | "Null" | "NULL" | "~" => Value::Null,
        "true" | "True" | "TRUE" => Value::Bool(true),
        "false" | "False" | "FALSE" => Value::Bool(false),
        ".nan" | ".NaN" | ".NAN" => {
            Value::Number(Number::from(f64::NAN))
        }
        ".inf" | ".Inf" | ".INF" => {
            Value::Number(Number::from(f64::INFINITY))
        }
        "-.inf" | "-.Inf" | "-.INF" => {
            Value::Number(Number::from(f64::NEG_INFINITY))
        }
        _ => {
            // Try integer
            if let Some(n) = parse_integer(s) {
                return n;
            }
            // Try float
            if let Some(n) = parse_float(s) {
                return n;
            }
            Value::String(s.to_owned())
        }
    }
}

fn parse_integer(s: &str) -> Option<Value> {
    if s.starts_with("0x") || s.starts_with("0X") {
        u64::from_str_radix(&s[2..], 16)
            .ok()
            .map(|n| Value::Number(Number::from(n)))
    } else if s.starts_with("0o") || s.starts_with("0O") {
        u64::from_str_radix(&s[2..], 8)
            .ok()
            .map(|n| Value::Number(Number::from(n)))
    } else if s.starts_with('-') || s.starts_with('+') {
        s.parse::<i64>()
            .ok()
            .map(|n| Value::Number(Number::from(n)))
    } else {
        // Only parse as integer if it's all digits
        // (or digits with underscores)
        let clean = s.replace('_', "");
        if clean.chars().all(|c| c.is_ascii_digit()) {
            clean
                .parse::<u64>()
                .ok()
                .map(|n| Value::Number(Number::from(n)))
        } else {
            None
        }
    }
}

fn parse_float(s: &str) -> Option<Value> {
    // Must contain a '.' or 'e'/'E' to be a float
    if !s.contains('.') && !s.contains('e') && !s.contains('E') {
        return None;
    }
    let clean = s.replace('_', "");
    clean
        .parse::<f64>()
        .ok()
        .map(|f| Value::Number(Number::from(f)))
}

// ---- SeqDeserializer / MapDeserializer ----
// Used by Value's Deserializer impl.

pub(crate) struct SeqDeserializer {
    iter: std::vec::IntoIter<Value>,
}

impl SeqDeserializer {
    pub(crate) fn new(seq: Vec<Value>) -> Self {
        SeqDeserializer {
            iter: seq.into_iter(),
        }
    }
}

impl<'de> serde::Deserializer<'de> for SeqDeserializer {
    type Error = Error;

    fn deserialize_any<V>(
        self,
        visitor: V,
    ) -> std::result::Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64
        char str string bytes byte_buf option unit
        unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> std::result::Result<Option<T::Value>, Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }
}

pub(crate) struct MapDeserializer {
    iter: crate::mapping::IntoIter,
    value: Option<Value>,
}

impl MapDeserializer {
    pub(crate) fn new(mapping: Mapping) -> Self {
        MapDeserializer {
            iter: mapping.into_iter(),
            value: None,
        }
    }
}

impl<'de> serde::Deserializer<'de> for MapDeserializer {
    type Error = Error;

    fn deserialize_any<V>(
        self,
        visitor: V,
    ) -> std::result::Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64
        char str string bytes byte_buf option unit
        unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<K>(
        &mut self,
        seed: K,
    ) -> std::result::Result<Option<K::Value>, Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(key).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(
        &mut self,
        seed: V,
    ) -> std::result::Result<V::Value, Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(Error::msg("value called before key")),
        }
    }
}

// ---- Streaming Deserializer (Phase 3) ----
// Implements serde::Deserializer directly on the parser
// to avoid building an intermediate Value tree.

/// Streaming YAML deserializer that drives the parser
/// directly via serde's visitor pattern.
pub(crate) struct StreamDeserializer<'p, 'i> {
    parser: &'p mut Parser<'i>,
    min_indent: usize,
}

impl<'p, 'i> StreamDeserializer<'p, 'i> {
    fn new(parser: &'p mut Parser<'i>, min_indent: usize) -> Self {
        StreamDeserializer { parser, min_indent }
    }
}

impl<'de> serde::Deserializer<'de> for StreamDeserializer<'_, '_> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.parser.skip_blanks_and_comments();
        if self.parser.is_eof() || self.parser.at_document_marker() {
            return visitor.visit_unit();
        }

        let indent = self.parser.peek_line_indent();
        if indent < self.min_indent {
            return visitor.visit_unit();
        }

        let rest = self.parser.rest().trim_start_matches(' ');
        let first_char = match rest.chars().next() {
            Some(c) => c,
            None => return visitor.visit_unit(),
        };

        match first_char {
            // Alias
            '*' => {
                self.parser.advance_to_content();
                let v = self.parser.parse_alias()?;
                v.deserialize_any(visitor)
            }
            // Anchor — parse name, then stream the value
            '&' => {
                self.parser.advance_to_content();
                let name = self.parser.parse_anchor_name()?;
                // Fall back to Value path for anchored
                // values (need to store them)
                let mi = self.min_indent;
                let check_eol = self.parser.peek() == Some('\n')
                    || self.parser.peek() == Some('\r')
                    || self.parser.is_eof();
                let value = if check_eol {
                    self.parser.parse_value(mi)?
                } else {
                    self.parser.parse_inline_value(mi)?
                };
                self.parser.anchors.insert(name, value.clone());
                value.deserialize_any(visitor)
            }
            // Flow sequence
            '[' => {
                self.parser.advance_to_content();
                self.parser.advance_by(1); // skip '['
                visitor.visit_seq(StreamFlowSeqAccess {
                    parser: self.parser,
                    first: true,
                })
            }
            // Flow mapping
            '{' => {
                self.parser.advance_to_content();
                self.parser.advance_by(1); // skip '{'
                visitor.visit_map(StreamFlowMapAccess {
                    parser: self.parser,
                    first: true,
                    value: None,
                })
            }
            // Block sequence
            '-' if self.parser.is_sequence_dash(indent) => visitor
                .visit_seq(StreamBlockSeqAccess {
                    parser: self.parser,
                    indent,
                }),
            // Quoted string
            '\'' | '"' => {
                self.parser.advance_to_content();
                let s = self.parser.parse_quoted_string(first_char)?;
                // Check if this is a mapping key
                self.parser.skip_inline_spaces();
                if self.parser.peek() == Some(':')
                    && self.parser.is_mapping_colon()
                {
                    // It's a mapping — fall back to Value
                    let v = self.parser.parse_mapping_from_first_key(
                        Value::String(s),
                        indent,
                    )?;
                    v.deserialize_any(visitor)
                } else {
                    visitor.visit_string(s)
                }
            }
            // Block scalar
            '|' | '>' => {
                self.parser.advance_to_content();
                let v = self
                    .parser
                    .parse_block_scalar(first_char, indent)?;
                v.deserialize_any(visitor)
            }
            // Tag
            '!' => {
                self.parser.advance_to_content();
                let v = self.parser.parse_tagged_value(indent)?;
                v.deserialize_any(visitor)
            }
            // Mapping or plain scalar
            _ => {
                if self.parser.line_has_mapping_colon(indent) {
                    visitor.visit_map(StreamBlockMapAccess {
                        parser: self.parser,
                        indent,
                        value: None,
                    })
                } else {
                    self.parser.advance_to_content();
                    let v = self.parser.parse_plain_scalar()?;
                    v.deserialize_any(visitor)
                }
            }
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.parser.skip_blanks_and_comments();
        if self.parser.is_eof() {
            return visitor.visit_none();
        }
        let indent = self.parser.peek_line_indent();
        if indent < self.min_indent {
            return visitor.visit_none();
        }
        // Peek at scalar to check for null
        let rest = self.parser.rest().trim_start_matches(' ');
        let line = match rest.find('\n') {
            Some(i) => &rest[..i],
            None => rest,
        };
        let trimmed = line.trim();
        if matches!(trimmed, "" | "null" | "Null" | "NULL" | "~") {
            // Consume the null
            self.parser.advance_to_content();
            let _ = self.parser.parse_plain_scalar();
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Fall back to Value path for enums
        let value = self.parser.parse_value(self.min_indent)?;
        value.deserialize_enum(name, variants, visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64
        char str string bytes byte_buf unit unit_struct seq
        tuple tuple_struct map struct identifier ignored_any
    }
}

// ---- Block Mapping Stream Access ----

struct StreamBlockMapAccess<'p, 'i> {
    parser: &'p mut Parser<'i>,
    indent: usize,
    value: Option<Value>,
}

impl<'de> MapAccess<'de> for StreamBlockMapAccess<'_, '_> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        self.parser.skip_blanks_and_comments();
        if self.parser.is_eof() {
            return Ok(None);
        }
        let cur_indent = self.parser.peek_line_indent();
        if cur_indent != self.indent {
            return Ok(None);
        }
        let rest = self.parser.rest().trim_start_matches(' ');
        if rest.starts_with("---") || rest.starts_with("...") {
            return Ok(None);
        }
        self.parser.advance_by(self.indent);
        let key = self.parser.parse_mapping_key()?;
        // Skip ':'
        if self.parser.peek() == Some(':') {
            self.parser.advance_by(1);
        }
        self.parser.skip_inline_spaces();

        // Parse value
        let value = if self.parser.peek() == Some('\n')
            || self.parser.peek() == Some('\r')
            || self.parser.peek() == Some('#')
            || self.parser.is_eof()
        {
            if self.parser.peek() == Some('#') {
                self.parser.skip_to_eol();
            }
            self.parser.parse_value(self.indent + 1)?
        } else {
            self.parser.parse_inline_value(self.indent)?
        };

        self.value = Some(value);
        seed.deserialize(key).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(v) => seed.deserialize(v),
            None => Err(Error::msg("value before key")),
        }
    }
}

// ---- Block Sequence Stream Access ----

struct StreamBlockSeqAccess<'p, 'i> {
    parser: &'p mut Parser<'i>,
    indent: usize,
}

impl<'de> SeqAccess<'de> for StreamBlockSeqAccess<'_, '_> {
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        self.parser.skip_blanks_and_comments();
        if self.parser.is_eof() {
            return Ok(None);
        }
        let cur_indent = self.parser.peek_line_indent();
        if cur_indent != self.indent {
            return Ok(None);
        }
        let rest = self.parser.rest().trim_start_matches(' ');
        if !rest.starts_with("- ")
            && rest != "-"
            && !rest.starts_with("-\n")
            && !rest.starts_with("-\r")
        {
            return Ok(None);
        }
        // Skip indent + "-"
        self.parser.advance_by(self.indent);
        self.parser.advance_by(1);
        if self.parser.peek() == Some(' ') {
            self.parser.advance_by(1);
        }
        self.parser.skip_inline_spaces();

        // Parse the item value
        let value = if self.parser.peek() == Some('\n')
            || self.parser.peek() == Some('\r')
            || self.parser.peek() == Some('#')
            || self.parser.is_eof()
        {
            if self.parser.peek() == Some('#') {
                self.parser.skip_to_eol();
            }
            self.parser.parse_value(self.indent + 2)?
        } else {
            // Check if it's inline mapping
            let line = self.parser.current_line();
            if self.parser.find_mapping_colon(line).is_some() {
                let item_indent = self.indent + 2;
                let (key, val) =
                    self.parser.parse_mapping_entry(item_indent)?;
                let mut m = Mapping::new();
                m.insert(key, val);
                loop {
                    self.parser.skip_blanks_and_comments();
                    if self.parser.is_eof() {
                        break;
                    }
                    let ci = self.parser.peek_line_indent();
                    if ci != item_indent {
                        break;
                    }
                    let rt = self.parser.rest().trim_start_matches(' ');
                    if self
                        .parser
                        .find_mapping_colon(match rt.find('\n') {
                            Some(i) => &rt[..i],
                            None => rt,
                        })
                        .is_none()
                    {
                        break;
                    }
                    self.parser.advance_by(item_indent);
                    let (k, v) =
                        self.parser.parse_mapping_entry(item_indent)?;
                    m.insert(k, v);
                }
                Value::Mapping(m)
            } else {
                self.parser.parse_inline_value(self.indent)?
            }
        };

        seed.deserialize(value).map(Some)
    }
}

// ---- Flow Sequence Stream Access ----

struct StreamFlowSeqAccess<'p, 'i> {
    parser: &'p mut Parser<'i>,
    first: bool,
}

impl<'de> SeqAccess<'de> for StreamFlowSeqAccess<'_, '_> {
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        self.parser.skip_flow_whitespace();
        match self.parser.peek() {
            None => {
                return Err(self
                    .parser
                    .error("unterminated flow sequence"))
            }
            Some(']') => {
                self.parser.advance_by(1);
                return Ok(None);
            }
            Some(',') if !self.first => {
                self.parser.advance_by(1);
                self.parser.skip_flow_whitespace();
                if self.parser.peek() == Some(']') {
                    self.parser.advance_by(1);
                    return Ok(None);
                }
            }
            _ => {}
        }
        self.first = false;
        let value = self.parser.parse_flow_value()?;
        seed.deserialize(value).map(Some)
    }
}

// ---- Flow Mapping Stream Access ----

struct StreamFlowMapAccess<'p, 'i> {
    parser: &'p mut Parser<'i>,
    first: bool,
    value: Option<Value>,
}

impl<'de> MapAccess<'de> for StreamFlowMapAccess<'_, '_> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        self.parser.skip_flow_whitespace();
        match self.parser.peek() {
            None => {
                return Err(self
                    .parser
                    .error("unterminated flow mapping"))
            }
            Some('}') => {
                self.parser.advance_by(1);
                return Ok(None);
            }
            Some(',') if !self.first => {
                self.parser.advance_by(1);
                self.parser.skip_flow_whitespace();
                if self.parser.peek() == Some('}') {
                    self.parser.advance_by(1);
                    return Ok(None);
                }
            }
            _ => {}
        }
        self.first = false;
        let key = self.parser.parse_flow_value()?;
        self.parser.skip_flow_whitespace();
        if self.parser.peek() != Some(':') {
            return Err(self
                .parser
                .error("expected ':' in flow mapping"));
        }
        self.parser.advance_by(1);
        self.parser.skip_flow_whitespace();
        let value = self.parser.parse_flow_value()?;
        self.value = Some(value);
        seed.deserialize(key).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(v) => seed.deserialize(v),
            None => Err(Error::msg("value before key")),
        }
    }
}
