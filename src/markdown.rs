//! Core Markdown processing functionality.
//!
//! This module handles the conversion of Markdown content into HTML,
//! with support for custom blocks, enhanced tables, and syntax highlighting.
//!
//! ## Processing Pipeline
//!
//! 1. **Parse** — Markdown source → comrak AST (arena-allocated).
//! 2. **Transform** — Walk the AST to rewrite custom-block `HtmlBlock`
//!    nodes in-place.
//! 3. **Render** — Convert the (possibly modified) AST to HTML, using
//!    comrak's plugin system for syntax highlighting.
//! 4. **Enhance** — Post-process table HTML for responsive wrappers and
//!    alignment classes.
//! 5. **Sanitize** — When `allow_unsafe_html` is `false`, run ammonia to
//!    strip dangerous tags while preserving safe structural markup.

use crate::error::MarkdownError;
use crate::extensions::{
    process_custom_block_nodes, process_tables, CustomBlockConfig,
};
use comrak::options::{Plugins, RenderPlugins};
use comrak::{markdown_to_html_with_plugins, Arena, Options};
use log::{debug, info, warn};
use std::fmt;

#[cfg(feature = "syntax_highlighting")]
use crate::highlight::SyntectAdapter;

/// Default maximum input size: 1 MiB.
pub const DEFAULT_MAX_INPUT_SIZE: usize = 1_048_576;

/// Options for configuring Markdown processing behavior.
#[derive(Clone)]
pub struct MarkdownOptions<'a> {
    /// Options for the underlying Comrak Markdown parser.
    pub comrak_options: Options<'a>,
    /// Enable or disable processing of custom blocks.
    pub enable_custom_blocks: bool,
    /// Enable or disable syntax highlighting for code blocks.
    pub enable_syntax_highlighting: bool,
    /// Enable or disable enhanced table formatting.
    pub enable_enhanced_tables: bool,
    /// Optional custom theme for syntax highlighting.
    pub syntax_theme: Option<String>,
    /// Allow raw HTML pass-through in Markdown output.
    ///
    /// When `true`, raw HTML in the Markdown source is passed through
    /// unchanged. When `false`, output is sanitized with ammonia to
    /// strip dangerous tags while preserving safe structural HTML
    /// (our generated alert divs, tables, code blocks, etc.).
    pub allow_unsafe_html: bool,
    /// Configuration for custom block rendering.
    pub custom_block_config: CustomBlockConfig,
    /// Maximum input size in bytes. `0` means no limit.
    pub max_input_size: usize,
}

impl<'a> Default for MarkdownOptions<'a> {
    fn default() -> Self {
        Self {
            comrak_options: Options::default(),
            enable_custom_blocks: true,
            enable_syntax_highlighting: true,
            enable_enhanced_tables: true,
            syntax_theme: None,
            allow_unsafe_html: true,
            custom_block_config: CustomBlockConfig::default(),
            max_input_size: DEFAULT_MAX_INPUT_SIZE,
        }
    }
}

impl<'a> MarkdownOptions<'a> {
    /// Creates a new instance with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables or disables custom blocks.
    pub fn with_custom_blocks(mut self, enable: bool) -> Self {
        self.enable_custom_blocks = enable;
        self
    }

    /// Enables or disables syntax highlighting for code blocks.
    pub fn with_syntax_highlighting(mut self, enable: bool) -> Self {
        self.enable_syntax_highlighting = enable;
        self
    }

    /// Enables or disables enhanced table formatting.
    pub fn with_enhanced_tables(mut self, enable: bool) -> Self {
        self.enable_enhanced_tables = enable;
        self
    }

    /// Sets a custom theme for syntax highlighting.
    pub fn with_custom_theme(mut self, theme: String) -> Self {
        self.syntax_theme = Some(theme);
        self
    }

    /// Sets custom Comrak options.
    ///
    /// Also syncs `allow_unsafe_html` from `render.unsafe`.
    pub fn with_comrak_options(mut self, options: Options<'a>) -> Self {
        self.allow_unsafe_html = options.render.r#unsafe;
        self.comrak_options = options;
        self
    }

