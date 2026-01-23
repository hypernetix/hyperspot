#![no_main]

use libfuzzer_sys::fuzz_target;
use modkit_odata::CursorV1;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Attempt to decode cursor
        let _ = CursorV1::decode(s);
    }
});
