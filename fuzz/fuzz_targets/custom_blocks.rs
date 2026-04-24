#![no_main]

use libfuzzer_sys::fuzz_target;
use mdx_gen::extensions::process_custom_blocks;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };
    let _ = process_custom_blocks(input);
});
