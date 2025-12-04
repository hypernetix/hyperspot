#![allow(clippy::unwrap_used, clippy::expect_used)]

use modkit_security::{
    decode_bin, encode_bin, AccessScope, SecurityCtx, Subject, SECCTX_BIN_VERSION,
};
use uuid::Uuid;

#[test]
fn round_trips_security_ctx_binary_payload() {
    let tenant_ids = vec![
        Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa),
        Uuid::from_u128(0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb),
    ];
    let resource_ids = vec![
        Uuid::from_u128(0x11111111111111111111111111111111),
        Uuid::from_u128(0x22222222222222222222222222222222),
    ];
    let subject_id = Uuid::from_u128(0xdeadbeefdeadbeefdeadbeefdeadbeef);

    let scope = AccessScope::both(tenant_ids.clone(), resource_ids.clone());
    let subject = Subject::new(subject_id);
    let ctx = SecurityCtx::new(scope.clone(), subject.clone());

    let encoded = encode_bin(&ctx).expect("security context encodes");
    let decoded = decode_bin(&encoded).expect("security context decodes");

    assert_eq!(decoded.scope(), ctx.scope());
    assert_eq!(decoded.scope().tenant_ids(), tenant_ids.as_slice());
    assert_eq!(decoded.scope().resource_ids(), resource_ids.as_slice());
    assert_eq!(decoded.subject(), ctx.subject());
    assert_eq!(decoded.subject_id(), subject.id());
}

#[test]
fn decode_rejects_unknown_version() {
    let tenant_ids = vec![Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa)];
    let subject_id = Uuid::from_u128(0x33333333333333333333333333333333);

    let ctx = SecurityCtx::for_tenants(tenant_ids, subject_id);
    let mut encoded = encode_bin(&ctx).expect("encodes context");
    encoded[0] = SECCTX_BIN_VERSION.wrapping_add(1);

    let err = decode_bin(&encoded).expect_err("version mismatch should error");
    let message = err.to_string();
    assert!(
        message.contains("unsupported secctx version"),
        "expected version error, got: {message}"
    );
}
