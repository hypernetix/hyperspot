#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Test that repository operations work with multi-tenant access.
//!
//! These tests verify that explicit tenant lists work correctly for accessing
//! users across multiple tenants. There is no "root bypass" - all access
//! requires explicit tenant IDs.

mod support;

use modkit_db::secure::SecureConn;
use modkit_odata::ODataQuery;
use support::{ctx_allow_tenants, ctx_deny_all, inmem_db, seed_user};
use users_info::{
    domain::repo::UsersRepository, infra::storage::sea_orm_repo::SeaOrmUsersRepository,
};
use uuid::Uuid;

#[tokio::test]
async fn multi_tenant_scope_can_access_listed_tenants() {
    // Arrange: Create database with users in different tenants
    let db = inmem_db().await;
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();

    seed_user(&db, user1_id, tenant1, "user1@example.com", "User 1").await;
    seed_user(&db, user2_id, tenant2, "user2@example.com", "User 2").await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);

    // Create context with access to both tenants
    let ctx = ctx_allow_tenants(&[tenant1, tenant2]);

    // Act: List users with multi-tenant context
    let query = ODataQuery::default();
    let result = repo.list_users_page(ctx.scope(), &query).await;

    // Assert: Should return users from both tenants
    assert!(result.is_ok(), "Multi-tenant scope query should succeed");
    let page = result.unwrap();
    assert_eq!(
        page.items.len(),
        2,
        "Multi-tenant scope should access all listed tenants"
    );
}

#[tokio::test]
async fn multi_tenant_scope_can_find_users_in_listed_tenants() {
    // Arrange: Create database with users in different tenants
    let db = inmem_db().await;
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();

    let user1 = seed_user(&db, user1_id, tenant1, "user1@example.com", "User 1").await;
    let user2 = seed_user(&db, user2_id, tenant2, "user2@example.com", "User 2").await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);

    // Create context with access to both tenants
    let ctx_multi = ctx_allow_tenants(&[tenant1, tenant2]);

    // Act & Assert: Multi-tenant context can find users in tenant1
    let result1 = repo.find_by_id(ctx_multi.scope(), user1.id).await.unwrap();
    assert!(
        result1.is_some(),
        "Multi-tenant scope should find user in tenant1"
    );
    assert_eq!(result1.unwrap().email, "user1@example.com");

    // Act & Assert: Multi-tenant context can find users in tenant2
    let result2 = repo.find_by_id(ctx_multi.scope(), user2.id).await.unwrap();
    assert!(
        result2.is_some(),
        "Multi-tenant scope should find user in tenant2"
    );
    assert_eq!(result2.unwrap().email, "user2@example.com");
}

#[tokio::test]
async fn multi_tenant_vs_single_tenant_scope() {
    // Arrange: Create database with users in different tenants
    let db = inmem_db().await;
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();

    seed_user(&db, user1_id, tenant1, "user1@example.com", "User 1").await;
    seed_user(&db, user2_id, tenant2, "user2@example.com", "User 2").await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);

    // Create different contexts
    let ctx_multi = ctx_allow_tenants(&[tenant1, tenant2]);
    let ctx_single = ctx_allow_tenants(&[tenant1]);

    // Act: List with multi-tenant context
    let query = ODataQuery::default();
    let multi_result = repo
        .list_users_page(ctx_multi.scope(), &query)
        .await
        .unwrap();

    // Act: List with single-tenant context
    let single_result = repo
        .list_users_page(ctx_single.scope(), &query)
        .await
        .unwrap();

    // Assert: Multi-tenant context sees users in both tenants
    assert_eq!(
        multi_result.items.len(),
        2,
        "Multi-tenant scope should see users in both tenants"
    );

    // Assert: Single-tenant context sees only its tenant
    assert_eq!(
        single_result.items.len(),
        1,
        "Single-tenant scope should see only its tenant"
    );
    assert_eq!(single_result.items[0].tenant_id, tenant1);
}

#[tokio::test]
async fn empty_scope_denies_all_access() {
    // Arrange: Create database with a test user
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);

    // Create deny-all context (empty scope)
    let ctx_empty = ctx_deny_all();

    // Verify scope properties
    assert!(ctx_empty.scope().is_empty(), "Should be empty scope");
    assert!(ctx_empty.is_denied(), "Empty scope should be denied");

    // Act: List with empty context
    let query = ODataQuery::default();
    let result = repo.list_users_page(ctx_empty.scope(), &query).await;

    // Assert: Should succeed but return no data (deny-all)
    assert!(result.is_ok(), "Empty scope query should succeed");
    let page = result.unwrap();
    assert_eq!(page.items.len(), 0, "Empty scope should return no data");
}

#[tokio::test]
async fn cannot_access_unlisted_tenant() {
    // Arrange: Create database with users in different tenants
    let db = inmem_db().await;
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();
    let tenant3 = Uuid::new_v4(); // Not in context
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();
    let user3_id = Uuid::new_v4();

    seed_user(&db, user1_id, tenant1, "user1@example.com", "User 1").await;
    seed_user(&db, user2_id, tenant2, "user2@example.com", "User 2").await;
    let user3 = seed_user(&db, user3_id, tenant3, "user3@example.com", "User 3").await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);

    // Create context with access to only tenant1 and tenant2
    let ctx = ctx_allow_tenants(&[tenant1, tenant2]);

    // Act: List users
    let query = ODataQuery::default();
    let result = repo.list_users_page(ctx.scope(), &query).await.unwrap();

    // Assert: Should only return users from listed tenants
    assert_eq!(
        result.items.len(),
        2,
        "Should only see users in listed tenants"
    );
    assert!(
        result.items.iter().all(|u| u.tenant_id != tenant3),
        "Should not include users from unlisted tenant"
    );

    // Act: Try to find user in unlisted tenant
    let find_result = repo.find_by_id(ctx.scope(), user3.id).await.unwrap();

    // Assert: Should not find user in unlisted tenant
    assert!(
        find_result.is_none(),
        "Should not find user in unlisted tenant"
    );
}
