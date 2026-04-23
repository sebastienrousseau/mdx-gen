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
    collect_headings, enhance_table_nodes, process_custom_block_nodes,
    CustomBlockConfig, Heading,
};
use comrak::options::{Plugins, RenderPlugins};
use comrak::{Arena, Options};
use log::{debug, info, warn};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io::Write;
use std::sync::LazyLock;

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
    /// Enable automatic `id` attributes on headings for anchor links.
    ///
    /// When `Some(prefix)`, headings get `id="prefix-slug"` attributes.
    /// Use `Some("")` for bare `id="slug"` without a prefix.
    /// `None` disables header IDs (default).
    pub header_ids: Option<String>,
    /// Optional extensions to the default HTML sanitizer allow-list.
    ///
    /// When `None`, the cached default sanitizer is used — the hot
    /// path. When `Some`, a fresh `ammonia::Builder` is constructed
    /// per call that merges the defaults with the extras declared in
    /// [`SanitizerConfig`].
    pub sanitizer_config: Option<SanitizerConfig>,
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
            header_ids: None,
            sanitizer_config: None,
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

    /// Enables automatic `id` attributes on headings.
    ///
    /// Pass `""` for bare slugs, or a prefix like `"user-content-"`
    /// to namespace them (GitHub-style).
    pub fn with_header_ids(
        mut self,
        prefix: impl Into<String>,
    ) -> Self {
        self.header_ids = Some(prefix.into());
        self
    }

    /// Extends the HTML sanitizer allow-list.
    ///
    /// Setting this disables the cached default sanitizer for calls
    /// made with these options — a fresh `ammonia::Builder` is
    /// constructed per call that merges the defaults with the extras.
    /// Only used when `allow_unsafe_html` is `false`.
    pub fn with_sanitizer_config(
        mut self,
        config: SanitizerConfig,
    ) -> Self {
        self.sanitizer_config = Some(config);
        self
    }

    /// Validates that options are internally consistent and that
    /// every string the pipeline will splice into HTML is well-
    /// formed.
    ///
    /// Uses [`commons::validation::Validator`] so every failing
    /// check is reported in one pass — callers get the full list
    /// of problems, not just the first. Each entry in the returned
    /// `Vec` is `(field_name, ValidationError)`.
    ///
    /// # Checks
    ///
    /// 1. `enhanced_tables` requires `comrak.extension.table`.
    /// 2. `syntax_theme`, if set, must name a theme bundled with
    ///    syntect (feature-gated).
    /// 3. `syntax_theme` set but `enable_syntax_highlighting =
    ///    false` is a silent no-op — rejected so the mistake
    ///    surfaces.
    /// 4. `sanitizer_config` set but `allow_unsafe_html = true`
    ///    skips sanitization entirely — rejected.
    /// 5. `header_ids` prefix must not contain whitespace or any
    ///    of `" ' < > & =` (would break the emitted `id="…"`).
    /// 6. `sanitizer_config.extra_tags` / `extra_tag_attributes`
    ///    keys must be valid HTML names; attribute lists must
    ///    contain only valid HTML names.
    /// 7. `sanitizer_config.extra_generic_attributes` must be
    ///    valid HTML names.
    /// 8. `sanitizer_config.extra_allowed_classes` keys must be
    ///    valid HTML names; class values must be non-empty and
    ///    free of whitespace / quote characters.
    /// 9. `custom_block_config` override values must be non-empty
    ///    and free of whitespace / quote characters (class
    ///    overrides) or non-empty (title overrides).
    ///
    /// # Errors
    ///
    /// Returns `Err(errors)` when any check fails. The pipeline
    /// converts the list into a single
    /// [`MarkdownError::InvalidOptionsError`] via the `From` impl
    /// in [`crate::error`].
    pub fn validate(
        &self,
    ) -> Result<(), Vec<(String, commons::validation::ValidationError)>>
    {
        use commons::validation::{ValidationError, Validator};

        let mut v = Validator::new();

        // 1. enhanced_tables requires comrak.extension.table
        v.check("enable_enhanced_tables", || {
            if self.enable_enhanced_tables
                && !self.comrak_options.extension.table
            {
                Err(ValidationError::Custom(
                    "enhanced_tables = true requires comrak_options.extension.table = true"
                        .into(),
                ))
            } else {
                Ok(())
            }
        });

        // 2. syntax_theme must be a bundled theme.
        #[cfg(feature = "syntax_highlighting")]
        v.check("syntax_theme", || {
            if let Some(ref theme) = self.syntax_theme {
                let available =
                    crate::highlight::SyntectAdapter::available_themes(
                    );
                if !available.contains(&theme.as_str()) {
                    return Err(ValidationError::NotInSet {
                        allowed: available
                            .iter()
                            .map(|s| (*s).to_string())
                            .collect(),
                    });
                }
            }
            Ok(())
        });

        // 3. syntax_theme + highlighter disabled is a no-op.
        v.check("syntax_theme", || {
            if !self.enable_syntax_highlighting
                && self.syntax_theme.is_some()
            {
                Err(ValidationError::Custom(
                    "syntax_theme is set but enable_syntax_highlighting = false (theme would be ignored)"
                        .into(),
                ))
            } else {
                Ok(())
            }
        });

        // 4. sanitizer_config + unsafe_html is a no-op (sanitizer
        //    never runs when unsafe_html is true).
        v.check("sanitizer_config", || {
            if self.allow_unsafe_html && self.sanitizer_config.is_some()
            {
                Err(ValidationError::Custom(
                    "sanitizer_config is set but allow_unsafe_html = true (sanitizer is skipped)"
                        .into(),
                ))
            } else {
                Ok(())
            }
        });

        // 5. header_ids prefix — no chars that would escape out of
        //    the `id="…"` attribute.
        v.check("header_ids", || {
            if let Some(ref prefix) = self.header_ids {
                if let Some(c) = prefix.chars().find(|c| {
                    c.is_whitespace()
                        || matches!(
                            c,
                            '"' | '\''
                                | '<'
                                | '>'
                                | '&'
                                | '='
                        )
                }) {
                    return Err(ValidationError::InvalidPattern {
                        pattern: format!(
                            "no whitespace or HTML-special chars (found {c:?})"
                        ),
                    });
                }
            }
            Ok(())
        });

        // 6. & 7. & 8. — SanitizerConfig.
        if let Some(ref cfg) = self.sanitizer_config {
            check_tag_list(
                &mut v,
                "sanitizer_config.extra_tags",
                &cfg.extra_tags,
            );
            check_tag_attr_map(
                &mut v,
                "sanitizer_config.extra_tag_attributes",
                &cfg.extra_tag_attributes,
            );
            check_attr_list(
                &mut v,
                "sanitizer_config.extra_generic_attributes",
                &cfg.extra_generic_attributes,
            );
            check_allowed_classes_map(
                &mut v,
                "sanitizer_config.extra_allowed_classes",
                &cfg.extra_allowed_classes,
            );
        }

        // 9. CustomBlockConfig override values.
        for (block_type, class) in
            &self.custom_block_config.class_overrides
        {
            let field = format!(
                "custom_block_config.class_overrides[{block_type:?}]"
            );
            let c = class.clone();
            v.check(&field, move || {
                if c.is_empty() {
                    return Err(ValidationError::Empty);
                }
                if let Some(ch) = c.chars().find(|c| {
                    c.is_whitespace() || matches!(c, '"' | '\'')
                }) {
                    return Err(ValidationError::InvalidPattern {
                        pattern: format!(
                            "non-empty, no whitespace or quotes (found {ch:?})"
                        ),
                    });
                }
                Ok(())
            });
        }
        for (block_type, title) in
            &self.custom_block_config.title_overrides
        {
            let field = format!(
                "custom_block_config.title_overrides[{block_type:?}]"
            );
            let t = title.clone();
            v.check(&field, move || {
                if t.trim().is_empty() {
                    return Err(ValidationError::Empty);
                }
                Ok(())
            });
        }

        v.finish()
    }
}

