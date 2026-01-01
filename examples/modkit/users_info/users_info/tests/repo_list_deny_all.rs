#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Test that list operations respect deny-all security context.

mod support;

use modkit_db::secure::SecureConn;
use modkit_odata::ODataQuery;
use std::sync::Arc;
use support::{ctx_deny_all, inmem_db, seed_user, MockAuditPort, MockEventPublisher};
use users_info::domain::service::{Service, ServiceConfig};
use uuid::Uuid;

#[tokio::test]
async fn list_with_deny_all_returns_empty_page() {
    // Arrange: Create database with test users
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    seed_user(
        &db,
        Uuid::new_v4(),
        tenant_id,
        "user1@example.com",
        "User 1",
    )
    .await;
    seed_user(
        &db,
        Uuid::new_v4(),
        tenant_id,
        "user2@example.com",
        "User 2",
    )
    .await;
    seed_user(
        &db,
        Uuid::new_v4(),
        tenant_id,
        "user3@example.com",
        "User 3",
    )
    .await;

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    // Create deny-all context
    let ctx = ctx_deny_all();

    // Act: List users with deny-all context
    let query = ODataQuery::default();
    let result = service.list_users_page(&ctx, &query).await;

    // Assert: Should succeed but return empty page
    assert!(result.is_ok(), "Query should succeed");
    let page = result.unwrap();
    assert!(
        page.items.is_empty(),
        "Deny-all context should return no items"
    );
    assert_eq!(page.items.len(), 0);
}

#[tokio::test]
async fn list_with_empty_database_returns_empty_page() {
    // Arrange: Create empty database
    let db = inmem_db().await;
    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    // Create any context (even allow-all would return empty)
    let ctx = ctx_deny_all();

    // Act: List users from empty database
    let query = ODataQuery::default();
    let result = service.list_users_page(&ctx, &query).await;

    // Assert: Should succeed with empty page
    assert!(result.is_ok());
    let page = result.unwrap();
    assert!(page.items.is_empty());
}
