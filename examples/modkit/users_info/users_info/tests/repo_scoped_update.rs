#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Test that repository update operations respect security scope.

mod support;

use modkit_db::secure::SecureConn;
use support::{ctx_allow_tenants, ctx_deny_all, inmem_db, seed_user};
use users_info::{
    domain::repo::UsersRepository, infra::storage::sea_orm_repo::SeaOrmUsersRepository,
};
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
    let repo = SeaOrmUsersRepository::new(sec);

    // Create context for a different tenant
    let ctx_deny = ctx_allow_tenants(&[tenant2]);

    // Act: Try to update user outside scope
    let mut updated = user.clone();
    updated.email = "hacker@example.com".to_owned();
    updated.updated_at = time::OffsetDateTime::now_utc();

    let result = repo.update(ctx_deny.scope(), updated).await;

    // Assert: Should fail (ScopeError -> anyhow::Error)
    assert!(
        result.is_err(),
        "Update should fail for out-of-scope tenant"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not accessible")
            || err_msg.contains("denied")
            || err_msg.contains("Secure update"),
        "Error should indicate access denial: {err_msg}"
    );
}

#[tokio::test]
async fn update_succeeds_within_scope() {
    // Arrange: Create database with a test user
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let user = seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);

    // Create context with access to this tenant
    let ctx_ok = ctx_allow_tenants(&[tenant_id]);

    // Act: Update user within scope
    let mut updated = user.clone();
    updated.email = "updated@example.com".to_owned();
    updated.updated_at = time::OffsetDateTime::now_utc();

    let result = repo.update(ctx_ok.scope(), updated).await;

    // Assert: Should succeed
    assert!(result.is_ok(), "Update should succeed for in-scope tenant");

    // Verify the update persisted
    let loaded = repo.find_by_id(ctx_ok.scope(), user_id).await.unwrap();
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().email, "updated@example.com");
}

#[tokio::test]
async fn update_with_deny_all_fails() {
    // Arrange: Create database with a test user
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let user = seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);

    // Create deny-all context
    let ctx = ctx_deny_all();

    // Act: Try to update with deny-all context
    let mut updated = user.clone();
    updated.email = "blocked@example.com".to_owned();

    let result = repo.update(ctx.scope(), updated).await;

    // Assert: Should fail
    assert!(result.is_err(), "Deny-all context should prevent updates");
}
