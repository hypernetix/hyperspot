#![allow(clippy::unwrap_used, clippy::expect_used)]

use modkit_security::constants::DEFAULT_TENANT_ID;
use modkit_security::AccessScope;

#[test]
fn empty_scope_is_deny_all() {
    // Empty scope means deny all access
    let scope = AccessScope::default();
    assert!(scope.is_empty());
    assert!(scope.tenant_ids().is_empty());
    assert!(scope.resource_ids().is_empty());
}

#[test]
fn tenant_scope_is_not_empty() {
    let scope = AccessScope::tenant(DEFAULT_TENANT_ID);

    assert!(!scope.is_empty());
    assert_eq!(scope.tenant_ids(), &[DEFAULT_TENANT_ID]);
}
