#![forbid(unsafe_code)]
// src/lib.rs
#![doc = include_str!("../README.md")]
#![doc(
    html_favicon_url = "https://kura.pro/mdx-gen/images/favicon.ico",
    html_logo_url = "https://kura.pro/mdx-gen/images/logos/mdx-gen.svg",
    html_root_url = "https://docs.rs/mdx-gen"
)]
#![crate_name = "mdx_gen"]
#![crate_type = "lib"]

/// The `error` module contains error types for Markdown processing.
pub mod error;

/// The `extensions` module contains custom block and table extensions.
pub mod extensions;

/// Syntax highlighting adapter for comrak's plugin system.
#[cfg(feature = "syntax_highlighting")]
pub mod highlight;

/// YAML frontmatter extraction and parsing.
pub mod frontmatter;

/// The `markdown` module contains the core processing pipeline.
pub mod markdown;

// ── Re-exports ──────────────────────────────────────────────────────

pub use error::MarkdownError;

/// Applies syntax highlighting to code (standalone usage).
///
/// # Example
/// ```
/// use mdx_gen::apply_syntax_highlighting;
/// let highlighted = apply_syntax_highlighting("fn main() {}", "rust");
/// ```
#[cfg(feature = "syntax_highlighting")]
pub use highlight::apply_syntax_highlighting;

/// Generates a CSS stylesheet for a built-in syntect theme so that
/// callers can render the class-based output produced by the
/// highlighter and the comrak adapter.
#[cfg(feature = "syntax_highlighting")]
pub use highlight::theme_css;

pub use extensions::ColumnAlignment;
pub use extensions::CustomBlockConfig;
pub use extensions::CustomBlockType;

/// Processes a Markdown string and converts it into HTML.
///
/// # Example
/// ```
/// use mdx_gen::{process_markdown, MarkdownOptions};
/// use comrak::Options;
///
/// let markdown_input = "# Hello, World!";
/// let mut comrak_options = Options::default();
/// comrak_options.extension.table = true;
///
/// let options = MarkdownOptions::default()
///     .with_custom_blocks(false)
///     .with_comrak_options(comrak_options);
///
/// let html_output = process_markdown(markdown_input, &options).expect("Failed to process markdown");
/// assert!(html_output.contains("<h1>Hello, World!</h1>"));
/// ```
///
/// # Errors
///
/// Returns a `MarkdownError` if options are invalid, input exceeds
/// the size limit, or rendering fails.
pub use markdown::process_markdown;

/// Streams processed HTML directly to a `Write` sink.
///
/// See [`process_markdown`] for the pipeline semantics; this variant
/// writes the output through a caller-provided writer instead of
/// allocating a `String`.
pub use markdown::process_markdown_to_writer;

pub use markdown::MarkdownOptions;
pub use markdown::SanitizerConfig;

/// Re-export comrak's options for convenience.
///
/// # Usage
/// ```
/// use mdx_gen::Options;
///
/// let mut options = Options::default();
/// options.extension.strikethrough = true;
/// ```
pub use comrak::Options;
