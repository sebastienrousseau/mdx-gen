//! Syntax highlighting adapter for comrak.
//!
//! Implements comrak's `SyntaxHighlighterAdapter` trait using
//! syntect's class-based generator. Output uses CSS class names
//! (`<span class="…">`) rather than inline `style="…"` attributes,
//! which means callers must ship a stylesheet — see [`theme_css`]
//! for generating one from any built-in theme.

use comrak::adapters::{SyntaxHighlightMeta, SyntaxHighlighterAdapter};
use comrak::nodes::NodeValue;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Write};
use std::sync::LazyLock;
use syntect::highlighting::ThemeSet;
use syntect::html::{
    css_for_theme_with_class_style, ClassStyle, ClassedHTMLGenerator,
};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// Cached `SyntaxSet` to avoid reloading on every function call.
static SYNTAX_SET: LazyLock<SyntaxSet> =
    LazyLock::new(SyntaxSet::load_defaults_newlines);
/// Cached `ThemeSet` to avoid reloading on every function call.
static THEME_SET: LazyLock<ThemeSet> =
    LazyLock::new(ThemeSet::load_defaults);

/// Default theme used when none is specified.
pub const DEFAULT_THEME: &str = "base16-ocean.dark";

/// Class style used by every generator we construct.
///
/// `ClassStyle::Spaced` emits ` class="foo bar"` form, which works
/// with the CSS produced by [`theme_css`].
const CLASS_STYLE: ClassStyle = ClassStyle::Spaced;

/// A syntect-backed adapter for comrak's rendering plugin system.
///
/// Performs syntax highlighting during HTML rendering using
/// class-based output. The adapter does not emit `<pre>` / `<code>`
/// itself — comrak's renderer handles those tags via
/// [`SyntaxHighlighterAdapter::write_pre_tag`] and
/// [`SyntaxHighlighterAdapter::write_code_tag`].
pub struct SyntectAdapter {
    theme_name: String,
    prepared: Prepared,
}

/// Highlighted output computed ahead of the render, keyed by
/// language token, then by code. Filled by [`prehighlight`].
pub(crate) type Prepared = HashMap<String, HashMap<String, String>>;

/// A document needs at least this many distinct code blocks per
/// worker thread before [`prehighlight`] spreads them across threads.
/// Below it, thread start-up costs more than it saves.
pub(crate) const MIN_BLOCKS_PER_THREAD: usize = 8;

/// Upper bound on worker threads used by [`prehighlight`].
pub(crate) const MAX_HIGHLIGHT_THREADS: usize = 8;

impl SyntectAdapter {
    /// Creates a new adapter, optionally with a named theme.
    ///
    /// The theme name is retained so callers can recover it via
    /// [`SyntectAdapter::theme_name`] and pass it to [`theme_css`]
    /// when generating a stylesheet for the rendered output. Falls
    /// back to [`DEFAULT_THEME`] when the requested theme is not in
    /// syntect's built-in set.
    pub fn new(theme: Option<&str>) -> Self {
        let theme_name = theme
            .filter(|t| THEME_SET.themes.contains_key(*t))
            .unwrap_or(DEFAULT_THEME)
            .to_owned();
        Self {
            theme_name,
            prepared: Prepared::new(),
        }
    }

    /// Serves blocks found in `prepared` instead of highlighting
    /// them again during the render.
    pub(crate) fn with_prepared(mut self, prepared: Prepared) -> Self {
        self.prepared = prepared;
        self
    }

    /// Returns the resolved theme name (after fallback).
    pub fn theme_name(&self) -> &str {
        &self.theme_name
    }

    /// Returns the list of available theme names.
    pub fn available_themes() -> Vec<&'static str> {
        THEME_SET.themes.keys().map(|s| s.as_str()).collect()
    }
}

