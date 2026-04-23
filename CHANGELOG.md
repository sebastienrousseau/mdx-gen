# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Pre-1.0 caveat: cargo treats every `0.x` bump as fully incompatible. Read the
"Breaking changes" section before upgrading.

## [Unreleased]

## [0.0.3] — 2026-04-23

### Breaking

- **Syntax highlighter switched to class-based output.** Code blocks now render
  as `<span class="…">` tokens instead of inline `style="color:#…"`. Callers
  must serve a matching stylesheet — generate one with the new
  `mdx_gen::theme_css(theme_name)` helper. CSS that targeted the old
  inline-style output needs to switch to syntect's
  [`ClassStyle::Spaced`](https://docs.rs/syntect/latest/syntect/html/enum.ClassStyle.html)
  class names.
- **Sanitizer no longer permits `style` on any tag.** With class-based
  highlighting nothing in the pipeline emits `style="…"` any longer, and the
  global allowance was a clickjacking / UI-redress vector for raw HTML in
  untrusted Markdown. Opt back in (for trusted content only) via
  `SanitizerConfig::with_generic_attribute("style")`.
- **Frontmatter requires `---` at byte 0.** Leading whitespace before the
  opening delimiter no longer triggers detection — matches Jekyll, Hugo, and
  most CommonMark front-matter consumers. Strip leading whitespace upstream
  if you depended on the previous forgiving behaviour.

### Added

- `process_markdown_to_writer<W: Write>(content, &mut writer, options)` —
  streams rendered HTML directly into any `std::io::Write` sink, skipping the
  intermediate `String` allocation. Sanitised path uses
  `ammonia::Document::write_to()` end-to-end.
- `process_markdown_with_toc(content, options) -> (String, Vec<Heading>)` and
  the streaming variant `process_markdown_with_toc_to_writer`. Both return a
  document-order outline alongside the HTML, with anchor ids that match what
  comrak renders.
- `Heading { level, text, id }` and the building-block walker
  `collect_headings(root, prefix)`.
- `SanitizerConfig` for extending the ammonia allow-list (extra tags,
  per-tag attributes, generic attributes, per-tag allowed classes) without
  leaking `ammonia::Builder` through the public API.
- `theme_css(theme_name) -> Option<String>` for generating a stylesheet
  matched to the class-based highlighter output.
- `SyntectAdapter::theme_name()` so callers can recover the resolved theme
  after fallback.
- `MarkdownError::IoError(std::io::Error)` with a `From` impl for `?`
  propagation through writer-based entry points.
- `fuzz/` workspace with three `libfuzzer-sys` targets (`process_markdown`,
  `custom_blocks`, `sanitize_roundtrip`). Run with
  `cargo +nightly fuzz run <target>` from `fuzz/`.
- 16 standalone examples in `examples/` covering every public surface:
  `basic`, `quickstart`, `blog`, `typed`, `docs`, `alerts`, `cms`,
  `security`, `site`, `styling`, `gallery`, `streaming`, `pipe`, `search`,
  `bulk`, `errors`, plus an `all` runner.
- `examples/support.rs` — shared spinner + checkmark helper for example
  output, ported from the noyalib idiom.

### Changed

- `comrak` 0.50 → 0.52. The deprecated `extension.header_ids` field is now
  `extension.header_id_prefix`; mdx-gen forwards the value transparently.
- `criterion` (dev-dep) 0.5 → 0.8. `criterion::black_box` deprecated in
  favour of `std::hint::black_box`; the bench was updated.
- Default features now include `yaml_support` so `cargo run --example blog`
  works out of the box. Use `--no-default-features` for a minimal build.
- `crates/yaml_safe` vendored from the upstream
  `~/Code/Public/Rust/yaml_safe`. Now supports block-style sequences under
  mapping keys, literal/folded block scalars, anchors + aliases + merge keys
  (`<<`), and multi-document YAML (`DocumentIter`). The frontmatter examples
  switched to idiomatic block YAML accordingly.

### Performance

- `ammonia::Builder` constructed once per process via `LazyLock<Builder<'static>>`
  instead of per-call. The configuration is all `'static` strs, so
  `Builder::clean(&self, …)` reuses a single builder for every default-config
  sanitize.
- Table enhancement moved to AST-level (a pre-render pass over comrak nodes)
  rather than post-render regex on rendered HTML.
- Literal-string `<table>` / `</table>` regex replacements switched to
  `str::replace`. The capturing `<td …>` rewrite stays as a regex — it
  genuinely needs pattern capture.

### Security

- `cargo-fuzz` scaffolding for ongoing parser hardening (no crashes across
  ~2.3 M iterations on initial smoke run).
- `style` attribute removed from the default sanitizer allow-list (see
  Breaking). Regression test asserts a `position:fixed` overlay is stripped.
- Examples include a dedicated `security` red-team scenario covering XSS via
  `<script>`, `on*` event handlers, clickjacking via inline style,
  `javascript:` / `data:` URLs, oversized input via the cap, and deeply
  nested blockquote bombs.

### Internal

- Pipeline body in `src/markdown.rs` factored into a private `pipeline()`
  helper that all four entry points (`process_markdown`,
  `process_markdown_to_writer`, `process_markdown_with_toc`,
  `process_markdown_with_toc_to_writer`) share.
- `#![deny(missing_docs)]` enforced on the public crate; every public item
  carries a docstring.
- `deny.toml` added for `cargo-deny` (license allow-list, advisory deny,
  source allow-list).

### Closed

- Closed 8 stale Dependabot PRs (#19–#26). Two (`comrak`, `criterion`) were
  applied to this branch; six were obsoleted by the consolidated CI workflow
  + dropped `toml` dependency on `feat/v0.0.3`.

[Unreleased]: https://github.com/sebastienrousseau/mdx-gen/compare/v0.0.3...HEAD
[0.0.3]: https://github.com/sebastienrousseau/mdx-gen/releases/tag/v0.0.3