/// ASCII-alphabetic first character, ASCII alphanumeric + `-` + `_`
/// after. Empty strings rejected. Conservative shape for HTML tag
/// and attribute names.
fn is_html_name(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn check_tag_list(
    v: &mut commons::validation::Validator,
    field: &str,
    tags: &[String],
) {
    for (i, tag) in tags.iter().enumerate() {
        let f = format!("{field}[{i}]");
        let t = tag.clone();
        v.check(&f, move || {
            if !is_html_name(&t) {
                Err(commons::validation::ValidationError::InvalidPattern {
                    pattern: format!(
                        "valid HTML tag name (got {t:?})"
                    ),
                })
            } else {
                Ok(())
            }
        });
    }
}

fn check_attr_list(
    v: &mut commons::validation::Validator,
    field: &str,
    attrs: &[String],
) {
    for (i, attr) in attrs.iter().enumerate() {
        let f = format!("{field}[{i}]");
        let a = attr.clone();
        v.check(&f, move || {
            if !is_html_name(&a) {
                Err(commons::validation::ValidationError::InvalidPattern {
                    pattern: format!(
                        "valid HTML attribute name (got {a:?})"
                    ),
                })
            } else {
                Ok(())
            }
        });
    }
}

fn check_tag_attr_map(
    v: &mut commons::validation::Validator,
    field: &str,
    map: &HashMap<String, Vec<String>>,
) {
    for (tag, attrs) in map {
        let f_tag = format!("{field}.{tag}");
        let t = tag.clone();
        v.check(&f_tag, move || {
            if !is_html_name(&t) {
                Err(commons::validation::ValidationError::InvalidPattern {
                    pattern: format!(
                        "valid HTML tag name (got {t:?})"
                    ),
                })
            } else {
                Ok(())
            }
        });
        check_attr_list(v, &f_tag, attrs);
    }
}

fn check_allowed_classes_map(
    v: &mut commons::validation::Validator,
    field: &str,
    map: &HashMap<String, Vec<String>>,
) {
    for (tag, classes) in map {
        let f_tag = format!("{field}.{tag}");
        let t = tag.clone();
        v.check(&f_tag, move || {
            if !is_html_name(&t) {
                Err(commons::validation::ValidationError::InvalidPattern {
                    pattern: format!(
                        "valid HTML tag name (got {t:?})"
                    ),
                })
            } else {
                Ok(())
            }
        });
        for (i, class) in classes.iter().enumerate() {
            let f = format!("{f_tag}[{i}]");
            let c = class.clone();
            v.check(&f, move || {
                if c.is_empty() {
                    return Err(
                        commons::validation::ValidationError::Empty,
                    );
                }
                if let Some(ch) = c.chars().find(|c| {
                    c.is_whitespace() || matches!(c, '"' | '\'')
                }) {
                    return Err(
                        commons::validation::ValidationError::InvalidPattern {
                            pattern: format!(
                                "non-empty, no whitespace or quotes (got {ch:?})"
                            ),
                        },
                    );
                }
                Ok(())
            });
        }
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
            .field("header_ids", &self.header_ids)
            .field("sanitizer_config", &self.sanitizer_config)
            .finish()
    }
}

// ── Sanitizer configuration ─────────────────────────────────────────

/// User-supplied extensions to the default HTML sanitizer allow-list.
///
/// Each field is additive: values here are merged on top of the
/// defaults that ship with `mdx-gen`. Wire an instance into
/// [`MarkdownOptions::with_sanitizer_config`].
#[derive(Debug, Clone, Default)]
pub struct SanitizerConfig {
    /// Additional tags to allow (beyond the defaults).
    pub extra_tags: Vec<String>,
    /// Additional attributes per tag, in the form `tag -> attrs`.
    pub extra_tag_attributes: HashMap<String, Vec<String>>,
    /// Additional generic attributes that may appear on any tag.
    pub extra_generic_attributes: Vec<String>,
    /// Additional allowed class values per tag.
    pub extra_allowed_classes: HashMap<String, Vec<String>>,
}

impl SanitizerConfig {
    /// Creates a new, empty config (equivalent to `Default`).
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one extra tag to the allow-list.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.extra_tags.push(tag.into());
        self
    }

    /// Adds one extra attribute for a specific tag.
    pub fn with_tag_attribute(
        mut self,
        tag: impl Into<String>,
        attr: impl Into<String>,
    ) -> Self {
        self.extra_tag_attributes
            .entry(tag.into())
            .or_default()
            .push(attr.into());
        self
    }

    /// Adds one extra generic attribute (applies to any allowed tag).
    pub fn with_generic_attribute(
        mut self,
        attr: impl Into<String>,
    ) -> Self {
        self.extra_generic_attributes.push(attr.into());
        self
    }

    /// Adds one extra allowed class value for a specific tag.
    pub fn with_allowed_class(
        mut self,
        tag: impl Into<String>,
        class: impl Into<String>,
    ) -> Self {
        self.extra_allowed_classes
            .entry(tag.into())
            .or_default()
            .push(class.into());
        self
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
    let mut buf: Vec<u8> = Vec::new();
    process_markdown_to_writer(content, &mut buf, options)?;
    // comrak and ammonia both emit valid UTF-8, so this should never
    // fail in practice — but surface the error rather than panic.
    String::from_utf8(buf).map_err(|e| {
        MarkdownError::RenderError(format!(
            "non-UTF-8 output from pipeline: {e}"
        ))
    })
}

/// Streams processed HTML directly to a `Write` sink.
///
/// Semantically equivalent to [`process_markdown`], but avoids one
/// intermediate allocation when callers already have a `Write`
/// destination (a file, a buffered network writer, a template engine).
/// The comrak render stage still produces a `String` internally — the
/// 1 MiB default input cap means end-to-end streaming would add API
/// surface without meaningful memory savings.
///
/// # Errors
///
/// Returns [`MarkdownError::IoError`] if the writer fails. All other
/// error conditions mirror [`process_markdown`].
pub fn process_markdown_to_writer<W: Write>(
    content: &str,
    writer: &mut W,
    options: &MarkdownOptions,
) -> Result<(), MarkdownError> {
    pipeline(content, writer, options, None)
}

/// Processes Markdown and returns both the rendered HTML and a
/// document-order table of contents.
///
/// Each [`Heading`] carries the level, the plain-text content, and
/// the anchor id that comrak emits for that heading. To make those
/// ids actually appear in the rendered HTML, set
/// [`MarkdownOptions::header_ids`] (the same prefix is reflected in
/// `Heading::id`).
///
/// # Errors
///
/// Mirrors [`process_markdown`].
pub fn process_markdown_with_toc(
    content: &str,
    options: &MarkdownOptions,
) -> Result<(String, Vec<Heading>), MarkdownError> {
    let mut buf: Vec<u8> = Vec::new();
    let mut toc = Vec::new();
    pipeline(content, &mut buf, options, Some(&mut toc))?;
    let html = String::from_utf8(buf).map_err(|e| {
        MarkdownError::RenderError(format!(
            "non-UTF-8 output from pipeline: {e}"
        ))
    })?;
    Ok((html, toc))
}

/// Streams processed HTML to `writer` and returns the table of
/// contents collected during the AST walk.
///
/// Same shape as [`process_markdown_to_writer`] but with the toc
/// metadata returned alongside the IO result.
pub fn process_markdown_with_toc_to_writer<W: Write>(
    content: &str,
    writer: &mut W,
    options: &MarkdownOptions,
) -> Result<Vec<Heading>, MarkdownError> {
    let mut toc = Vec::new();
    pipeline(content, writer, options, Some(&mut toc))?;
    Ok(toc)
}

/// Extracts plain-text content from Markdown, stripping all
/// formatting and markup.
///
/// This is useful for building search indexes, generating
/// plain-text excerpts, or calculating reading time.
///
/// # Errors
///
/// Returns a [`MarkdownError`] if input exceeds the size limit.
pub fn process_markdown_to_plain_text(
    content: &str,
    options: &MarkdownOptions,
) -> Result<String, MarkdownError> {
    if options.max_input_size > 0
        && content.len() > options.max_input_size
    {
        return Err(MarkdownError::InputTooLarge {
            size: content.len(),
            limit: options.max_input_size,
        });
    }

    let arena = Arena::new();
    let root = comrak::parse_document(
        &arena,
        content,
        &options.comrak_options,
    );

    Ok(crate::extensions::collect_all_text(root))
}

/// Internal pipeline shared by every public entry point. When
/// `toc_out` is `Some`, headings are collected during the AST pass
/// using [`collect_headings`].
fn pipeline<W: Write>(
    content: &str,
    writer: &mut W,
    options: &MarkdownOptions,
    toc_out: Option<&mut Vec<Heading>>,
) -> Result<(), MarkdownError> {
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
    if let Err(errors) = options.validate() {
        for (field, err) in &errors {
            warn!("Invalid MarkdownOptions.{field}: {err}");
        }
        return Err(MarkdownError::from(errors));
    }

    // ── 2. Build comrak options ─────────────────────────────────
    let mut comrak_opts = options.comrak_options.clone();
    // Always enable unsafe for internal rendering — we sanitize
    // at the end if the caller wants safety.
    comrak_opts.render.r#unsafe = true;

    // Wire header_ids into comrak's extension
    if let Some(ref prefix) = options.header_ids {
        comrak_opts.extension.header_id_prefix = Some(prefix.clone());
    }

    // ── 3. Parse → AST ─────────────────────────────────────────
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, content, &comrak_opts);

    // ── 4. AST transforms ───────────────────────────────────────
    if options.enable_custom_blocks {
        debug!("Processing custom blocks at AST level");
        process_custom_block_nodes(root, &options.custom_block_config);
    }
    if let Some(toc) = toc_out {
        // Collect after custom-block transforms (which may add
        // structural divs) but before table enhancement (which
        // detaches table nodes — irrelevant to headings, but keeps
        // the heading walk on a stable subtree).
        debug!("Collecting headings for table of contents");
        *toc = collect_headings(root, options.header_ids.as_deref());
    }
    if options.enable_enhanced_tables {
        debug!("Enhancing tables at AST level");
        enhance_table_nodes(root, &arena, &comrak_opts);
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

    let mut html = String::new();
    comrak::format_html_with_plugins(
        root,
        &comrak_opts,
        &mut html,
        &plugins,
    )
    .map_err(|e| MarkdownError::RenderError(e.to_string()))?;

    // ── 6. Sanitize and emit ────────────────────────────────────
    if options.allow_unsafe_html {
        writer.write_all(html.as_bytes())?;
    } else {
        debug!("Sanitizing HTML output");
        sanitize_html_to_writer(
            &html,
            writer,
            options.sanitizer_config.as_ref(),
        )?;
    }

    info!("Markdown processing completed successfully");
    Ok(())
}

// ── HTML sanitization ───────────────────────────────────────────────

/// Pre-generated `language-*` class names for code elements,
/// allocated once and reused across all sanitize calls.
static CODE_LANG_CLASSES: LazyLock<HashSet<String>> =
    LazyLock::new(|| {
        [
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
        ]
        .iter()
        .map(|lang| format!("language-{lang}"))
        .collect()
    });

/// Applies the default sanitizer allow-list to a `Builder<'a>`.
///
/// Kept separate so the cached default builder and any per-call
/// builder (used when the caller supplies a [`SanitizerConfig`])
/// share one source of truth for the base policy. All strings
/// threaded through here are `'static`, which coerces into any `'a`.
fn configure_default_sanitizer<'a>(builder: &mut ammonia::Builder<'a>) {
    let code_class_refs: HashSet<&'static str> =
        CODE_LANG_CLASSES.iter().map(|s| s.as_str()).collect();

    let mut allowed_classes: HashMap<
        &'static str,
        HashSet<&'static str>,
    > = HashMap::new();

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
    allowed_classes.insert("table", ["table"].into_iter().collect());
    allowed_classes.insert(
        "td",
        ["text-left", "text-center", "text-right"]
            .into_iter()
            .collect(),
    );
    allowed_classes.insert("code", code_class_refs);

    builder
        .add_tags(["div", "pre", "code", "span", "input"])
        .add_tag_attributes("div", &["role", "id"])
        .add_tag_attributes("td", &["align"])
        .add_tag_attributes("th", &["align"])
        .add_tag_attributes("input", &["type", "checked", "disabled"])
        .add_tag_attributes("h1", &["id"])
        .add_tag_attributes("h2", &["id"])
        .add_tag_attributes("h3", &["id"])
        .add_tag_attributes("h4", &["id"])
        .add_tag_attributes("h5", &["id"])
        .add_tag_attributes("h6", &["id"])
        .add_tag_attributes("a", &["id"])
        .allowed_classes(allowed_classes)
        // Syntect's class-based highlighter emits open-ended class
        // names on <span> (one per grammar scope). Whitelisting them
        // individually is impractical, so we allow `class` on <span>
        // with unrestricted values — class attributes are CSS hooks,
        // they cannot execute script.
        .add_tag_attributes("span", &["class", "data-math-style"]);
}

