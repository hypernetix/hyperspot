#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Compile-fail tests for `domain_model` macro enforcement.
//!
//! These tests verify that the `domain_model` macro correctly rejects
//! types with infrastructure dependencies at compile time.

#[test]
#[cfg(not(coverage_nightly))]
#[ignore = "TODO: Enable after generating .stderr files"]
fn domain_model_compile_fail_tests() {
    // On MinGW (windows-gnu), native deps like `ring` may fail to build in trybuild sandboxes.
    if cfg!(all(target_os = "windows", target_env = "gnu")) {
        eprintln!("Skipping trybuild compile-fail tests on windows-gnu host");
        return;
    }

    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/domain_model_with_infra_type.rs");
    t.pass("tests/ui/domain_model_valid.rs");
}

#[cfg(test)]
mod unit_tests {
    use modkit::domain::{DomainModel, DomainSafe};

    // Test that marker traits work correctly
    #[allow(dead_code)]
    fn assert_domain_safe<T: DomainSafe>() {}
    #[allow(dead_code)]
    fn assert_domain_model<T: DomainModel>() {}

    #[test]
    fn test_primitives_are_domain_safe() {
        assert_domain_safe::<bool>();
        assert_domain_safe::<i32>();
        assert_domain_safe::<i64>();
        assert_domain_safe::<u32>();
        assert_domain_safe::<f64>();
        assert_domain_safe::<String>();
        assert_domain_safe::<char>();
    }

    #[test]
    fn test_collections_are_domain_safe() {
        assert_domain_safe::<Vec<String>>();
        assert_domain_safe::<Option<i32>>();
        assert_domain_safe::<std::collections::HashMap<String, i32>>();
        assert_domain_safe::<std::collections::HashSet<String>>();
        assert_domain_safe::<std::collections::BTreeMap<String, i32>>();
    }

    #[test]
    fn test_uuid_is_domain_safe() {
        assert_domain_safe::<uuid::Uuid>();
    }

    #[test]
    fn test_nested_types_are_domain_safe() {
        assert_domain_safe::<Vec<Option<uuid::Uuid>>>();
        assert_domain_safe::<Option<Vec<String>>>();
        assert_domain_safe::<Result<String, String>>();
        assert_domain_safe::<Box<String>>();
        assert_domain_safe::<std::sync::Arc<String>>();
    }

    #[test]
    fn test_tuples_are_domain_safe() {
        assert_domain_safe::<(i32,)>();
        assert_domain_safe::<(i32, String)>();
        assert_domain_safe::<(i32, String, bool)>();
    }

    #[test]
    fn test_modkit_page_is_domain_safe() {
        assert_domain_safe::<modkit_odata::Page<String>>();
        assert_domain_safe::<modkit_odata::PageInfo>();
    }
}
