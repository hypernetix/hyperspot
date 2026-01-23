#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_yaml::Value;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Parse YAML into generic Value
        let _ = serde_yaml::from_str::<Value>(s);
    }
});