/// Pre-configured ammonia sanitizer, built once and reused across
/// every default-config call to the sanitizer.
///
/// Why: `ammonia::Builder` is relatively expensive to construct — it
/// allocates several tag/attribute hash sets and the allowed-classes
/// map. Since the default configuration is static (all `'static`
/// strs), we build a single `Builder<'static>` behind a `LazyLock`
/// and call `clean(&self, …)` on it repeatedly.
static SANITIZE_BUILDER: LazyLock<ammonia::Builder<'static>> =
    LazyLock::new(|| {
        let mut builder = ammonia::Builder::default();
        configure_default_sanitizer(&mut builder);
        builder
    });

/// Writes sanitized HTML to the given writer.
///
/// Uses the cached default sanitizer when `cfg` is `None` (hot path).
/// When `cfg` is `Some`, builds a fresh `Builder` that merges the
/// defaults with the caller's extras — per-call cost, but scoped to
/// the uncommon case.
fn sanitize_html_to_writer<W: Write>(
    html: &str,
    writer: &mut W,
    cfg: Option<&SanitizerConfig>,
) -> std::io::Result<()> {
    match cfg {
        None => SANITIZE_BUILDER.clean(html).write_to(writer),
        Some(custom) => {
            build_custom_sanitizer(custom).clean(html).write_to(writer)
        }
    }
}

