#![allow(clippy::unwrap_used, clippy::expect_used)]

use modkit_security::constants::{DEFAULT_SUBJECT_ID, DEFAULT_TENANT_ID};
use modkit_security::{AccessScope, SecurityCtx, Subject};

#[test]
fn default_subject_and_context() {
    // Create subject with default ID
    let subj = Subject::new(DEFAULT_SUBJECT_ID);
    assert_eq!(subj.id(), DEFAULT_SUBJECT_ID);

    // Context operates within the default tenant
    #[allow(deprecated)]
    let ctx = SecurityCtx::for_tenant(DEFAULT_TENANT_ID, DEFAULT_SUBJECT_ID);

    assert_eq!(ctx.subject_id(), DEFAULT_SUBJECT_ID);
    assert!(!ctx.scope().is_empty(), "Context should have tenant scope");
    assert_eq!(ctx.scope().tenant_ids(), &[DEFAULT_TENANT_ID]);
}

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
