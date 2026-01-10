//! Integration tests for the secure ORM layer.
//!
//! Note: Full integration tests with `SeaORM` entities should be written in actual
//! application code where real entities are defined. These are basic unit tests
//! for the core condition building logic.
//!
//! See `USAGE_EXAMPLE.md` for complete usage examples with real `SeaORM` entities.

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(clippy::disallowed_methods)]
mod integration_tests {
    use crate::secure::AccessScope;
    use uuid::Uuid;

    #[test]
    fn test_access_scope_is_empty() {
        // Empty scope = deny all
        let scope = AccessScope::default();
        assert!(scope.is_empty());

        // Scope with tenants is not empty
        let scope = AccessScope::tenants_only(vec![Uuid::new_v4()]);
        assert!(!scope.is_empty());

        // Scope with resources is not empty
        let scope = AccessScope::resources_only(vec![Uuid::new_v4()]);
        assert!(!scope.is_empty());

        // Scope with both is not empty
        let scope = AccessScope::both(vec![Uuid::new_v4()], vec![Uuid::new_v4()]);
        assert!(!scope.is_empty());
    }

    #[test]
    fn test_empty_scope_is_deny_all() {
        let empty_scope = AccessScope::default();

        // Empty scope should be marked as empty (deny all)
        assert!(empty_scope.is_empty());
        assert!(empty_scope.tenant_ids().is_empty());
        assert!(empty_scope.resource_ids().is_empty());
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
    fn test_security_ctx_deny_all() {
        use crate::secure::SecurityCtx;

        #[allow(deprecated)]
        let deny_all_ctx = SecurityCtx::deny_all(Uuid::new_v4());

        // Deny-all context should be denied
        assert!(deny_all_ctx.is_denied());
        assert!(deny_all_ctx.scope().is_empty());
    }

    #[test]
    fn test_security_ctx_for_tenant() {
        use crate::secure::SecurityCtx;

        let tenant_id = Uuid::new_v4();
        let subject_id = Uuid::new_v4();
        #[allow(deprecated)]
        let ctx = SecurityCtx::for_tenant(tenant_id, subject_id);

        // Context with tenant is not denied
        assert!(!ctx.is_denied());
        assert!(!ctx.scope().is_empty());
        assert_eq!(ctx.scope().tenant_ids(), &[tenant_id]);
        assert_eq!(ctx.subject_id(), subject_id);
    }

    // Note: Full entity integration tests should be written in application code
    // where actual SeaORM entities are available. See USAGE_EXAMPLE.md for patterns.
}
