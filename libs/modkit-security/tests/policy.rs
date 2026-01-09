#![allow(clippy::unwrap_used, clippy::expect_used)]

use modkit_security::SecurityCtx;
use uuid::Uuid;

#[test]
fn deny_all() {
    #[allow(deprecated)]
    let ctx = SecurityCtx::deny_all(Uuid::nil());
    assert!(ctx.is_denied());
    assert!(ctx.scope().is_empty());
}

#[test]
fn tenants_only() {
    let t = Uuid::new_v4();
    #[allow(deprecated)]
    let ctx = SecurityCtx::for_tenant(t, Uuid::nil());
    assert!(ctx.has_tenant_access());
    assert_eq!(ctx.scope().tenant_ids(), &[t]);
}

#[test]
fn resources_only() {
    let r = Uuid::new_v4();
    #[allow(deprecated)]
    let ctx = SecurityCtx::for_resource(r, Uuid::nil());
    assert!(ctx.has_resource_access());
    assert_eq!(ctx.scope().resource_ids(), &[r]);
}
