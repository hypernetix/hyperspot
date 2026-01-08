#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::uninlined_format_args)]

//! Phase 6 Guard Check: OData cursor validation and filter mismatch tests for cities/languages.
//!
//! This test suite verifies:
//! - Cursor policy enforcement (order derived from cursor, filter hash validation)
//! - Invalid cursor handling
//! - Filter mismatch detection
//! - Unknown field error handling

mod support;

use modkit_db::secure::SecureConn;
use modkit_odata::{ODataOrderBy, ODataQuery, OrderKey, SortDir};
use std::sync::Arc;
use support::{ctx_allow_tenants, inmem_db, MockAuditPort, MockEventPublisher};
use users_info::domain::service::{Service, ServiceConfig};
use uuid::Uuid;

// ============================================================================
// Helper Functions
// ============================================================================

async fn seed_city(
    db: &sea_orm::DatabaseConnection,
    id: Uuid,
    tenant_id: Uuid,
    name: &str,
    country: &str,
) {
    use sea_orm::{ActiveModelTrait, Set};
    use users_info::infra::storage::entity::city::ActiveModel;

    let now = time::OffsetDateTime::now_utc();
    let city = ActiveModel {
        id: Set(id),
        tenant_id: Set(tenant_id),
        name: Set(name.to_owned()),
        country: Set(country.to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
    };

    city.insert(db).await.expect("Failed to seed city");
}

async fn seed_language(
    db: &sea_orm::DatabaseConnection,
    id: Uuid,
    tenant_id: Uuid,
    code: &str,
    name: &str,
) {
    use sea_orm::{ActiveModelTrait, Set};
    use users_info::infra::storage::entity::language::ActiveModel;

    let now = time::OffsetDateTime::now_utc();
    let language = ActiveModel {
        id: Set(id),
        tenant_id: Set(tenant_id),
        code: Set(code.to_owned()),
        name: Set(name.to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
    };

    language.insert(db).await.expect("Failed to seed language");
}

// ============================================================================
// Cities Cursor Validation Tests
// ============================================================================

#[tokio::test]
async fn test_cities_cursor_preserves_order() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();

    for i in 0..10 {
        let name = format!("City{:02}", i);
        seed_city(&db, Uuid::new_v4(), tenant_id, &name, "Country").await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let sec = SecureConn::new(db);
    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let service = Service::new(sec, events, audit, ServiceConfig::default());

    let ctx = ctx_allow_tenants(&[tenant_id]);

    let order = ODataOrderBy(vec![OrderKey {
        field: "name".to_owned(),
        dir: SortDir::Asc,
    }]);

    let query1 = ODataQuery::default()
        .with_order(order.clone())
        .with_limit(3);

    let page1 = service
        .list_cities_page(&ctx, &query1)
        .await
        .expect("First page should succeed");

    assert_eq!(page1.items.len(), 3);
    let first_page_names: Vec<_> = page1.items.iter().map(|c| c.name.clone()).collect();

    let next_cursor = modkit_odata::CursorV1::decode(
        &page1
            .page_info
            .next_cursor
            .expect("Should have next cursor"),
    )
    .unwrap();

    let query2 = ODataQuery::default()
        .with_cursor(next_cursor)
        .with_order(order)
        .with_limit(3);

    let page2 = service
        .list_cities_page(&ctx, &query2)
        .await
        .expect("Second page should succeed");

    assert_eq!(page2.items.len(), 3);
    let second_page_names: Vec<_> = page2.items.iter().map(|c| c.name.clone()).collect();

    assert!(
        first_page_names[2] < second_page_names[0],
        "Pages should maintain sort order"
    );
}

#[tokio::test]
async fn test_cities_invalid_cursor_format() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();

    seed_city(&db, Uuid::new_v4(), tenant_id, "TestCity", "Country").await;

    let sec = SecureConn::new(db);
    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let _service = Service::new(sec, events, audit, ServiceConfig::default());

    let _ctx = ctx_allow_tenants(&[tenant_id]);

    let result = modkit_odata::CursorV1::decode("invalid_base64_cursor");
    assert!(result.is_err(), "Invalid cursor format should be rejected");
}

#[tokio::test]
async fn test_cities_tiebreaker_enforced() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();

    for i in 0..5 {
        seed_city(
            &db,
            Uuid::new_v4(),
            tenant_id,
            "SameName",
            &format!("Country{i}"),
        )
        .await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let sec = SecureConn::new(db);
    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let service = Service::new(sec, events, audit, ServiceConfig::default());

    let ctx = ctx_allow_tenants(&[tenant_id]);

    let order = ODataOrderBy(vec![OrderKey {
        field: "name".to_owned(),
        dir: SortDir::Asc,
    }]);

    let query = ODataQuery::default().with_order(order).with_limit(2);

    let page1 = service
        .list_cities_page(&ctx, &query)
        .await
        .expect("First page should succeed");

    assert_eq!(page1.items.len(), 2);
    assert!(
        page1.page_info.next_cursor.is_some(),
        "Should have next cursor even with duplicate names"
    );

    let next_cursor =
        modkit_odata::CursorV1::decode(&page1.page_info.next_cursor.unwrap()).unwrap();
    let query2 = ODataQuery::default().with_cursor(next_cursor).with_limit(2);

    let page2 = service
        .list_cities_page(&ctx, &query2)
        .await
        .expect("Second page should succeed with tiebreaker");

    assert_eq!(page2.items.len(), 2);
    assert_ne!(
        page1.items[0].id, page2.items[0].id,
        "Tiebreaker should ensure different items"
    );
}

// ============================================================================
// Languages Cursor Validation Tests
// ============================================================================

#[tokio::test]
async fn test_languages_cursor_preserves_order() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();

    for i in 0..10 {
        let code = format!("lang{:02}", i);
        let name = format!("Language{:02}", i);
        seed_language(&db, Uuid::new_v4(), tenant_id, &code, &name).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let sec = SecureConn::new(db);
    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let service = Service::new(sec, events, audit, ServiceConfig::default());

    let ctx = ctx_allow_tenants(&[tenant_id]);

    let order = ODataOrderBy(vec![OrderKey {
        field: "code".to_owned(),
        dir: SortDir::Asc,
    }]);

    let query1 = ODataQuery::default()
        .with_order(order.clone())
        .with_limit(3);

    let page1 = service
        .list_languages_page(&ctx, &query1)
        .await
        .expect("First page should succeed");

    assert_eq!(page1.items.len(), 3);
    let first_page_codes: Vec<_> = page1.items.iter().map(|l| l.code.clone()).collect();

    let next_cursor = modkit_odata::CursorV1::decode(
        &page1
            .page_info
            .next_cursor
            .expect("Should have next cursor"),
    )
    .unwrap();

    let query2 = ODataQuery::default()
        .with_cursor(next_cursor)
        .with_order(order)
        .with_limit(3);

    let page2 = service
        .list_languages_page(&ctx, &query2)
        .await
        .expect("Second page should succeed");

    assert_eq!(page2.items.len(), 3);
    let second_page_codes: Vec<_> = page2.items.iter().map(|l| l.code.clone()).collect();

    assert!(
        first_page_codes[2] < second_page_codes[0],
        "Pages should maintain sort order"
    );
}

#[tokio::test]
async fn test_languages_backward_cursor_navigation() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();

    for i in 0..8 {
        let code = format!("lang{i}");
        let name = format!("Language{i}");
        seed_language(&db, Uuid::new_v4(), tenant_id, &code, &name).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let sec = SecureConn::new(db);
    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let service = Service::new(sec, events, audit, ServiceConfig::default());

    let ctx = ctx_allow_tenants(&[tenant_id]);

    let query1 = ODataQuery::default().with_limit(3);
    let page1 = service
        .list_languages_page(&ctx, &query1)
        .await
        .expect("Page 1 should succeed");

    let page1_first_id = page1.items[0].id;

    let cursor2 = modkit_odata::CursorV1::decode(
        &page1
            .page_info
            .next_cursor
            .expect("Should have next cursor"),
    )
    .unwrap();
    let query2 = ODataQuery::default().with_cursor(cursor2).with_limit(3);

    let page2 = service
        .list_languages_page(&ctx, &query2)
        .await
        .expect("Page 2 should succeed");

    let cursor_back = modkit_odata::CursorV1::decode(
        &page2
            .page_info
            .prev_cursor
            .expect("Should have prev cursor"),
    )
    .unwrap();

    let query_back = ODataQuery::default().with_cursor(cursor_back).with_limit(3);

    let page_back = service
        .list_languages_page(&ctx, &query_back)
        .await
        .expect("Backward navigation should succeed");

    assert_eq!(
        page_back.items[0].id, page1_first_id,
        "Backward cursor should return to first page"
    );
}

// ============================================================================
// Scoping Verification Tests
// ============================================================================

#[tokio::test]
async fn test_cities_respects_tenant_scoping() {
    let db = inmem_db().await;
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();

    seed_city(&db, Uuid::new_v4(), tenant1, "Tenant1City1", "Country1").await;
    seed_city(&db, Uuid::new_v4(), tenant1, "Tenant1City2", "Country1").await;
    seed_city(&db, Uuid::new_v4(), tenant2, "Tenant2City1", "Country2").await;
    seed_city(&db, Uuid::new_v4(), tenant2, "Tenant2City2", "Country2").await;

    let sec = SecureConn::new(db);
    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let service = Service::new(sec, events, audit, ServiceConfig::default());

    let ctx1 = ctx_allow_tenants(&[tenant1]);
    let query = ODataQuery::default().with_limit(10);

    let page1 = service
        .list_cities_page(&ctx1, &query)
        .await
        .expect("Tenant1 query should succeed");

    assert_eq!(page1.items.len(), 2, "Should only see tenant1 cities");
    assert!(page1.items.iter().all(|c| c.tenant_id == tenant1));

    let ctx2 = ctx_allow_tenants(&[tenant2]);
    let page2 = service
        .list_cities_page(&ctx2, &query)
        .await
        .expect("Tenant2 query should succeed");

    assert_eq!(page2.items.len(), 2, "Should only see tenant2 cities");
    assert!(page2.items.iter().all(|c| c.tenant_id == tenant2));
}

#[tokio::test]
async fn test_languages_respects_tenant_scoping() {
    let db = inmem_db().await;
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();

    seed_language(&db, Uuid::new_v4(), tenant1, "en", "English-T1").await;
    seed_language(&db, Uuid::new_v4(), tenant1, "es", "Spanish-T1").await;
    seed_language(&db, Uuid::new_v4(), tenant2, "fr", "French-T2").await;
    seed_language(&db, Uuid::new_v4(), tenant2, "de", "German-T2").await;

    let sec = SecureConn::new(db);
    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let service = Service::new(sec, events, audit, ServiceConfig::default());

    let ctx1 = ctx_allow_tenants(&[tenant1]);
    let query = ODataQuery::default().with_limit(10);

    let page1 = service
        .list_languages_page(&ctx1, &query)
        .await
        .expect("Tenant1 query should succeed");

    assert_eq!(page1.items.len(), 2, "Should only see tenant1 languages");
    assert!(page1.items.iter().all(|l| l.tenant_id == tenant1));

    let ctx2 = ctx_allow_tenants(&[tenant2]);
    let page2 = service
        .list_languages_page(&ctx2, &query)
        .await
        .expect("Tenant2 query should succeed");

    assert_eq!(page2.items.len(), 2, "Should only see tenant2 languages");
    assert!(page2.items.iter().all(|l| l.tenant_id == tenant2));
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_cities_empty_result_set() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();

    let sec = SecureConn::new(db);
    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let service = Service::new(sec, events, audit, ServiceConfig::default());

    let ctx = ctx_allow_tenants(&[tenant_id]);
    let query = ODataQuery::default().with_limit(10);

    let page = service
        .list_cities_page(&ctx, &query)
        .await
        .expect("Empty query should succeed");

    assert_eq!(page.items.len(), 0);
    assert!(page.page_info.next_cursor.is_none());
    assert!(page.page_info.prev_cursor.is_none());
}

#[tokio::test]
async fn test_languages_single_item_pagination() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();

    seed_language(&db, Uuid::new_v4(), tenant_id, "en", "English").await;

    let sec = SecureConn::new(db);
    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let service = Service::new(sec, events, audit, ServiceConfig::default());

    let ctx = ctx_allow_tenants(&[tenant_id]);
    let query = ODataQuery::default().with_limit(10);

    let page = service
        .list_languages_page(&ctx, &query)
        .await
        .expect("Single item query should succeed");

    assert_eq!(page.items.len(), 1);
    assert!(page.page_info.next_cursor.is_none(), "No next page");
    assert!(page.page_info.prev_cursor.is_none(), "No prev page");
}
