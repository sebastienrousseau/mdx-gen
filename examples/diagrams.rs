// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Client-side diagram rendering — mermaid, geojson, topojson, stl.
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

const SOURCE: &str = r##"# Diagrams showcase

## Mermaid

```mermaid
graph TD
  A[Source Markdown] --> B[mdx-gen]
  B --> C[HTML + hydrator]
  C --> D[Inline SVG]
```

## GeoJSON

Three features that actually look like something — a recognisable
triangle (UK-shaped), a hollow ring, and a small dot — each with
its own colour via `properties.fill` / `properties.stroke`.

```geojson
{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "properties": { "fill": "#4a90d9", "stroke": "#1b3d6e" },
      "geometry": {
        "type": "Polygon",
        "coordinates": [[
          [-6, 58], [-2, 58], [ 2, 55], [ 2, 51], [-1, 50],
          [-5, 50], [-5, 54], [-8, 55], [-6, 58]
        ]]
      }
    },
    {
      "type": "Feature",
      "properties": { "fill": "#f2c14e", "stroke": "#8a6b1e" },
      "geometry": {
        "type": "Polygon",
        "coordinates": [
          [[10, 45], [20, 45], [20, 40], [10, 40], [10, 45]],
          [[13, 43], [17, 43], [17, 42], [13, 42], [13, 43]]
        ]
      }
    },
    {
      "type": "Feature",
      "properties": { "fill": "#e94e77", "stroke": "#7a2540" },
      "geometry": {
        "type": "Polygon",
        "coordinates": [[
          [ 4, 48], [ 6, 48], [ 6, 46], [ 4, 46], [ 4, 48]
        ]]
      }
    }
  ]
}
```

## TopoJSON

A shared-border topology: two rectangles meeting at a common
arc, as per the TopoJSON spec. The hydrator colours each object
independently.

```topojson
{
  "type": "Topology",
  "arcs": [
    [[-3, 0], [0, 0], [0, 2], [-3, 2], [-3, 0]],
    [[ 0, 0], [3, 0], [3, 2], [ 0, 2], [ 0, 0]]
  ],
  "objects": {
    "west": {
      "type": "Polygon",
      "arcs": [[0]],
      "properties": { "fill": "#4a90d9", "stroke": "#1b3d6e" }
    },
    "east": {
      "type": "Polygon",
      "arcs": [[1]],
      "properties": { "fill": "#f2c14e", "stroke": "#8a6b1e" }
    }
  }
}
```

## ASCII STL

A full cube (12 triangles, six faces). Phong-shaded via three.js
so directional lighting gives real 3-D depth cues.

```stl
solid cube
  facet normal 0 0 -1
    outer loop
      vertex 0 0 0
      vertex 1 1 0
      vertex 1 0 0
    endloop
  endfacet
  facet normal 0 0 -1
    outer loop
      vertex 0 0 0
      vertex 0 1 0
      vertex 1 1 0
    endloop
  endfacet
  facet normal 0 0 1
    outer loop
      vertex 0 0 1
      vertex 1 0 1
      vertex 1 1 1
    endloop
  endfacet
  facet normal 0 0 1
    outer loop
      vertex 0 0 1
      vertex 1 1 1
      vertex 0 1 1
    endloop
  endfacet
  facet normal 0 -1 0
    outer loop
      vertex 0 0 0
      vertex 1 0 0
      vertex 1 0 1
    endloop
  endfacet
  facet normal 0 -1 0
    outer loop
      vertex 0 0 0
      vertex 1 0 1
      vertex 0 0 1
    endloop
  endfacet
  facet normal 0 1 0
    outer loop
      vertex 0 1 0
      vertex 0 1 1
      vertex 1 1 1
    endloop
  endfacet
  facet normal 0 1 0
    outer loop
      vertex 0 1 0
      vertex 1 1 1
      vertex 1 1 0
    endloop
  endfacet
  facet normal -1 0 0
    outer loop
      vertex 0 0 0
      vertex 0 0 1
      vertex 0 1 1
    endloop
  endfacet
  facet normal -1 0 0
    outer loop
      vertex 0 0 0
      vertex 0 1 1
      vertex 0 1 0
    endloop
  endfacet
  facet normal 1 0 0
    outer loop
      vertex 1 0 0
      vertex 1 1 0
      vertex 1 1 1
    endloop
  endfacet
  facet normal 1 0 0
    outer loop
      vertex 1 0 0
      vertex 1 1 1
      vertex 1 0 1
    endloop
  endfacet
endsolid cube
```
"##;

fn main() {
    support::header("mdx-gen -- diagrams");

    let out_dir: PathBuf = support::task("Prepare target dir", || {
        let dir = PathBuf::from("target/examples/diagrams");
        fs::create_dir_all(&dir).unwrap();
        dir
    });

    let fragment =
        support::task("Render Markdown with diagrams on", || {
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

    support::task_with_output(
        "Verify each container is present",
        || {
            vec![
                format!(
                    "mermaid : {}",
                    fragment.contains("<pre class=\"mermaid\">")
                ),
                format!(
                    "geojson : {}",
                    fragment.contains("data-mdx-diagram=\"geojson\"")
                ),
                format!(
                    "topojson: {}",
                    fragment.contains("data-mdx-diagram=\"topojson\"")
                ),
                format!(
                    "stl     : {}",
                    fragment.contains("data-mdx-diagram=\"stl\"")
                ),
            ]
        },
    );

    let out_path = support::task(
        "Assemble standalone index.html",
        || {
            let page = format!(
                r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>mdx-gen diagrams</title>
  <style>
    body {{ font: 16px/1.6 system-ui, sans-serif; max-width: 48rem; margin: 2rem auto; padding: 0 1rem; }}
    h1, h2 {{ line-height: 1.2; }}
    .mdx-diagram {{ border: 1px solid #e5e5e5; border-radius: 4px; padding: 1rem; margin: 1rem 0; background: #fafafa; }}
    .mdx-diagram svg {{ max-width: 100%; height: auto; }}
    pre.mermaid {{ background: none; padding: 0; border: 1px solid #e5e5e5; border-radius: 4px; text-align: center; }}
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
        },
    );

    support::task_with_output("Inspect artefacts", || {
        let bytes = fs::metadata(&out_path).unwrap().len();
        vec![
            format!("path  : {}", out_path.display()),
            format!("bytes : {bytes}"),
            format!(
                "open  : open {} (loads mermaid / d3-geo / three.js from jsdelivr)",
                out_path.display()
            ),
        ]
    });

    support::summary(5);
}
