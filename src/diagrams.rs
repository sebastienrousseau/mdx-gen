//! Mermaid diagram rendering for fenced code blocks tagged
//! `mermaid`.
//!
//! Rather than rasterising server-side (which would require a
//! headless browser or equivalent heavy runtime), mdx-gen rewrites
//! each `mermaid` code block into a sanitizer-safe
//! `<pre class="mermaid">…</pre>` container that the client-side
//! [mermaid.js] library hydrates into inline SVG at page-load
//! time. This matches what github.com does natively in README
//! rendering.
//!
//! # Usage
//!
//! Enable the transform with
//! [`crate::MarkdownOptions::with_diagrams`]. Emit the hydration
//! script into your page shell exactly once with
//! [`hydration_script_html`]. The output is SVG.
//!
//! ```
//! use mdx_gen::{process_markdown, MarkdownOptions};
//!
//! let md = "```mermaid\ngraph TD\nA --> B\n```\n";
//! let options = MarkdownOptions::new()
//!     .with_custom_blocks(false)
//!     .with_enhanced_tables(false)
//!     .with_syntax_highlighting(false)
//!     .with_diagrams(true);
//! let html = process_markdown(md, &options).unwrap();
//! assert!(html.contains("<pre class=\"mermaid\">"));
//! ```
//!
//! [mermaid.js]: https://mermaid.js.org/

use comrak::nodes::{AstNode, NodeHtmlBlock, NodeValue};

/// Walks the comrak AST and replaces every `NodeValue::CodeBlock`
/// whose info-string names `mermaid` with a
/// `NodeValue::HtmlBlock` containing the sanitizer-safe
/// `<pre class="mermaid">` container.
///
/// Kind-matching is done on the first whitespace-delimited token
/// of the info string so `mermaid classDiagram` still counts as
/// mermaid. Non-mermaid code blocks pass through unchanged — the
/// syntax highlighter still sees them downstream.
pub fn process_diagram_code_blocks<'a>(root: &'a AstNode<'a>) {
    for node in root.descendants() {
        let mut ast = node.data.borrow_mut();
        let replacement = match ast.value {
            NodeValue::CodeBlock(ref block) => {
                let kind = block
                    .info
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if kind == "mermaid" {
                    Some(render_mermaid(&block.literal))
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(html) = replacement {
            ast.value = NodeValue::HtmlBlock(NodeHtmlBlock {
                block_type: 6,
                literal: html,
            });
        }
    }
}

/// Builds the `<pre class="mermaid">` container for a single
/// block. Content is HTML-escaped so raw `<` / `>` inside the
/// source cannot break out of the `<pre>` context.
fn render_mermaid(source: &str) -> String {
    let escaped = html_escape::encode_text(source);
    format!("<pre class=\"mermaid\">{escaped}</pre>\n")
}

/// Returns the `<script type="module">…</script>` block users
/// should drop into their page shell (usually just before
/// `</body>`) to hydrate every `<pre class="mermaid">` container
/// on the page. The script is safe to include on pages that have
/// no diagrams — it short-circuits when no mermaid container is
/// present.
///
/// The return value is a `'static` string; embed it verbatim.
#[must_use]
pub fn hydration_script_html() -> &'static str {
    HYDRATION_SCRIPT
}

const HYDRATION_SCRIPT: &str = include_str!("diagrams_hydrator.js");

#[cfg(test)]
mod tests {
    use super::*;
    use comrak::{parse_document, Arena, Options};

    /// Walk the root looking for an HtmlBlock whose literal
    /// contains `needle`. Returns the matching literal or `None`.
    fn find_html_containing<'a>(
        root: &'a AstNode<'a>,
        needle: &str,
    ) -> Option<String> {
        for node in root.descendants() {
            if let NodeValue::HtmlBlock(ref block) =
                node.data.borrow().value
            {
                if block.literal.contains(needle) {
                    return Some(block.literal.clone());
                }
            }
        }
        None
    }

    /// Helper that parses Markdown, runs the diagram transform,
    /// and returns any HtmlBlock literal containing `needle`.
    fn transform_and_find(
        source: &str,
        needle: &str,
    ) -> Option<String> {
        let arena = Arena::new();
        let root = parse_document(&arena, source, &Options::default());
        process_diagram_code_blocks(root);
        find_html_containing(root, needle)
    }

    #[test]
    fn test_mermaid_block_rewritten() {
        let md = "```mermaid\ngraph TD\n  A --> B\n```\n";
        let found = transform_and_find(md, "class=\"mermaid\"")
            .expect("mermaid container missing");
        assert!(found.starts_with("<pre class=\"mermaid\">"));
        assert!(found.contains("graph TD"));
        assert!(found.contains("A --&gt; B"), "content escaped");
    }

    #[test]
    fn test_info_string_with_attributes() {
        // `mermaid classDiagram` should still match on the first
        // whitespace-delimited token.
        let md =
            "```mermaid classDiagram\nclassDiagram\n  A<|--B\n```\n";
        assert!(transform_and_find(md, "class=\"mermaid\"").is_some());
    }

    #[test]
    fn test_unknown_lang_passes_through() {
        let md = "```rust\nfn main() {}\n```\n";
        assert!(transform_and_find(md, "class=\"mermaid\"").is_none());
    }

    #[test]
    fn test_non_matching_diagram_langs_pass_through() {
        // Formats that previously had first-class support (geojson,
        // topojson, stl) are no longer recognised — they should
        // pass through to the syntax highlighter like any other
        // unknown language.
        for lang in ["geojson", "topojson", "stl"] {
            let md = format!("```{lang}\n{{\"a\":1}}\n```\n");
            assert!(
                transform_and_find(&md, "class=\"mermaid\"").is_none(),
                "{lang} should not produce a mermaid container"
            );
        }
    }

    #[test]
    fn test_content_is_html_escaped() {
        let md = "```mermaid\ngraph <script>alert(1)</script>\n```\n";
        let found =
            transform_and_find(md, "class=\"mermaid\"").unwrap();
        assert!(!found.contains("<script>"));
        assert!(found.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_hydration_script_imports_mermaid() {
        let s = hydration_script_html();
        assert!(s.contains("pre.mermaid"));
        assert!(s.contains("mermaid"));
        // Wrapped in <script type="module">…</script>.
        assert!(s.starts_with("<script type=\"module\">"));
        assert!(s.trim_end().ends_with("</script>"));
    }
}
