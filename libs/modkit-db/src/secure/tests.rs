//! Integration tests for the secure ORM layer.

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(clippy::disallowed_methods)]
mod integration_tests {
    use crate::secure::AccessScope;
    use modkit_security::access_scope::{FilterOp, ScopeConstraint, ScopeFilter, pep_properties};
    use uuid::Uuid;

    #[test]
    fn test_access_scope_is_deny_all() {
        // Empty scope = deny all
        let scope = AccessScope::default();
        assert!(scope.is_deny_all());

        // Scope with tenants is not deny-all
        let scope = AccessScope::for_tenants(vec![Uuid::new_v4()]);
        assert!(!scope.is_deny_all());

        // Scope with resources is not deny-all
        let scope = AccessScope::for_resources(vec![Uuid::new_v4()]);
        assert!(!scope.is_deny_all());

        // Scope with both is not deny-all
        let scope = AccessScope::single(ScopeConstraint::new(vec![
            ScopeFilter::new(
                pep_properties::OWNER_TENANT_ID,
                FilterOp::In,
                vec![Uuid::new_v4()],
            ),
            ScopeFilter::new(
                pep_properties::RESOURCE_ID,
                FilterOp::In,
                vec![Uuid::new_v4()],
            ),
        ]));
        assert!(!scope.is_deny_all());
    }

    #[test]
    fn test_empty_scope_is_deny_all() {
        let empty_scope = AccessScope::default();

        assert!(empty_scope.is_deny_all());
        assert!(
            empty_scope
                .all_values_for(pep_properties::OWNER_TENANT_ID)
                .is_empty()
        );
        assert!(
            empty_scope
                .all_values_for(pep_properties::RESOURCE_ID)
                .is_empty()
        );
    }

    #[test]
    fn test_access_scope_builders() {
        let tid = Uuid::new_v4();
        let rid = Uuid::new_v4();

        let scope = AccessScope::for_tenants(vec![tid]);
        assert_eq!(
            scope.all_values_for(pep_properties::OWNER_TENANT_ID),
            &[tid]
        );
        assert!(scope.all_values_for(pep_properties::RESOURCE_ID).is_empty());

        let scope = AccessScope::for_resources(vec![rid]);
        assert!(
            scope
                .all_values_for(pep_properties::OWNER_TENANT_ID)
                .is_empty()
        );
        assert_eq!(scope.all_values_for(pep_properties::RESOURCE_ID), &[rid]);

        let scope = AccessScope::single(ScopeConstraint::new(vec![
            ScopeFilter::new(pep_properties::OWNER_TENANT_ID, FilterOp::In, vec![tid]),
            ScopeFilter::new(pep_properties::RESOURCE_ID, FilterOp::In, vec![rid]),
        ]));
        assert_eq!(
            scope.all_values_for(pep_properties::OWNER_TENANT_ID),
            &[tid]
        );
        assert_eq!(scope.all_values_for(pep_properties::RESOURCE_ID), &[rid]);
    }

    #[test]
    fn test_allow_all_is_unconstrained() {
        let scope = AccessScope::allow_all();
        assert!(scope.is_unconstrained());
        assert!(!scope.is_deny_all());
    }

    #[test]
    fn test_or_constraints() {
        let t1 = Uuid::new_v4();
        let t2 = Uuid::new_v4();
        let r1 = Uuid::new_v4();

        let scope = AccessScope::from_constraints(vec![
            ScopeConstraint::new(vec![
                ScopeFilter::new(pep_properties::OWNER_TENANT_ID, FilterOp::In, vec![t1]),
                ScopeFilter::new(pep_properties::RESOURCE_ID, FilterOp::In, vec![r1]),
            ]),
            ScopeConstraint::new(vec![ScopeFilter::new(
                pep_properties::OWNER_TENANT_ID,
                FilterOp::In,
                vec![t2],
            )]),
        ]);

        assert_eq!(scope.constraints().len(), 2);
        assert!(scope.contains_value(pep_properties::OWNER_TENANT_ID, t1));
        assert!(scope.contains_value(pep_properties::OWNER_TENANT_ID, t2));
        assert!(scope.contains_value(pep_properties::RESOURCE_ID, r1));
        // all_values_for collects from all constraints
        assert_eq!(
            scope.all_values_for(pep_properties::OWNER_TENANT_ID).len(),
            2
        );
    }

    #[test]
    fn test_has_property() {
        let tid = Uuid::new_v4();
        let scope = AccessScope::for_tenant(tid);
        assert!(scope.has_property(pep_properties::OWNER_TENANT_ID));
        assert!(!scope.has_property(pep_properties::RESOURCE_ID));
    }
}
