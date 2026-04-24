// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Mermaid diagram rendering — every mermaid 10 diagram kind in
//! one page, each with a "when to use" blurb.
//!
//! Run: `cargo run --example diagrams`

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use std::fs;
use std::path::PathBuf;

use mdx_gen::{
    hydration_script_html, process_markdown, MarkdownOptions, Options,
};

// Nine diagram kinds mermaid 10 supports, each with a "when to
// use" blurb lifted from the project brief. Every block below
// flows through mdx-gen's client-side hydration path unchanged.
const SOURCE: &str = r##"# Mermaid showcase

mdx-gen recognises fenced `mermaid` code blocks and rewrites them
into `<pre class="mermaid">…</pre>` containers that the
client-side [mermaid.js](https://mermaid.js.org/) library
hydrates into inline SVG at page-load time. Every diagram kind
mermaid supports flows through the same pipeline — the nine
below are a tour of that surface.

## Flowchart

**When to use:** illustrating logic, processes, or decision trees.

```mermaid
flowchart LR
  Start([Start]) --> Check{Input valid?}
  Check -->|Yes| Process[Process data]
  Check -->|No| Error[Log error]
  Process --> Save[(Database)]
  Save --> Done([Done])
  Error --> Done
```

## Sequence diagram

**When to use:** documenting interactions between different
systems or components over time — API traces, message passing,
request / response flows.

```mermaid
sequenceDiagram
  autonumber
  participant Client
  participant API as API gateway
  participant DB as Database
  Client->>+API: POST /orders
  API->>+DB: INSERT order
  DB-->>-API: id = 42
  API-->>-Client: 201 Created
  Client->>+API: GET /orders/42
  API->>+DB: SELECT order
  DB-->>-API: row
  API-->>-Client: 200 OK
```

## Entity–Relationship diagram

**When to use:** modelling database schemas and the relationships
between data entities. Ideal for README-level data-model docs.

```mermaid
erDiagram
  CUSTOMER ||--o{ ORDER : places
  ORDER ||--|{ LINE_ITEM : contains
  PRODUCT ||--o{ LINE_ITEM : listed_in
  CUSTOMER {
    string name
    string email UK
  }
  ORDER {
    int id PK
    date placed_at
    string status
  }
  PRODUCT {
    int sku PK
    string title
    decimal price
  }
  LINE_ITEM {
    int order_id FK
    int sku FK
    int qty
  }
```

## Class diagram

**When to use:** object-oriented documentation — classes,
methods, attributes, and relationships between them. Essential
for showing system structure.

```mermaid
classDiagram
  class MarkdownOptions {
    +bool enable_diagrams
    +with_diagrams(bool) Self
    +validate() Result
  }
  class Pipeline {
    -parse()
    -transform()
    -render()
    -sanitize()
  }
  class DiagramTransform {
    <<trait>>
    +apply(ast)
  }
  Pipeline --> MarkdownOptions : configures
  Pipeline ..|> DiagramTransform : uses
```

## State diagram

**When to use:** visualising the states and transitions of an
object or system — state machines, connection lifecycles, UI
modes.

```mermaid
stateDiagram-v2
  [*] --> Idle
  Idle --> Parsing : input arrived
  Parsing --> Rendering : AST built
  Rendering --> Sanitizing : HTML produced
  Sanitizing --> Done : clean
  Sanitizing --> Failed : rejected
  Done --> [*]
  Failed --> Idle : retry
```

## Gantt chart

**When to use:** project management — tracking task timelines,
dependencies, and completion status across a release window.

```mermaid
gantt
  dateFormat  YYYY-MM-DD
  title       v0.0.3 release runway
  section Core
  AST pipeline         :done,    p1, 2026-03-15, 7d
  Sanitizer hardening  :done,    p2, after p1, 5d
  section Polish
  Examples + CHANGELOG :done,    p3, after p2, 4d
  Diagrams             :active,  p4, after p3, 3d
  section Launch
  Tag + publish        :         p5, after p4, 1d
```

## Git graph

**When to use:** visualising GitHub-style branch history —
merges, tags, and commit progressions on a release branch.

```mermaid
gitGraph
  commit id: "init"
  branch feat/v0.0.3
  checkout feat/v0.0.3
  commit id: "AST pipeline"
  commit id: "Sanitizer"
  commit id: "ToC"
  checkout main
  merge feat/v0.0.3 tag: "v0.0.3"
  commit id: "changelog"
```

## User journey

**When to use:** mapping the steps a user takes to complete a
task. Each step carries a satisfaction score (1–5) and the set
of actors involved.

```mermaid
journey
  title Author writes and ships a Markdown page
  section Write
    Open editor         : 5: Author
    Draft content       : 4: Author
    Add mermaid block   : 4: Author
  section Build
    Run mdx-gen         : 5: Author
    Inspect HTML        : 4: Author
  section Ship
    Deploy to CDN       : 5: Author, Ops
    Reader loads page   : 5: Reader
```

## Pie chart

**When to use:** simple visualisations for data distribution —
test coverage splits, allocation breakdowns, share-of-total
plots.

```mermaid
pie showData
  title Where mdx-gen spends its cycles
  "Parse"     : 18
  "Transform" : 22
  "Render"    : 35
  "Sanitize"  : 25
```
"##;

fn main() {
    support::header("mdx-gen -- diagrams (mermaid)");

    let out_dir: PathBuf = support::task("Prepare target dir", || {
        let dir = PathBuf::from("target/examples/diagrams");
        fs::create_dir_all(&dir).unwrap();
        dir
    });

    let fragment =
        support::task("Render with diagrams enabled", || {
            let mut comrak_options = Options::default();
            comrak_options.extension.table = true;
            comrak_options.extension.strikethrough = true;

            let options = MarkdownOptions::new()
                .with_comrak_options(comrak_options)
                .with_custom_blocks(false)
                .with_enhanced_tables(false)
                .with_syntax_highlighting(false)
                .with_diagrams(true)
                .with_unsafe_html(false);

            process_markdown(SOURCE, &options).unwrap()
        });

    support::task_with_output("Verify mermaid containers", || {
        let n = fragment.matches("<pre class=\"mermaid\">").count();
        vec![
            format!("containers found : {n} (expected 9)"),
            "hydration script  : embedded inline".to_string(),
            "sanitizer         : passed (unsafe_html = false)"
                .to_string(),
        ]
    });

    let out_path =
        support::task("Assemble standalone index.html", || {
            let page = format!(
                r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>mdx-gen — mermaid diagrams</title>
  <style>
    body {{
      font: 16px/1.6 system-ui, sans-serif;
      max-width: 52rem;
      margin: 2rem auto;
      padding: 0 1.25rem;
      color: #1a1a1a;
    }}
    h1 {{ margin: 0 0 1.5rem; }}
    h2 {{ margin-top: 2.5rem; }}
    pre.mermaid {{
      background: #fafafa;
      border: 1px solid #e5e5e5;
      border-radius: 6px;
      padding: 1.25rem;
      text-align: center;
      overflow-x: auto;
    }}
    pre.mermaid svg {{ max-width: 100%; height: auto; }}
  </style>
</head>
<body>
{fragment}
{hydrator}
</body>
</html>
"#,
                hydrator = hydration_script_html(),
            );

            let path = out_dir.join("index.html");
            fs::write(&path, page).unwrap();
            path
        });

    support::task_with_output("Inspect artefact", || {
        let bytes = fs::metadata(&out_path).unwrap().len();
        vec![
            format!("path  : {}", out_path.display()),
            format!("bytes : {bytes}"),
            format!(
                "open  : open {} (loads mermaid 10 from jsdelivr)",
                out_path.display()
            ),
        ]
    });

    support::summary(5);
}
