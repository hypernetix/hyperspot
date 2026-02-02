//! Demonstration test proving domain enforcement works at compile time.
//!
//! This test file demonstrates TWO levels of enforcement:
//!
//! 1. **Macro-level**: `#[domain_model]` macro validates field types at macro expansion time
//! 2. **Trait-level**: Trait bounds require types to implement `DomainModel`
//!
//! The validation happens during macro expansion by checking field type names
//! against forbidden patterns (e.g., `sqlx::`, `http::`, `sea_orm::`).
//!
//! To verify enforcement works, uncomment the `ENFORCEMENT_TEST_*` sections below
//! and run `cargo check -p cf-modkit`. Each should produce a compile error.

#![allow(clippy::str_to_string)]

use modkit::domain::DomainModel;
use modkit_macros::domain_model;
use uuid::Uuid;

// ============================================================================
// VALID: Domain model with only allowed field types - COMPILES
// ============================================================================

#[domain_model]
#[derive(Debug, Clone)]
pub struct ValidUser {
    pub id: Uuid,
    pub email: String,
    pub active: bool,
}

// ============================================================================
// ENFORCEMENT_TEST_1: Macro-level validation
// Error: field 'status' has type 'http::StatusCode' which matches forbidden pattern
// ============================================================================

// #[domain_model]
// pub struct BadModelWithHttpType {
//     pub id: Uuid,
//     pub status: http::StatusCode,  // FORBIDDEN - will fail at macro expansion!
// }

// ============================================================================
// ENFORCEMENT_TEST_2: Macro-level validation with database type
// Error: field 'pool' has type 'sqlx::PgPool' which matches forbidden pattern
// ============================================================================

// #[domain_model]
// pub struct BadModelWithDbType {
//     pub id: Uuid,
//     pub pool: sqlx::PgPool,  // FORBIDDEN - will fail at macro expansion!
// }

// ============================================================================
// ENFORCEMENT_TEST_3: Trait-level enforcement
// Error: `DomainModel` is not implemented for `UnmarkedModel`
// ============================================================================

// pub struct UnmarkedModel {
//     pub id: Uuid,
// }
//
// pub trait TestRepository
// where
//     UnmarkedModel: DomainModel,  // This bound will fail - no #[domain_model] macro!
// {
//     fn find(&self) -> Option<UnmarkedModel>;
// }

// ============================================================================
// Compile-time assertion that ValidUser implements DomainModel
// ============================================================================

const _: () = {
    #[allow(dead_code)]
    fn assert_domain_model<T: DomainModel>() {}

    fn _compile_time_checks() {
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
