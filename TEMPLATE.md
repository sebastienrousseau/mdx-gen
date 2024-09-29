<!-- markdownlint-disable MD033 MD041 -->
<img src="https://kura.pro/mdx-gen/images/logos/mdx-gen.svg"
alt="MDX Gen logo" height="66" align="right" />
<!-- markdownlint-enable MD033 MD041 -->

# MDX Generator (mdx-gen)

A robust Rust library for processing Markdown into responsive HTML, offering custom blocks, syntax highlighting, and enhanced table formatting for richer content.

<!-- markdownlint-disable MD033 MD041 -->
<center>
<!-- markdownlint-enable MD033 MD041 -->

[![Made With Love][made-with-rust]][01] [![Crates.io][crates-badge]][06] [![Lib.rs][libs-badge]][08] [![Docs.rs][docs-badge]][07] [![License][license-badge]][03]

• [Website][00] • [Documentation][07] • [Report Bug][04] • [Request Feature][04] • [Contributing Guidelines][05]

<!-- markdownlint-disable MD033 MD041 -->
</center>
<!-- markdownlint-enable MD033 MD041 -->

## Overview

`mdx-gen` is a flexible Rust library that converts Markdown into HTML, providing enhanced features like custom block extensions, syntax highlighting, and table formatting.

`mdx-gen` uses the high-performance `comrak` library for Markdown parsing and offers flexible options for modifying and extending Markdown behavior.

### Key Features

- **Markdown to HTML Conversion**: Converts Markdown to responsive HTML using the `comrak` parser, ensuring fast and accurate rendering of Markdown content.
- **Custom Block Extensions**: Allows the use of custom blocks such as notes, warnings, and tips, transforming them into structured HTML elements for improved content formatting.
- **Syntax Highlighting**: Automatically applies syntax highlighting to code blocks for a wide range of programming languages, making code snippets more readable and professional.
- **Enhanced Table Formatting**: Converts Markdown tables into responsive HTML tables with proper alignment and additional styling for better usability across devices.
- **Flexible Configuration**: Provides a customizable `MarkdownOptions` structure that allows developers to enable or disable specific features (e.g., custom blocks, enhanced tables, or syntax highlighting).
- **Error Handling**: Comprehensive error handling system with detailed error reporting to ensure smooth Markdown processing, even in complex cases.

[00]: https://mdxgen.com/ 'MDX Generator'
[01]: https://www.rust-lang.org/ 'Rust Programming Language'
[03]: https://opensource.org/licenses/MIT "MIT license"
[04]: https://github.com/sebastienrousseau/mdx-gen/issues "Report Bug"
[05]: https://github.com/sebastienrousseau/mdx-gen/blob/main/CONTRIBUTING.md "Contributing Guidelines"
[06]: https://crates.io/crates/mdx-gen 'Crates.io'
[07]: https://docs.rs/mdx-gen 'Docs.rs'
[08]: https://lib.rs/crates/mdx-gen 'Lib.rs'

[crates-badge]: https://img.shields.io/crates/v/mdx-gen-html.svg?style=for-the-badge 'Crates.io badge'
[docs-badge]: https://img.shields.io/docsrs/mdx-gen-html.svg?style=for-the-badge 'Docs.rs badge'
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.1.0-orange.svg?style=for-the-badge 'Lib.rs badge'
[license-badge]: https://img.shields.io/crates/l/mdx-gen-html.svg?style=for-the-badge 'License badge'
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust badge'

## Changelog 📚
