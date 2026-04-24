// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Stream rendered HTML to a File and a Vec<u8>.
//!
//! Run: `cargo run --example streaming`

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;

use mdx_gen::{
    process_markdown, process_markdown_to_writer, MarkdownOptions,
    Options,
};

const SOURCE: &str = r#"# Streaming demo

mdx-gen can emit rendered HTML directly to any `std::io::Write`
sink — here we target a file.

| Path | Kind |
|:-----|:-----|
| stdout | `io::Stdout` |
| disk   | `File`       |
| socket | `TcpStream`  |

```rust
let out = BufWriter::new(File::create("out.html")?);
process_markdown_to_writer(md, &mut out, &options)?;
```
"#;

fn main() {
    support::header("mdx-gen -- streaming");

    let options =
        support::task("Build streaming-friendly options", || {
            let mut comrak_options = Options::default();
            comrak_options.extension.table = true;
            comrak_options.extension.strikethrough = true;
            MarkdownOptions::new()
                .with_comrak_options(comrak_options)
                .with_custom_blocks(false)
                .with_enhanced_tables(true)
                .with_syntax_highlighting(true)
                .with_unsafe_html(false)
        });

    let out_path =
        support::task("Stream through BufWriter<File>", || {
            let dir: PathBuf = PathBuf::from("target/examples");
            fs::create_dir_all(&dir).unwrap();
            let path = dir.join("streaming.html");
            {
                let mut writer =
                    BufWriter::new(File::create(&path).unwrap());
                process_markdown_to_writer(
                    SOURCE,
                    &mut writer,
                    &options,
                )
                .unwrap();
            } // flush on drop
            path
        });

    support::task_with_output("Inspect artefact on disk", || {
        let bytes = fs::metadata(&out_path).unwrap().len();
        vec![
            format!("path  : {}", out_path.display()),
            format!("bytes : {bytes}"),
        ]
    });

    support::task(
        "Verify byte-equivalence with process_markdown",
        || {
            let from_string =
                process_markdown(SOURCE, &options).unwrap();
            let from_disk = fs::read_to_string(&out_path).unwrap();
            assert_eq!(
                from_string, from_disk,
                "streaming and String variants must agree"
            );
        },
    );

    support::task_with_output("Stream into a Vec<u8>", || {
        let mut buf: Vec<u8> = Vec::new();
        process_markdown_to_writer(SOURCE, &mut buf, &options).unwrap();
        let decoded = String::from_utf8(buf.clone()).unwrap();
        vec![
            format!("bytes : {}", buf.len()),
            format!("chars : {}", decoded.chars().count()),
        ]
    });

    support::summary(5);
}
