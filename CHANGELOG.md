# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Pre-1.0 caveat: cargo treats every `0.x` bump as fully incompatible. Read the
"Breaking changes" section before upgrading.

## [Unreleased]

### Added

- **Diagram code-block rendering.** `mdx-gen` now recognises fenced
  code blocks with info strings `mermaid`, `geojson`, `topojson`,
  and `stl` and rewrites them into sanitizer-safe HTML containers
  that a small client-side JS module hydrates into inline SVG at
  page-load time. This mirrors how github.com handles mermaid and
  geojson natively. Enable with
  [`MarkdownOptions::with_diagrams(true)`]; embed
  [`mdx_gen::hydration_script_html()`] once per page. New
  `mdx_gen::diagrams` module + `examples/diagrams.rs`. See
  [`mdx_gen::diagrams`] rustdoc for the container contract and
  why this is client-side rather than server-rasterised.
- Sanitizer allow-list extended with `pre.mermaid`,
  `div.mdx-diagram` / `div.mdx-diagram-{geojson,topojson,stl}`,
  and `div[data-mdx-diagram]` so diagram containers survive
  `allow_unsafe_html = false`.

### Breaking

- **MSRV raised to 1.88.0** (from 1.85.0). Required by the newly
  vendored `commons` crate (EUXIS ecosystem shared utilities) which
  ships on edition 2024. `build.rs`, `Cargo.toml` `rust-version`,
  and the README install note all updated in lockstep.

### Added

- `crates/commons/` — vendored copy of the EUXIS `commons` utility
  crate (upstream: github.com/sebastienrousseau/commons, v0.0.3).
  Kept under `publish = false` in the workspace so mdx-gen can
  evolve against the source without a separate registry release.
  Pulled in with `error` + `validation` features only.
- `impl From<commons::error::CommonError> for MarkdownError` — the
  EUXIS-wide error type flows through `?` into mdx-gen's `Result`
  types. Domain-specific variants stay intact
  (`InvalidInput`/`Parse` → `ParseError`, `Io` → `IoError`, rest →
  `ConversionError` with Display preserved).
- `impl From<commons::validation::ValidationError> for MarkdownError`
  for the validation bridge.

### Changed

- `MarkdownOptions::validate` now runs a comprehensive
  multi-check pass via [`commons::validation::Validator`] and
  returns `Result<(), Vec<(String, ValidationError)>>`. Every
  failing check surfaces with the field that tripped it — the
  validator no longer bails on the first problem. Nine checks
  now run:
    1. `enhanced_tables` requires `comrak.extension.table`.
    2. `syntax_theme` (if set) must be a bundled syntect theme.
    3. `syntax_theme` set with `enable_syntax_highlighting = false`
       is rejected (silent no-op).
    4. `sanitizer_config` set with `allow_unsafe_html = true` is
       rejected (sanitizer would never run).
    5. `header_ids` prefix rejects whitespace and HTML-breaking
       characters (`"`, `'`, `<`, `>`, `&`, `=`).
    6. `sanitizer_config.extra_tags` / `extra_tag_attributes`
       must use valid HTML tag / attribute names.
    7. `sanitizer_config.extra_generic_attributes` must be valid
       HTML attribute names.
    8. `sanitizer_config.extra_allowed_classes` tags must be
       valid; class values must be non-empty without whitespace
       or quotes.
    9. `custom_block_config.class_overrides` values must be
       non-empty without whitespace/quotes;
       `title_overrides` values must be non-blank.
  The pipeline converts the resulting `Vec` into a single
  `MarkdownError::InvalidOptionsError` via a new `From` impl
  that preserves every failing field in the message.
- Closed PR #17 (pre-release proposal to add commons) — the
  vendored + migrated form landed directly on this branch.

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
