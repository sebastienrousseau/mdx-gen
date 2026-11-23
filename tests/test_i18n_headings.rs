// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Heading ids for non-Latin scripts (issue #34).
//!
//! Audits the anchor ids mdx-gen produces for headings written in
//! eight scripts. Each case asserts `Heading::{level, text, id}` from
//! `process_markdown_with_toc` and checks that the same `id="…"`
//! survives the sanitizer in the rendered HTML.

use mdx_gen::{process_markdown_with_toc, Heading, MarkdownOptions};

/// (script, heading source, expected id).
const MATRIX: &[(&str, &str, &str)] = &[
    ("Latin", "Résumé & Café Notes", "résumé--café-notes"),
    ("Cyrillic", "Привет, Мир", "привет-мир"),
    ("Greek", "Καλημέρα Κόσμε", "καλημέρα-κόσμε"),
    ("Arabic", "مرحبا بالعالم", "مرحبا-بالعالم"),
    (
        "Japanese",
        "はじめに：日本語の見出し",
        "はじめに日本語の見出し",
    ),
    ("Chinese", "快速入门 指南", "快速入门-指南"),
    ("Korean", "한국어 제목입니다", "한국어-제목입니다"),
    ("Thai", "สวัสดีชาวโลก", "สวัสดีชาวโลก"),
];

fn render(md: &str, prefix: &str) -> (String, Vec<Heading>) {
    let options = MarkdownOptions::default().with_header_ids(prefix);
    process_markdown_with_toc(md, &options).expect("renders")
}

#[test]
fn every_script_keeps_its_letters_in_the_id() {
    for (script, text, id) in MATRIX {
        let (html, headings) = render(&format!("## {text}\n"), "");
        assert_eq!(
            headings,
            vec![Heading {
                level: 2,
                text: (*text).to_string(),
                id: (*id).to_string(),
            }],
            "{script}"
        );
        assert!(
            html.contains(&format!(r#"id="{id}""#)),
            "{script}: rendered HTML lacks id {id:?}: {html}"
        );
    }
}

#[test]
fn prefix_applies_to_non_latin_ids() {
    let (html, headings) = render("# Привет\n", "user-content-");
    assert_eq!(headings[0].id, "user-content-привет");
    assert!(html.contains(r#"id="user-content-привет""#), "{html}");
}

#[test]
fn repeated_non_latin_headings_are_deduplicated() {
    let (html, headings) = render("# 概要\n\n# 概要\n\n# 概要\n", "");
    let ids: Vec<&str> =
        headings.iter().map(|h| h.id.as_str()).collect();
    assert_eq!(ids, ["概要", "概要-1", "概要-2"]);
    for id in ids {
        assert!(html.contains(&format!(r#"id="{id}""#)), "{html}");
    }
}

#[test]
fn mixed_script_heading_keeps_both_scripts() {
    let (_, headings) = render("## API リファレンス v2\n", "");
    assert_eq!(headings[0].id, "api-リファレンス-v2");
    assert_eq!(headings[0].text, "API リファレンス v2");
}

/// Documented limitation, shared with GitHub: a heading with no
/// letters, marks or digits anchorizes to an empty id.
#[test]
fn emoji_only_heading_has_an_empty_id() {
    let (_, headings) = render("# 🎉\n\n# 🎉\n", "");
    let ids: Vec<&str> =
        headings.iter().map(|h| h.id.as_str()).collect();
    assert_eq!(ids, ["", "-1"]);
    assert_eq!(headings[0].text, "🎉");
}
