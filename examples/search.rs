// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Build a per-section search index by walking the comrak AST.
//!
//! Run: `cargo run --example search`

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, Options};

const SOURCE: &str = r#"# Getting started

A short introduction to the library. We focus on three things:
parsing, rendering, and sanitisation.

## Installation

Add `mdx-gen` to your `Cargo.toml`. The default features cover
most needs.

## First conversion

Call `process_markdown` with a `MarkdownOptions`. You get back
an HTML `String`. Use `process_markdown_to_writer` if you want
to stream into a `File` or `TcpStream` instead.

# Configuration

Every knob lives on `MarkdownOptions`.

## Custom blocks

Wrap content in `<div class="note">…</div>` and the renderer
turns it into a Bootstrap-shaped alert. See the `alerts`
example for the override mechanism.

## Sanitisation

`SanitizerConfig` extends the ammonia allow-list. Read the
`cms` example for the safe-rendering scenario.
"#;

#[derive(Debug)]
struct Section {
    id: String,
    level: u8,
    title: String,
    text: String,
}

fn main() {
    support::header("mdx-gen -- search");

    let sections = support::task("Walk AST + group by heading", || {
        let arena = Arena::new();
        let mut comrak_options = Options::default();
        comrak_options.extension.table = true;
        let root = parse_document(&arena, SOURCE, &comrak_options);

        let mut anchorizer = comrak::Anchorizer::new();
        let mut sections: Vec<Section> = Vec::new();
        let mut current: Option<Section> = None;

        for child in root.children() {
            let value = child.data.borrow().value.clone();
            if let NodeValue::Heading(h) = value {
                if let Some(s) = current.take() {
                    sections.push(s);
                }
                let title = collect_text(child);
                let id = anchorizer.anchorize(&title);
                current = Some(Section {
                    id,
                    level: h.level,
                    title,
                    text: String::new(),
                });
            } else if let Some(s) = current.as_mut() {
                let mut buf = String::new();
                walk_text(child, &mut buf);
                if !buf.is_empty() {
                    if !s.text.is_empty() {
                        s.text.push(' ');
                    }
                    s.text.push_str(buf.trim());
                }
            }
        }
        if let Some(s) = current.take() {
            sections.push(s);
        }
        sections
    });

    support::task_with_output("Inspect index entries", || {
        sections
            .iter()
            .map(|s| {
                let snippet = snippet(&s.text, 64);
                format!(
                    "H{lvl} #{id:<24} {title}",
                    lvl = s.level,
                    id = s.id,
                    title = s.title,
                ) + &format!("\n           text: {snippet}")
            })
            .collect()
    });

    support::task_with_output("Emit faux-JSON for an indexer", || {
        let mut lines = vec!["[".to_string()];
        for (i, s) in sections.iter().enumerate() {
            let comma = if i + 1 == sections.len() { "" } else { "," };
            lines.push(format!(
                r#"  {{"id":"{}","level":{},"title":{:?},"text":{:?}}}{}"#,
                s.id,
                s.level,
                s.title,
                s.text,
                comma
            ));
        }
        lines.push("]".to_string());
        lines
    });

    support::task_with_output("Stats", || {
        vec![
            format!("sections    : {}", sections.len()),
            format!(
                "total chars : {}",
                sections.iter().map(|s| s.text.len()).sum::<usize>()
            ),
            format!(
                "max depth   : {}",
                sections.iter().map(|s| s.level).max().unwrap_or(0)
            ),
        ]
    });

    support::summary(4);
}

/// Collect plain text from a node's subtree, matching what comrak
/// renders inside an `<h*>` tag.
fn collect_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut out = String::new();
    walk_text(node, &mut out);
    out.trim().to_string()
}

fn walk_text<'a>(node: &'a AstNode<'a>, out: &mut String) {
    for d in node.descendants() {
        match &d.data.borrow().value {
            NodeValue::Text(t) => out.push_str(t),
            NodeValue::Code(c) => out.push_str(&c.literal),
            NodeValue::SoftBreak | NodeValue::LineBreak => {
                out.push(' ')
            }
            _ => {}
        }
    }
}

fn snippet(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        return s.to_string();
    }
    let cut: String = s.chars().take(n).collect();
    format!("{cut}…")
}
