// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `MarkdownOptions` from config files (issue #33, `serde` feature).

#![cfg(feature = "serde")]

use mdx_gen::{
    process_markdown, ColumnAlignment, CustomBlockConfig,
    CustomBlockType, MarkdownOptions, SanitizerConfig,
};

/// `MarkdownOptions` holds comrak's `Options`, which is neither
/// `PartialEq` nor serialisable, so compare through the serialised
/// form plus the comrak fields mdx-gen relies on.
fn assert_same(a: &MarkdownOptions, b: &MarkdownOptions) {
    assert_eq!(
        toml::to_string(a).unwrap(),
        toml::to_string(b).unwrap()
    );
    assert_eq!(
        a.comrak_options.extension.table,
        b.comrak_options.extension.table
    );
}

#[test]
fn default_round_trips_through_toml() {
    let original = MarkdownOptions::default();
    let text = toml::to_string(&original).unwrap();
    let back: MarkdownOptions = toml::from_str(&text).unwrap();
    assert_same(&original, &back);
    assert!(back.validate().is_ok());
}

#[test]
fn customised_options_round_trip_including_maps() {
    let original = MarkdownOptions::default()
        .with_lazy_images(true)
        .with_header_ids("user-content-")
        .with_max_input_size(4096)
        .with_custom_block_config(
            CustomBlockConfig::new()
                .with_class(CustomBlockType::Note, "callout")
                .with_title(CustomBlockType::Warning, "Careful"),
        )
        .with_sanitizer_config(
            SanitizerConfig::new()
                .with_tag("mark")
                .with_tag_attribute("abbr", "title")
                .with_generic_attribute("lang")
                .with_allowed_class("span", "kbd"),
        );
    let text = toml::to_string(&original).unwrap();
    let back: MarkdownOptions = toml::from_str(&text).unwrap();
    assert_same(&original, &back);
    let cfg = back.sanitizer_config.as_ref().unwrap();
    assert_eq!(cfg.extra_tag_attributes["abbr"], ["title"]);
    assert_eq!(cfg.extra_allowed_classes["span"], ["kbd"]);
    assert_eq!(
        back.custom_block_config.title_overrides
            [&CustomBlockType::Warning],
        "Careful"
    );
}

#[test]
fn partial_kebab_case_config_keeps_defaults() {
    let options: MarkdownOptions = toml::from_str(
        r#"
        enable-lazy-images = true
        header-ids = ""

        [custom-block-config.title-overrides]
        note = "Heads up"
        "#,
    )
    .unwrap();
    assert!(options.enable_lazy_images);
    assert!(options.enable_custom_blocks, "default kept");
    assert!(options.enable_enhanced_tables, "default kept");
    assert!(
        options.comrak_options.extension.table,
        "comrak default kept"
    );
    assert_eq!(
        options.max_input_size,
        mdx_gen::markdown::DEFAULT_MAX_INPUT_SIZE
    );

    let html = process_markdown(
        "# Hi\n\n<div class=\"note\">x</div>\n",
        &options,
    )
    .unwrap();
    assert!(html.contains("Heads up:"), "{html}");
    assert!(html.contains(r#"id="hi""#), "{html}");
}

#[test]
fn unknown_keys_are_rejected() {
    let err =
        toml::from_str::<MarkdownOptions>("enable-lazy-imagez = true")
            .unwrap_err();
    assert!(err.to_string().contains("enable-lazy-imagez"), "{err}");
}

#[test]
fn enums_use_kebab_case() {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Wrap {
        block: CustomBlockType,
        align: ColumnAlignment,
    }
    let text = toml::to_string(&Wrap {
        block: CustomBlockType::Important,
        align: ColumnAlignment::Center,
    })
    .unwrap();
    assert!(text.contains(r#"block = "important""#), "{text}");
    assert!(text.contains(r#"align = "center""#), "{text}");
    let back: Wrap = toml::from_str(&text).unwrap();
    assert_eq!(back.block, CustomBlockType::Important);
}

#[test]
fn loads_from_json_too() {
    let options: MarkdownOptions =
        serde_json::from_str(r#"{"enable-diagrams": true}"#).unwrap();
    assert!(options.enable_diagrams);
}
