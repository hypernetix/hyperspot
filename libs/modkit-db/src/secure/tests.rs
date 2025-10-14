//! Integration tests for the secure ORM layer.
//!
//! Note: Full integration tests with SeaORM entities should be written in actual
//! application code where real entities are defined. These are basic unit tests
//! for the core condition building logic.
//!
//! See USAGE_EXAMPLE.md for complete usage examples with real SeaORM entities.

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod integration_tests {
    use crate::secure::AccessScope;
    use uuid::Uuid;

    #[test]
    fn test_access_scope_is_empty() {
        let scope = AccessScope::default();
        assert!(scope.is_empty());

        let scope = AccessScope::tenants_only(vec![Uuid::new_v4()]);
        assert!(!scope.is_empty());

        let scope = AccessScope::resources_only(vec![Uuid::new_v4()]);
        assert!(!scope.is_empty());

        let scope = AccessScope::both(vec![Uuid::new_v4()], vec![Uuid::new_v4()]);
        assert!(!scope.is_empty());
    }

    #[test]
    fn test_access_scope_builders() {
        let tid = Uuid::new_v4();
        let rid = Uuid::new_v4();

        let scope = AccessScope::tenants_only(vec![tid]);
        assert_eq!(scope.tenant_ids(), &[tid]);
        assert!(scope.resource_ids().is_empty());

        let scope = AccessScope::resources_only(vec![rid]);
        assert!(scope.tenant_ids().is_empty());
        assert_eq!(scope.resource_ids(), &[rid]);

        let scope = AccessScope::both(vec![tid], vec![rid]);
        assert_eq!(scope.tenant_ids(), &[tid]);
        assert_eq!(scope.resource_ids(), &[rid]);
    }

    // Note: Full entity integration tests should be written in application code
    // where actual SeaORM entities are available. See USAGE_EXAMPLE.md for patterns.
}
