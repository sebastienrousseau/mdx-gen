# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Pre-1.0 caveat: cargo treats every `0.x` bump as fully incompatible. Read the
"Breaking changes" section before upgrading.

## [Unreleased]

## [0.0.4] — 2026-04-24

First crates.io-shipping release of the 0.0.x line. `0.0.3` was tagged
locally but never published because the root `mdx-gen` crate depended
on two workspace members (`crates/commons`, `crates/yaml_safe`) that
were set to `publish = false`, so `cargo publish` could not resolve
them on the registry. `0.0.4` removes both dependencies and ships as
a self-contained crate.

### Breaking

- **`yaml_support` feature removed.** `mdx_gen::frontmatter` and the
  three frontmatter-using examples (`blog`, `typed`, `site`) are gone.
  The vendored `crates/yaml_safe/` parser remains on disk as reference
  for the upcoming standalone replacement but is not a workspace
  member and not a dependency of `mdx-gen`.
- **`commons` dependency removed.** The only two modules mdx-gen used
  (`commons::error`, `commons::validation`) have been surgically
  inlined. `ValidationError` and `Validator` are now
  [`mdx_gen::validation::ValidationError`](crate::validation::ValidationError)
  and [`mdx_gen::validation::Validator`](crate::validation::Validator).
  `From<commons::error::CommonError> for MarkdownError` and
  `From<commons::validation::ValidationError> for MarkdownError` are
  gone; `From<ValidationError>` and
  `From<Vec<(String, ValidationError)>>` remain against the inlined
  types. The vendored `crates/commons/` sits on disk as reference.
- **`MarkdownError::FrontmatterError` variant removed** — no longer a
  meaningful failure mode now that frontmatter parsing is out of tree.
- **Workspace shrunk to `members = ["."]`.** `crates/commons` and
  `crates/yaml_safe` are in `exclude`, preserved but not built.

### Fixed

- `cargo publish --dry-run` now succeeds for the root crate — the
  underlying blocker that kept `0.0.3` unpublished.
- Release CI: job-level permissions (`contents` / `id-token` /
  `attestations`) now granted on the reusable-workflow caller so
  `workflow_dispatch` no longer fails at startup.

## [0.0.3] — 2026-04-24

A full rewrite of mdx-gen's internals plus a ground-up refresh of the
developer experience. Replaces the legacy regex-based processing with
an AST-level pipeline, adds a hardened sanitizer, streaming / ToC /
plain-text / mermaid APIs, and vendors the EUXIS `commons` + `yaml_safe`
utilities into a workspace.

### Breaking

- **MSRV raised to 1.88.0** (from 1.85.0). Required by the newly vendored
  `commons` crate (edition 2024). `build.rs`, `Cargo.toml` `rust-version`,
  and the README install note all updated in lockstep.
