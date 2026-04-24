// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 MDX Gen. All rights reserved.

//! Custom `std::io::Write` target — pipe the rendered HTML through
//! a writer that instruments and forwards.
//!
//! Run: `cargo run --example pipe`

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support.rs"]
mod support;

use std::io::{self, Write};

use mdx_gen::{process_markdown_to_writer, MarkdownOptions, Options};

const SOURCE: &str = r#"# Pipe demo

mdx-gen streams rendered HTML to any `std::io::Write` you hand
it. That makes it a drop-in for network sockets, compressors,
hash computers, embedded peripherals, or anything else with a
`Write` impl.

```rust
let mut sink = MeteredSink::new(File::create("out.html")?);
process_markdown_to_writer(md, &mut sink, &options)?;
println!("wrote {} bytes / {} lines", sink.bytes(), sink.lines());
```
"#;

/// Wraps an inner `Write`, counting bytes and lines on the way
/// past, and computing a 64-bit FNV-1a hash of every byte.
///
/// FNV-1a is used here only because it fits in ~10 lines and
/// demonstrates the wrapping pattern — pick any real hash for a
/// real workload.
struct MeteredSink<W: Write> {
    inner: W,
    bytes: u64,
    lines: u64,
    hash: u64,
}

impl<W: Write> MeteredSink<W> {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    fn new(inner: W) -> Self {
        Self {
            inner,
            bytes: 0,
            lines: 0,
            hash: Self::FNV_OFFSET,
        }
    }

    fn bytes(&self) -> u64 {
        self.bytes
    }

    fn lines(&self) -> u64 {
        self.lines
    }

    fn hash(&self) -> u64 {
        self.hash
    }
}

impl<W: Write> Write for MeteredSink<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        let written = &buf[..n];
        self.bytes += n as u64;
        self.lines +=
            written.iter().filter(|&&b| b == b'\n').count() as u64;
        for &b in written {
            self.hash ^= u64::from(b);
            self.hash = self.hash.wrapping_mul(Self::FNV_PRIME);
        }
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

fn main() {
    support::header("mdx-gen -- pipe");

    let options = support::task("Build options", || {
        let mut comrak_options = Options::default();
        comrak_options.extension.table = true;
        MarkdownOptions::new()
            .with_comrak_options(comrak_options)
            .with_custom_blocks(false)
            .with_enhanced_tables(true)
            .with_syntax_highlighting(true)
            .with_unsafe_html(false)
    });

    // Inner sink: a Vec<u8>. Outer sink: MeteredSink that forwards
    // every byte to the Vec while keeping running counts.
    let metered: MeteredSink<Vec<u8>> =
        support::task("Stream through MeteredSink<Vec<u8>>", || {
            let mut sink = MeteredSink::new(Vec::new());
            process_markdown_to_writer(SOURCE, &mut sink, &options)
                .unwrap();
            sink
        });

    support::task_with_output("Inspect counters + hash", || {
        vec![
            format!("bytes : {}", metered.bytes()),
            format!("lines : {}", metered.lines()),
            format!("hash  : {:#018x}", metered.hash()),
        ]
    });

    // The inner Write is recoverable — destructure to get the
    // Vec<u8> back. This pattern works the same for File,
    // TcpStream, BufWriter, or anything else with a `Write` impl.
    support::task_with_output("Recover the inner Write target", || {
        let MeteredSink { inner, bytes, .. } = metered;
        let html = String::from_utf8(inner).unwrap();
        vec![
            format!("inner bytes : {}", html.len()),
            format!("matches counter: {}", html.len() as u64 == bytes),
            format!(
                "starts with: {}",
                html.lines().next().unwrap_or("")
            ),
        ]
    });

    support::summary(4);
}
