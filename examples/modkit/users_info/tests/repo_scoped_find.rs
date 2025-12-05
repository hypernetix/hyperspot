#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Test that repository find operations respect security scope.

mod support;

use modkit_db::secure::SecureConn;
use support::{ctx_allow_tenants, ctx_deny_all, inmem_db, seed_user};
use users_info::{
    domain::repo::UsersRepository, infra::storage::sea_orm_repo::SeaOrmUsersRepository,
};
use uuid::Uuid;

#[tokio::test]
async fn find_by_id_respects_tenant_scope() {
    // Arrange: Create database with test users in different tenants
    let db = inmem_db().await;
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let user = seed_user(&db, user_id, tenant1, "test@example.com", "Test User").await;

    // Create SecureConn from the same database
    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);

    // Create contexts: one with access to tenant1, one with access to tenant2
    let ctx_ok = ctx_allow_tenants(&[tenant1]);
    let ctx_deny = ctx_allow_tenants(&[tenant2]); // Different tenant

    // Act & Assert: With access to correct tenant should find user
    let result_ok = repo.find_by_id(&ctx_ok, user.id).await.unwrap();
    assert!(result_ok.is_some());
    assert_eq!(result_ok.unwrap().email, "test@example.com");

    // Act & Assert: Without access to tenant should return None
    let result_deny = repo.find_by_id(&ctx_deny, user.id).await.unwrap();
    assert!(
        result_deny.is_none(),
        "Should not find user outside tenant scope"
    );
}

#[tokio::test]
async fn find_by_id_with_deny_all_returns_none() {
    // Arrange: Create database with a test user
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let user = seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    // Create SecureConn from the same database
    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);

    // Create deny-all context
    let ctx = ctx_deny_all();

    // Act: Try to find user
    let result = repo.find_by_id(&ctx, user.id).await.unwrap();

    // Assert: Should return None (empty scope = no access)
    assert!(
        result.is_none(),
        "Deny-all context should not return any data"
    );
}

#[tokio::test]
async fn email_exists_respects_tenant_scope() {
    // Arrange: Create database with users in different tenants
    let db = inmem_db().await;
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    seed_user(&db, user_id, tenant1, "test@example.com", "Test User").await;

    // Create SecureConn from the same database
    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);

    // Create contexts
    let ctx_ok = ctx_allow_tenants(&[tenant1]);
    let ctx_deny = ctx_allow_tenants(&[tenant2]); // Different tenant

    // Act & Assert: With access to correct tenant should find email
    let exists_ok = repo
        .email_exists(&ctx_ok, "test@example.com")
        .await
        .unwrap();
    assert!(exists_ok, "Email should exist in accessible tenant scope");

    // Act & Assert: Without access to tenant should not find email
    let exists_deny = repo
        .email_exists(&ctx_deny, "test@example.com")
        .await
        .unwrap();
    assert!(
        !exists_deny,
        "Email should not be found in different tenant"
    );
}
