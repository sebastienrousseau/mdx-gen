//! Syntax highlighting adapter for comrak.
//!
//! Implements comrak's `SyntaxHighlighterAdapter` trait using syntect,
//! allowing syntax highlighting to run during the rendering phase
//! instead of as a fragile regex post-processing step.

use comrak::adapters::SyntaxHighlighterAdapter;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{self, Write};
use std::sync::LazyLock;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

/// Cached `SyntaxSet` to avoid reloading on every function call.
static SYNTAX_SET: LazyLock<SyntaxSet> =
    LazyLock::new(SyntaxSet::load_defaults_newlines);
/// Cached `ThemeSet` to avoid reloading on every function call.
static THEME_SET: LazyLock<ThemeSet> =
    LazyLock::new(ThemeSet::load_defaults);

/// Default theme used when none is specified.
pub const DEFAULT_THEME: &str = "base16-ocean.dark";

/// A syntect-backed adapter for comrak's rendering plugin system.
///
/// This performs syntax highlighting during HTML rendering rather
/// than via regex post-processing, which is both faster and more
/// robust.
pub struct SyntectAdapter {
    theme_name: String,
}

impl SyntectAdapter {
    /// Creates a new adapter with the given theme name.
    ///
    /// Falls back to [`DEFAULT_THEME`] if the requested theme
    /// is not found in syntect's built-in theme set.
    pub fn new(theme: Option<&str>) -> Self {
        let theme_name = theme
            .filter(|t| THEME_SET.themes.contains_key(*t))
            .unwrap_or(DEFAULT_THEME)
            .to_owned();
        Self { theme_name }
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
        let theme = &THEME_SET.themes[&self.theme_name];
        let syntax = lang
            .and_then(|l| SYNTAX_SET.find_syntax_by_token(l))
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

        match highlighted_html_for_string(
            code,
            &SYNTAX_SET,
            syntax,
            theme,
        ) {
            Ok(html) => {
                // syntect wraps output in <pre style="...">...</pre>.
                // Comrak already emits <pre><code> via write_pre_tag/
                // write_code_tag, so strip syntect's wrapper.
                let inner = strip_pre_wrapper(&html);
                output.write_str(inner)
            }
            // Fall back to plain text on highlighting failure
            Err(_) => output.write_str(&html_escape::encode_text(code)),
        }
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

/// Strips the `<pre style="...">...</pre>` wrapper that syntect adds,
/// returning only the inner highlighted spans.
fn strip_pre_wrapper(html: &str) -> &str {
    let s = html.trim();
    // syntect output: <pre style="...">CONTENT\n</pre>
    let after_open = s.find('>').map(|i| &s[i + 1..]).unwrap_or(s);
    after_open
        .strip_suffix("</pre>")
        .unwrap_or(after_open)
        .strip_suffix('\n')
        .unwrap_or(after_open)
}

/// Applies syntax highlighting to a code string (standalone usage).
///
/// This is the public API for callers who want to highlight code
/// outside of the markdown pipeline.
pub fn apply_syntax_highlighting(
    code: &str,
    lang: &str,
) -> Result<String, crate::error::MarkdownError> {
    let theme = &THEME_SET.themes[DEFAULT_THEME];
    let syntax = SYNTAX_SET
        .find_syntax_by_token(lang)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    highlighted_html_for_string(code, &SYNTAX_SET, syntax, theme)
        .map_err(|e| {
            crate::error::MarkdownError::SyntaxHighlightError(
                e.to_string(),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntect_adapter_highlights_rust() {
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
            output.contains("<span"),
            "should contain syntax spans"
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
        assert_eq!(adapter.theme_name, DEFAULT_THEME);
    }

    #[test]
    fn test_available_themes_not_empty() {
        let themes = SyntectAdapter::available_themes();
        assert!(!themes.is_empty());
        assert!(themes.contains(&DEFAULT_THEME));
    }

    #[test]
    fn test_standalone_highlighting() {
        let result = apply_syntax_highlighting("fn main() {}", "rust");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("<span"));
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
}