    /// Enables or disables raw HTML pass-through.
    ///
    /// This is the authoritative control. Call **after**
    /// `with_comrak_options` if you need to override.
    pub fn with_unsafe_html(mut self, enable: bool) -> Self {
        self.allow_unsafe_html = enable;
        self
    }

    /// Sets the custom block configuration.
    pub fn with_custom_block_config(
        mut self,
        config: CustomBlockConfig,
    ) -> Self {
        self.custom_block_config = config;
        self
    }

    /// Sets the maximum input size in bytes. `0` means no limit.
    pub fn with_max_input_size(mut self, size: usize) -> Self {
        self.max_input_size = size;
        self
    }

    /// Validates that options are consistent.
    pub fn validate(&self) -> Result<(), String> {
        if self.enable_enhanced_tables
            && !self.comrak_options.extension.table
        {
            return Err(
                "Enhanced tables are enabled, but Comrak table \
                 extension is disabled."
                    .to_string(),
            );
        }
        Ok(())
    }
}

impl fmt::Debug for MarkdownOptions<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MarkdownOptions")
            .field("enable_custom_blocks", &self.enable_custom_blocks)
            .field(
                "enable_syntax_highlighting",
                &self.enable_syntax_highlighting,
            )
            .field(
                "enable_enhanced_tables",
                &self.enable_enhanced_tables,
            )
            .field("syntax_theme", &self.syntax_theme)
            .field("allow_unsafe_html", &self.allow_unsafe_html)
            .field("max_input_size", &self.max_input_size)
            .finish()
    }
}

/// Creates a convenience set of options with all features enabled.
pub fn default_markdown_options() -> MarkdownOptions<'static> {
    MarkdownOptions::new()
        .with_custom_blocks(true)
        .with_syntax_highlighting(true)
        .with_enhanced_tables(true)
        .with_comrak_options({
            let mut opts = Options::default();
            opts.extension.table = true;
            opts
        })
        .with_unsafe_html(true)
}

// ── Core processing pipeline ────────────────────────────────────────

/// Processes the input Markdown content and converts it into HTML.
///
/// The pipeline:
/// 1. Validate options and check resource limits.
/// 2. Parse Markdown to a comrak AST.
/// 3. (Optional) Transform custom-block `HtmlBlock` nodes in the AST.
/// 4. Render to HTML, using comrak's syntax-highlighting plugin.
/// 5. (Optional) Enhance tables with responsive wrappers.
/// 6. (Optional) Sanitize HTML when `allow_unsafe_html` is `false`.
pub fn process_markdown(
    content: &str,
    options: &MarkdownOptions,
) -> Result<String, MarkdownError> {
    info!("Starting markdown processing");
    debug!("Markdown options: {:?}", options);

    // ── 0. Resource limits ──────────────────────────────────────
    if options.max_input_size > 0
        && content.len() > options.max_input_size
    {
        return Err(MarkdownError::InputTooLarge {
            size: content.len(),
            limit: options.max_input_size,
        });
    }

    // ── 1. Validate options ─────────────────────────────────────
    if let Err(msg) = options.validate() {
        warn!("Invalid MarkdownOptions: {}", msg);
        return Err(MarkdownError::InvalidOptionsError(msg));
    }

    // ── 2. Build comrak options ─────────────────────────────────
    let mut comrak_opts = options.comrak_options.clone();
    // Always enable unsafe for internal rendering — we sanitize
    // at the end if the caller wants safety.
    comrak_opts.render.r#unsafe = true;

    // ── 3. Parse → AST ─────────────────────────────────────────
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, content, &comrak_opts);

    // ── 4. AST transforms ───────────────────────────────────────
    if options.enable_custom_blocks {
        debug!("Processing custom blocks at AST level");
        process_custom_block_nodes(root, &options.custom_block_config);
    }

    // ── 5. Render to HTML ───────────────────────────────────────
    debug!("Rendering AST to HTML");

    #[cfg(feature = "syntax_highlighting")]
    let adapter;
    #[cfg(feature = "syntax_highlighting")]
    let plugins = if options.enable_syntax_highlighting {
        adapter = SyntectAdapter::new(options.syntax_theme.as_deref());
        Plugins {
            render: RenderPlugins {
                codefence_syntax_highlighter: Some(&adapter),
                ..Default::default()
            },
        }
    } else {
        Plugins::default()
    };
    #[cfg(not(feature = "syntax_highlighting"))]
    let plugins = Plugins::default();

    let mut html =
        markdown_to_html_with_plugins(content, &comrak_opts, &plugins);

    // We rendered from *content* (not AST) above, but custom block
    // transforms are on the AST. Re-render from the AST so
    // transforms take effect.
    html.clear();
    comrak::format_html_with_plugins(
        root,
        &comrak_opts,
        &mut html,
        &plugins,
    )
    .map_err(|e| MarkdownError::RenderError(e.to_string()))?;

    // ── 6. Enhance tables ───────────────────────────────────────
    if options.enable_enhanced_tables {
        debug!("Processing enhanced tables");
        html = process_tables(&html);
    }

    // ── 7. Sanitize ─────────────────────────────────────────────
    if !options.allow_unsafe_html {
        debug!("Sanitizing HTML output");
        html = sanitize_html(&html);
    }

    info!("Markdown processing completed successfully");
    Ok(html)
}

