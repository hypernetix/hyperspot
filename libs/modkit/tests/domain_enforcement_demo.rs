//! Demonstration test proving domain enforcement works at compile time.
//!
//! This test file demonstrates TWO levels of enforcement:
//!
//! 1. **Field-level**: `#[domain_model]` macro checks that all fields implement `DomainSafe`
//! 2. **Repository-level**: Trait bounds require types to implement `DomainModel`
//!
//! To verify enforcement works, uncomment the `ENFORCEMENT_TEST_*` sections below
//! and run `cargo check -p cf-modkit`. Each should produce a compile error.

#![allow(clippy::str_to_string)]

use modkit::domain::{DomainModel, DomainSafe};
use modkit_macros::domain_model;
use uuid::Uuid;

// ============================================================================
// VALID: Domain model with only DomainSafe fields - COMPILES
// ============================================================================

#[domain_model]
#[derive(Debug, Clone)]
pub struct ValidUser {
    pub id: Uuid,
    pub email: String,
    pub active: bool,
}

// ============================================================================
// ENFORCEMENT_TEST_1: Field-level enforcement
// Error: `DomainSafe` is not implemented for `http::StatusCode`
// ============================================================================

// #[domain_model]
// pub struct BadModelWithInfraField {
//     pub id: Uuid,
//     pub status: http::StatusCode,  // INFRA TYPE - will fail!
// }

// ============================================================================
// ENFORCEMENT_TEST_2: Uncomment to see repository-level enforcement
// Error: `DomainModel` is not implemented for `UnmarkedModel`
// ============================================================================

// pub struct UnmarkedModel {
//     pub id: Uuid,
// }
//
// pub trait TestRepository
// where
//     UnmarkedModel: DomainModel,  // This bound will fail!
// {
//     fn find(&self) -> Option<UnmarkedModel>;
// }

// ============================================================================
// Compile-time assertion that ValidUser implements required traits
// ============================================================================

const _: () = {
    #[allow(dead_code)]
    fn assert_domain_safe<T: DomainSafe>() {}
    #[allow(dead_code)]
    fn assert_domain_model<T: DomainModel>() {}

    fn _compile_time_checks() {
        assert_domain_safe::<ValidUser>();
        assert_domain_model::<ValidUser>();
    }
};

#[test]
fn test_valid_user_is_domain_model() {
    fn requires_domain_model<T: DomainModel>(_: &T) {}

    let user = ValidUser {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        active: true,
    };

    requires_domain_model(&user);
}

/// This test demonstrates the repository pattern with trait bounds.
/// If `ValidUser` didn't have `#[domain_model]`, this wouldn't compile.
mod repository_pattern_demo {
    use super::*;

    pub trait UserRepository
    where
        ValidUser: DomainModel,
    {
        fn find_by_id(&self, id: Uuid) -> Option<ValidUser>;
    }

    struct MockRepo;

    impl UserRepository for MockRepo {
        fn find_by_id(&self, _id: Uuid) -> Option<ValidUser> {
            Some(ValidUser {
                id: Uuid::new_v4(),
                email: "mock@test.com".to_string(),
                active: true,
            })
        }
    }

    #[test]
    fn test_repository_with_domain_model_bound() {
        let repo = MockRepo;
        let user = repo.find_by_id(Uuid::new_v4());
        assert!(user.is_some());
    }
}
