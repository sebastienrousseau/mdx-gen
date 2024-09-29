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

/// The `extensions` module contains custom block extensions for Markdown processing.
pub mod extensions;

/// The `markdown` module contains functions for parsing, converting, and rendering Markdown.
pub mod markdown;

// Re-exporting key items for easier access by the library's users.

/// Represents errors that may occur during Markdown processing.
///
/// This includes errors related to syntax, rendering, and custom block handling.
pub use error::MarkdownError;

/// Applies syntax highlighting to code blocks within the processed Markdown.
///
/// # Example
/// ```
/// use mdx_gen::apply_syntax_highlighting;
/// let highlighted = apply_syntax_highlighting("fn main() {}", "rust");
/// ```
pub use extensions::apply_syntax_highlighting;

/// Represents different alignment options for table columns in enhanced Markdown tables.
pub use extensions::ColumnAlignment;

/// Represents the type of custom block, such as admonitions or custom embedded content.
pub use extensions::CustomBlockType;

/// Processes a Markdown string and converts it into HTML, applying custom blocks and syntax highlighting.
///
/// # Example
/// ```
/// use mdx_gen::{process_markdown, MarkdownOptions};
/// use comrak::ComrakOptions;
///
/// let markdown_input = "# Hello, World!";
/// let mut comrak_options = ComrakOptions::default();
/// comrak_options.extension.table = true;  // Enable Comrak table extension
///
/// let options = MarkdownOptions::default().with_comrak_options(comrak_options);  // Chaining method call
///
/// let html_output = process_markdown(markdown_input, &options).expect("Failed to process markdown");
/// assert!(html_output.contains("<h1>Hello, World!</h1>"));
/// ```
///
/// # Errors
///
/// This function will return a `MarkdownError` if the input contains invalid syntax or cannot be parsed.
pub use markdown::process_markdown;

/// Options for configuring how Markdown is processed, including syntax highlighting and custom block support.
pub use markdown::MarkdownOptions;

/// Re-export comrak's options for convenience when customizing Markdown processing.
///
/// # Usage
/// ```
/// use mdx_gen::ComrakOptions;
///
/// let mut options = ComrakOptions::default();
/// options.extension.strikethrough = true;
/// ```
pub use comrak::ComrakOptions;