// ── HTML sanitization ───────────────────────────────────────────────

/// Sanitizes HTML output, stripping dangerous tags while preserving
/// safe structural markup that this library generates.
fn sanitize_html(html: &str) -> String {
    use std::collections::{HashMap, HashSet};

    let mut allowed_classes: HashMap<&str, HashSet<&str>> =
        HashMap::new();

    // Allow our generated alert classes on divs
    allowed_classes.insert(
        "div",
        [
            "alert",
            "alert-info",
            "alert-warning",
            "alert-success",
            "alert-primary",
            "alert-danger",
            "alert-secondary",
            "table-responsive",
        ]
        .into_iter()
        .collect(),
    );

    // Allow table classes
    allowed_classes.insert("table", ["table"].into_iter().collect());

    // Allow alignment classes on td
    allowed_classes.insert(
        "td",
        ["text-left", "text-center", "text-right"]
            .into_iter()
            .collect(),
    );

    // Allow common language classes on code elements.
    // We generate a broad set covering popular languages.
    let mut code_classes: HashSet<&str> = HashSet::new();
    for lang in [
        "rust",
        "python",
        "javascript",
        "typescript",
        "java",
        "c",
        "cpp",
        "csharp",
        "go",
        "ruby",
        "swift",
        "kotlin",
        "php",
        "html",
        "css",
        "sql",
        "bash",
        "shell",
        "json",
        "yaml",
        "toml",
        "xml",
        "markdown",
        "plaintext",
        "text",
    ] {
        code_classes.insert(
            // Leak is fine — these are static strings
            Box::leak(format!("language-{lang}").into_boxed_str()),
        );
    }
    allowed_classes.insert("code", code_classes);

    ammonia::Builder::default()
        .add_tags(["div", "pre", "code", "span", "input"])
        .add_tag_attributes("div", &["role"])
        .add_tag_attributes("td", &["align"])
        .add_tag_attributes("th", &["align"])
        .add_tag_attributes("input", &["type", "checked", "disabled"])
        .add_generic_attributes(["style"])
        .allowed_classes(allowed_classes)
        .clean(html)
        .to_string()
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_markdown_with_all_features() {
        let markdown = r#"
# Test Markdown

| Left | Center | Right |
|:-----|:------:|------:|
| 1    |   2    |     3 |

```rust
fn main() {
    println!("Hello, world!");
}
```

<div class="note">This is a note.</div>
<div class="warning">This is a warning.</div>
<div class="tip">This is a tip.</div>
"#;

        let options = default_markdown_options();
        let result = process_markdown(markdown, &options);
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        let html = result.unwrap();
        assert!(html.contains("table-responsive"));
        assert!(html.contains("language-rust"));
        assert!(html.contains("alert alert-info"));
        assert!(html.contains("alert alert-warning"));
        assert!(html.contains("alert alert-success"));
    }

    #[test]
    fn test_process_markdown_without_custom_blocks() {
        let markdown = "# Test\n<div class=\"note\">Note.</div>";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_comrak_options({
                let mut opts = Options::default();
                opts.extension.table = true;
                opts
            })
            .with_unsafe_html(true);

        let html = process_markdown(markdown, &options).unwrap();
        // The div should remain as-is (not converted to alert)
        assert!(html.contains("<div class=\"note\">"));
        assert!(!html.contains("alert"));
    }

    #[test]
    fn test_process_markdown_without_enhanced_tables() {
        let markdown = "| H1 | H2 |\n|---|---|\n| A | B |";
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_custom_blocks(false)
            .with_comrak_options({
                let mut opts = Options::default();
                opts.extension.table = true;
                opts
            });

        let html = process_markdown(markdown, &options).unwrap();
        assert!(!html.contains("table-responsive"));
        assert!(html.contains("<table>"));
    }

    #[test]
    fn test_validation_enhanced_tables_without_extension() {
        let options = MarkdownOptions::new()
            .with_enhanced_tables(true)
            .with_custom_blocks(false)
            .with_comrak_options({
                let mut opts = Options::default();
                opts.extension.table = false;
                opts
            });
        assert!(options.validate().is_err());
    }

    #[test]
    fn test_empty_content() {
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_custom_blocks(false);
        let html = process_markdown("", &options).unwrap();
        assert!(html.trim().is_empty());
    }

    #[test]
    fn test_no_features_enabled() {
        let markdown = "# Title\n\nPlain text.";
        let options = MarkdownOptions::new()
            .with_syntax_highlighting(false)
            .with_custom_blocks(false)
            .with_enhanced_tables(false);

        let html = process_markdown(markdown, &options).unwrap();
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("Plain text."));
    }

    #[test]
    fn test_sanitization_strips_script() {
        let markdown = "<script>alert('xss')</script>";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_unsafe_html(false);

        let html = process_markdown(markdown, &options).unwrap();
        assert!(
            !html.contains("<script>"),
            "Script tags should be stripped"
        );
    }

    #[test]
    fn test_sanitization_preserves_alerts() {
        let markdown = "<div class=\"note\">Important info.</div>";
        let options = MarkdownOptions::new()
            .with_custom_blocks(true)
            .with_enhanced_tables(false)
            .with_unsafe_html(false);

        let html = process_markdown(markdown, &options).unwrap();
        assert!(
            html.contains("alert alert-info"),
            "Alert divs should survive sanitization"
        );
    }

    #[test]
    fn test_input_too_large() {
        let options = MarkdownOptions::new()
            .with_max_input_size(10)
            .with_custom_blocks(false)
            .with_enhanced_tables(false);

        let result =
            process_markdown("a]".repeat(20).as_str(), &options);
        assert!(matches!(
            result,
            Err(MarkdownError::InputTooLarge { .. })
        ));
    }

    #[test]
    fn test_syntax_theme_customization() {
        let markdown = "```rust\nfn main() {}\n```";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_custom_theme("InspiredGitHub".to_string());

        let result = process_markdown(markdown, &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_custom_block_config() {
        let markdown = "<div class=\"note\">Custom styled.</div>";
        let config = CustomBlockConfig::new()
            .with_class(
                crate::extensions::CustomBlockType::Note,
                "my-note",
            )
            .with_title(
                crate::extensions::CustomBlockType::Note,
                "Heads up",
            );

        let options = MarkdownOptions::new()
            .with_custom_blocks(true)
            .with_enhanced_tables(false)
            .with_custom_block_config(config)
            .with_unsafe_html(true);

        let html = process_markdown(markdown, &options).unwrap();
        assert!(html.contains("my-note"));
        assert!(html.contains("Heads up:"));
    }

    #[test]
    fn test_builder_order_comrak_then_unsafe() {
        let options = MarkdownOptions::new()
            .with_comrak_options(Options::default())
            .with_unsafe_html(true);
        assert!(options.allow_unsafe_html);
    }

    #[test]
    fn test_comrak_options_syncs_unsafe() {
        let mut opts = Options::default();
        opts.render.r#unsafe = true;
        let options = MarkdownOptions::new().with_comrak_options(opts);
        assert!(options.allow_unsafe_html);
    }
}
