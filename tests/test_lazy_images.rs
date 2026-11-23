// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `loading="lazy"` / `decoding="async"` on images (issue #32).

use mdx_gen::{process_markdown, MarkdownOptions, Options};

const IMG: &str =
    "![Alt text](https://example.com/a.png \"A title\")\n";

fn lazy() -> MarkdownOptions<'static> {
    MarkdownOptions::default().with_lazy_images(true)
}

#[test]
fn lazy_images_are_off_by_default() {
    let html =
        process_markdown(IMG, &MarkdownOptions::default()).unwrap();
    assert!(html.contains("<img"), "{html}");
    assert!(!html.contains("loading="), "{html}");
    assert!(!html.contains("decoding="), "{html}");
}

#[test]
fn test_lazy_images_survive_sanitizer() {
    let html = process_markdown(IMG, &lazy()).unwrap();
    assert!(html.contains(r#"loading="lazy""#), "{html}");
    assert!(html.contains(r#"decoding="async""#), "{html}");
}

#[test]
fn lazy_images_without_sanitizer() {
    let html =
        process_markdown(IMG, &lazy().with_unsafe_html(true)).unwrap();
    assert_eq!(
        html,
        "<p><img src=\"https://example.com/a.png\" alt=\"Alt text\" \
         title=\"A title\" loading=\"lazy\" decoding=\"async\" /></p>\n"
    );
}

#[test]
fn lazy_images_preserve_existing_attributes() {
    let html = process_markdown(IMG, &lazy()).unwrap();
    assert!(
        html.contains(r#"src="https://example.com/a.png""#),
        "{html}"
    );
    assert!(html.contains(r#"alt="Alt text""#), "{html}");
    assert!(html.contains(r#"title="A title""#), "{html}");
}

#[test]
fn lazy_images_without_title_and_with_markup_in_alt() {
    let html =
        process_markdown("![*bold* alt](x.png)\n", &lazy()).unwrap();
    assert!(html.contains(r#"alt="bold alt""#), "{html}");
    assert!(!html.contains("title="), "{html}");
    assert!(html.contains(r#"loading="lazy""#), "{html}");
}

#[test]
fn lazy_images_inside_enhanced_tables() {
    let md = "| pic |\n|-----|\n| ![a](t.png) |\n";
    let html = process_markdown(md, &lazy()).unwrap();
    assert!(html.contains("table-responsive"), "{html}");
    assert!(html.contains(r#"loading="lazy""#), "{html}");
}

#[test]
fn lazy_images_with_figure_captions() {
    let mut comrak = Options::default();
    comrak.extension.table = true;
    comrak.render.figure_with_caption = true;
    let options =
        lazy().with_comrak_options(comrak).with_unsafe_html(true);
    let html = process_markdown(IMG, &options).unwrap();
    assert!(
        html.contains(
            "decoding=\"async\" /><figcaption>A title</figcaption></figure>"
        ),
        "{html}"
    );
}

#[test]
fn every_image_gets_the_attributes() {
    let md = "![a](1.png) and ![b](2.png)\n\n![c](3.png)\n";
    let html = process_markdown(md, &lazy()).unwrap();
    assert_eq!(html.matches(r#"loading="lazy""#).count(), 3, "{html}");
    assert_eq!(
        html.matches(r#"decoding="async""#).count(),
        3,
        "{html}"
    );
}

#[test]
fn lazy_images_survive_a_custom_sanitizer_config() {
    let options = lazy().with_sanitizer_config(
        mdx_gen::SanitizerConfig::new().with_tag("mark"),
    );
    let html = process_markdown(IMG, &options).unwrap();
    assert!(html.contains(r#"loading="lazy""#), "{html}");
    assert!(html.contains(r#"decoding="async""#), "{html}");
}

#[test]
fn raw_html_img_is_left_alone() {
    let html =
        process_markdown("<img src=\"r.png\">\n", &lazy()).unwrap();
    assert!(html.contains("r.png"), "{html}");
    assert!(!html.contains("loading="), "{html}");
}
