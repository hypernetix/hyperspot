//! Test that list operations respect deny-all security context.

mod support;

use modkit_db::secure::SecureConn;
use modkit_odata::ODataQuery;
use support::{ctx_deny_all, inmem_db, seed_user};
use users_info::{
    domain::repo::UsersRepository, infra::storage::sea_orm_repo::SeaOrmUsersRepository,
};
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
    let repo = SeaOrmUsersRepository::new(sec);

    // Create deny-all context
    let ctx = ctx_deny_all();

    // Act: List users with deny-all context
    let query = ODataQuery::default();
    let result = repo.list_users_page(&ctx, &query).await;

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
    let repo = SeaOrmUsersRepository::new(sec);

    // Create any context (even allow-all would return empty)
    let ctx = ctx_deny_all();

    // Act: List users from empty database
    let query = ODataQuery::default();
    let result = repo.list_users_page(&ctx, &query).await;

    // Assert: Should succeed with empty page
    assert!(result.is_ok());
    let page = result.unwrap();
    assert!(page.items.is_empty());
}
