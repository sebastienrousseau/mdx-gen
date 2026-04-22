//! YAML frontmatter extraction and parsing.
//!
//! Extracts YAML frontmatter delimited by `---` from the beginning of
//! Markdown content. Requires the `yaml_support` feature.

#[cfg(feature = "yaml_support")]
use crate::error::MarkdownError;

/// Extracts YAML frontmatter from Markdown content.
///
/// Looks for content between `---` delimiters at the start of the
/// document. Returns `(frontmatter_yaml, remaining_markdown)`.
/// If no frontmatter is found, returns `(None, original_content)`.
pub fn extract_frontmatter(content: &str) -> (Option<&str>, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (None, content);
    }

    // Skip the opening "---" and any trailing whitespace on that line
    let after_open = &trimmed[3..];
    let after_open = after_open
        .strip_prefix('\n')
        .or_else(|| after_open.strip_prefix("\r\n"))
        .unwrap_or(after_open);

    // Find the closing "---" (either at the start or after a newline)
    let close_pos = if after_open.starts_with("---") {
        Some(0)
    } else {
        after_open.find("\n---").map(|i| i + 1) // +1 to skip the \n
    };

    if let Some(pos) = close_pos {
        let yaml = &after_open
            [..pos.saturating_sub(if pos > 0 { 1 } else { 0 })];
        let after_close = &after_open[pos + 3..]; // skip "---"
        let rest = after_close
            .strip_prefix('\n')
            .or_else(|| after_close.strip_prefix("\r\n"))
            .unwrap_or(after_close);
        (Some(yaml), rest)
    } else {
        (None, content)
    }
}

/// Parses a YAML frontmatter string into a `yaml_safe::Value`.
#[cfg(feature = "yaml_support")]
pub fn parse_frontmatter(
    yaml: &str,
) -> Result<yaml_safe::Value, MarkdownError> {
    yaml_safe::from_str(yaml)
        .map_err(|e| MarkdownError::FrontmatterError(e.to_string()))
}

/// Parses a YAML frontmatter string into a typed value.
#[cfg(feature = "yaml_support")]
pub fn parse_frontmatter_as<T>(yaml: &str) -> Result<T, MarkdownError>
where
    T: for<'de> yaml_safe::Deserialize<'de>,
{
    yaml_safe::from_str(yaml)
        .map_err(|e| MarkdownError::FrontmatterError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_frontmatter() {
        let input = "---\ntitle: Hello\nauthor: World\n---\n# Content";
        let (fm, rest) = extract_frontmatter(input);
        assert_eq!(fm, Some("title: Hello\nauthor: World"));
        assert_eq!(rest, "# Content");
    }

    #[test]
    fn test_no_frontmatter() {
        let input = "# Just content\n\nNo frontmatter here.";
        let (fm, rest) = extract_frontmatter(input);
        assert!(fm.is_none());
        assert_eq!(rest, input);
    }

    #[test]
    fn test_unclosed_frontmatter() {
        let input = "---\ntitle: Hello\nNo closing delimiter";
        let (fm, rest) = extract_frontmatter(input);
        assert!(fm.is_none());
        assert_eq!(rest, input);
    }

    #[test]
    fn test_empty_frontmatter() {
        let input = "---\n---\n# Content";
        let (fm, rest) = extract_frontmatter(input);
        assert_eq!(fm, Some(""));
        assert_eq!(rest, "# Content");
    }

    #[cfg(feature = "yaml_support")]
    #[test]
    fn test_parse_frontmatter() {
        let yaml = "title: Hello\nauthor: World";
        let value = parse_frontmatter(yaml).unwrap();
        let mapping = value.as_mapping().expect("should be mapping");
        let key = yaml_safe::Value::String("title".into());
        let title = mapping.get(&key).expect("should have title");
        assert_eq!(*title, yaml_safe::Value::String("Hello".into()));
    }
}