impl SyntaxHighlighterAdapter for SyntectAdapter {
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        highlight: &SyntaxHighlightMeta<'_>,
    ) -> fmt::Result {
        let SyntaxHighlightMeta { lang, code, .. } = *highlight;
        let ready = self
            .prepared
            .get(lang)
            .and_then(|by_code| by_code.get(code));
        match ready {
            Some(html) => output.write_str(html),
            None => {
                output.write_str(&highlight_block(Some(lang), code))
            }
        }
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn Write,
        _highlight: &SyntaxHighlightMeta<'_>,
        attributes: BTreeMap<&str, Cow<'_, str>>,
    ) -> fmt::Result {
        write!(output, "<pre")?;
        for (attr, value) in &attributes {
            write!(
                output,
                " {}=\"{}\"",
                attr,
                html_escape::encode_double_quoted_attribute(value)
            )?;
        }
        write!(output, ">")
    }

    fn write_code_tag(
        &self,
        output: &mut dyn Write,
        _highlight: &SyntaxHighlightMeta<'_>,
        attributes: BTreeMap<&str, Cow<'_, str>>,
    ) -> fmt::Result {
        write!(output, "<code")?;
        for (attr, value) in &attributes {
            // Skip empty attributes
            if value.is_empty() {
                continue;
            }
            write!(
                output,
                " {}=\"{}\"",
                attr,
                html_escape::encode_double_quoted_attribute(value)
            )?;
        }
        write!(output, ">")
    }
}

/// Highlights one code block with class-based spans. Falls back to
/// plain escaped text if syntect gives up part-way, rather than
/// emitting a half-built span tree.
fn highlight_block(lang: Option<&str>, code: &str) -> String {
    let syntax = lang
        .and_then(|l| SYNTAX_SET.find_syntax_by_token(l))
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    let mut generator = ClassedHTMLGenerator::new_with_class_style(
        syntax,
        &SYNTAX_SET,
        CLASS_STYLE,
    );

    for line in LinesWithEndings::from(code) {
        if generator
            .parse_html_for_line_which_includes_newline(line)
            .is_err()
        {
            return html_escape::encode_text(code).into_owned();
        }
    }

    generator.finalize()
}

/// Splits a fenced code block's info string the way comrak does
/// before calling the highlighter: the language token is everything
/// up to the first ASCII whitespace.
fn info_lang(info: &str) -> &str {
    let end = info
        .find(|c: char| {
            matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0b' | '\x0c')
        })
        .unwrap_or(info.len());
    &info[..end]
}

/// Distinct `(language, code)` pairs of the code blocks under `root`,
/// in document order. `math` blocks are skipped: comrak renders them
/// without the highlighter.
fn collect_code_blocks<'a>(
    root: comrak::nodes::Node<'a>,
) -> Vec<(String, String)> {
    let mut seen = std::collections::HashSet::new();
    let mut jobs = Vec::new();
    for node in root.descendants() {
        if let NodeValue::CodeBlock(ref cb) = node.data.borrow().value {
            let lang = info_lang(&cb.info);
            if lang != "math"
                && seen.insert((lang.to_owned(), cb.literal.clone()))
            {
                jobs.push((lang.to_owned(), cb.literal.clone()));
            }
        }
    }
    jobs
}

/// Worker threads worth starting for `blocks` code blocks: none when
/// the machine has one core or the document has too few blocks.
fn worker_count(blocks: usize) -> usize {
    let cores = std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1);
    cores
        .min(MAX_HIGHLIGHT_THREADS)
        .min(blocks / MIN_BLOCKS_PER_THREAD)
}

/// Highlights the document's code blocks on scoped worker threads
/// ahead of the render, for documents with enough blocks to make it
/// pay (see [`MIN_BLOCKS_PER_THREAD`]).
///
/// Returns an empty map when the document is too small or threads
/// are unavailable; blocks missing from the map (including any whose
/// worker could not start) are highlighted serially during the
/// render, so the output is identical either way.
pub(crate) fn prehighlight<'a>(
    root: comrak::nodes::Node<'a>,
) -> Prepared {
    let jobs = collect_code_blocks(root);
    let threads = worker_count(jobs.len());
    if threads < 2 {
        return Prepared::new();
    }
    // Stripe blocks across workers so long and short blocks spread
    // evenly instead of landing in one contiguous chunk.
    let outputs: Vec<Option<Vec<String>>> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let jobs = &jobs;
                std::thread::Builder::new().spawn_scoped(s, move || {
                    jobs.iter()
                        .skip(t)
                        .step_by(threads)
                        .map(|(lang, code)| {
                            highlight_block(Some(lang), code)
                        })
                        .collect::<Vec<String>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.ok().and_then(|h| h.join().ok()))
            .collect()
    });
    assemble(jobs, outputs, threads)
}

