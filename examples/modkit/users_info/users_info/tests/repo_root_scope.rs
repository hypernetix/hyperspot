#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Test that service operations work with root scope (system-level access).

mod support;

use modkit_db::secure::SecureConn;
use modkit_odata::ODataQuery;
use std::sync::Arc;
use support::{
    ctx_allow_tenants, ctx_root, inmem_db, seed_user, MockAuditPort, MockEventPublisher,
};
use users_info::domain::service::{Service, ServiceConfig};
use uuid::Uuid;

#[tokio::test]
async fn root_scope_can_access_all_tenants() {
    // Arrange: Create database with users in different tenants
    let db = inmem_db().await;
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();

    seed_user(&db, user1_id, tenant1, "user1@example.com", "User 1").await;
    seed_user(&db, user2_id, tenant2, "user2@example.com", "User 2").await;

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    // Create root context
    let ctx = ctx_root();

    // Act: List users with root context
    let query = ODataQuery::default();
    let result = service.list_users_page(&ctx, &query).await;

    // Assert: Should return users from all tenants
    assert!(result.is_ok(), "Root scope query should succeed");
    let page = result.unwrap();
    assert_eq!(page.items.len(), 2, "Root scope should access all tenants");
}

#[tokio::test]
async fn root_scope_can_find_users_in_any_tenant() {
    // Arrange: Create database with users in different tenants
    let db = inmem_db().await;
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();

    let user1 = seed_user(&db, user1_id, tenant1, "user1@example.com", "User 1").await;
    let user2 = seed_user(&db, user2_id, tenant2, "user2@example.com", "User 2").await;

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    // Create root context
    let ctx_root = ctx_root();

    // Act & Assert: Root context can find users in tenant1
    let result1 = service.get_user(&ctx_root, user1.id).await;
    assert!(result1.is_ok(), "Root scope should find user in tenant1");
    assert_eq!(result1.unwrap().email, "user1@example.com");

    // Act & Assert: Root context can find users in tenant2
    let result2 = service.get_user(&ctx_root, user2.id).await;
    assert!(result2.is_ok(), "Root scope should find user in tenant2");
    assert_eq!(result2.unwrap().email, "user2@example.com");
}

#[tokio::test]
async fn root_scope_vs_tenant_scope() {
    // Arrange: Create database with users in different tenants
    let db = inmem_db().await;
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();

    seed_user(&db, user1_id, tenant1, "user1@example.com", "User 1").await;
    seed_user(&db, user2_id, tenant2, "user2@example.com", "User 2").await;

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    // Create different contexts
    let ctx_root = ctx_root();
    let ctx_tenant1 = ctx_allow_tenants(&[tenant1]);

    // Act: List with root context
    let query = ODataQuery::default();
    let root_result = service.list_users_page(&ctx_root, &query).await.unwrap();

    // Act: List with tenant-scoped context
    let tenant_result = service.list_users_page(&ctx_tenant1, &query).await.unwrap();

    // Assert: Root context sees all users
    assert_eq!(
        root_result.items.len(),
        2,
        "Root scope should see all users"
    );

    // Assert: Tenant-scoped context sees only its tenant
    assert_eq!(
        tenant_result.items.len(),
        1,
        "Tenant scope should see only its tenant"
    );
    assert_eq!(tenant_result.items[0].tenant_id, tenant1);
}

#[tokio::test]
async fn root_scope_is_not_empty_scope() {
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

    // Create contexts
    let ctx_root = ctx_root();

    // Act: List with root context (root context should access all data)
    let query = ODataQuery::default();
    let result = service.list_users_page(&ctx_root, &query).await;

    // Assert: Should succeed and return data
    assert!(result.is_ok(), "Root scope query should succeed");
    let page = result.unwrap();
    assert_eq!(page.items.len(), 1, "Root scope should access data");
}
