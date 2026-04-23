// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Strongly-typed frontmatter — parse YAML into custom structs.
//!
//! Run: `cargo run --example typed`

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use mdx_gen::frontmatter::{
    extract_frontmatter, parse_frontmatter, parse_frontmatter_as,
};
use serde::{Deserialize, Serialize};

// Three different documents shaped for three different consumers.
// yaml_safe is minimal — inline `[…]` sequences only.

const POST: &str = r#"---
title: "Shipping 0.0.3"
slug: "shipping-003"
date: "2026-04-23"
tags: [release, notes]
draft: false
---

# Body
"#;

const SITE_CONFIG: &str = r#"---
name: "mdx-gen handbook"
base_url: "https://example.com"
locale: "en-GB"
analytics: false
---

# Page
"#;

const PARTIAL: &str = r#"---
title: "Quick note"
---

# Body
"#;

#[derive(Debug, Deserialize, Serialize)]
struct Post {
    title: String,
    slug: String,
    date: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    draft: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct SiteConfig {
    name: String,
    base_url: String,
    locale: String,
    #[serde(default)]
    analytics: bool,
}

/// `slug` is missing on PARTIAL; `#[serde(default)]` keeps the
/// parse infallible and yields `String::default()` instead.
#[derive(Debug, Deserialize, Serialize)]
struct Page {
    title: String,
    #[serde(default)]
    slug: String,
}

fn main() {
    support::header("mdx-gen -- typed");

    let post: Post = support::task("Parse Post (full struct)", || {
        let (yaml, _body) = extract_frontmatter(POST);
        parse_frontmatter_as(yaml.unwrap()).unwrap()
    });
    support::task_with_output("Inspect Post fields", || {
        vec![
            format!("title : {}", post.title),
            format!("slug  : {}", post.slug),
            format!("date  : {}", post.date),
            format!("tags  : {:?}", post.tags),
            format!("draft : {}", post.draft),
        ]
    });

    let cfg: SiteConfig =
        support::task("Parse SiteConfig (different shape)", || {
            let (yaml, _body) = extract_frontmatter(SITE_CONFIG);
            parse_frontmatter_as(yaml.unwrap()).unwrap()
        });
    support::task_with_output("Inspect SiteConfig fields", || {
        vec![
            format!("name      : {}", cfg.name),
            format!("base_url  : {}", cfg.base_url),
            format!("locale    : {}", cfg.locale),
            format!("analytics : {}", cfg.analytics),
        ]
    });

    let page: Page = support::task(
        "Parse Page (missing optional field via #[serde(default)])",
        || {
            let (yaml, _body) = extract_frontmatter(PARTIAL);
            parse_frontmatter_as(yaml.unwrap()).unwrap()
        },
    );
    support::task_with_output("Inspect Page fields", || {
        vec![
            format!("title : {}", page.title),
            format!("slug  : {:?} (defaulted)", page.slug),
        ]
    });

    // Untyped path — useful for schemaless flows or exploration.
    support::task_with_output(
        "Untyped Value walk (for schemaless flows)",
        || {
            let (yaml, _body) = extract_frontmatter(POST);
            let value = parse_frontmatter(yaml.unwrap()).unwrap();
            let mapping = value.as_mapping().unwrap();
            mapping
                .iter()
                .map(|(k, v)| {
                    let key = k.as_str().unwrap_or("?");
                    let kind = match v {
                        v if v.is_string() => "string",
                        v if v.is_bool() => "bool",
                        v if v.is_sequence() => "sequence",
                        v if v.is_number() => "number",
                        _ => "other",
                    };
                    format!("{key:<8} -> {kind}")
                })
                .collect()
        },
    );

    support::summary(7);
}
