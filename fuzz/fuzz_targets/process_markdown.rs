#![no_main]

use libfuzzer_sys::fuzz_target;
use mdx_gen::{MarkdownOptions, Options, process_markdown};

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };

    let mut comrak_options = Options::default();
    comrak_options.extension.table = true;
    comrak_options.extension.strikethrough = true;
    comrak_options.extension.tasklist = true;

    let options = MarkdownOptions::default()
        .with_comrak_options(comrak_options)
        .with_unsafe_html(false);

    let _ = process_markdown(input, &options);
});
