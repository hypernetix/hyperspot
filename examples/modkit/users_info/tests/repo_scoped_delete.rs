#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Test that repository delete operations respect security scope.

mod support;

use modkit_db::secure::SecureConn;
use support::{ctx_allow_tenants, ctx_deny_all, inmem_db, seed_user};
use users_info::{
    domain::repo::UsersRepository, infra::storage::sea_orm_repo::SeaOrmUsersRepository,
};
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
    let repo = SeaOrmUsersRepository::new(sec);

    // Create context for a different tenant
    let ctx_deny = ctx_allow_tenants(&[tenant2]);

    // Act: Try to delete user outside scope
    let result = repo.delete(&ctx_deny, user_id).await;

    // Assert: Should return Ok(false) - no rows deleted
    assert!(result.is_ok());
    let deleted = result.unwrap();
    assert!(!deleted, "Should not delete user outside tenant scope");
}

#[tokio::test]
async fn delete_succeeds_within_scope() {
    // Arrange: Create database with a test user
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);

    // Create context with access to this tenant
    let ctx_ok = ctx_allow_tenants(&[tenant_id]);

    // Act: Delete user within scope
    let result = repo.delete(&ctx_ok, user_id).await;

    // Assert: Should succeed
    assert!(result.is_ok());
    let deleted = result.unwrap();
    assert!(deleted, "Should successfully delete user in scope");

    // Verify the user is gone
    let loaded = repo.find_by_id(&ctx_ok, user_id).await.unwrap();
    assert!(loaded.is_none(), "User should be deleted");
}

#[tokio::test]
async fn delete_with_deny_all_returns_false() {
    // Arrange: Create database with a test user
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);

    // Create deny-all context
    let ctx = ctx_deny_all();

    // Act: Try to delete with deny-all context
    let result = repo.delete(&ctx, user_id).await;

    // Assert: Should return Ok(false)
    assert!(result.is_ok());
    let deleted = result.unwrap();
    assert!(!deleted, "Deny-all context should not delete any data");
}
