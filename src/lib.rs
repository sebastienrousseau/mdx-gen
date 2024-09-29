// src/lib.rs

//! # MDX Generator (mdx-gen)
//!
//! A robust Rust library for processing Markdown into responsive HTML, offering custom blocks, syntax highlighting, and enhanced table formatting for richer content.
//!
//! ## Overview
//!
//! `mdx-gen` is a flexible Rust library that converts Markdown into HTML, providing enhanced features like custom block extensions, syntax highlighting, and table formatting.
//!
//! `mdx-gen` uses the high-performance `comrak` library for Markdown parsing and offers flexible options for modifying and extending Markdown behavior.
//!
//! ### Key Features
//!
//! - **Markdown to HTML Conversion**: Converts Markdown to responsive HTML using the `comrak` parser, ensuring fast and accurate rendering of Markdown content.
//! - **Custom Block Extensions**: Allows the use of custom blocks such as notes, warnings, and tips, transforming them into structured HTML elements for improved content formatting.
//! - **Syntax Highlighting**: Automatically applies syntax highlighting to code blocks for a wide range of programming languages, making code snippets more readable and professional.
//! - **Enhanced Table Formatting**: Converts Markdown tables into responsive HTML tables with proper alignment and additional styling for better usability across devices.
//! - **Flexible Configuration**: Provides a customizable `MarkdownOptions` structure that allows developers to enable or disable specific features (e.g., custom blocks, enhanced tables, or syntax highlighting).
//! - **Error Handling**: Comprehensive error handling system with detailed error reporting to ensure smooth Markdown processing, even in complex cases.
//!
//! ### Supported Extensions
//!
//! `mdx-gen` offers the following extensions, which can be enabled or disabled individually via `MarkdownOptions`:
//!
//! - Custom blocks (notes, warnings, tips)
//! - Enhanced table formatting with responsive design
//! - Syntax highlighting for code blocks
//! - Strikethrough and autolink support
//! - Advanced error reporting for improved debugging
//!
//! ## Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! mdx-gen = "0.0.1"
//! ```
//!
//! ## Usage
//!
//! Here are some examples of how to use the library:
//!
//! ### Basic Usage
//!
//! ```rust
//! use mdx_gen::{process_markdown, MarkdownOptions};
//!
//! let markdown_content = "# Hello, world!\n\nThis is a paragraph.";
//! let options = MarkdownOptions::new();
//! let html = process_markdown(markdown_content, &options).unwrap();
//! println!("HTML output: {}", html);
//! ```
//!
//! ### Custom Blocks and Syntax Highlighting
//!
//! ```rust
//! use mdx_gen::{process_markdown, MarkdownOptions};
//!
//! let markdown_content = r#"
//! # Example
//!
//! <div class="note">This is a note.</div>
//!
//! ```rust
//! fn main() {
//!     println!("Hello, world!");
//! }
//!
//! "#;
//!
//! let options = MarkdownOptions::new()
//!     .with_custom_blocks(true)
//!     .with_syntax_highlighting(true);
//!
//! let html = process_markdown(markdown_content, &options).unwrap();
//! println!("HTML output: {}", html);
//!
//! ```

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

pub use error::MarkdownError;
pub use extensions::{
    apply_syntax_highlighting, ColumnAlignment, CustomBlockType,
};
pub use markdown::{process_markdown, MarkdownOptions};

/// Re-export comrak options for convenience
pub use comrak::ComrakOptions;