/// Files each worker's striped output back under its block's key.
fn assemble(
    jobs: Vec<(String, String)>,
    outputs: Vec<Option<Vec<String>>>,
    threads: usize,
) -> Prepared {
    let mut prepared = Prepared::new();
    for (t, out) in outputs.into_iter().enumerate() {
        let Some(out) = out else { continue };
        let keys = jobs.iter().skip(t).step_by(threads);
        for ((lang, code), html) in keys.zip(out) {
            let _ = prepared
                .entry(lang.clone())
                .or_default()
                .insert(code.clone(), html);
        }
    }
    prepared
}

/// Highlights a code string with class-based spans (standalone API).
///
/// This is the public entry point for callers who want to highlight
/// code outside of the markdown pipeline. Output is a sequence of
/// `<span class="…">` tags — pair it with [`theme_css`] to produce
/// a stylesheet that renders the colours.
pub fn apply_syntax_highlighting(
    code: &str,
    lang: &str,
) -> Result<String, crate::error::MarkdownError> {
    let syntax = SYNTAX_SET
        .find_syntax_by_token(lang)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    let mut generator = ClassedHTMLGenerator::new_with_class_style(
        syntax,
        &SYNTAX_SET,
        CLASS_STYLE,
    );

    for line in LinesWithEndings::from(code) {
        generator
            .parse_html_for_line_which_includes_newline(line)
            .map_err(|e| {
                crate::error::MarkdownError::SyntaxHighlightError(
                    e.to_string(),
                )
            })?;
    }

    Ok(generator.finalize())
}

