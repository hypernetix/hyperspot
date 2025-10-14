// Compile-fail tests for the Scopable derive macro.
// IMPORTANT: These files must not import external crates like sea_orm or uuid.
// We only validate macro input diagnostics here.

#[test]
fn trybuild_tests() {
    let t = trybuild::TestCases::new();

    t.compile_fail("tests/ui/err_unknown_attr.rs");
    t.compile_fail("tests/ui/err_non_struct.rs");
    t.compile_fail("tests/ui/err_duplicate_tenant_col.rs");
    t.compile_fail("tests/ui/err_unrestricted_with_tenant.rs");
}
