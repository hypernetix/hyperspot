#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Test that service delete operations respect security scope.

mod support;

use modkit_db::secure::SecureConn;
use std::sync::Arc;
use support::{ctx_allow_tenants, ctx_deny_all, inmem_db, seed_user, MockAuditPort, MockEventPublisher};
use users_info::domain::service::{Service, ServiceConfig};
use uuid::Uuid;

#[tokio::test]
async fn delete_respects_tenant_scope() {
    // Arrange: Create database with a test user
    let db = inmem_db().await;
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    seed_user(&db, user_id, tenant1, "test@example.com", "Test User").await;

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    // Create context for a different tenant
    let ctx_deny = ctx_allow_tenants(&[tenant2]);

    // Act: Try to delete user outside scope
    let result = service.delete_user(&ctx_deny, user_id).await;

    // Assert: Should return error - user not found in scope
    assert!(result.is_err(), "Should not delete user outside tenant scope");
}

#[tokio::test]
async fn delete_succeeds_within_scope() {
    // Arrange: Create database with a test user
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    // Create context with access to this tenant
    let ctx_ok = ctx_allow_tenants(&[tenant_id]);

    // Act: Delete user within scope
    let result = service.delete_user(&ctx_ok, user_id).await;

    // Assert: Should succeed
    assert!(result.is_ok(), "Should successfully delete user in scope");

    // Verify the user is gone
    let loaded = service.get_user(&ctx_ok, user_id).await;
    assert!(loaded.is_err(), "User should be deleted");
}

#[tokio::test]
async fn delete_with_deny_all_returns_false() {
    // Arrange: Create database with a test user
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    // Create deny-all context
    let ctx = ctx_deny_all();

    // Act: Try to delete with deny-all context
    let result = service.delete_user(&ctx, user_id).await;

    // Assert: Should return error
    assert!(result.is_err(), "Deny-all context should not delete any data");
}
