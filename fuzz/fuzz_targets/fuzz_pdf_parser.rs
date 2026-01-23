#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // pdf-extract may panic - this is OK for fuzzing
    let _ = pdf_extract::extract_text_from_mem(data);
});
