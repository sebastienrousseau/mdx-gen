<p align="center">
  <img src="https://cloudcdn.pro/mdx-gen/v1/logos/mdx-gen.svg" alt="MDX Gen logo" width="128" />
</p>

<h1 align="center">MDX Gen</h1>

<p align="center">
  <strong>A Rust library for processing Markdown into HTML with custom blocks, enhanced tables, class-based syntax highlighting, a hardened sanitizer, table-of-contents extraction, streaming output, and client-side mermaid diagrams.</strong>
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/mdx-gen/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/mdx-gen/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/mdx-gen"><img src="https://img.shields.io/crates/v/mdx-gen.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/mdx-gen"><img src="https://img.shields.io/badge/docs.rs-mdx--gen-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://codecov.io/gh/sebastienrousseau/mdx-gen"><img src="https://img.shields.io/codecov/c/github/sebastienrousseau/mdx-gen?style=for-the-badge&logo=codecov" alt="Coverage" /></a>
  <a href="https://lib.rs/crates/mdx-gen"><img src="https://img.shields.io/badge/lib.rs-v0.0.4-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
</p>

---

## Install

```bash
cargo add mdx-gen
```

Or add to `Cargo.toml`:

```toml
[dependencies]
mdx-gen = "0.0.4"
```

Requires [Rust](https://rustup.rs/) 1.88.0 or later. Works on macOS, Linux, and Windows.

---

## Overview

MDX Gen is an AST-first Markdown-to-HTML pipeline built on [comrak](https://crates.io/crates/comrak) with optional [syntect](https://crates.io/crates/syntect) highlighting and [ammonia](https://crates.io/crates/ammonia)-backed sanitization. Every transformation happens on the comrak AST — not as post-render regex — which keeps the output predictable and fast.

Pipeline stages:

1. **Parse** — Markdown → comrak AST (arena-allocated).
2. **Transform** — custom blocks, mermaid diagrams, table enhancement.
3. **Collect** — headings for table-of-contents callers.
4. **Render** — AST → HTML, with class-based syntax highlighting.
5. **Sanitize** — ammonia strips dangerous tags while preserving safe structural markup.

---

## Features

| | |
| :--- | :--- |
| **CommonMark + GFM** | Tables, strikethrough, task lists, autolinks, footnotes, math |
| **Class-based syntax highlighting** | 20+ syntect themes, CSS stylesheet helper |
| **Custom blocks** | `::note`, `::warning`, `::tip`, and user-defined variants |
| **Enhanced tables** | AST-level responsive wrappers and alignment classes |
| **Table of contents** | `Vec<Heading>` in document order with anchor ids |
| **Streaming output** | Write directly into any `std::io::Write` sink |
| **Mermaid diagrams** | Client-side hydration of fenced `mermaid` blocks |
| **Plain-text extraction** | For search indexes, excerpts, reading-time |
| **Hardened sanitizer** | No inline `style`, cached `ammonia::Builder` |
| **Input size cap** | Default 1 MiB, overridable |
| **Fuzz harness** | 3 `cargo-fuzz` targets exercising the public surface |
| **Strict validation** | Nine-check `MarkdownOptions::validate` |

---

## Quick start

```rust
use mdx_gen::{process_markdown, MarkdownOptions};

fn main() {
    let markdown = "# Hello\n\nParagraph with **bold** and *italic*.";
    let html = process_markdown(markdown, &MarkdownOptions::default()).unwrap();
    println!("{html}");
}
```

For a richer starting template (feature flags, options, pretty output), see [`examples/quickstart.rs`](examples/quickstart.rs).

---

## Usage

### Table of contents

```rust
use mdx_gen::{process_markdown_with_toc, MarkdownOptions};

let md = "# Intro\n\n## Background\n\n## Details\n";
let (html, headings) = process_markdown_with_toc(md, &MarkdownOptions::default()).unwrap();

for h in &headings {
    println!("H{} {} (#{})", h.level, h.text, h.id);
}
// H1 Intro (#intro)
// H2 Background (#background)
// H2 Details (#details)
```

Each `Heading { level, text, id }` carries the anchor id that the rendered HTML actually uses, including comrak's dedup suffixes (`-1`, `-2`, …).

### Streaming into a writer

```rust
use mdx_gen::{process_markdown_to_writer, MarkdownOptions};
use std::io::stdout;

let mut out = stdout().lock();
process_markdown_to_writer("# streamed\n", &mut out, &MarkdownOptions::default()).unwrap();
```

`process_markdown_to_writer` and `process_markdown_with_toc_to_writer` skip the intermediate `String` — the sanitized path uses `ammonia::Document::write_to()` end-to-end.

### Plain-text extraction

```rust
use mdx_gen::{process_markdown_to_plain_text, MarkdownOptions};

let text = process_markdown_to_plain_text(
    "# Hello\n\nWorld with `code` inside.",
    &MarkdownOptions::default(),
).unwrap();
assert!(text.contains("Hello"));
assert!(text.contains("code"));
```

Useful for feeding search indexes, excerpt generators, or reading-time estimators.

### Class-based syntax highlighting

```rust
use mdx_gen::{process_markdown, theme_css, MarkdownOptions};

let md = "```rust\nfn main() { println!(\"hi\"); }\n```\n";
let options = MarkdownOptions::default().with_syntax_highlighting(true);
let html = process_markdown(md, &options).unwrap();

// Serve this CSS alongside the HTML:
let css = theme_css("base16-ocean.dark").unwrap();
```

The highlighter emits `<span class="…">` tokens (no inline `style=`), so you control the colour palette via CSS.

### Mermaid diagrams

```rust
use mdx_gen::{hydration_script_html, process_markdown, MarkdownOptions};

let md = "```mermaid\ngraph TD\n  A --> B\n```\n";
let options = MarkdownOptions::default().with_diagrams(true);
let body = process_markdown(md, &options).unwrap();

// Drop the hydration script once, before </body>:
let script = hydration_script_html();
let page = format!("<!doctype html><body>{body}\n{script}</body>");
```

Fenced `mermaid` blocks become sanitizer-safe `<pre class="mermaid">` containers; the bundled hydration script loads mermaid.js from jsDelivr and turns every container into inline SVG. This mirrors how `github.com` renders mermaid diagrams in READMEs. `examples/diagrams.rs` covers all nine mermaid kinds — flowchart, sequence, ER, class, state, gantt, git graph, user journey, pie.

### Custom blocks

```rust
use mdx_gen::{process_markdown, MarkdownOptions};

let md = "\
::note
Heads up — custom blocks.
::
";

let options = MarkdownOptions::default().with_custom_blocks(true);
let html = process_markdown(md, &options).unwrap();
assert!(html.contains("note"));
```

Built-ins: `note`, `info`, `warning`, `tip`, `important`, `caution`. Class and title overrides come from `CustomBlockConfig`.

### Enhanced tables

```rust
use comrak::Options;
use mdx_gen::{process_markdown, MarkdownOptions};

let md = "| a | b |\n|---|---|\n| 1 | 2 |\n";
let mut comrak = Options::default();
comrak.extension.table = true;

let options = MarkdownOptions::default()
    .with_comrak_options(comrak)
    .with_enhanced_tables(true);
let html = process_markdown(md, &options).unwrap();
assert!(html.contains("<table"));
```

### Sanitizer configuration

```rust
use mdx_gen::{process_markdown, MarkdownOptions, SanitizerConfig};

let cfg = SanitizerConfig::new()
    .with_generic_attribute("style")   // opt back in for trusted content
    .with_tag("mark");

let options = MarkdownOptions::default()
    .with_unsafe_html(false)
    .with_sanitizer_config(cfg);

let html = process_markdown("<mark>selected</mark>", &options).unwrap();
```

`SanitizerConfig` is the typed extension point for the ammonia allow-list — extra tags, per-tag attributes, generic attributes, per-tag allowed classes — without leaking `ammonia::Builder` through the public API.

### Error handling

```rust
use mdx_gen::{process_markdown, MarkdownError, MarkdownOptions};

let huge = "x".repeat(2_000_000);
let options = MarkdownOptions::default().with_max_input_size(1_048_576);

match process_markdown(&huge, &options) {
    Err(MarkdownError::InputTooLarge { size, limit }) => {
        eprintln!("input {size} exceeds limit {limit}");
    }
    Err(e) => eprintln!("other error: {e}"),
    Ok(html) => println!("{html}"),
}
```

`MarkdownError` covers `ParseError`, `RenderError`, `CustomBlockError`, `InputTooLarge`, `IoError` (writer path), `InvalidOptionsError` (from `validate`), and others. It implements `From<std::io::Error>` and `From<ValidationError>` so errors flow via `?`.

### Structured validation errors

`MarkdownOptions::validate()` returns `Result<(), Vec<(String, ValidationError)>>` — every failing check surfaces with its field name and a typed `ValidationError` variant. The pipeline folds this into a single `MarkdownError::InvalidOptionsError` when you call `process_markdown`, but callers can inspect the structured form directly before running the pipeline:

```rust
use mdx_gen::{MarkdownOptions, ValidationError};

let options = MarkdownOptions::default().with_header_ids("bad id");
if let Err(errors) = options.validate() {
    for (field, err) in &errors {
        match err {
            ValidationError::InvalidPattern { pattern } => {
                eprintln!("{field}: expected {pattern}");
            }
            other => eprintln!("{field}: {other}"),
        }
    }
}
```

---

## Examples (14 standalone + 1 runner)

Each example is an independently-runnable binary (`cargo run --example <name>`):

| Group | Examples |
| --- | --- |
| Onboarding | `basic`, `quickstart` |
| Scenarios | `docs`, `alerts`, `cms`, `security`, `diagrams` |
| Output channels | `styling`, `gallery`, `streaming`, `pipe` |
| Integrators | `search`, `bulk`, `errors` |
| Runner | `all` |

Start with `quickstart` or `docs` for a realistic walkthrough; `security` doubles as a red-team regression suite (XSS, clickjacking, `javascript:` URLs, oversized input, blockquote bombs).

---

## Feature flags

| Flag                  | Default | Description |
| --------------------- | :-----: | --- |
| `syntax_highlighting` | ✓       | Enable `syntect`-backed highlighter, `theme_css`, `apply_syntax_highlighting` |

Minimal build: `cargo build --no-default-features`.

---

## Breaking changes in 0.0.4

If you are upgrading from `0.0.3` (source only — `0.0.3` never shipped to crates.io):

- **`yaml_support` feature removed.** `mdx_gen::frontmatter` is gone. The vendored `yaml_safe` parser is parked on disk and will return through a standalone crate in a later release.
- **`commons` dependency removed.** `MarkdownOptions::validate` now returns `Result<(), Vec<(String, ValidationError)>>` using the in-crate [`mdx_gen::validation::ValidationError`](crate::validation::ValidationError). The old `From<commons::error::CommonError>` and `From<commons::validation::ValidationError>` impls on `MarkdownError` are gone.
- **`MarkdownError::FrontmatterError` variant removed.** No more frontmatter path = no variant to carry.

If you are upgrading from `0.0.2`:

- **MSRV raised to 1.88.**
- **Syntax highlighter is now class-based.** Code blocks render as `<span class="…">` tokens instead of inline `style="color:#…"`. Generate a matching stylesheet with `mdx_gen::theme_css(theme_name)`.
- **Sanitizer no longer permits `style` on any tag.** Opt back in (trusted content only) via `SanitizerConfig::with_generic_attribute("style")`.
- **`MarkdownOptions::validate`** runs nine consistency checks; every failing check surfaces with its field name. The pipeline still converts the result into a single `MarkdownError::InvalidOptionsError` for callers of `process_markdown`.

New surfaces since `0.0.2`:

- `process_markdown_to_writer`, `process_markdown_with_toc`, `process_markdown_with_toc_to_writer`, `process_markdown_to_plain_text`.
- `MarkdownOptions::with_diagrams` + `hydration_script_html()` — client-side mermaid rendering.
- `SanitizerConfig`, `Heading`, `collect_headings`, `theme_css`.
- `MarkdownError::IoError`, plus `From<ValidationError>` impls for composing validation results.
- `fuzz/` workspace with three `libfuzzer-sys` targets for ongoing parser hardening.

Full release notes: [CHANGELOG.md](CHANGELOG.md).

---

## Security

- Default sanitizer is **on**. Raw HTML flows through ammonia unless you set `allow_unsafe_html(true)`.
- `style` attribute is stripped from every tag by default — explicit opt-in required.
- Input size capped at 1 MiB by default; override via `with_max_input_size`.
- `cargo-fuzz` harness under `fuzz/` with three targets (`process_markdown`, `custom_blocks`, `sanitize_roundtrip`). No crashes across ~2.3 M iterations on the initial smoke run.
- `cargo-deny` configured (`deny.toml`): license allow-list, advisory deny + `yanked = "deny"`, wildcard bans, crates.io-only sources.

Report a vulnerability: [SECURITY.md](https://github.com/sebastienrousseau/mdx-gen/security/policy).

---

## Development

```bash
cargo build                                                      # Build the project
cargo test --workspace --all-features                            # Run all tests
cargo clippy --all-targets --all-features --workspace -- -D warnings  # Lint
cargo fmt --all                                                  # Format
cargo llvm-cov --workspace --all-features                        # Coverage
cargo +nightly fuzz run process_markdown                         # Fuzz (from fuzz/)
```

Opt in to the pre-push hook (runs fmt + clippy + tests before every push):

```bash
git config core.hooksPath .githooks
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for signed commits and PR guidelines.

---

**THE ARCHITECT** ᛫ [Sebastien Rousseau](https://sebastienrousseau.com)
**THE ENGINE** ᛞ [EUXIS](https://euxis.co) ᛫ Enterprise Unified Execution Intelligence System

---

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.

<p align="right"><a href="#mdx-gen">Back to Top</a></p>