- **Syntax highlighter switched to class-based output.** Code blocks now
  render as `<span class="…">` tokens instead of inline
  `style="color:#…"`. Callers must serve a matching stylesheet — generate
  one with the new `mdx_gen::theme_css(theme_name)` helper. CSS that
  targeted the old inline-style output needs to switch to syntect's
  [`ClassStyle::Spaced`](https://docs.rs/syntect/latest/syntect/html/enum.ClassStyle.html)
  class names.
- **Sanitizer no longer permits `style` on any tag.** With class-based
  highlighting nothing in the pipeline emits `style="…"` any longer, and
  the global allowance was a clickjacking / UI-redress vector for raw
  HTML in untrusted Markdown. Opt back in (trusted content only) via
  `SanitizerConfig::with_generic_attribute("style")`.
- **Frontmatter requires `---` at byte 0.** Leading whitespace before the
  opening delimiter no longer triggers detection — matches Jekyll, Hugo,
  and most CommonMark front-matter consumers. Strip leading whitespace
  upstream if you depended on the previous forgiving behaviour.
- **`MarkdownOptions::validate` signature change.** Now returns
  `Result<(), Vec<(String, ValidationError)>>` and runs nine consistency
  checks via `commons::validation::Validator`; every failing check
  surfaces with its field name. The pipeline converts the resulting
  `Vec` into a single `MarkdownError::InvalidOptionsError` via a new
  `From` impl so `process_markdown` callers see the same error shape.
- **`MarkdownOptions::default()` is now internally consistent.** Default
  comrak options enable `extension.table = true` to match the default
  `enable_enhanced_tables = true`. Previously the combination tripped
  `validate()` and required callers to either disable enhanced tables
  or wire comrak tables by hand.

### Added

- **AST-level processing pipeline.** Parse → custom-block transform →
  diagram transform → ToC collection → table enhancement → comrak render
  → sanitize. Every transformation happens on the comrak AST rather than
  as post-render regex on HTML.
- `process_markdown_to_writer<W: Write>(content, &mut writer, options)`
  — streams rendered HTML directly into any `std::io::Write` sink,
  skipping the intermediate `String` allocation. Sanitised path uses
  `ammonia::Document::write_to()` end-to-end.
- `process_markdown_with_toc(content, options) -> (String, Vec<Heading>)`
  and the streaming variant `process_markdown_with_toc_to_writer`. Both
  return a document-order outline alongside the HTML, with anchor ids
  that match what comrak renders, including dedup suffixes (`-1`, `-2`).
- `process_markdown_to_plain_text` — AST-walking text extractor for
  search indexes, excerpts, reading-time calculation. Captures inline
  code and fenced code-block literals; inserts space separators between
  structural elements.
- `Heading { level, text, id }` and the building-block walker
  `collect_headings(root, prefix)`.
- `SanitizerConfig` — typed extension point for the ammonia allow-list
  (extra tags, per-tag attributes, generic attributes, per-tag allowed
  classes) without leaking `ammonia::Builder` through the public API.
  Per-tag class restriction correctly swaps the tag out of the permissive
  default mode.
- `theme_css(theme_name) -> Option<String>` for generating a stylesheet
  matched to the class-based highlighter output.
- `SyntectAdapter::theme_name()` so callers can recover the resolved
  theme after fallback.
- `MarkdownError::IoError(std::io::Error)` with `From` impls for `?`
  propagation through writer-based entry points.
- **Mermaid diagram rendering.** Fenced code blocks tagged `mermaid` are
  rewritten into sanitizer-safe `<pre class="mermaid">…</pre>` containers
  that a small client-side JS module hydrates into inline SVG at
  page-load time. Mirrors how github.com renders mermaid in READMEs.
  Enable with `MarkdownOptions::with_diagrams(true)`; embed
  `mdx_gen::hydration_script_html()` once per page. New `mdx_gen::diagrams`
  module + `examples/diagrams.rs` showcase all nine mermaid kinds —
  flowchart, sequence, ER, class, state, gantt, git graph, user journey,
  pie — each with a "when to use" blurb.
- **Comprehensive `MarkdownOptions::validate`.** Nine consistency checks:
  1. `enable_enhanced_tables = true` requires `comrak.extension.table`.
  2. `syntax_theme` (if set) must be a bundled syntect theme.
  3. `syntax_theme` set with `enable_syntax_highlighting = false` is
     rejected (silent no-op otherwise).
  4. `sanitizer_config` set with `allow_unsafe_html = true` is rejected
     (sanitizer would never run).
  5. `header_ids` prefix rejects whitespace and HTML-breaking characters
     (`"`, `'`, `<`, `>`, `&`, `=`).
  6. `sanitizer_config.extra_tags` / `extra_tag_attributes` must use
     valid HTML tag / attribute names.
  7. `sanitizer_config.extra_generic_attributes` must be valid HTML
     attribute names.
  8. `sanitizer_config.extra_allowed_classes` tags must be valid; class
     values must be non-empty without whitespace or quotes.
  9. `custom_block_config.class_overrides` values must be non-empty
     without whitespace/quotes; `title_overrides` values must be
     non-blank.
- **Vendored `crates/commons/`** — EUXIS ecosystem shared utilities
  (error, validation, time, retry, fs, env, logging, collections, config,
  id). `publish = false` so mdx-gen can evolve against the source without
  a separate registry release. Pulled in with `error` + `validation`
  features only for now.
- **Vendored `crates/yaml_safe/`** — replaces the previous in-tree YAML
  parser. Adds block-style sequences under mapping keys, literal/folded
  block scalars, anchors + aliases + merge keys (`<<`), and multi-document
  YAML (`DocumentIter`). The frontmatter examples switched to idiomatic
  block YAML accordingly.
- `impl From<commons::error::CommonError> for MarkdownError` — the
  EUXIS-wide error type flows through `?` into mdx-gen's `Result` types.
  Domain-specific variants stay intact (`InvalidInput`/`Parse` →
  `ParseError`, `Io` → `IoError`, rest → `ConversionError` with Display
  preserved).
- `impl From<commons::validation::ValidationError> for MarkdownError`
  for the validation bridge.
- `fuzz/` workspace with three `libfuzzer-sys` targets
  (`process_markdown`, `custom_blocks`, `sanitize_roundtrip`). Run with
  `cargo +nightly fuzz run <target>` from `fuzz/`.
- **18 standalone examples in `examples/`** covering every public
  surface: `basic`, `quickstart`, `blog`, `typed`, `docs`, `alerts`,
  `cms`, `security`, `site`, `diagrams`, `styling`, `gallery`,
  `streaming`, `pipe`, `search`, `bulk`, `errors`, plus an `all` runner.
- `examples/support.rs` — shared spinner + checkmark helper for example
  output, ported from the noyalib idiom.
- Sanitizer allow-list extended with `pre.mermaid` so mermaid containers
  survive `allow_unsafe_html = false`.

### Changed

- `comrak` 0.50 → 0.52. The deprecated `extension.header_ids` field is
  now `extension.header_id_prefix`; mdx-gen forwards the value
  transparently.
- `criterion` (dev-dep) 0.5 → 0.8. `criterion::black_box` deprecated in
  favour of `std::hint::black_box`; the bench was updated.
- Default features now include `yaml_support` so `cargo run --example
  blog` works out of the box. Use `--no-default-features` for a minimal
  build.
- **CDN host migrated** from `https://kura.pro/mdx-gen/images/` to
  `https://cloudcdn.pro/mdx-gen/v1/` across `src/lib.rs` (favicon +
  logo), `README.md`, and `TEMPLATE.md`.
- Closed PR #17 (pre-release proposal to add commons) — the vendored +
  migrated form landed directly on this branch.

### Performance

- `ammonia::Builder` constructed once per process via
  `LazyLock<Builder<'static>>` instead of per-call. The configuration is
  all `'static` strs, so `Builder::clean(&self, …)` reuses a single
  builder for every default-config sanitize.
- Table enhancement moved to AST-level (a pre-render pass over comrak
  nodes) rather than post-render regex on rendered HTML.
- Literal-string `<table>` / `</table>` regex replacements switched to
  `str::replace`. The capturing `<td …>` rewrite stays as a regex — it
  genuinely needs pattern capture.
- Pipeline body factored into a private `pipeline()` helper so every
  public entry point shares one implementation.

### Security

- `cargo-fuzz` scaffolding for ongoing parser hardening. 0 crashes
  across ~2.3 M iterations on the initial smoke run.
- `style` attribute removed from the default sanitizer allow-list (see
  Breaking). Regression test asserts a `position:fixed` overlay is
  stripped.
- Input size cap (`DEFAULT_MAX_INPUT_SIZE = 1 MiB`, overridable via
  `with_max_input_size`). Oversized input returns
  `MarkdownError::InputTooLarge`.
- Math (`extension.math_dollars`) and footnote (`extension.footnotes`)
  output survive sanitization — verified by regression tests that feed
  real comrak output through the default sanitizer.
- `examples/security.rs` — dedicated red-team scenario covering XSS via
  `<script>`, `on*` event handlers, clickjacking via inline style,
  `javascript:` / `data:` URLs, oversized input via the cap, and deeply
  nested blockquote bombs.

### Tests & coverage

- **857 tests pass** across the workspace (`cargo test --workspace
  --all-features`): mdx-gen lib + integration + doc-tests, `commons`
  unit + env integration (13 tests in `crates/commons/tests/
  env_mutation.rs`, isolated from the library's `forbid(unsafe_code)`),
  `yaml_safe` unit + integration + coverage suite.
- **Workspace line coverage: 95.64 %** (`cargo llvm-cov --workspace
  --all-features`). Every file sits at ≥ 95 % except the explicitly
  excluded `yaml_safe/src/de.rs` (85.71 %, the upstream parser).

### CI / tooling

- `rust-toolchain.toml` pins `stable` with `clippy` + `rustfmt`
  components.
- `.githooks/pre-push` runs `cargo fmt --check`, `cargo clippy
  --all-targets --all-features --workspace -- -D warnings`, and `cargo
  test --workspace --all-features --no-fail-fast` before every push.
  Opt-in: `git config core.hooksPath .githooks`.
- Workflow consolidation: old per-job `.github/workflows/{audit,check,
  coverage,document,lint,release,test}.yml` replaced with a single
  `ci.yml` that delegates to the centralised `sebastienrousseau/
  pipelines/.github/workflows/rust-ci.yml`.
- **GitHub Pages migrated to Actions-based deployment** via the shared
  `docs.yml` pipeline with `cname: doc.mdxgen.com`. The `gh-pages`
  branch is retired.
- `codecov.yml` — project and patch checks marked `informational: true`
  until the first upload on `main` establishes a baseline; targets
  (90 % project / 85 % patch) stay in place for post-merge tightening.

### Internal

- `#![deny(missing_docs)]` enforced on the public crate; every public
  item carries a docstring.
- `deny.toml` added for `cargo-deny` (license allow-list, advisory deny
  + `yanked = "deny"`, wildcard bans, crates.io-only sources).

### Closed

- Closed 8 stale Dependabot PRs (#17, #19–#26). Two (`comrak`,
  `criterion`) applied here; six obsoleted by the consolidated CI
  workflow + dropped `toml` dependency on `feat/v0.0.3`.

[Unreleased]: https://github.com/sebastienrousseau/mdx-gen/compare/v0.0.3...HEAD
[0.0.3]: https://github.com/sebastienrousseau/mdx-gen/releases/tag/v0.0.3
