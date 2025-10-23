use modkit_security::{AccessScope, SecurityCtx, Subject, ROOT_SUBJECT_ID, ROOT_TENANT_ID};

#[test]
fn root_constants_and_helpers() {
    let ctx = SecurityCtx::root_ctx();
    assert_eq!(ctx.subject_id(), ROOT_SUBJECT_ID);
    assert!(ctx.scope().tenant_ids().contains(&ROOT_TENANT_ID));
    assert!(ctx.subject().is_root());
    assert!(ctx.scope().includes_root_tenant());

    let scope = AccessScope::root_tenant();
    assert!(scope.includes_root_tenant());

    let subj = Subject::root();
    assert!(subj.is_root());
}
