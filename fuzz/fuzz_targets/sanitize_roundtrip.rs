#![no_main]

// Exercises the sanitize-to-writer path with arbitrary bytes. The
// writer is a sink — we care about crashes, panics, and unbounded
// CPU, not the output.

use libfuzzer_sys::fuzz_target;
use mdx_gen::{MarkdownOptions, Options, process_markdown_to_writer};

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };

    let mut comrak_options = Options::default();
    comrak_options.extension.table = true;

    let options = MarkdownOptions::default()
        .with_comrak_options(comrak_options)
        .with_unsafe_html(false);

    let mut sink = std::io::sink();
    let _ = process_markdown_to_writer(input, &mut sink, &options);
});
