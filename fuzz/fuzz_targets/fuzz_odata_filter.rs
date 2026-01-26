#![no_main]

use libfuzzer_sys::fuzz_target;
use modkit_odata::parse_filter_string;

fuzz_target!(|data: &[u8]| {
    // Limit input size to avoid OOM on pathological inputs
    if data.len() > 1024 {
        return;
    }
    // Convert bytes to string (may be invalid UTF-8)
    if let Ok(s) = std::str::from_utf8(data) {
        // Don't panic on invalid input - this is expected
        let _ = parse_filter_string(s);
    }
});
