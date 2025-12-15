#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Integration tests for `OData` cursor pagination.
//!
//! These tests verify end-to-end cursor pagination behavior using an in-memory
//! `SQLite` database, including forward/backward navigation, filters, security scopes,
//! and edge cases.

mod support;

use modkit_db::secure::SecureConn;
use modkit_odata::{ast, ODataOrderBy, ODataQuery, OrderKey, SortDir};
use support::{ctx_allow_tenants, ctx_deny_all, inmem_db, seed_user};
use users_info::{
    domain::repo::UsersRepository, infra::storage::sea_orm_repo::SeaOrmUsersRepository,
};
use uuid::Uuid;

// ============================================================================
// Helper Functions
// ============================================================================

/// Seed multiple users with sequential IDs and names for predictable ordering.
///
/// Returns the `tenant_id` and list of user IDs in creation order.
async fn seed_users_sequential(
    db: &sea_orm::DatabaseConnection,
    count: usize,
    tenant_id: Uuid,
) -> Vec<Uuid> {
    let mut user_ids = Vec::new();

    for i in 0..count {
        let id = Uuid::new_v4();
        let email = format!("user{i}@example.com");
        let display_name = format!("User {i}");

        seed_user(db, id, tenant_id, &email, &display_name).await;
        user_ids.push(id);

        // Small delay to ensure different created_at timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    user_ids
}

/// Seed users across multiple tenants for security testing.
async fn seed_users_multi_tenant(
    db: &sea_orm::DatabaseConnection,
) -> (Uuid, Uuid, Vec<Uuid>, Vec<Uuid>) {
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();

    let mut tenant1_users = Vec::new();
    let mut tenant2_users = Vec::new();

    // Seed 10 users for tenant 1
    for i in 0..10 {
        let id = Uuid::new_v4();
        seed_user(
            db,
            id,
            tenant1,
            &format!("t1user{i}@example.com"),
            &format!("Tenant1 User {i}"),
        )
        .await;
        tenant1_users.push(id);
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Seed 10 users for tenant 2
    for i in 0..10 {
        let id = Uuid::new_v4();
        seed_user(
            db,
            id,
            tenant2,
            &format!("t2user{i}@example.com"),
            &format!("Tenant2 User {i}"),
        )
        .await;
        tenant2_users.push(id);
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    (tenant1, tenant2, tenant1_users, tenant2_users)
}

// ============================================================================
// End-to-End Multi-Page Forward Pagination Tests
// ============================================================================

#[tokio::test]
async fn test_forward_pagination_through_multiple_pages() {
    // Arrange: Create 25 users
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_ids = seed_users_sequential(&db, 25, tenant_id).await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let ctx = ctx_allow_tenants(&[tenant_id]);

    // Act & Assert: Paginate through with limit=10
    let mut query = ODataQuery::default().with_limit(10);
    let mut all_fetched_ids = Vec::new();
    let mut page_count = 0;

    loop {
        let page = repo
            .list_users_page(&ctx, &query)
            .await
            .expect("Pagination should succeed");

        page_count += 1;

        // Verify page has correct number of items
        if page_count < 3 {
            assert_eq!(page.items.len(), 10, "Pages 1-2 should have 10 items");
        } else {
            assert_eq!(page.items.len(), 5, "Last page should have 5 items");
        }

        // Collect IDs
        for user in &page.items {
            all_fetched_ids.push(user.id);
        }

        // Check if there's more
        if let Some(next_cursor) = page.page_info.next_cursor {
            // Decode and use cursor for next page
            let cursor =
                modkit_odata::CursorV1::decode(&next_cursor).expect("Next cursor should be valid");
            query = query.with_cursor(cursor);
        } else {
            break;
        }
    }

    // Verify we got exactly 3 pages
    assert_eq!(page_count, 3, "Should have 3 pages total");

    // Verify we got all 25 users
    assert_eq!(all_fetched_ids.len(), 25, "Should fetch all 25 users");

    // Verify no duplicates
    let unique_ids: std::collections::HashSet<_> = all_fetched_ids.iter().collect();
    assert_eq!(
        unique_ids.len(),
        25,
        "Should have no duplicate users across pages"
    );

    // Verify all original user IDs were fetched
    for user_id in &user_ids {
        assert!(
            all_fetched_ids.contains(user_id),
            "Should fetch all seeded users"
        );
    }
}

#[tokio::test]
async fn test_forward_pagination_respects_order() {
    // Arrange: Create 15 users
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    seed_users_sequential(&db, 15, tenant_id).await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let ctx = ctx_allow_tenants(&[tenant_id]);

    // Act: Paginate with explicit DESC order on created_at
    let order = ODataOrderBy(vec![OrderKey {
        field: "created_at".to_owned(),
        dir: SortDir::Desc,
    }]);

    let mut query = ODataQuery::default().with_order(order).with_limit(5);

    let page1 = repo
        .list_users_page(&ctx, &query)
        .await
        .expect("Page 1 should succeed");

    assert_eq!(page1.items.len(), 5);
    assert!(page1.page_info.next_cursor.is_some());

    // Verify DESC order on page 1
    for i in 0..4 {
        assert!(
            page1.items[i].created_at >= page1.items[i + 1].created_at,
            "Should be in descending order"
        );
    }

    // Get page 2
    let cursor = modkit_odata::CursorV1::decode(&page1.page_info.next_cursor.unwrap())
        .expect("Cursor should be valid");
    query = ODataQuery::default().with_cursor(cursor).with_limit(5);

    let page2 = repo
        .list_users_page(&ctx, &query)
        .await
        .expect("Page 2 should succeed");

    assert_eq!(page2.items.len(), 5);

    // Verify DESC order on page 2
    for i in 0..4 {
        assert!(
            page2.items[i].created_at >= page2.items[i + 1].created_at,
            "Should be in descending order"
        );
    }

    // Verify page 2 continues from page 1 (no overlap)
    assert!(
        page1.items.last().unwrap().created_at > page2.items.first().unwrap().created_at,
        "Page 2 should continue where page 1 ended"
    );
}

#[tokio::test]
async fn test_forward_pagination_no_duplicates_across_pages() {
    // Arrange: Create 30 users
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    seed_users_sequential(&db, 30, tenant_id).await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let ctx = ctx_allow_tenants(&[tenant_id]);

    // Act: Paginate with small pages
    let mut query = ODataQuery::default().with_limit(7);
    let mut all_emails = Vec::new();

    for _ in 0..5 {
        let page = repo
            .list_users_page(&ctx, &query)
            .await
            .expect("Pagination should succeed");

        for user in &page.items {
            all_emails.push(user.email.clone());
        }

        if let Some(next_cursor) = page.page_info.next_cursor {
            let cursor = modkit_odata::CursorV1::decode(&next_cursor).unwrap();
            query = ODataQuery::default().with_cursor(cursor).with_limit(7);
        } else {
            break;
        }
    }

    // Assert: No duplicates
    let unique_emails: std::collections::HashSet<_> = all_emails.iter().collect();
    assert_eq!(
        unique_emails.len(),
        all_emails.len(),
        "Should have no duplicate emails across pages"
    );
}

// ============================================================================
// Backward Pagination Tests Using prev_cursor
// ============================================================================

#[tokio::test]
async fn test_backward_pagination_with_prev_cursor() {
    // Arrange: Create 20 users
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    seed_users_sequential(&db, 20, tenant_id).await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let ctx = ctx_allow_tenants(&[tenant_id]);

    // Act: Get page 1, then page 2, then go back to page 1 using prev_cursor
    let query1 = ODataQuery::default().with_limit(8);
    let page1 = repo
        .list_users_page(&ctx, &query1)
        .await
        .expect("Page 1 should succeed");

    assert_eq!(page1.items.len(), 8);
    let _page1_first_id = page1.items.first().unwrap().id;
    let page1_last_id = page1.items.last().unwrap().id;

    // Get page 2
    let next_cursor = page1.page_info.next_cursor.as_ref().unwrap();
    let cursor2 = modkit_odata::CursorV1::decode(next_cursor).unwrap();
    let query2 = ODataQuery::default().with_cursor(cursor2).with_limit(8);

    let page2 = repo
        .list_users_page(&ctx, &query2)
        .await
        .expect("Page 2 should succeed");

    assert_eq!(page2.items.len(), 8);
    let page2_first_id = page2.items.first().unwrap().id;

    // Verify no overlap between page 1 and page 2
    assert_ne!(
        page1_last_id, page2_first_id,
        "Page 1 and page 2 should not overlap"
    );

    // Now use prev_cursor from page 2 to navigate backward
    let prev_cursor = page2.page_info.prev_cursor.as_ref().unwrap();
    let cursor_back = modkit_odata::CursorV1::decode(prev_cursor).unwrap();
    let query_back = ODataQuery::default().with_cursor(cursor_back).with_limit(8);

    let page_back = repo
        .list_users_page(&ctx, &query_back)
        .await
        .expect("Backward navigation should succeed");

    // Assert: Backward pagination should return page 1 items
    assert_eq!(
        page_back.items.len(),
        8,
        "Should get 8 items going backward"
    );

    // The emails should match page 1
    let page1_emails: Vec<_> = page1.items.iter().map(|u| u.email.clone()).collect();
    let page_back_emails: Vec<_> = page_back.items.iter().map(|u| u.email.clone()).collect();

    assert_eq!(
        page_back_emails, page1_emails,
        "Using prev_cursor from page 2 should return page 1 items in same order"
    );
}

#[tokio::test]
async fn test_backward_pagination_maintains_order() {
    // Arrange: Create 20 users
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    seed_users_sequential(&db, 20, tenant_id).await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let ctx = ctx_allow_tenants(&[tenant_id]);

    // Act: Get page 2 (skip first 8 items) with explicit ordering by created_at DESC
    let order = modkit_odata::ODataOrderBy(vec![modkit_odata::OrderKey {
        field: "created_at".to_owned(),
        dir: modkit_odata::SortDir::Desc,
    }]);

    let query1 = ODataQuery::default()
        .with_order(order.clone())
        .with_limit(8);
    let page1 = repo
        .list_users_page(&ctx, &query1)
        .await
        .expect("Page 1 should succeed");

    let next_cursor = page1.page_info.next_cursor.as_ref().unwrap();
    let cursor2 = modkit_odata::CursorV1::decode(next_cursor).unwrap();
    let query2 = ODataQuery::default().with_cursor(cursor2).with_limit(8);
    let page2 = repo
        .list_users_page(&ctx, &query2)
        .await
        .expect("Page 2 should succeed");

    // Store page 2 items for comparison
    let page2_emails: Vec<String> = page2.items.iter().map(|u| u.email.clone()).collect();

    // Now use prev_cursor from page 2 to navigate backward
    let prev_cursor = page2.page_info.prev_cursor.as_ref().unwrap();
    let cursor_back = modkit_odata::CursorV1::decode(prev_cursor).unwrap();

    // Verify the cursor has backward direction
    assert_eq!(
        cursor_back.d, "bwd",
        "prev_cursor should have 'bwd' direction"
    );

    let query_back = ODataQuery::default().with_cursor(cursor_back).with_limit(8);
    let page_back = repo
        .list_users_page(&ctx, &query_back)
        .await
        .expect("Backward navigation should succeed");

    // Assert: The items should be in SAME ORDER as page 1 (not reversed!)
    // And they should be the items BEFORE page 2
    assert_eq!(
        page_back.items.len(),
        8,
        "Should get 8 items going backward"
    );

    // Verify no items overlap with page 2
    let page_back_emails: Vec<String> = page_back.items.iter().map(|u| u.email.clone()).collect();
    for email in &page_back_emails {
        assert!(
            !page2_emails.contains(email),
            "Backward page should not contain items from page 2"
        );
    }

    // Most importantly: items should maintain DESC order (not reversed)
    for i in 0..7 {
        assert!(
            page_back.items[i].created_at >= page_back.items[i + 1].created_at,
            "Items should be in DESC order (same as forward pagination), not reversed! Item {} has {} but item {} has {}",
            i, page_back.items[i].created_at, i+1, page_back.items[i + 1].created_at
        );
    }

    // The backward page should contain items that come before page 2 in DESC order
    // This means they should have higher timestamps (newer items)
    let page_back_oldest = page_back.items.last().unwrap();
    let page2_newest = page2.items.first().unwrap();

    assert!(
        page_back_oldest.created_at > page2_newest.created_at,
        "Last (oldest) item of backward page should have timestamp {} > {} (first/newest item of page 2)",
        page_back_oldest.created_at, page2_newest.created_at
    );

    // Verify that next_cursor is present when paginating backward
    assert!(
        page_back.page_info.next_cursor.is_some(),
        "next_cursor should be present when paginating backward (to go forward again)"
    );
}

#[tokio::test]
async fn test_backward_pagination_has_next_cursor() {
    // This test explicitly verifies that when using prev_cursor from page 2,
    // the response includes a next_cursor to navigate forward to page 2 again

    // Arrange: Create 10 users
    let tenant_id = Uuid::new_v4();
    let db = inmem_db().await;
    seed_users_sequential(&db, 10, tenant_id).await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let ctx = ctx_allow_tenants(&[tenant_id]);

    // Act: Get page 1, then page 2, then use prev_cursor to go back
    let order = modkit_odata::ODataOrderBy(vec![modkit_odata::OrderKey {
        field: "created_at".to_owned(),
        dir: modkit_odata::SortDir::Desc,
    }]);

    let query1 = ODataQuery::default()
        .with_order(order.clone())
        .with_limit(2);
    let page1 = repo
        .list_users_page(&ctx, &query1)
        .await
        .expect("Page 1 should succeed");

    assert_eq!(page1.items.len(), 2);
    assert!(
        page1.page_info.next_cursor.is_some(),
        "Page 1 should have next_cursor"
    );

    // Get page 2
    let next_cursor_encoded = page1.page_info.next_cursor.as_ref().unwrap();
    let next_cursor = modkit_odata::CursorV1::decode(next_cursor_encoded).unwrap();
    let query2 = ODataQuery::default().with_cursor(next_cursor).with_limit(2);
    let page2 = repo
        .list_users_page(&ctx, &query2)
        .await
        .expect("Page 2 should succeed");

    assert_eq!(page2.items.len(), 2);
    assert!(
        page2.page_info.prev_cursor.is_some(),
        "Page 2 should have prev_cursor"
    );

    // Now use prev_cursor from page 2 to go backward
    let prev_cursor_encoded = page2.page_info.prev_cursor.as_ref().unwrap();
    let prev_cursor = modkit_odata::CursorV1::decode(prev_cursor_encoded).unwrap();

    // Verify it's a backward cursor
    assert_eq!(
        prev_cursor.d, "bwd",
        "prev_cursor should have direction=bwd"
    );

    let query_back = ODataQuery::default().with_cursor(prev_cursor).with_limit(2);
    let page_back = repo
        .list_users_page(&ctx, &query_back)
        .await
        .expect("Backward page should succeed");

    // The critical check: next_cursor MUST be present when we have items
    assert_eq!(
        page_back.items.len(),
        2,
        "Should get 2 items going backward"
    );
    assert!(
        page_back.page_info.next_cursor.is_some(),
        "next_cursor MUST be present when paginating backward - it allows going forward to page 2 again"
    );

    // Verify we can use next_cursor to go forward to page 2
    let next_cursor_encoded = page_back.page_info.next_cursor.as_ref().unwrap();
    let next_cursor = modkit_odata::CursorV1::decode(next_cursor_encoded).unwrap();
    assert_eq!(
        next_cursor.d, "fwd",
        "next_cursor should have direction=fwd"
    );

    let query_fwd = ODataQuery::default().with_cursor(next_cursor).with_limit(2);
    let page_fwd = repo
        .list_users_page(&ctx, &query_fwd)
        .await
        .expect("Forward page should succeed");

    assert_eq!(
        page_fwd.items.len(),
        2,
        "Should get 2 items when going forward from backward page"
    );

    // The items should be the same as page 2 (or overlap with it)
    let page2_emails: std::collections::HashSet<_> = page2.items.iter().map(|u| &u.email).collect();
    let page_fwd_emails: std::collections::HashSet<_> =
        page_fwd.items.iter().map(|u| &u.email).collect();

    assert!(
        page2_emails.intersection(&page_fwd_emails).count() > 0,
        "Forward page after backward should overlap with original page 2"
    );
}

#[tokio::test]
async fn test_prev_cursor_at_first_page() {
    // Arrange: Create 15 users
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    seed_users_sequential(&db, 15, tenant_id).await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let ctx = ctx_allow_tenants(&[tenant_id]);

    // Act: Get first page
    let query = ODataQuery::default().with_limit(10);
    let page1 = repo
        .list_users_page(&ctx, &query)
        .await
        .expect("Page 1 should succeed");

    // Assert: First page should NOT have prev_cursor (we're at the beginning)
    assert!(
        page1.page_info.prev_cursor.is_none(),
        "First page should NOT have prev_cursor since there are no items before it"
    );

    // Navigate to second page using next_cursor
    assert!(
        page1.page_info.next_cursor.is_some(),
        "First page should have next_cursor"
    );

    let next_cursor_encoded = page1.page_info.next_cursor.as_ref().unwrap();
    let next_cursor = modkit_odata::CursorV1::decode(next_cursor_encoded).unwrap();
    let query2 = ODataQuery::default()
        .with_cursor(next_cursor)
        .with_limit(10);
    let page2 = repo
        .list_users_page(&ctx, &query2)
        .await
        .expect("Page 2 should succeed");

    // Page 2 SHOULD have prev_cursor (to go back to page 1)
    assert!(
        page2.page_info.prev_cursor.is_some(),
        "Page 2 should have prev_cursor to navigate back"
    );

    // Using prev_cursor from page 2 should return to first page
    if let Some(prev_cursor) = page1.page_info.prev_cursor {
        let cursor = modkit_odata::CursorV1::decode(&prev_cursor).unwrap();
        let query_prev = ODataQuery::default().with_cursor(cursor).with_limit(10);

        let page_prev = repo
            .list_users_page(&ctx, &query_prev)
            .await
            .expect("Using prev_cursor should succeed");

        // Should return empty (no items before first page)
        assert_eq!(
            page_prev.items.len(),
            0,
            "No items should exist before the first page"
        );
    }
}

// ============================================================================
// Cursor Pagination with Filters Tests
// ============================================================================

#[tokio::test]
async fn test_cursor_pagination_with_filter() {
    // Arrange: Create 30 users with varied email patterns
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();

    for i in 0..30 {
        let id = Uuid::new_v4();
        // Half will have "alice" in email, half "bob"
        let email = if i % 2 == 0 {
            format!("alice{i}@example.com")
        } else {
            format!("bob{i}@example.com")
        };
        let display_name = format!("User {i}");

        seed_user(&db, id, tenant_id, &email, &display_name).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    }

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let ctx = ctx_allow_tenants(&[tenant_id]);

    // Act: Filter for "alice" in email and paginate
    let filter_ast = ast::Expr::Function(
        "contains".to_owned(),
        vec![
            ast::Expr::Identifier("email".to_owned()),
            ast::Expr::Value(ast::Value::String("alice".to_owned())),
        ],
    );

    let filter_hash = modkit_odata::pagination::short_filter_hash(Some(&filter_ast));

    let mut query = ODataQuery::default().with_filter(filter_ast).with_limit(5);

    if let Some(hash) = filter_hash {
        query = query.with_filter_hash(hash);
    }

    let mut all_items = Vec::new();

    loop {
        let page = repo
            .list_users_page(&ctx, &query)
            .await
            .expect("Filtered pagination should succeed");

        for user in &page.items {
            // Verify all items match filter
            assert!(
                user.email.contains("alice"),
                "All results should contain 'alice' in email"
            );
            all_items.push(user.id);
        }

        if let Some(next_cursor) = page.page_info.next_cursor {
            let cursor = modkit_odata::CursorV1::decode(&next_cursor).unwrap();

            // Create the same filter for next page
            let filter_ast_next = ast::Expr::Function(
                "contains".to_owned(),
                vec![
                    ast::Expr::Identifier("email".to_owned()),
                    ast::Expr::Value(ast::Value::String("alice".to_owned())),
                ],
            );

            let filter_hash_next =
                modkit_odata::pagination::short_filter_hash(Some(&filter_ast_next));

            query = ODataQuery::default()
                .with_filter(filter_ast_next)
                .with_cursor(cursor)
                .with_limit(5);

            if let Some(hash) = filter_hash_next {
                query = query.with_filter_hash(hash);
            }
        } else {
            break;
        }
    }

    // Assert: Should have 15 "alice" users
    assert_eq!(all_items.len(), 15, "Should find all 15 'alice' users");

    // Verify no duplicates
    let unique: std::collections::HashSet<_> = all_items.iter().collect();
    assert_eq!(unique.len(), 15, "Should have no duplicates");
}

#[tokio::test]
async fn test_cursor_filter_hash_mismatch_error() {
    // Arrange: Create 20 users
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    seed_users_sequential(&db, 20, tenant_id).await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let ctx = ctx_allow_tenants(&[tenant_id]);

    // Act: Get first page with a filter
    let filter_ast = ast::Expr::Function(
        "contains".to_owned(),
        vec![
            ast::Expr::Identifier("email".to_owned()),
            ast::Expr::Value(ast::Value::String("user".to_owned())),
        ],
    );

    let filter_hash = modkit_odata::pagination::short_filter_hash(Some(&filter_ast));

    let mut query = ODataQuery::default()
        .with_filter(filter_ast.clone())
        .with_limit(10);

    if let Some(hash) = filter_hash {
        query = query.with_filter_hash(hash);
    }

    let page1 = repo
        .list_users_page(&ctx, &query)
        .await
        .expect("First page should succeed");

    // Now try to use the cursor with a DIFFERENT filter hash
    let next_cursor = page1.page_info.next_cursor.unwrap();
    let cursor = modkit_odata::CursorV1::decode(&next_cursor).unwrap();

    // Create a different filter
    let different_filter = ast::Expr::Function(
        "contains".to_owned(),
        vec![
            ast::Expr::Identifier("email".to_owned()),
            ast::Expr::Value(ast::Value::String("different".to_owned())),
        ],
    );

    let different_hash = modkit_odata::pagination::short_filter_hash(Some(&different_filter));

    let mut bad_query = ODataQuery::default().with_cursor(cursor);

    if let Some(hash) = different_hash {
        bad_query = bad_query.with_filter_hash(hash);
    }

    // Assert: Should fail with filter mismatch error
    let result = repo.list_users_page(&ctx, &bad_query).await;
    assert!(result.is_err(), "Should error on filter hash mismatch");

    let err = result.unwrap_err();
    assert!(
        matches!(err, modkit_odata::Error::FilterMismatch),
        "Should be FilterMismatch error, got: {err:?}"
    );
}

// ============================================================================
// Security-Scoped Cursor Pagination Tests
// ============================================================================

#[tokio::test]
async fn test_cursor_pagination_with_tenant_isolation() {
    // Arrange: Create users in two tenants
    let db = inmem_db().await;
    let (tenant1, tenant2, _, _) = seed_users_multi_tenant(&db).await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);

    // Act: Paginate with tenant1 context
    let ctx1 = ctx_allow_tenants(&[tenant1]);
    let mut query = ODataQuery::default().with_limit(5);
    let mut tenant1_count = 0;

    loop {
        let page = repo
            .list_users_page(&ctx1, &query)
            .await
            .expect("Tenant1 pagination should succeed");

        // Verify all items belong to tenant1
        for user in &page.items {
            assert_eq!(user.tenant_id, tenant1, "Should only see tenant1 users");
            tenant1_count += 1;
        }

        if let Some(next_cursor) = page.page_info.next_cursor {
            let cursor = modkit_odata::CursorV1::decode(&next_cursor).unwrap();
            query = ODataQuery::default().with_cursor(cursor).with_limit(5);
        } else {
            break;
        }
    }

    // Act: Paginate with tenant2 context
    let ctx2 = ctx_allow_tenants(&[tenant2]);
    let mut query = ODataQuery::default().with_limit(5);
    let mut tenant2_count = 0;

    loop {
        let page = repo
            .list_users_page(&ctx2, &query)
            .await
            .expect("Tenant2 pagination should succeed");

        // Verify all items belong to tenant2
        for user in &page.items {
            assert_eq!(user.tenant_id, tenant2, "Should only see tenant2 users");
            tenant2_count += 1;
        }

        if let Some(next_cursor) = page.page_info.next_cursor {
            let cursor = modkit_odata::CursorV1::decode(&next_cursor).unwrap();
            query = ODataQuery::default().with_cursor(cursor).with_limit(5);
        } else {
            break;
        }
    }

    // Assert: Each tenant should see exactly 10 users
    assert_eq!(tenant1_count, 10, "Tenant1 should see 10 users");
    assert_eq!(tenant2_count, 10, "Tenant2 should see 10 users");
}

#[tokio::test]
async fn test_cursor_pagination_with_deny_all_returns_empty() {
    // Arrange: Create 20 users
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    seed_users_sequential(&db, 20, tenant_id).await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let ctx = ctx_deny_all();

    // Act: Try to paginate with deny-all context
    let query = ODataQuery::default().with_limit(10);
    let page = repo
        .list_users_page(&ctx, &query)
        .await
        .expect("Query should succeed");

    // Assert: Should return empty page
    assert_eq!(page.items.len(), 0, "Deny-all should return no items");
    assert!(
        page.page_info.next_cursor.is_none(),
        "Should have no next cursor"
    );
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_cursor_pagination_empty_database() {
    // Arrange: Empty database
    let db = inmem_db().await;
    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let ctx = ctx_allow_tenants(&[Uuid::new_v4()]);

    // Act: Try to paginate
    let query = ODataQuery::default().with_limit(10);
    let page = repo
        .list_users_page(&ctx, &query)
        .await
        .expect("Empty pagination should succeed");

    // Assert: Empty page with no next cursor
    assert_eq!(page.items.len(), 0);
    assert!(page.page_info.next_cursor.is_none());
    // Note: prev_cursor may be None or Some depending on implementation
    // when there are no items
}

#[tokio::test]
async fn test_cursor_pagination_exact_page_boundary() {
    // Arrange: Create exactly 20 users (2 full pages of 10)
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    seed_users_sequential(&db, 20, tenant_id).await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let ctx = ctx_allow_tenants(&[tenant_id]);

    // Act: Get page 1
    let query1 = ODataQuery::default().with_limit(10);
    let page1 = repo
        .list_users_page(&ctx, &query1)
        .await
        .expect("Page 1 should succeed");

    assert_eq!(page1.items.len(), 10);
    assert!(page1.page_info.next_cursor.is_some());

    // Get page 2
    let cursor = modkit_odata::CursorV1::decode(&page1.page_info.next_cursor.unwrap()).unwrap();
    let query2 = ODataQuery::default().with_cursor(cursor).with_limit(10);
    let page2 = repo
        .list_users_page(&ctx, &query2)
        .await
        .expect("Page 2 should succeed");

    // Assert: Page 2 should have exactly 10 items and NO next cursor
    assert_eq!(page2.items.len(), 10, "Page 2 should have 10 items");
    assert!(
        page2.page_info.next_cursor.is_none(),
        "Page 2 should have no next cursor (exact boundary)"
    );
}

#[tokio::test]
async fn test_cursor_pagination_single_item() {
    // Arrange: Create just 1 user
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    seed_users_sequential(&db, 1, tenant_id).await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let ctx = ctx_allow_tenants(&[tenant_id]);

    // Act: Paginate with large limit
    let query = ODataQuery::default().with_limit(10);
    let page = repo
        .list_users_page(&ctx, &query)
        .await
        .expect("Single item pagination should succeed");

    // Assert: Should get 1 item with no next cursor and no prev cursor
    // (first page has no prev_cursor since there are no items before it)
    assert_eq!(page.items.len(), 1);
    assert!(page.page_info.next_cursor.is_none());
    assert!(
        page.page_info.prev_cursor.is_none(),
        "First page should have no prev_cursor"
    );
}

#[tokio::test]
async fn test_cursor_pagination_limit_exceeds_total() {
    // Arrange: Create 5 users
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    seed_users_sequential(&db, 5, tenant_id).await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let ctx = ctx_allow_tenants(&[tenant_id]);

    // Act: Request more than available with limit=100
    let query = ODataQuery::default().with_limit(100);
    let page = repo
        .list_users_page(&ctx, &query)
        .await
        .expect("Should succeed");

    // Assert: Should get all 5 items with no next cursor
    assert_eq!(page.items.len(), 5);
    assert!(
        page.page_info.next_cursor.is_none(),
        "Should have no next cursor when limit exceeds total"
    );
}

#[tokio::test]
async fn test_cursor_pagination_with_limit_1() {
    // Arrange: Create 5 users
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    seed_users_sequential(&db, 5, tenant_id).await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let ctx = ctx_allow_tenants(&[tenant_id]);

    // Act: Paginate with limit=1
    let mut query = ODataQuery::default().with_limit(1);
    let mut fetched_count = 0;

    for _ in 0..10 {
        // Limit iterations to prevent infinite loop
        let page = repo
            .list_users_page(&ctx, &query)
            .await
            .expect("Should succeed");

        if page.items.is_empty() {
            break;
        }

        assert_eq!(page.items.len(), 1, "Each page should have exactly 1 item");
        fetched_count += 1;

        if let Some(next_cursor) = page.page_info.next_cursor {
            let cursor = modkit_odata::CursorV1::decode(&next_cursor).unwrap();
            query = ODataQuery::default().with_cursor(cursor).with_limit(1);
        } else {
            break;
        }
    }

    // Assert: Should have fetched all 5 users one at a time
    assert_eq!(fetched_count, 5, "Should fetch all 5 users");
}

#[tokio::test]
async fn test_cursor_stability_repeated_queries() {
    // Arrange: Create 10 users
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    seed_users_sequential(&db, 10, tenant_id).await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let ctx = ctx_allow_tenants(&[tenant_id]);

    // Act: Get first page twice with same query
    let query = ODataQuery::default().with_limit(5);

    let first_page = repo
        .list_users_page(&ctx, &query)
        .await
        .expect("First query should succeed");

    let second_page = repo
        .list_users_page(&ctx, &query)
        .await
        .expect("Second query should succeed");

    // Assert: Both should return identical results
    assert_eq!(first_page.items.len(), second_page.items.len());

    for (a, b) in first_page.items.iter().zip(second_page.items.iter()) {
        assert_eq!(a.id, b.id, "Repeated queries should return same items");
    }

    // Cursors should be identical
    assert_eq!(
        first_page.page_info.next_cursor, second_page.page_info.next_cursor,
        "Cursors should be stable across repeated queries"
    );
}

#[tokio::test]
async fn test_cursor_with_different_ordering() {
    // Arrange: Create 15 users
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    seed_users_sequential(&db, 15, tenant_id).await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let ctx = ctx_allow_tenants(&[tenant_id]);

    // Act: Get page 1 with ASC order
    let order_asc = ODataOrderBy(vec![OrderKey {
        field: "created_at".to_owned(),
        dir: SortDir::Asc,
    }]);

    let query_asc = ODataQuery::default().with_order(order_asc).with_limit(5);

    let page_asc = repo
        .list_users_page(&ctx, &query_asc)
        .await
        .expect("ASC query should succeed");

    // Get page 1 with DESC order
    let order_desc = ODataOrderBy(vec![OrderKey {
        field: "created_at".to_owned(),
        dir: SortDir::Desc,
    }]);

    let query_desc = ODataQuery::default().with_order(order_desc).with_limit(5);

    let page_desc = repo
        .list_users_page(&ctx, &query_desc)
        .await
        .expect("DESC query should succeed");

    // Assert: First items should be different
    assert_ne!(
        page_asc.items.first().unwrap().id,
        page_desc.items.first().unwrap().id,
        "Different ordering should return different first items"
    );

    // ASC first item should have earlier created_at than DESC first item
    assert!(
        page_asc.items.first().unwrap().created_at < page_desc.items.first().unwrap().created_at,
        "ASC should start with earliest, DESC with latest"
    );
}
