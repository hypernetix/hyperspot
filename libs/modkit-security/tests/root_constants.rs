use modkit_security::{AccessScope, SecurityCtx, Subject, ROOT_SUBJECT_ID, ROOT_TENANT_ID};

#[test]
fn root_constants_and_helpers() {
    // Root context now uses explicit is_root flag
    let ctx = SecurityCtx::root_ctx();
    assert_eq!(ctx.subject_id(), ROOT_SUBJECT_ID);
    assert!(
        ctx.scope().is_root(),
        "Root context should have is_root=true"
    );
    assert!(
        ctx.scope().tenant_ids().is_empty(),
        "Root scope should have empty tenant_ids (no longer contains ROOT_TENANT_ID)"
    );
    assert!(
        !ctx.scope().is_empty(),
        "Root scope should not be considered empty"
    );
    assert!(ctx.subject().is_root());

    // Root scope no longer contains ROOT_TENANT_ID by default
    let scope = AccessScope::root_tenant();
    assert!(scope.is_root());
    assert!(
        !scope.includes_root_tenant(),
        "Root scope no longer uses ROOT_TENANT_ID"
    );

    let subj = Subject::root();
    assert!(subj.is_root());

    // The constants still exist for backward compatibility or explicit usage
    assert_eq!(
        ROOT_TENANT_ID.as_u128(),
        0x00000000_df51_5b42_9538_d2b56b7ee953
    );
    assert_eq!(
        ROOT_SUBJECT_ID.as_u128(),
        0x11111111_6a88_4768_9dfc_6bcd5187d9ed
    );
}
