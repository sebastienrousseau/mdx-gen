//! Client-side diagram rendering for `mermaid`, `geojson`,
//! `topojson`, and ASCII `stl` fenced code blocks.
//!
//! The rendering strategy mirrors what github.com does for mermaid
//! and geojson: rather than rasterising server-side (which would
//! require a headless browser or equivalent heavy runtime), mdx-gen
//! rewrites each recognised code block into sanitizer-safe HTML
//! containers that a small client-side JS module hydrates into
//! inline `<svg>` at page-load time.
//!
//! # Supported formats
//!
//! | Info string | Container                                                     | Renderer (client-side) |
//! |:------------|:--------------------------------------------------------------|:-----------------------|
//! | `mermaid`   | `<pre class="mermaid">…</pre>`                                | [mermaid.js] v10 (ESM) |
//! | `geojson`   | `<div data-mdx-diagram="geojson"><pre>…</pre></div>`          | [d3-geo] v3 (SVG out)  |
//! | `topojson`  | `<div data-mdx-diagram="topojson"><pre>…</pre></div>`         | [topojson-client] + d3-geo |
//! | `stl`       | `<div data-mdx-diagram="stl"><pre>…</pre></div>`              | [three.js] + STLLoader + SVGRenderer |
//!
//! Content inside `<pre>` is plain text — the hydrator reads
//! `.textContent` at load time. HTML injection vectors are therefore
//! neutralised by the sanitizer's standard string-escaping.
//!
//! # Usage
//!
//! Enable the transform with
//! [`crate::MarkdownOptions::with_diagrams`]. Emit the hydration
//! script into your page shell exactly once with
//! [`hydration_script_html`]. The output is SVG in every case.
//!
//! # Why client-side?
//!
//! Server-side rasterisation of these four disparate formats would
//! need a headless Chromium (mermaid), a geospatial rasteriser
//! (geojson/topojson), and a 3-D rendering pipeline (stl). The
//! hydration-script approach keeps mdx-gen pure-Rust + dependency-
//! light while matching github.com's actual behaviour for the two
//! formats GitHub supports natively.
//!
//! [mermaid.js]: https://mermaid.js.org/
//! [d3-geo]: https://github.com/d3/d3-geo
//! [topojson-client]: https://github.com/topojson/topojson-client
//! [three.js]: https://threejs.org/

use comrak::nodes::{AstNode, NodeHtmlBlock, NodeValue};

/// Set of info-string tokens the transform recognises. Matched on
/// the first whitespace-delimited token of the fenced code block's
/// info string so `mermaid classDiagram` still counts as mermaid.
const DIAGRAM_KINDS: &[&str] =
    &["mermaid", "geojson", "topojson", "stl"];

/// Walks the comrak AST and replaces every `NodeValue::CodeBlock`
/// whose info-string names a supported diagram format with a
/// `NodeValue::HtmlBlock` containing the sanitizer-safe container
/// markup.
///
/// Non-diagram code blocks pass through unchanged — the syntax
/// highlighter still sees them downstream. Call sites should
/// invoke this before table enhancement and rendering.
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
                if DIAGRAM_KINDS.contains(&kind.as_str()) {
                    Some(render_container(&kind, &block.literal))
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

/// Builds the HTML container for a single diagram block. Content
/// is HTML-escaped so raw `<` / `>` inside the source cannot break
/// out of the `<pre>` context.
fn render_container(kind: &str, source: &str) -> String {
    let escaped = html_escape::encode_text(source);
    if kind == "mermaid" {
        format!("<pre class=\"mermaid\">{escaped}</pre>\n")
    } else {
        format!(
            "<div class=\"mdx-diagram mdx-diagram-{kind}\" data-mdx-diagram=\"{kind}\"><pre>{escaped}</pre></div>\n"
        )
    }
}

/// Returns the `<script type="module">…</script>` block users should
/// drop into their page shell (usually just before `</body>`) to
/// hydrate every mdx-gen diagram container on the page.
///
/// The script loads mermaid, d3, topojson-client, and three.js
/// (+ STLLoader + SVGRenderer) from jsdelivr lazily per diagram
/// kind — only what the page actually uses is fetched.
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
    fn test_geojson_block_rewritten() {
        let md = "```geojson\n{\"type\":\"Feature\"}\n```\n";
        let found =
            transform_and_find(md, "data-mdx-diagram=\"geojson\"")
                .expect("geojson container missing");
        assert!(
            found.contains("class=\"mdx-diagram mdx-diagram-geojson\"")
        );
        assert!(found.contains("<pre>"));
        // Content round-trips through `html_escape::encode_text`
        // which leaves ASCII-safe characters (`"`, `:`, etc.)
        // alone; only `<`, `>`, and `&` are entity-encoded.
        assert!(found.contains(r#""type":"Feature""#));
    }

    #[test]
    fn test_topojson_block_rewritten() {
        let md = "```topojson\n{\"type\":\"Topology\"}\n```\n";
        let found =
            transform_and_find(md, "data-mdx-diagram=\"topojson\"")
                .expect("topojson container missing");
        assert!(found
            .contains("class=\"mdx-diagram mdx-diagram-topojson\""));
    }

    #[test]
    fn test_stl_block_rewritten() {
        let md = "```stl\nsolid cube\nendsolid cube\n```\n";
        let found = transform_and_find(md, "data-mdx-diagram=\"stl\"")
            .expect("stl container missing");
        assert!(found.contains("class=\"mdx-diagram mdx-diagram-stl\""));
        assert!(found.contains("solid cube"));
    }

    #[test]
    fn test_info_string_with_attributes() {
        // `mermaid classDiagram` should still match on the first
        // token.
        let md =
            "```mermaid classDiagram\nclassDiagram\n  A<|--B\n```\n";
        assert!(transform_and_find(md, "class=\"mermaid\"").is_some());
    }

    #[test]
    fn test_unknown_lang_passes_through() {
        let md = "```rust\nfn main() {}\n```\n";
        assert!(transform_and_find(md, "mdx-diagram").is_none());
        assert!(transform_and_find(md, "class=\"mermaid\"").is_none());
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
    fn test_hydration_script_mentions_each_renderer() {
        let s = hydration_script_html();
        assert!(s.contains("pre.mermaid"));
        assert!(s.contains("data-mdx-diagram=\"geojson\""));
        assert!(s.contains("data-mdx-diagram=\"topojson\""));
        assert!(s.contains("data-mdx-diagram=\"stl\""));
        // Wrapped in <script type="module">...</script>.
        assert!(s.starts_with("<script type=\"module\">"));
        assert!(s.trim_end().ends_with("</script>"));
    }
}
