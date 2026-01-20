#![allow(clippy::unwrap_used, clippy::expect_used)]

use modkit_security::{Permission, SECCTX_BIN_VERSION, SecurityContext, decode_bin, encode_bin};
use uuid::Uuid;

#[test]
#[allow(clippy::unreadable_literal)] // UUID hex patterns are intentionally repeating
fn round_trips_security_ctx_binary_payload() {
    let tenant_id = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa);
    let resource_ids = [
        Uuid::from_u128(0x11111111111111111111111111111111),
        Uuid::from_u128(0x22222222222222222222222222222222),
    ];
    let subject_id = Uuid::from_u128(0xdeadbeefdeadbeefdeadbeefdeadbeef);

    let ctx = SecurityContext::builder()
        .tenant_id(tenant_id)
        .subject_id(subject_id)
        .add_permission(
            Permission::builder()
                .resource_pattern("book")
                .resource_id(resource_ids[0])
                .action("read")
                .build()
                .unwrap(),
        )
        .add_permission(
            Permission::builder()
                .resource_pattern("book")
                .resource_id(resource_ids[1])
                .action("read")
                .build()
                .unwrap(),
        )
        .build();

    let encoded = encode_bin(&ctx).expect("security context encodes");
    let decoded = decode_bin(&encoded).expect("security context decodes");

    // Validate core fields round-trip
    assert_eq!(decoded.tenant_id(), ctx.tenant_id());
    assert_eq!(decoded.subject_id(), ctx.subject_id());

    let decoded_perms = decoded.permissions();
    let orig_perms = ctx.permissions();
    assert_eq!(decoded_perms.len(), orig_perms.len());

    for (i, p) in decoded_perms.iter().enumerate() {
        assert_eq!(p.resource_pattern(), orig_perms[i].resource_pattern());
        assert_eq!(p.resource_id(), orig_perms[i].resource_id());
        assert_eq!(p.action(), orig_perms[i].action());
        assert_eq!(p.tenant_id(), orig_perms[i].tenant_id());
    }
}

#[test]
#[allow(clippy::unreadable_literal)] // UUID hex patterns are intentionally repeating
fn decode_rejects_unknown_version() {
    let tenant_id = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa);
    let subject_id = Uuid::from_u128(0x33333333333333333333333333333333);

    let ctx = SecurityContext::builder()
        .tenant_id(tenant_id)
        .subject_id(subject_id)
        .build();

    let mut encoded = encode_bin(&ctx).expect("encodes context");
    encoded[0] = SECCTX_BIN_VERSION.wrapping_add(1);

    let err = decode_bin(&encoded).expect_err("version mismatch should error");
    let message = err.to_string();
    assert!(
        message.contains("unsupported secctx version"),
        "expected version error, got: {message}"
    );
}
