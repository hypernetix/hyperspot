//! Compile-fail UI tests for modkit-db.
//!
//! These tests verify that certain incorrect usages of the secure database API
//! produce compile-time errors, ensuring security properties are enforced by
//! the type system.

#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail/*.rs");
}
