#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Test that service update operations respect security scope.

mod support;

use modkit_db::secure::SecureConn;
use std::sync::Arc;
use support::{
    ctx_allow_tenants, ctx_deny_all, inmem_db, seed_user, MockAuditPort, MockEventPublisher,
};
use user_info_sdk::UserPatch;
use users_info::domain::service::{Service, ServiceConfig};
use uuid::Uuid;

#[tokio::test]
async fn update_with_scoped_ctx_denies_out_of_scope() {
    // Arrange: Create database with a test user
    let db = inmem_db().await;
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let user = seed_user(&db, user_id, tenant1, "test@example.com", "Test User").await;

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    // Create context for a different tenant
    let ctx_deny = ctx_allow_tenants(&[tenant2]);

    // Act: Try to update user outside scope
    let patch = UserPatch {
        email: Some("hacker@example.com".to_owned()),
        display_name: None,
    };

    let result = service.update_user(&ctx_deny, user.id, patch).await;

    // Assert: Should fail - user not found in scope
    assert!(
        result.is_err(),
        "Update should fail for out-of-scope tenant"
    );
}

#[tokio::test]
async fn update_succeeds_within_scope() {
    // Arrange: Create database with a test user
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let _user = seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    // Create context with access to this tenant
    let ctx_ok = ctx_allow_tenants(&[tenant_id]);

    // Act: Update user within scope
    let patch = UserPatch {
        email: Some("updated@example.com".to_owned()),
        display_name: None,
    };

    let result = service.update_user(&ctx_ok, user_id, patch).await;

    // Assert: Should succeed
    assert!(result.is_ok(), "Update should succeed for in-scope tenant");

    // Verify the update persisted
    let loaded = service.get_user(&ctx_ok, user_id).await.unwrap();
    assert_eq!(loaded.email, "updated@example.com");
}

#[tokio::test]
async fn update_with_deny_all_fails() {
    // Arrange: Create database with a test user
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let user = seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    // Create deny-all context
    let ctx = ctx_deny_all();

    // Act: Try to update with deny-all context
    let patch = UserPatch {
        email: Some("blocked@example.com".to_owned()),
        display_name: None,
    };

    let result = service.update_user(&ctx, user.id, patch).await;

    // Assert: Should fail
    assert!(result.is_err(), "Deny-all context should prevent updates");
}
