<p align="center">
  <img src="https://kura.pro/mdx-gen/images/logos/mdx-gen.svg" alt="MDX Gen logo" width="128" />
</p>

<h1 align="center">MDX Gen</h1>

<p align="center">
  <strong>A Rust library for processing Markdown into HTML with custom blocks and enhanced table formatting.</strong>
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/mdx-gen/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/mdx-gen/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/mdx-gen"><img src="https://img.shields.io/crates/v/mdx-gen.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/mdx-gen"><img src="https://img.shields.io/badge/docs.rs-mdx-gen-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://codecov.io/gh/sebastienrousseau/mdx-gen"><img src="https://img.shields.io/codecov/c/github/sebastienrousseau/mdx-gen?style=for-the-badge&logo=codecov" alt="Coverage" /></a>
  <a href="https://lib.rs/crates/mdx-gen"><img src="https://img.shields.io/badge/lib.rs-v0.0.3-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
</p>

---

## Install

```bash
cargo add mdx-gen
```

Or add to `Cargo.toml`:

```toml
[dependencies]
mdx-gen = "0.0.3"
```

You need [Rust](https://rustup.rs/) 1.85.0 or later. Works on macOS, Linux, and Windows.

---

## Overview

MDX Gen converts Markdown to HTML with support for custom blocks, enhanced table formatting, and syntax highlighting.

- **CommonMark-compliant** Markdown parsing
- **Custom block elements** for extended content types
- **Enhanced table formatting** with alignment control
- **Configurable rendering** options

---

## Features

| | |
| :--- | :--- |
| **Markdown to HTML** | Convert Markdown to HTML with extensions |
| **Custom blocks** | Support for custom block-level elements |
| **Tables** | Enhanced table formatting and alignment |
| **Syntax highlighting** | Code block syntax highlighting with ComrakOptions |
| **Configurable** | Flexible configuration for parsing and rendering |

---

## Usage

```rust
use mdx_gen::{process_markdown, MarkdownOptions};

fn main() {
    let markdown = "# Title\n\nParagraph with **bold**.";
    let options = MarkdownOptions::new()
        .with_syntax_highlighting(false)
        .with_custom_blocks(false)
        .with_enhanced_tables(false);
    let html = process_markdown(markdown, &options).unwrap();
    println!("{}", html);
}
```

---

## Development

```bash
cargo build        # Build the project
cargo test         # Run all tests
cargo clippy       # Lint with Clippy
cargo fmt          # Format with rustfmt
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, signed commits, and PR guidelines.

---

**THE ARCHITECT** ᴫ [Sebastien Rousseau](https://sebastienrousseau.com)
**THE ENGINE** ᵞ [EUXIS](https://euxis.co) ᴫ Enterprise Unified Execution Intelligence System

---

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.

<p align="right"><a href="#mdx-gen">Back to Top</a></p>
