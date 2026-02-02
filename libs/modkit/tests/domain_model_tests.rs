#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    dead_code,
    clippy::box_collection,
    clippy::rc_buffer
)]

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
    use modkit::domain::DomainModel;
    use modkit_macros::domain_model;

    // Test that DomainModel trait works correctly
    #[allow(dead_code)]
    fn assert_domain_model<T: DomainModel>() {}

    #[test]
    fn test_domain_model_macro_with_primitives() {
        #[domain_model]
        struct User {
            id: i32,
            name: String,
            active: bool,
        }

        assert_domain_model::<User>();
    }

    #[test]
    fn test_domain_model_macro_with_collections() {
        #[domain_model]
        struct Container {
            items: Vec<String>,
            maybe_value: Option<i32>,
            tags: std::collections::HashSet<String>,
        }

        assert_domain_model::<Container>();
    }

    #[test]
    fn test_domain_model_macro_with_uuid() {
        #[domain_model]
        struct Entity {
            id: uuid::Uuid,
            name: String,
        }

        assert_domain_model::<Entity>();
    }

    #[test]
    fn test_domain_model_macro_with_nested_types() {
        #[domain_model]
        struct Nested {
            maybe_items: Vec<Option<uuid::Uuid>>,
            boxed: Box<String>,
            arc: std::sync::Arc<String>,
        }

        assert_domain_model::<Nested>();
    }

    #[test]
    fn test_domain_model_macro_with_tuples() {
        #[domain_model]
        struct WithTuples {
            pair: (i32, String),
            triple: (i32, String, bool),
        }

        assert_domain_model::<WithTuples>();
    }

    #[test]
    fn test_domain_model_macro_with_page() {
        #[domain_model]
        struct WithPagination {
            results: modkit_odata::Page<String>,
            page_info: modkit_odata::PageInfo,
        }

        assert_domain_model::<WithPagination>();
    }

    #[test]
    fn test_domain_model_macro_with_enum() {
        #[domain_model]
        enum Status {
            Active,
            Inactive { reason: String },
            Pending(i32),
        }

        assert_domain_model::<Status>();
    }

    #[test]
    fn test_domain_model_macro_unit_struct() {
        #[domain_model]
        struct Marker;

        assert_domain_model::<Marker>();
    }

    #[test]
    fn test_domain_model_macro_tuple_struct() {
        #[domain_model]
        struct UserId(uuid::Uuid);

        assert_domain_model::<UserId>();
    }
}
