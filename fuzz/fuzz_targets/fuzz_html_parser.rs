#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // HTML parsing via tl crate
    if let Ok(html) = std::str::from_utf8(data) {
        let _ = tl::parse(html, tl::ParserOptions::default());
    }
});
