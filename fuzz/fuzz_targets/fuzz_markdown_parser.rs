#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Test UTF-8 text processing similar to PlainTextParser
    if let Ok(text) = std::str::from_utf8(data) {
        // Simulate paragraph splitting (as in PlainTextParser::text_to_blocks)
        let _blocks: Vec<&str> = text
            .split("\n\n")
            .filter(|para| !para.trim().is_empty())
            .map(|para| para.trim())
            .collect();
    }
});
