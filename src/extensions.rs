//! Extension functionality for the MDX Gen library.
//!
//! This module provides utilities for enhancing Markdown processing,
//! including custom block handling and table formatting.
//! Syntax highlighting has moved to [`crate::highlight`].

use crate::error::MarkdownError;
use comrak::nodes::{NodeHtmlBlock, NodeValue};
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::LazyLock;

// ── Table regexes (cached, for legacy process_tables) ───────────────
//
// Opening and closing `<table>` tags are literal substrings, handled
// by `str::replace` in `process_tables` below — no regex needed. The
// `<td …>` rewrite does need a pattern to capture the attribute run,
// so it stays as a cached `Regex`.

static TABLE_CELL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<td([^>]*)>").unwrap());

/// Regex matching known custom block div elements inside HTML blocks.
static CUSTOM_BLOCK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?si)<div\s+class=["']?(note|warning|tip|info|important|caution)["']?>(.*?)</div>"#,
    )
    .unwrap()
});

// ── Column alignment ────────────────────────────────────────────────

/// Alignment options for table columns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnAlignment {
    /// Align the column to the left.
    Left,
    /// Align the column to the center.
    Center,
    /// Align the column to the right.
    Right,
}

// ── Custom block types ──────────────────────────────────────────────

/// Represents different types of custom blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CustomBlockType {
    /// A note block.
    Note,
    /// A warning block.
    Warning,
    /// A tip block.
    Tip,
    /// An info block.
    Info,
    /// An important block.
    Important,
    /// A caution block.
    Caution,
}

impl CustomBlockType {
    /// Returns the default Bootstrap alert class.
    pub fn default_alert_class(&self) -> &'static str {
        match self {
            Self::Note => "alert-info",
            Self::Warning => "alert-warning",
            Self::Tip => "alert-success",
            Self::Info => "alert-primary",
            Self::Important => "alert-danger",
            Self::Caution => "alert-secondary",
        }
    }

    /// Returns the default human-readable title.
    pub fn default_title(&self) -> &'static str {
        match self {
            Self::Note => "Note",
            Self::Warning => "Warning",
            Self::Tip => "Tip",
            Self::Info => "Info",
            Self::Important => "Important",
            Self::Caution => "Caution",
        }
    }

    /// Returns the default Bootstrap alert class for this block type.
    pub fn get_alert_class(&self) -> &'static str {
        self.default_alert_class()
    }

    /// Returns the default title for this block type.
    pub fn get_title(&self) -> &'static str {
        self.default_title()
    }

    /// Returns the alert class, respecting config overrides.
    pub fn alert_class_with<'a>(
        &self,
        config: &'a CustomBlockConfig,
    ) -> &'a str {
        config
            .class_overrides
            .get(self)
            .map(|s| s.as_str())
            .unwrap_or_else(move || self.default_alert_class())
    }

    /// Returns the title, respecting config overrides.
    pub fn title_with<'a>(
        &self,
        config: &'a CustomBlockConfig,
    ) -> &'a str {
        config
            .title_overrides
            .get(self)
            .map(|s| s.as_str())
            .unwrap_or_else(move || self.default_title())
    }
}

impl FromStr for CustomBlockType {
    type Err = MarkdownError;

    fn from_str(block_type: &str) -> Result<Self, Self::Err> {
        match block_type.to_lowercase().as_str() {
            "note" => Ok(Self::Note),
            "warning" => Ok(Self::Warning),
            "tip" => Ok(Self::Tip),
            "info" => Ok(Self::Info),
            "important" => Ok(Self::Important),
            "caution" => Ok(Self::Caution),
            _ => Err(MarkdownError::CustomBlockError(format!(
                "Unknown block type: {block_type}"
            ))),
        }
    }
}

// ── Custom block configuration ──────────────────────────────────────

/// Configuration for custom block rendering.
///
/// Allows overriding the default CSS class and title for each
/// block type, enabling use with CSS frameworks other than Bootstrap.
#[derive(Debug, Clone, Default)]
pub struct CustomBlockConfig {
    /// Override the CSS alert class per block type.
    pub class_overrides: HashMap<CustomBlockType, String>,
    /// Override the display title per block type.
    pub title_overrides: HashMap<CustomBlockType, String>,
}

impl CustomBlockConfig {
    /// Creates a new empty configuration (uses all defaults).
    pub fn new() -> Self {
        Self::default()
    }

    /// Overrides the CSS class for a specific block type.
    pub fn with_class(
        mut self,
        block_type: CustomBlockType,
        class: impl Into<String>,
    ) -> Self {
        self.class_overrides.insert(block_type, class.into());
        self
    }

    /// Overrides the display title for a specific block type.
    pub fn with_title(
        mut self,
        block_type: CustomBlockType,
        title: impl Into<String>,
    ) -> Self {
        self.title_overrides.insert(block_type, title.into());
        self
    }
}

// ── AST-level custom block processing ───────────────────────────────

/// Walks the comrak AST and transforms `HtmlBlock` nodes that contain
/// known custom block divs into styled alert HTML.
///
/// This is safer than regex on rendered HTML because it only touches
/// nodes the parser explicitly identified as raw HTML blocks.
pub fn process_custom_block_nodes<'a>(
    root: comrak::nodes::Node<'a>,
    config: &CustomBlockConfig,
) {
    for node in root.descendants() {
        let mut ast = node.data.borrow_mut();
        if let NodeValue::HtmlBlock(ref mut block) = ast.value {
            block.literal =
                transform_custom_blocks(&block.literal, config);
        }
    }
}

