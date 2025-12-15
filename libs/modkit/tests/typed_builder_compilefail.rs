#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Compile-fail tests to verify type-safe API builder enforcement
//!
//! These tests ensure that the type-state pattern correctly prevents
//! invalid API operations from compiling.

#[test]
#[ignore = "TODO: Need to turn it on after fix issue with CR/LF"]
fn compile_fail_tests() {
    // On MinGW (windows-gnu), native deps like `ring` may fail to build in trybuild sandboxes.
    // Skip these compile-fail tests in that environment.
    if cfg!(all(target_os = "windows", target_env = "gnu")) {
        eprintln!("Skipping trybuild compile-fail tests on windows-gnu host");
        return;
    }

    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/no_handler.rs");
    t.compile_fail("tests/ui/no_response.rs");
    t.compile_fail("tests/ui/no_handler_no_response.rs");
}
