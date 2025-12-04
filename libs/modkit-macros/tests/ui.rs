#![allow(clippy::unwrap_used, clippy::expect_used)]

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    // Passing cases
    t.pass("tests/ui/pass/*.rs");
    // Compile-fail cases with snapshot comparison
    t.compile_fail("tests/ui/fail/*.rs");
}
