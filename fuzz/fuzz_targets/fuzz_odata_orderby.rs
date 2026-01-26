#![no_main]

use libfuzzer_sys::fuzz_target;
use modkit_odata::ODataOrderBy;

fuzz_target!(|data: &[u8]| {
    // Limit input size to avoid OOM on pathological inputs
    if data.len() > 1024 {
        return;
    }
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = ODataOrderBy::from_signed_tokens(s);
    }
});