/// Transforms custom block divs in a raw HTML string.
fn transform_custom_blocks(
    html: &str,
    config: &CustomBlockConfig,
) -> String {
    CUSTOM_BLOCK_RE
        .replace_all(html, |caps: &regex::Captures| {
            let block_type = CustomBlockType::from_str(
                caps.get(1).unwrap().as_str(),
            )
            .expect("regex only matches known block types");
            generate_custom_block_html(block_type, &caps[2], config)
        })
        .to_string()
}

/// Generates the HTML for a custom block.
fn generate_custom_block_html(
    block_type: CustomBlockType,
    content: &str,
    config: &CustomBlockConfig,
) -> String {
    format!(
        r#"<div class="alert {}" role="alert"><strong>{}:</strong> {}</div>"#,
        block_type.alert_class_with(config),
        block_type.title_with(config),
        content
    )
}

// ── AST-level table enhancement ─────────────────────────────────────

/// Walks the comrak AST and replaces `Table` nodes with `HtmlBlock`
/// nodes containing responsive-wrapped, class-enhanced table HTML.
///
/// This eliminates the last regex pass over rendered HTML.
pub fn enhance_table_nodes<'a>(
    root: comrak::nodes::Node<'a>,
    arena: &'a comrak::Arena<'a>,
    options: &comrak::Options,
) {
    // Collect table nodes first to avoid borrow issues during mutation
    let table_nodes: Vec<comrak::nodes::Node<'a>> = root
        .descendants()
        .filter(|node| {
            matches!(node.data.borrow().value, NodeValue::Table(_))
        })
        .collect();

    for table_node in table_nodes {
        // Render this table subtree to HTML
        let mut table_html = String::new();
        if comrak::format_html(table_node, options, &mut table_html)
            .is_err()
        {
            continue;
        }

        // Apply the responsive wrapper and alignment classes
        let enhanced = process_tables(&table_html);

        // Create a replacement HtmlBlock node
        let start = comrak::nodes::LineColumn { line: 0, column: 0 };
        let replacement = arena.alloc(comrak::nodes::AstNode::new(
            RefCell::new(comrak::nodes::Ast::new(
                NodeValue::HtmlBlock(NodeHtmlBlock {
                    block_type: 6, // generic block
                    literal: enhanced,
                }),
                start,
            )),
        ));

        // Insert replacement and remove original
        table_node.insert_before(replacement);
        table_node.detach();
    }
}

// ── Legacy string-level custom block processing ─────────────────────

/// Processes custom blocks in an HTML string.
///
/// Provided for backward compatibility. Prefer
/// [`process_custom_block_nodes`] for AST-level processing.
pub fn process_custom_blocks(content: &str) -> String {
    transform_custom_blocks(content, &CustomBlockConfig::default())
}

// ── Table post-processing ───────────────────────────────────────────

/// Processes tables, enhancing them with responsive design and alignment classes.
pub fn process_tables(table_html: &str) -> String {
    let table_html = table_html.replace(
        "<table>",
        r#"<div class="table-responsive"><table class="table">"#,
    );
    let table_html = table_html.replace("</table>", "</table></div>");

    TABLE_CELL_RE
        .replace_all(&table_html, |caps: &regex::Captures| {
            let attrs = &caps[1];
            if attrs.contains("align=\"center\"") {
                format!(r#"<td{attrs} class="text-center">"#)
            } else if attrs.contains("align=\"right\"") {
                format!(r#"<td{attrs} class="text-right">"#)
            } else {
                format!(r#"<td{attrs} class="text-left">"#)
            }
        })
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_custom_blocks_default_config() {
        let input = r#"
            <div class="note">This is a note.</div>
            <div class="WARNING">This is a warning.</div>
            <div class="Tip">This is a tip.</div>
        "#;
        let processed = process_custom_blocks(input);
        assert!(processed.contains(r#"alert alert-info"#));
        assert!(processed.contains(r#"alert alert-warning"#));
        assert!(processed.contains(r#"alert alert-success"#));
    }

    #[test]
    fn test_custom_block_config_overrides() {
        let config = CustomBlockConfig::new()
            .with_class(CustomBlockType::Note, "callout-info")
            .with_title(CustomBlockType::Note, "Did you know?");

        let html = generate_custom_block_html(
            CustomBlockType::Note,
            "test content",
            &config,
        );
        assert!(html.contains("callout-info"));
        assert!(html.contains("Did you know?:"));
    }

    #[test]
    fn test_unknown_block_passthrough() {
        let input =
            r#"<div class="unknown">Should pass through.</div>"#;
        let processed = process_custom_blocks(input);
        assert_eq!(processed, input);
    }

    #[test]
    fn test_process_tables() {
        let input = r#"<table><tr><td align="center">Center</td><td align="right">Right</td><td>Left</td></tr></table>"#;
        let processed = process_tables(input);
        assert!(processed.contains(r#"table-responsive"#));
        assert!(processed.contains(r#"text-center"#));
        assert!(processed.contains(r#"text-right"#));
        assert!(processed.contains(r#"text-left"#));
    }

    #[test]
    fn test_process_multiple_tables() {
        let input = "<table><tr><td>A</td></tr></table>\n<table><tr><td>B</td></tr></table>";
        let processed = process_tables(input);
        assert_eq!(processed.matches("table-responsive").count(), 2);
    }

    #[test]
    fn test_unknown_block_type_from_str() {
        let result = CustomBlockType::from_str("unknown");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Unknown block type: unknown"),
            "Error message should contain the unknown type"
        );
    }

    #[test]
    fn test_unknown_block_type_from_str_various() {
        for name in ["foobar", "alert", "danger", "success", ""] {
            let result = CustomBlockType::from_str(name);
            assert!(
                result.is_err(),
                "'{name}' should not parse as a valid block type"
            );
        }
    }
}
