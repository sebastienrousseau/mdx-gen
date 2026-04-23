// Copyright © 2024 - 2026 MDX Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! # Streaming — Render directly to a file or socket
//!
//! ## What this example is
//!
//! Shows the writer-flavoured entry point
//! [`process_markdown_to_writer`] writing rendered HTML straight to
//! a `std::io::BufWriter<File>`. Compared to
//! [`process_markdown`], this variant skips one intermediate
//! `String` allocation when the caller already has a `Write` sink
//! — a file, a network socket, a template buffer.
//!
//! ## What it demonstrates
//!
//! - **Streaming render** — pipeline output goes byte-for-byte to
//!   the writer; the sanitised path uses `ammonia::Document::
//!   write_to()` so nothing is re-allocated as a `String` on the
//!   way.
//! - **Error surface** — failed writes bubble up as
//!   [`MarkdownError::IoError`] via the `From<io::Error>` impl, so
//!   your caller can handle them with `?`.
//! - **Byte-equivalent output** — the example also runs
//!   `process_markdown` on the same input and asserts the two
//!   paths produce identical bytes.
//!
//! ## When to use this pattern
//!
//! When your pipeline lives downstream of a file system, network
//! handler, or templating layer that already takes an `impl Write`.
//! Avoids an unnecessary `String::from_utf8` round-trip.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example streaming
//! ```
//!
//! The example writes to `target/examples/streaming.html`. Open
//! that file in a browser to verify the rendered document.

use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;

use mdx_gen::{
    process_markdown, process_markdown_to_writer, MarkdownOptions,
    Options,
};

const SAMPLE: &str = r#"# Streaming demo

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Streaming render");
    println!("───────────────────");

    // ── Step 1: Work out where the output goes ────────────────────
    let out_dir: PathBuf = PathBuf::from("target/examples");
    fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join("streaming.html");

    // ── Step 2: Configure the pipeline ────────────────────────────
    let mut comrak_options = Options::default();
    comrak_options.extension.table = true;
    comrak_options.extension.strikethrough = true;

    let options = MarkdownOptions::new()
        .with_comrak_options(comrak_options)
        .with_custom_blocks(false)
        .with_enhanced_tables(true)
        .with_syntax_highlighting(true)
        .with_unsafe_html(false);

    // ── Step 3: Stream through a BufWriter<File> ──────────────────
    {
        let file = File::create(&out_path)?;
        let mut writer = BufWriter::new(file);
        process_markdown_to_writer(SAMPLE, &mut writer, &options)?;
    } // BufWriter flushes + File closes here on Drop.
    let bytes = fs::metadata(&out_path)?.len();
    println!("    ✅ wrote {} bytes → {}", bytes, out_path.display());

    // ── Step 4: Verify equivalence with the String variant ────────
    let from_string = process_markdown(SAMPLE, &options)?;
    let from_disk = fs::read_to_string(&out_path)?;
    assert_eq!(
        from_string, from_disk,
        "streaming and String variants must agree"
    );
    println!("    ✅ streamed output is byte-identical to `process_markdown`");

    // ── Step 5: Stream to an in-memory Vec<u8> ────────────────────
    //
    // Same entry point, different sink. `Vec<u8>` implements `Write`,
    // which makes it handy for tests and in-memory transforms.
    let mut buf: Vec<u8> = Vec::new();
    process_markdown_to_writer(SAMPLE, &mut buf, &options)?;
    println!(
        "    ✅ streamed {} bytes to Vec<u8> ({} chars once decoded)",
        buf.len(),
        String::from_utf8(buf)?.chars().count()
    );

    Ok(())
}
