#![allow(clippy::unwrap_used, clippy::expect_used)]

use modkit_security::AccessScope;
use modkit_security::constants::DEFAULT_TENANT_ID;
use modkit_security::pep_properties;

#[test]
fn empty_scope_is_deny_all() {
    // Empty scope means deny all access
    let scope = AccessScope::default();
    assert!(scope.is_deny_all());
    assert!(
        scope
            .all_values_for(pep_properties::OWNER_TENANT_ID)
            .is_empty()
    );
    assert!(scope.all_values_for(pep_properties::RESOURCE_ID).is_empty());
}

#[test]
fn tenant_scope_is_not_empty() {
    let scope = AccessScope::for_tenant(DEFAULT_TENANT_ID);

    assert!(!scope.is_deny_all());
    assert_eq!(
        scope.all_values_for(pep_properties::OWNER_TENANT_ID),
        &[DEFAULT_TENANT_ID]
    );
}

#[test]
fn allow_all_scope_is_unconstrained() {
    let scope = AccessScope::allow_all();
    assert!(scope.is_unconstrained());
    assert!(!scope.is_deny_all());
}

#[test]
fn contains_value_works() {
    let tid = uuid::Uuid::new_v4();
    let scope = AccessScope::for_tenant(tid);
    assert!(scope.contains_value(pep_properties::OWNER_TENANT_ID, tid));
    assert!(!scope.contains_value(pep_properties::OWNER_TENANT_ID, uuid::Uuid::new_v4()));
}