/// Generates a CSS stylesheet for the named built-in theme.
///
/// Returns `None` if the theme is not present. The generated CSS
/// targets the class names emitted by [`apply_syntax_highlighting`]
/// and the comrak adapter, so callers can either inline the result
/// in a `<style>` block or write it to a `.css` file.
pub fn theme_css(theme_name: &str) -> Option<String> {
    let theme = THEME_SET.themes.get(theme_name)?;
    css_for_theme_with_class_style(theme, CLASS_STYLE).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Code-block metadata for the tag-writing tests, which ignore it.
    fn meta() -> SyntaxHighlightMeta<'static> {
        SyntaxHighlightMeta::new("", "", "")
    }

    /// A document with `n` distinct fenced blocks in several
    /// languages, plus an indented block and a duplicate.
    fn many_blocks(n: usize) -> String {
        let langs = ["rust", "python", "toml", "", "nonexistent"];
        let mut md = String::new();
        for i in 0..n {
            let lang = langs[i % langs.len()];
            md.push_str(&format!(
                "```{lang} extra meta\nlet x_{i} = \"<{i}>\";\n```\n\n"
            ));
        }
        md.push_str(
            "    indented <block>\n\n```rust\nlet x_0 = 1;\n```\n",
        );
        md
    }

    fn render(md: &str, parallel: bool) -> String {
        let options = crate::MarkdownOptions::default()
            .with_parallel_highlighting(parallel);
        crate::process_markdown(md, &options).unwrap()
    }

    #[test]
    fn parallel_highlighting_matches_serial_output() {
        let md = many_blocks(MIN_BLOCKS_PER_THREAD * 4);
        let serial = render(&md, false);
        assert!(serial.contains("<span class="), "{serial}");
        assert_eq!(render(&md, true), serial);
    }

    #[test]
    fn prehighlight_covers_every_block_of_a_large_document() {
        let md = many_blocks(MIN_BLOCKS_PER_THREAD * 4);
        let arena = comrak::Arena::new();
        let root = comrak::parse_document(
            &arena,
            &md,
            &comrak::Options::default(),
        );
        let jobs = collect_code_blocks(root);
        // n fenced blocks + the indented block; the trailing rust
        // block is new code, so it counts too.
        assert_eq!(jobs.len(), MIN_BLOCKS_PER_THREAD * 4 + 2);
        let prepared = prehighlight(root);
        if worker_count(jobs.len()) < 2 {
            assert!(
                prepared.is_empty(),
                "single core: nothing prepared"
            );
            return;
        }
        for (lang, code) in &jobs {
            let html = &prepared[lang][code];
            assert_eq!(html, &highlight_block(Some(lang), code));
        }
    }

    #[test]
    fn small_documents_are_not_split() {
        let arena = comrak::Arena::new();
        let root = comrak::parse_document(
            &arena,
            &many_blocks(MIN_BLOCKS_PER_THREAD - 1),
            &comrak::Options::default(),
        );
        assert!(prehighlight(root).is_empty());
    }

    #[test]
    fn info_lang_matches_comrak_split() {
        assert_eq!(info_lang("rust"), "rust");
        assert_eq!(info_lang("rust ignore"), "rust");
        assert_eq!(info_lang("py\tx"), "py");
        assert_eq!(info_lang(""), "");
    }

    #[test]
    fn math_blocks_are_not_collected() {
        let arena = comrak::Arena::new();
        let root = comrak::parse_document(
            &arena,
            "```math\nx^2\n```\n",
            &comrak::Options::default(),
        );
        assert!(collect_code_blocks(root).is_empty());
    }

    #[test]
    fn test_syntect_adapter_emits_class_spans() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();
        adapter
            .write_highlighted(
                &mut output,
                &SyntaxHighlightMeta::new("rust", "", "fn main() {}"),
            )
            .unwrap();
        assert!(
            output.contains("<span class="),
            "should contain class-based syntax spans, got: {output}"
        );
        assert!(
            !output.contains(" style=\""),
            "must not contain inline styles: {output}"
        );
    }

    #[test]
    fn test_syntect_adapter_unknown_lang_fallback() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();
        adapter
            .write_highlighted(
                &mut output,
                &SyntaxHighlightMeta::new(
                    "nonexistent-lang-xyz",
                    "",
                    "hello world",
                ),
            )
            .unwrap();
        // Should not panic, should produce some output
        assert!(!output.is_empty());
    }

    #[test]
    fn test_syntect_adapter_invalid_theme_falls_back() {
        let adapter = SyntectAdapter::new(Some("no-such-theme"));
        assert_eq!(adapter.theme_name(), DEFAULT_THEME);
    }

    #[test]
    fn test_available_themes_not_empty() {
        let themes = SyntectAdapter::available_themes();
        assert!(!themes.is_empty());
        assert!(themes.contains(&DEFAULT_THEME));
    }

    #[test]
    fn test_standalone_highlighting_emits_classes() {
        let html =
            apply_syntax_highlighting("fn main() {}", "rust").unwrap();
        assert!(html.contains("<span class="));
        assert!(!html.contains(" style=\""));
    }

    #[test]
    fn test_write_pre_tag_with_attributes() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();
        let mut attrs = BTreeMap::new();
        attrs.insert("class", Cow::Borrowed("highlight"));
        attrs.insert("data-lang", Cow::Borrowed("rust"));
        adapter.write_pre_tag(&mut output, &meta(), attrs).unwrap();
        assert!(output.starts_with("<pre"));
        assert!(output.ends_with('>'));
        assert!(output.contains("class=\"highlight\""));
        assert!(output.contains("data-lang=\"rust\""));
    }

    #[test]
    fn test_write_pre_tag_no_attributes() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();
        adapter
            .write_pre_tag(&mut output, &meta(), BTreeMap::new())
            .unwrap();
        assert_eq!(output, "<pre>");
    }

    #[test]
    fn test_write_code_tag_with_attributes() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();
        let mut attrs = BTreeMap::new();
        attrs.insert("class", Cow::Borrowed("language-rust"));
        adapter.write_code_tag(&mut output, &meta(), attrs).unwrap();
        assert!(output.starts_with("<code"));
        assert!(output.ends_with('>'));
        assert!(output.contains("class=\"language-rust\""));
    }

    #[test]
    fn test_write_code_tag_skips_empty_attributes() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();
        let mut attrs = BTreeMap::new();
        attrs.insert("class", Cow::Borrowed(""));
        attrs.insert("id", Cow::Borrowed("my-code"));
        adapter.write_code_tag(&mut output, &meta(), attrs).unwrap();
        // Empty "class" value should be skipped
        assert!(!output.contains("class"));
        // Non-empty "id" should be present
        assert!(output.contains("id=\"my-code\""));
    }

    #[test]
    fn test_write_code_tag_no_attributes() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();
        adapter
            .write_code_tag(&mut output, &meta(), BTreeMap::new())
            .unwrap();
        assert_eq!(output, "<code>");
    }

    #[test]
    fn test_write_pre_tag_escapes_attribute_values() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();
        let mut attrs = BTreeMap::new();
        attrs.insert("data-info", Cow::Borrowed("a\"b"));
        adapter.write_pre_tag(&mut output, &meta(), attrs).unwrap();
        // The double quote inside the value should be escaped
        assert!(!output.contains("a\"b"));
        assert!(output.contains("data-info="));
    }

    #[test]
    fn test_write_highlighted_no_lang() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();
        adapter
            .write_highlighted(
                &mut output,
                &SyntaxHighlightMeta::new("", "", "plain text"),
            )
            .unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_standalone_highlighting_unknown_language() {
        // Unknown language should fall back to plain text, not error
        let result = apply_syntax_highlighting(
            "hello world",
            "nonexistent-language-xyz",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_theme_css_known_theme() {
        let css = theme_css(DEFAULT_THEME).expect("default theme");
        assert!(css.contains(".code"));
    }

    #[test]
    fn test_theme_css_unknown_theme() {
        assert!(theme_css("no-such-theme").is_none());
    }

    /// Minimal `fmt::Write` sink that rejects the first write. Used
    /// to exercise the `?`-on-write_str error paths in the adapter.
    struct FailingWrite;
    impl fmt::Write for FailingWrite {
        fn write_str(&mut self, _: &str) -> fmt::Result {
            Err(fmt::Error)
        }
    }

    #[test]
    fn test_write_pre_tag_propagates_write_error() {
        let adapter = SyntectAdapter::new(None);
        let err = adapter.write_pre_tag(
            &mut FailingWrite,
            &meta(),
            BTreeMap::new(),
        );
        assert!(err.is_err());
    }

    #[test]
    fn test_write_pre_tag_propagates_attribute_write_error() {
        // First write (`<pre`) succeeds into the throwaway String
        // wrapper below; the attribute write errors on first call.
        struct FailAfterFirst(usize);
        impl fmt::Write for FailAfterFirst {
            fn write_str(&mut self, _: &str) -> fmt::Result {
                if self.0 == 0 {
                    self.0 += 1;
                    Ok(())
                } else {
                    Err(fmt::Error)
                }
            }
        }

        let adapter = SyntectAdapter::new(None);
        let mut attrs = BTreeMap::new();
        attrs.insert("class", Cow::Borrowed("demo"));
        let err = adapter
            .write_pre_tag(&mut FailAfterFirst(0), &meta(), attrs)
            .unwrap_err();
        let _ = err; // just asserting it errored
    }

    #[test]
    fn test_write_code_tag_propagates_write_error() {
        let adapter = SyntectAdapter::new(None);
        let err = adapter.write_code_tag(
            &mut FailingWrite,
            &meta(),
            BTreeMap::new(),
        );
        assert!(err.is_err());
    }

    #[test]
    fn test_write_code_tag_propagates_attribute_write_error() {
        struct FailAfterFirst(usize);
        impl fmt::Write for FailAfterFirst {
            fn write_str(&mut self, _: &str) -> fmt::Result {
                if self.0 == 0 {
                    self.0 += 1;
                    Ok(())
                } else {
                    Err(fmt::Error)
                }
            }
        }

        let adapter = SyntectAdapter::new(None);
        let mut attrs = BTreeMap::new();
        attrs.insert("class", Cow::Borrowed("language-rust"));
        let err = adapter
            .write_code_tag(&mut FailAfterFirst(0), &meta(), attrs)
            .unwrap_err();
        let _ = err;
    }

    #[test]
    fn test_write_highlighted_propagates_write_error() {
        // FailingWrite rejects immediately, so both the fallback
        // branch and the normal finalize branch error out.
        let adapter = SyntectAdapter::new(None);
        let err = adapter
            .write_highlighted(
                &mut FailingWrite,
                &SyntaxHighlightMeta::new("rust", "", "fn x() {}"),
            )
            .unwrap_err();
        let _ = err;
    }
}
