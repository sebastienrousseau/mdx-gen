//! Syntax highlighting adapter for comrak.
//!
//! Implements comrak's `SyntaxHighlighterAdapter` trait using
//! syntect's class-based generator. Output uses CSS class names
//! (`<span class="…">`) rather than inline `style="…"` attributes,
//! which means callers must ship a stylesheet — see [`theme_css`]
//! for generating one from any built-in theme.

use comrak::adapters::SyntaxHighlighterAdapter;
use std::borrow::Cow;
use std::collections::HashMap;
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
}

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
        Self { theme_name }
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
        lang: Option<&str>,
        code: &str,
    ) -> fmt::Result {
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
                // Highlighter gave up part-way: fall back to plain
                // escaped text rather than emitting a half-built
                // span tree.
                return output
                    .write_str(&html_escape::encode_text(code));
            }
        }

        output.write_str(&generator.finalize())
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<&'static str, Cow<'_, str>>,
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
        attributes: HashMap<&'static str, Cow<'_, str>>,
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

    #[test]
    fn test_syntect_adapter_emits_class_spans() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();
        adapter
            .write_highlighted(
                &mut output,
                Some("rust"),
                "fn main() {}",
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
                Some("nonexistent-lang-xyz"),
                "hello world",
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
        let mut attrs = HashMap::new();
        attrs.insert("class", Cow::Borrowed("highlight"));
        attrs.insert("data-lang", Cow::Borrowed("rust"));
        adapter.write_pre_tag(&mut output, attrs).unwrap();
        assert!(output.starts_with("<pre"));
        assert!(output.ends_with('>'));
        assert!(output.contains("class=\"highlight\""));
        assert!(output.contains("data-lang=\"rust\""));
    }

    #[test]
    fn test_write_pre_tag_no_attributes() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();
        adapter.write_pre_tag(&mut output, HashMap::new()).unwrap();
        assert_eq!(output, "<pre>");
    }

    #[test]
    fn test_write_code_tag_with_attributes() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();
        let mut attrs = HashMap::new();
        attrs.insert("class", Cow::Borrowed("language-rust"));
        adapter.write_code_tag(&mut output, attrs).unwrap();
        assert!(output.starts_with("<code"));
        assert!(output.ends_with('>'));
        assert!(output.contains("class=\"language-rust\""));
    }

    #[test]
    fn test_write_code_tag_skips_empty_attributes() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();
        let mut attrs = HashMap::new();
        attrs.insert("class", Cow::Borrowed(""));
        attrs.insert("id", Cow::Borrowed("my-code"));
        adapter.write_code_tag(&mut output, attrs).unwrap();
        // Empty "class" value should be skipped
        assert!(!output.contains("class"));
        // Non-empty "id" should be present
        assert!(output.contains("id=\"my-code\""));
    }

    #[test]
    fn test_write_code_tag_no_attributes() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();
        adapter.write_code_tag(&mut output, HashMap::new()).unwrap();
        assert_eq!(output, "<code>");
    }

    #[test]
    fn test_write_pre_tag_escapes_attribute_values() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();
        let mut attrs = HashMap::new();
        attrs.insert("data-info", Cow::Borrowed("a\"b"));
        adapter.write_pre_tag(&mut output, attrs).unwrap();
        // The double quote inside the value should be escaped
        assert!(!output.contains("a\"b"));
        assert!(output.contains("data-info="));
    }

    #[test]
    fn test_write_highlighted_no_lang() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();
        adapter
            .write_highlighted(&mut output, None, "plain text")
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
        let err =
            adapter.write_pre_tag(&mut FailingWrite, HashMap::new());
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
        let mut attrs = HashMap::new();
        attrs.insert("class", Cow::Borrowed("demo"));
        let err = adapter
            .write_pre_tag(&mut FailAfterFirst(0), attrs)
            .unwrap_err();
        let _ = err; // just asserting it errored
    }

    #[test]
    fn test_write_code_tag_propagates_write_error() {
        let adapter = SyntectAdapter::new(None);
        let err =
            adapter.write_code_tag(&mut FailingWrite, HashMap::new());
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
        let mut attrs = HashMap::new();
        attrs.insert("class", Cow::Borrowed("language-rust"));
        let err = adapter
            .write_code_tag(&mut FailAfterFirst(0), attrs)
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
                Some("rust"),
                "fn x() {}",
            )
            .unwrap_err();
        let _ = err;
    }
}
