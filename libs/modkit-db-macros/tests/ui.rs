#![allow(clippy::unwrap_used, clippy::expect_used)]

// Compile-fail tests for the Scopable derive macro.
// IMPORTANT: These files must not import external crates like sea_orm or uuid.
// We only validate macro input diagnostics here.
//
// Note: We don't test successful macro expansions here because they would require
// importing the modkit-db crate. The macro is tested in actual usage in the main codebase.

#[test]
#[cfg(not(coverage_nightly))]
fn ui() {
    let t = trybuild::TestCases::new();

    // Error cases: Basic validation
    t.compile_fail("tests/ui/err_unknown_attr.rs");
    t.compile_fail("tests/ui/err_non_struct.rs");
    t.compile_fail("tests/ui/err_duplicate_tenant_col.rs");

    // Error cases: Missing explicit decisions
    t.compile_fail("tests/ui/err_missing_tenant_decision.rs");
    t.compile_fail("tests/ui/err_missing_resource_decision.rs");
    t.compile_fail("tests/ui/err_missing_owner_decision.rs");
    t.compile_fail("tests/ui/err_missing_type_decision.rs");

    // Error cases: Conflicting attributes
    t.compile_fail("tests/ui/err_conflicting_tenant.rs");
    t.compile_fail("tests/ui/err_conflicting_resource.rs");

    // Error cases: Unrestricted with other flags
    t.compile_fail("tests/ui/err_unrestricted_with_tenant.rs");
    t.compile_fail("tests/ui/err_unrestricted_with_resource.rs");

    // Error cases: pep_prop validation
    t.compile_fail("tests/ui/err_pep_reserved_owner_tenant_id.rs");
    t.compile_fail("tests/ui/err_pep_reserved_id.rs");
    t.compile_fail("tests/ui/err_pep_reserved_owner_id.rs");
    t.compile_fail("tests/ui/err_pep_duplicate_property.rs");
    t.compile_fail("tests/ui/err_unrestricted_with_pep.rs");

    // Note: Compile-pass tests (ok_*.rs) exist on disk for documentation but are
    // not registered here — successful expansion requires the modkit-db crate which
    // is not available in the trybuild environment. The macro is tested in actual
    // usage across the main codebase.
}