/// Builds a one-shot sanitizer that layers `cfg`'s extras over the
/// default allow-list. Lifetime is tied to `cfg` since the extras
/// are `String`-owned on the caller side.
fn build_custom_sanitizer(
    cfg: &SanitizerConfig,
) -> ammonia::Builder<'_> {
    let mut builder = ammonia::Builder::default();
    configure_default_sanitizer(&mut builder);

    // ammonia forbids a tag from appearing in both `tag_attributes`
    // (with "class") and `allowed_classes`. The default config
    // grants <span> open `class` via tag_attributes so syntect's
    // class-based highlighter survives sanitization. If the caller
    // is now restricting classes for any of those tags via
    // SanitizerConfig, swap them out of permissive mode before
    // adding the whitelist.
    for tag in cfg.extra_allowed_classes.keys() {
        builder.rm_tag_attributes(tag.as_str(), &["class"]);
    }

    if !cfg.extra_tags.is_empty() {
        builder.add_tags(cfg.extra_tags.iter().map(String::as_str));
    }
    for (tag, attrs) in &cfg.extra_tag_attributes {
        builder.add_tag_attributes(
            tag.as_str(),
            attrs.iter().map(String::as_str),
        );
    }
    if !cfg.extra_generic_attributes.is_empty() {
        builder.add_generic_attributes(
            cfg.extra_generic_attributes.iter().map(String::as_str),
        );
    }
    for (tag, classes) in &cfg.extra_allowed_classes {
        builder.add_allowed_classes(
            tag.as_str(),
            classes.iter().map(String::as_str),
        );
    }
    builder
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CustomBlockType;

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
        let errors = options.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|(f, _)| f == "enable_enhanced_tables"));
    }

    #[test]
    fn test_validation_default_options_pass() {
        // Defaults should pass every check in validate() — the
        // suite is "tight" but not hostile to normal config.
        // Note: defaults have enable_enhanced_tables = true but
        // Options::default() has extension.table = false, so this
        // catches check #1 by design. Enable the extension to see
        // the all-green path.
        let mut comrak = Options::default();
        comrak.extension.table = true;
        let options =
            MarkdownOptions::new().with_comrak_options(comrak);
        assert!(
            options.validate().is_ok(),
            "{:?}",
            options.validate().unwrap_err()
        );
    }

    #[test]
    fn test_validation_unknown_syntax_theme() {
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_custom_blocks(false)
            .with_custom_theme("no-such-theme-exists".into());
        let errors = options.validate().unwrap_err();
        assert!(
            errors.iter().any(|(f, _)| f == "syntax_theme"),
            "expected syntax_theme failure, got {errors:?}"
        );
    }

    #[test]
    fn test_validation_theme_without_highlighter_disabled() {
        // syntax_theme set + syntax_highlighting = false is a
        // silent no-op — rejected.
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_custom_blocks(false)
            .with_syntax_highlighting(false)
            .with_custom_theme("base16-ocean.dark".into());
        let errors = options.validate().unwrap_err();
        assert!(errors.iter().any(|(f, _)| f == "syntax_theme"));
    }

    #[test]
    fn test_validation_sanitizer_with_unsafe_html() {
        // sanitizer_config set + allow_unsafe_html = true skips
        // sanitization entirely — reject the silent no-op.
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_custom_blocks(false)
            .with_unsafe_html(true)
            .with_sanitizer_config(
                SanitizerConfig::new().with_tag("main"),
            );
        let errors = options.validate().unwrap_err();
        assert!(errors.iter().any(|(f, _)| f == "sanitizer_config"));
    }

    #[test]
    fn test_validation_header_ids_bad_chars() {
        for bad in [
            "user content ", // whitespace
            "quo\"te-",
            "ang<le-",
            "amp&-",
        ] {
            let options = MarkdownOptions::new()
                .with_enhanced_tables(false)
                .with_custom_blocks(false)
                .with_header_ids(bad);
            let errors = options.validate().unwrap_err();
            assert!(
                errors.iter().any(|(f, _)| f == "header_ids"),
                "expected header_ids failure for {bad:?}, got {errors:?}"
            );
        }
    }

    #[test]
    fn test_validation_header_ids_clean_prefix_ok() {
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_custom_blocks(false)
            .with_header_ids("user-content-");
        assert!(options.validate().is_ok());
    }

    #[test]
    fn test_validation_sanitizer_extra_tag_invalid() {
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_custom_blocks(false)
            .with_unsafe_html(false)
            .with_sanitizer_config(
                SanitizerConfig::new().with_tag("has space"),
            );
        let errors = options.validate().unwrap_err();
        assert!(
            errors
                .iter()
                .any(|(f, _)| f
                    .starts_with("sanitizer_config.extra_tags"))
        );
    }

    #[test]
    fn test_validation_sanitizer_extra_generic_attribute_invalid() {
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_custom_blocks(false)
            .with_unsafe_html(false)
            .with_sanitizer_config(
                SanitizerConfig::new().with_generic_attribute(""),
            );
        let errors = options.validate().unwrap_err();
        assert!(errors.iter().any(|(f, _)| f
            .starts_with("sanitizer_config.extra_generic_attributes")));
    }

    #[test]
    fn test_validation_sanitizer_allowed_class_with_whitespace() {
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_custom_blocks(false)
            .with_unsafe_html(false)
            .with_sanitizer_config(
                SanitizerConfig::new()
                    .with_allowed_class("span", "has space"),
            );
        let errors = options.validate().unwrap_err();
        assert!(errors.iter().any(|(f, _)| f
            .starts_with("sanitizer_config.extra_allowed_classes")));
    }

    #[test]
    fn test_validation_custom_block_class_override_empty() {
        let cfg = CustomBlockConfig::new()
            .with_class(CustomBlockType::Note, "");
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_custom_block_config(cfg);
        let errors = options.validate().unwrap_err();
        assert!(errors.iter().any(|(f, _)| f
            .starts_with("custom_block_config.class_overrides")));
    }

    #[test]
    fn test_validation_custom_block_title_override_blank() {
        let cfg = CustomBlockConfig::new()
            .with_title(CustomBlockType::Warning, "   ");
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_custom_block_config(cfg);
        let errors = options.validate().unwrap_err();
        assert!(errors.iter().any(|(f, _)| f
            .starts_with("custom_block_config.title_overrides")));
    }

    #[test]
    fn test_sanitizer_config_applies_extra_tag_attribute() {
        // Drives build_custom_sanitizer past validation with a
        // tag-specific attribute add — exercises the
        // add_tag_attributes branch in the sanitiser factory.
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_unsafe_html(false)
            .with_sanitizer_config(
                SanitizerConfig::new()
                    .with_tag("section")
                    .with_tag_attribute("section", "data-foo"),
            );

        let md = r#"<section data-foo="bar">hello</section>"#;
        let html = process_markdown(md, &options).unwrap();
        assert!(html.contains("<section"));
        assert!(html.contains("data-foo=\"bar\""));
    }

    #[test]
    fn test_sanitizer_config_applies_extra_generic_attribute() {
        // Drives build_custom_sanitizer past validation with a
        // generic-attr add — exercises the add_generic_attributes
        // branch.
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_unsafe_html(false)
            .with_sanitizer_config(
                SanitizerConfig::new().with_generic_attribute("data-x"),
            );

        let md = r#"<p data-x="v">hi</p>"#;
        let html = process_markdown(md, &options).unwrap();
        assert!(html.contains("data-x=\"v\""));
    }

    #[test]
    fn test_sanitizer_config_with_tag_attribute_direct() {
        // The builder method had zero direct coverage.
        let cfg = SanitizerConfig::new()
            .with_tag_attribute("div", "role")
            .with_tag_attribute("div", "id");
        let attrs = cfg
            .extra_tag_attributes
            .get("div")
            .expect("div should exist");
        assert_eq!(attrs, &vec!["role".to_string(), "id".to_string()]);
    }

    #[test]
    fn test_validation_sanitizer_tag_attr_invalid_tag() {
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_custom_blocks(false)
            .with_unsafe_html(false)
            .with_sanitizer_config(
                SanitizerConfig::new()
                    .with_tag_attribute("has space", "id"),
            );
        let errors = options.validate().unwrap_err();
        assert!(errors.iter().any(|(f, _)| f
            .starts_with("sanitizer_config.extra_tag_attributes")));
    }

    #[test]
    fn test_validation_sanitizer_tag_attr_invalid_attr_name() {
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_custom_blocks(false)
            .with_unsafe_html(false)
            .with_sanitizer_config(
                SanitizerConfig::new().with_tag_attribute("div", ""),
            );
        let errors = options.validate().unwrap_err();
        assert!(errors.iter().any(|(f, _)| f
            .starts_with("sanitizer_config.extra_tag_attributes")));
    }

    #[test]
    fn test_validation_sanitizer_allowed_class_invalid_tag() {
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_custom_blocks(false)
            .with_unsafe_html(false)
            .with_sanitizer_config(
                SanitizerConfig::new()
                    .with_allowed_class("bad tag", "foo"),
            );
        let errors = options.validate().unwrap_err();
        assert!(errors.iter().any(|(f, _)| f
            .starts_with("sanitizer_config.extra_allowed_classes")));
    }

    #[test]
    fn test_validation_sanitizer_allowed_class_empty() {
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_custom_blocks(false)
            .with_unsafe_html(false)
            .with_sanitizer_config(
                SanitizerConfig::new().with_allowed_class("span", ""),
            );
        let errors = options.validate().unwrap_err();
        assert!(errors.iter().any(|(f, _)| f
            .starts_with("sanitizer_config.extra_allowed_classes")));
    }

    #[test]
    fn test_validation_custom_block_class_override_whitespace() {
        let cfg = CustomBlockConfig::new()
            .with_class(CustomBlockType::Note, "bad class");
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_custom_block_config(cfg);
        let errors = options.validate().unwrap_err();
        assert!(errors.iter().any(|(f, _)| f
            .starts_with("custom_block_config.class_overrides")));
    }

    #[test]
    fn test_toc_extracts_image_title() {
        // Exercises the NodeValue::Image branch of `extract_text`.
        let md = "# See ![alt](logo.png \"Logo Title\") here\n";
        let options = MarkdownOptions::new()
            .with_enhanced_tables(false)
            .with_custom_blocks(false);
        let (_html, toc) =
            process_markdown_with_toc(md, &options).unwrap();
        assert_eq!(toc.len(), 1);
        // Image title text should make it into the heading's plain text.
        assert!(
            toc[0].text.contains("Logo Title")
                || toc[0].text.contains("alt"),
            "expected image title/alt in: {:?}",
            toc[0].text
        );
    }

    #[test]
    fn test_plain_text_soft_break_inserts_space() {
        // Two text nodes joined by a soft break should get a space
        // between them (covers SoftBreak/LineBreak arm in
        // collect_all_text).
        let md = "one\ntwo\nthree\n";
        let text = process_markdown_to_plain_text(
            md,
            &MarkdownOptions::default(),
        )
        .unwrap();
        assert_eq!(text, "one two three");
    }

    #[test]
    fn test_plain_text_image_title_included() {
        // Image titles end up in plain-text output too.
        let md = "Caption: ![alt](x.png \"Title Here\")\n";
        let text = process_markdown_to_plain_text(
            md,
            &MarkdownOptions::default(),
        )
        .unwrap();
        // Plain text should at minimum keep the surrounding caption.
        assert!(text.contains("Caption:"));
    }

    #[test]
    fn test_validation_reports_all_failures_at_once() {
        // Three independent violations in one options instance.
        // The validator must collect all of them, not bail on the
        // first.
        let cfg = CustomBlockConfig::new()
            .with_class(CustomBlockType::Note, "");
        let options = MarkdownOptions::new()
            .with_enhanced_tables(true) // ← 1: needs comrak.extension.table
            .with_header_ids("a b") // ← 2: whitespace
            .with_custom_block_config(cfg) // ← 3: empty override
            .with_comrak_options({
                let mut opts = Options::default();
                opts.extension.table = false;
                opts
            });
        let errors = options.validate().unwrap_err();
        assert!(
            errors.len() >= 3,
            "expected 3+ errors, got {errors:?}"
        );
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

    #[test]
    fn test_markdown_options_debug_impl() {
        let options = MarkdownOptions::new()
            .with_custom_blocks(true)
            .with_syntax_highlighting(false)
            .with_enhanced_tables(true)
            .with_custom_theme("InspiredGitHub".to_string())
            .with_unsafe_html(false)
            .with_max_input_size(2048);

        let debug_output = format!("{:?}", options);
        assert!(debug_output.contains("MarkdownOptions"));
        assert!(debug_output.contains("enable_custom_blocks: true"));
        assert!(
            debug_output.contains("enable_syntax_highlighting: false")
        );
        assert!(debug_output.contains("enable_enhanced_tables: true"));
        assert!(debug_output.contains("InspiredGitHub"));
        assert!(debug_output.contains("allow_unsafe_html: false"));
        assert!(debug_output.contains("max_input_size: 2048"));
    }

    #[test]
    fn test_header_ids() {
        let markdown = "# Hello World\n## Sub Section";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_header_ids("")
            .with_unsafe_html(true);

        let html = process_markdown(markdown, &options).unwrap();
        assert!(
            html.contains("id=\"hello-world\""),
            "H1 should have id attribute: {html}"
        );
        assert!(
            html.contains("id=\"sub-section\""),
            "H2 should have id attribute: {html}"
        );
    }

    #[test]
    fn test_header_ids_with_prefix() {
        let markdown = "# Title";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_header_ids("user-content-")
            .with_unsafe_html(true);

        let html = process_markdown(markdown, &options).unwrap();
        assert!(
            html.contains("id=\"user-content-title\""),
            "Should have prefixed id: {html}"
        );
    }

    #[test]
    fn test_header_ids_survive_sanitization() {
        let markdown = "# Hello World";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_header_ids("")
            .with_unsafe_html(false);

        let html = process_markdown(markdown, &options).unwrap();
        assert!(
            html.contains("id=\"hello-world\""),
            "Header id should survive ammonia sanitization: {html}"
        );
    }

    #[test]
    fn test_ast_table_enhancement() {
        let markdown =
            "| H1 | H2 |\n|:---|---:|\n| L | R |\n\nParagraph\n\n| A | B |\n|---|---|\n| C | D |";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_comrak_options({
                let mut opts = Options::default();
                opts.extension.table = true;
                opts
            })
            .with_unsafe_html(true);

        let html = process_markdown(markdown, &options).unwrap();
        // Both tables should be wrapped
        assert_eq!(
            html.matches("table-responsive").count(),
            2,
            "Both tables should get responsive wrapper: {html}"
        );
        assert!(
            html.contains("text-right"),
            "Right-aligned cells should have class"
        );
    }

    // ── Streaming API ───────────────────────────────────────────

    #[test]
    fn test_process_markdown_to_writer_matches_string_variant() {
        let markdown = "# Title\n\nParagraph with *emphasis*.";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_syntax_highlighting(false);

        let as_string = process_markdown(markdown, &options).unwrap();

        let mut buf: Vec<u8> = Vec::new();
        process_markdown_to_writer(markdown, &mut buf, &options)
            .unwrap();
        let as_bytes = String::from_utf8(buf).unwrap();

        assert_eq!(
            as_string, as_bytes,
            "writer variant must produce byte-identical output"
        );
    }

    #[test]
    fn test_process_markdown_to_writer_sanitizes() {
        let markdown = "<script>alert('xss')</script>\n\n# Safe";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_unsafe_html(false);

        let mut buf: Vec<u8> = Vec::new();
        process_markdown_to_writer(markdown, &mut buf, &options)
            .unwrap();
        let html = String::from_utf8(buf).unwrap();
        assert!(!html.contains("<script>"));
        assert!(html.contains("<h1>Safe</h1>"));
    }

    #[test]
    fn test_process_markdown_to_writer_propagates_io_error() {
        struct AlwaysFails;
        impl Write for AlwaysFails {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "nope",
                ))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false);
        let err = process_markdown_to_writer(
            "# hi",
            &mut AlwaysFails,
            &options,
        )
        .unwrap_err();
        assert!(matches!(err, MarkdownError::IoError(_)));
    }

    // ── SanitizerConfig ─────────────────────────────────────────

    #[test]
    fn test_sanitizer_config_allows_extra_tag() {
        // <main> is not in ammonia's default tag allow-list and is
        // not added by our defaults, so it's stripped to text unless
        // the SanitizerConfig extends the list.
        let markdown = "<main>wrapper</main>";

        let strict = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_unsafe_html(false);
        let stripped = process_markdown(markdown, &strict).unwrap();
        assert!(
            !stripped.contains("<main>"),
            "default sanitizer drops <main>: {stripped}"
        );

        let extended = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_unsafe_html(false)
            .with_sanitizer_config(
                SanitizerConfig::new().with_tag("main"),
            );
        let kept = process_markdown(markdown, &extended).unwrap();
        assert!(
            kept.contains("<main>wrapper</main>"),
            "extended sanitizer keeps <main>: {kept}"
        );
    }

    #[test]
    fn test_sanitizer_config_adds_allowed_class() {
        let markdown =
            "<span class=\"badge\">new</span> <span class=\"danger\">x</span>";

        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_unsafe_html(false)
            .with_sanitizer_config(
                SanitizerConfig::new()
                    .with_allowed_class("span", "badge"),
            );

        let html = process_markdown(markdown, &options).unwrap();
        assert!(
            html.contains("class=\"badge\""),
            "whitelisted class survives: {html}"
        );
        assert!(
            !html.contains("class=\"danger\""),
            "non-whitelisted class dropped: {html}"
        );
    }

    #[cfg(feature = "syntax_highlighting")]
    #[test]
    fn test_sanitized_output_keeps_syntect_span_classes() {
        // Sanitized pipeline must preserve the open-ended class
        // names that ClassedHTMLGenerator emits on <span> — without
        // them, code blocks render unstyled.
        let markdown = "```rust\nfn main() {}\n```";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_unsafe_html(false);

        let html = process_markdown(markdown, &options).unwrap();
        assert!(
            html.contains("<span class=\""),
            "syntect classes were stripped by sanitizer: {html}"
        );
    }

    #[test]
    fn test_sanitizer_config_restricts_span_class() {
        // Custom config with extra_allowed_classes for span must
        // override the default permissive span policy: only the
        // whitelisted class survives.
        let markdown = "<span class=\"badge\">a</span> <span class=\"danger\">b</span>";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_unsafe_html(false)
            .with_sanitizer_config(
                SanitizerConfig::new()
                    .with_allowed_class("span", "badge"),
            );

        let html = process_markdown(markdown, &options).unwrap();
        assert!(
            html.contains("class=\"badge\""),
            "whitelisted class survives: {html}"
        );
        assert!(
            !html.contains("class=\"danger\""),
            "non-whitelisted class dropped: {html}"
        );
    }

    #[test]
    fn test_sanitizer_strips_style_attribute() {
        // The default sanitizer no longer allows `style` on any tag.
        // Clickjacking via position:fixed/z-index would otherwise be
        // possible through raw HTML in user-supplied Markdown.
        let markdown =
            "<div style=\"position:fixed;top:0;left:0;width:100%;height:100%;z-index:9999;\">overlay</div>";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_unsafe_html(false);

        let html = process_markdown(markdown, &options).unwrap();
        assert!(
            !html.contains("style="),
            "style attribute must be stripped: {html}"
        );
        // The div itself is still allowed; only the attribute goes.
        assert!(html.contains("<div"), "div tag dropped: {html}");
    }

    // ── Table of contents ───────────────────────────────────────

    #[test]
    fn test_toc_collects_headings_in_document_order() {
        let markdown = "\
# First
Some text.

## Second-A
More text.

## Second-B

# Third\n";

        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false);

        let (_html, toc) =
            process_markdown_with_toc(markdown, &options).unwrap();
        let levels: Vec<u8> = toc.iter().map(|h| h.level).collect();
        let texts: Vec<&str> =
            toc.iter().map(|h| h.text.as_str()).collect();
        assert_eq!(levels, vec![1, 2, 2, 1]);
        assert_eq!(
            texts,
            vec!["First", "Second-A", "Second-B", "Third"]
        );
    }

    #[test]
    fn test_toc_ids_match_rendered_html() {
        // With header_ids enabled, the rendered HTML must contain
        // an id matching every Heading::id we report.
        let markdown = "# Hello World\n\n## A Second Heading\n";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_header_ids("");

        let (html, toc) =
            process_markdown_with_toc(markdown, &options).unwrap();
        assert_eq!(toc.len(), 2);
        for h in &toc {
            let needle = format!("id=\"{}\"", h.id);
            assert!(
                html.contains(&needle),
                "ToC id {:?} not found in HTML: {}",
                h.id,
                html
            );
        }
    }

    #[test]
    fn test_toc_prefix_propagates() {
        let markdown = "# Intro\n";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_header_ids("user-content-");

        let (html, toc) =
            process_markdown_with_toc(markdown, &options).unwrap();
        assert_eq!(toc.len(), 1);
        assert_eq!(toc[0].id, "user-content-intro");
        assert!(html.contains("id=\"user-content-intro\""));
    }

    #[test]
    fn test_toc_dedup_with_repeated_headings() {
        let markdown = "# Notes\n## Notes\n### Notes\n";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_header_ids("");

        let (_html, toc) =
            process_markdown_with_toc(markdown, &options).unwrap();
        let ids: Vec<&str> =
            toc.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids, vec!["notes", "notes-1", "notes-2"]);
    }

    #[test]
    fn test_toc_empty_document() {
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false);
        let (html, toc) =
            process_markdown_with_toc("", &options).unwrap();
        assert!(toc.is_empty());
        assert!(html.trim().is_empty());
    }

    #[test]
    fn test_toc_writer_variant_writes_html_and_returns_toc() {
        let markdown = "# Title\n\n## Sub\n";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false);

        let mut buf: Vec<u8> = Vec::new();
        let toc = process_markdown_with_toc_to_writer(
            markdown, &mut buf, &options,
        )
        .unwrap();
        let html = String::from_utf8(buf).unwrap();
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<h2>Sub</h2>"));
        assert_eq!(toc.len(), 2);
    }

    #[test]
    fn test_toc_extracts_inline_code_text() {
        let markdown = "# Using `&str` types\n";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false);

        let (_html, toc) =
            process_markdown_with_toc(markdown, &options).unwrap();
        assert_eq!(toc.len(), 1);
        assert_eq!(toc[0].text, "Using &str types");
    }

    #[test]
    fn test_sanitizer_config_default_path_unchanged() {
        // Options with no sanitizer_config must go through the cached
        // default builder and produce the same output as before the
        // feature was added.
        let markdown = "<script>x</script>\n<div class=\"alert alert-info\">safe</div>";
        let options = MarkdownOptions::new()
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_unsafe_html(false);

        let html = process_markdown(markdown, &options).unwrap();
        assert!(!html.contains("<script>"));
        assert!(html.contains("alert alert-info"));
    }

    // ── Plain text extractor ────────────────────────────────────

    #[test]
    fn test_plain_text_basic() {
        let md = "# Hello World\n\nA **bold** paragraph.";
        let text = process_markdown_to_plain_text(
            md,
            &MarkdownOptions::default(),
        )
        .unwrap();
        assert_eq!(text, "Hello World A bold paragraph.");
    }

    #[test]
    fn test_plain_text_lists_and_code() {
        let md =
            "# Title\n\nDesc.\n\n- one\n- two\n\n```\nfn main() {}\n```";
        let text = process_markdown_to_plain_text(
            md,
            &MarkdownOptions::default(),
        )
        .unwrap();
        assert!(text.contains("Title"));
        assert!(text.contains("Desc."));
        assert!(text.contains("one"));
        assert!(text.contains("two"));
        assert!(text.contains("fn main() {}"));
        // No words merged into each other.
        assert!(!text.contains("Titleone"));
        assert!(!text.contains("onetwo"));
    }

    #[test]
    fn test_plain_text_strips_html() {
        // Raw HTML should NOT appear in the plain-text output.
        let md = "A <strong>bold</strong> word";
        let text = process_markdown_to_plain_text(
            md,
            &MarkdownOptions::default(),
        )
        .unwrap();
        assert!(!text.contains("<strong>"));
        assert!(!text.contains("</strong>"));
        assert!(text.contains("bold"));
    }

    #[test]
    fn test_plain_text_respects_input_cap() {
        let options = MarkdownOptions::new().with_max_input_size(8);
        let err =
            process_markdown_to_plain_text(&"a".repeat(64), &options)
                .unwrap_err();
        assert!(matches!(err, MarkdownError::InputTooLarge { .. }));
    }

    // ── Math + footnote sanitizer survival ──────────────────────

    #[test]
    fn test_math_dollars_survives_sanitizer() {
        // `extension.math_dollars` renders as
        // `<span data-math-style="inline">…</span>`. The default
        // sanitizer must preserve the attribute so a frontend
        // library (KaTeX, MathJax) can find and render it.
        let mut comrak = Options::default();
        comrak.extension.math_dollars = true;
        let options = MarkdownOptions::new()
            .with_comrak_options(comrak)
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_syntax_highlighting(false)
            .with_unsafe_html(false);

        let html =
            process_markdown("Inline $a^2 + b^2$ math.", &options)
                .unwrap();
        assert!(
            html.contains("data-math-style"),
            "data-math-style attribute stripped by sanitizer: {html}"
        );
    }

    #[test]
    fn test_footnote_link_survives_sanitizer() {
        // GFM footnotes emit `<sup><a href="#fn-1" id="fnref-1">`
        // + a back-reference link. The default sanitizer must
        // preserve both so footnote navigation works.
        let mut comrak = Options::default();
        comrak.extension.footnotes = true;
        let options = MarkdownOptions::new()
            .with_comrak_options(comrak)
            .with_custom_blocks(false)
            .with_enhanced_tables(false)
            .with_syntax_highlighting(false)
            .with_unsafe_html(false);

        let md = "Claim[^1].\n\n[^1]: Reason.\n";
        let html = process_markdown(md, &options).unwrap();
        assert!(html.contains("<sup"), "missing <sup>: {html}");
        assert!(
            html.contains("href=\"#fn-1\"")
                || html.contains("href=\"#fn1\""),
            "missing forward link: {html}"
        );
        assert!(
            html.contains("href=\"#fnref-1\"")
                || html.contains("href=\"#fnref1\""),
            "missing back-reference link: {html}"
        );
    }
}
