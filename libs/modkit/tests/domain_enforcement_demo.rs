//! Demonstration test proving domain enforcement works at compile time.
//!
//! This test file demonstrates that `#[domain_model]` macro validates field types
//! at macro expansion time. The validation checks field type names against
//! forbidden patterns (e.g., `sqlx::`, `http::`, `sea_orm::`).
//!
//! For compile-fail tests, see `domain_model_tests.rs` which uses trybuild.

#![allow(clippy::str_to_string)]

use modkit::domain::DomainModel;
use modkit_macros::domain_model;
use uuid::Uuid;

#[domain_model]
#[derive(Debug, Clone)]
pub struct ValidUser {
    pub id: Uuid,
    pub email: String,
    pub active: bool,
}

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
