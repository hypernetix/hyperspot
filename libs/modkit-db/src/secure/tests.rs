//! Integration tests for the secure ORM layer.
//!
//! Note: Full integration tests with `SeaORM` entities should be written in actual
//! application code where real entities are defined. These are basic unit tests
//! for the core condition building logic.
//!
//! See `USAGE_EXAMPLE.md` for complete usage examples with real `SeaORM` entities.

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod integration_tests {
    use crate::secure::AccessScope;
    use uuid::Uuid;

    #[test]
    fn test_access_scope_is_empty() {
        let scope = AccessScope::default();
        assert!(scope.is_empty());
        assert!(!scope.is_root());

        let scope = AccessScope::tenants_only(vec![Uuid::new_v4()]);
        assert!(!scope.is_empty());
        assert!(!scope.is_root());

        let scope = AccessScope::resources_only(vec![Uuid::new_v4()]);
        assert!(!scope.is_empty());
        assert!(!scope.is_root());

        let scope = AccessScope::both(vec![Uuid::new_v4()], vec![Uuid::new_v4()]);
        assert!(!scope.is_empty());
        assert!(!scope.is_root());
    }

    #[test]
    fn test_root_scope_behavior() {
        let root_scope = AccessScope::root_tenant();

        // Root scope should never be considered empty
        assert!(!root_scope.is_empty());

        // Root scope should be marked as root
        assert!(root_scope.is_root());

        // Root scope should have no tenant or resource IDs by default
        assert!(root_scope.tenant_ids().is_empty());
        assert!(root_scope.resource_ids().is_empty());
    }

    #[test]
    fn test_root_vs_empty_scope() {
        let empty_scope = AccessScope::default();
        let root_scope = AccessScope::root_tenant();

        // Empty scope is empty, root scope is not
        assert!(empty_scope.is_empty());
        assert!(!root_scope.is_empty());

        // Only root scope is marked as root
        assert!(!empty_scope.is_root());
        assert!(root_scope.is_root());

        // Both have no tenant/resource IDs, but different semantics
        assert!(empty_scope.tenant_ids().is_empty());
        assert!(root_scope.tenant_ids().is_empty());
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

    #[test]
    fn test_security_ctx_root_ctx() {
        use crate::secure::SecurityCtx;

        let root_ctx = SecurityCtx::root_ctx();

        // Root context should have root scope
        assert!(root_ctx.scope().is_root());
        assert!(!root_ctx.scope().is_empty());

        // Root context should not be denied
        assert!(!root_ctx.is_denied());
    }

    #[test]
    fn test_security_ctx_deny_all_vs_root() {
        use crate::secure::SecurityCtx;

        let deny_all_ctx = SecurityCtx::deny_all(Uuid::new_v4());
        let root_ctx = SecurityCtx::root_ctx();

        // Deny-all context should be denied
        assert!(deny_all_ctx.is_denied());
        assert!(deny_all_ctx.scope().is_empty());
        assert!(!deny_all_ctx.scope().is_root());

        // Root context should not be denied
        assert!(!root_ctx.is_denied());
        assert!(!root_ctx.scope().is_empty());
        assert!(root_ctx.scope().is_root());
    }

    // Note: Full entity integration tests should be written in application code
    // where actual SeaORM entities are available. See USAGE_EXAMPLE.md for patterns.
}
