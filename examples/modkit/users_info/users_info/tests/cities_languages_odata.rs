#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::doc_markdown)]

//! Integration tests for OData cursor pagination on cities and languages.
//!
//! These tests verify end-to-end cursor pagination behavior for cities and languages
//! using an in-memory SQLite database, including filtering, ordering, and cursor validation.

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

async fn seed_cities_sequential(
    db: &sea_orm::DatabaseConnection,
    count: usize,
    tenant_id: Uuid,
) -> Vec<Uuid> {
    let mut city_ids = Vec::new();

    for i in 0..count {
        let id = Uuid::new_v4();
        let name = format!("City{i}");
        let country = format!("Country{}", i % 3);
        seed_city(db, id, tenant_id, &name, &country).await;
        city_ids.push(id);

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    city_ids
}

async fn seed_languages_sequential(
    db: &sea_orm::DatabaseConnection,
    count: usize,
    tenant_id: Uuid,
) -> Vec<Uuid> {
    let mut language_ids = Vec::new();

    for i in 0..count {
        let id = Uuid::new_v4();
        let code = format!("lang{i}");
        let name = format!("Language{i}");
        seed_language(db, id, tenant_id, &code, &name).await;
        language_ids.push(id);

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    language_ids
}

// ============================================================================
// Cities OData Pagination Tests
// ============================================================================

#[tokio::test]
async fn test_cities_forward_pagination() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let _city_ids = seed_cities_sequential(&db, 15, tenant_id).await;

    let sec = SecureConn::new(db);
    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let service = Service::new(sec, events, audit, ServiceConfig::default());

    let ctx = ctx_allow_tenants(&[tenant_id]);

    let mut query = ODataQuery::default().with_limit(5);
    let mut collected_ids = Vec::new();

    loop {
        let page = service
            .list_cities_page(&ctx, &query)
            .await
            .expect("Cities pagination should succeed");

        collected_ids.extend(page.items.iter().map(|c| c.id));

        if let Some(next_cursor) = page.page_info.next_cursor {
            let cursor = modkit_odata::CursorV1::decode(&next_cursor).unwrap();
            query = ODataQuery::default().with_cursor(cursor).with_limit(5);
        } else {
            break;
        }
    }

    assert_eq!(collected_ids.len(), 15, "Should collect all 15 cities");
}

#[tokio::test]
async fn test_cities_order_by_name() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();

    seed_city(&db, Uuid::new_v4(), tenant_id, "Zebra City", "Country1").await;
    seed_city(&db, Uuid::new_v4(), tenant_id, "Alpha City", "Country2").await;
    seed_city(&db, Uuid::new_v4(), tenant_id, "Beta City", "Country3").await;

    let sec = SecureConn::new(db);
    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let service = Service::new(sec, events, audit, ServiceConfig::default());

    let ctx = ctx_allow_tenants(&[tenant_id]);

    let order = ODataOrderBy(vec![OrderKey {
        field: "name".to_owned(),
        dir: SortDir::Asc,
    }]);

    let query = ODataQuery::default().with_order(order).with_limit(10);

    let page = service
        .list_cities_page(&ctx, &query)
        .await
        .expect("Ordered cities query should succeed");

    assert_eq!(page.items.len(), 3);
    assert_eq!(page.items[0].name, "Alpha City");
    assert_eq!(page.items[1].name, "Beta City");
    assert_eq!(page.items[2].name, "Zebra City");
}

#[tokio::test]
async fn test_cities_cursor_with_ordering() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let _city_ids = seed_cities_sequential(&db, 10, tenant_id).await;

    let sec = SecureConn::new(db);
    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let service = Service::new(sec, events, audit, ServiceConfig::default());

    let ctx = ctx_allow_tenants(&[tenant_id]);

    let order = ODataOrderBy(vec![OrderKey {
        field: "name".to_owned(),
        dir: SortDir::Asc,
    }]);

    let query = ODataQuery::default()
        .with_order(order.clone())
        .with_limit(5);

    let page1 = service
        .list_cities_page(&ctx, &query)
        .await
        .expect("First page should succeed");

    assert_eq!(page1.items.len(), 5);

    let next_cursor_encoded = page1
        .page_info
        .next_cursor
        .expect("Should have next cursor");

    let next_cursor = modkit_odata::CursorV1::decode(&next_cursor_encoded).unwrap();

    let query2 = ODataQuery::default()
        .with_cursor(next_cursor)
        .with_order(order)
        .with_limit(5);

    let page2 = service.list_cities_page(&ctx, &query2).await;
    assert!(page2.is_ok(), "Second page with cursor should succeed");
    assert_eq!(page2.unwrap().items.len(), 5);
}

// ============================================================================
// Languages OData Pagination Tests
// ============================================================================

#[tokio::test]
async fn test_languages_forward_pagination() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let _language_ids = seed_languages_sequential(&db, 12, tenant_id).await;

    let sec = SecureConn::new(db);
    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let service = Service::new(sec, events, audit, ServiceConfig::default());

    let ctx = ctx_allow_tenants(&[tenant_id]);

    let mut query = ODataQuery::default().with_limit(4);
    let mut collected_ids = Vec::new();

    loop {
        let page = service
            .list_languages_page(&ctx, &query)
            .await
            .expect("Languages pagination should succeed");

        collected_ids.extend(page.items.iter().map(|l| l.id));

        if let Some(next_cursor) = page.page_info.next_cursor {
            let cursor = modkit_odata::CursorV1::decode(&next_cursor).unwrap();
            query = ODataQuery::default().with_cursor(cursor).with_limit(4);
        } else {
            break;
        }
    }

    assert_eq!(collected_ids.len(), 12, "Should collect all 12 languages");
}

#[tokio::test]
async fn test_languages_order_by_name_desc() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();

    seed_language(&db, Uuid::new_v4(), tenant_id, "en", "English").await;
    seed_language(&db, Uuid::new_v4(), tenant_id, "es", "Spanish").await;
    seed_language(&db, Uuid::new_v4(), tenant_id, "fr", "French").await;

    let sec = SecureConn::new(db);
    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let service = Service::new(sec, events, audit, ServiceConfig::default());

    let ctx = ctx_allow_tenants(&[tenant_id]);

    let order = ODataOrderBy(vec![OrderKey {
        field: "name".to_owned(),
        dir: SortDir::Desc,
    }]);

    let query = ODataQuery::default().with_order(order).with_limit(10);

    let page = service
        .list_languages_page(&ctx, &query)
        .await
        .expect("Ordered languages query should succeed");

    assert_eq!(page.items.len(), 3);
    assert_eq!(page.items[0].name, "Spanish");
    assert_eq!(page.items[1].name, "French");
    assert_eq!(page.items[2].name, "English");
}

#[tokio::test]
async fn test_languages_backward_navigation() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let _language_ids = seed_languages_sequential(&db, 10, tenant_id).await;

    let sec = SecureConn::new(db);
    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let service = Service::new(sec, events, audit, ServiceConfig::default());

    let ctx = ctx_allow_tenants(&[tenant_id]);

    let query1 = ODataQuery::default().with_limit(4);
    let page1 = service
        .list_languages_page(&ctx, &query1)
        .await
        .expect("Page 1 should succeed");

    let cursor2 = modkit_odata::CursorV1::decode(
        &page1
            .page_info
            .next_cursor
            .expect("Should have next cursor"),
    )
    .unwrap();
    let query2 = ODataQuery::default().with_cursor(cursor2).with_limit(4);

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

    let query_back = ODataQuery::default().with_cursor(cursor_back).with_limit(4);

    let page_back = service
        .list_languages_page(&ctx, &query_back)
        .await
        .expect("Backward navigation should succeed");

    assert_eq!(page_back.items.len(), page1.items.len());
    assert_eq!(
        page_back.items[0].id, page1.items[0].id,
        "Backward navigation should return to first page"
    );
}

#[tokio::test]
async fn test_languages_multi_page_pagination() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();

    for i in 0..10 {
        let code = format!("lang{i}");
        let name = format!("Language {i}");
        seed_language(&db, Uuid::new_v4(), tenant_id, &code, &name).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let sec = SecureConn::new(db);
    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let service = Service::new(sec, events, audit, ServiceConfig::default());

    let ctx = ctx_allow_tenants(&[tenant_id]);

    let mut query = ODataQuery::default().with_limit(3);
    let mut collected = Vec::new();

    loop {
        let page = service
            .list_languages_page(&ctx, &query)
            .await
            .expect("Pagination should succeed");

        collected.extend(page.items);

        if let Some(next_cursor) = page.page_info.next_cursor {
            let cursor = modkit_odata::CursorV1::decode(&next_cursor).unwrap();
            query = ODataQuery::default().with_cursor(cursor).with_limit(3);
        } else {
            break;
        }
    }

    assert_eq!(collected.len(), 10, "Should collect all 10 languages");
}

#[tokio::test]
async fn test_cities_and_languages_independent_cursors() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();

    let _city_ids = seed_cities_sequential(&db, 5, tenant_id).await;
    let _language_ids = seed_languages_sequential(&db, 5, tenant_id).await;

    let sec = SecureConn::new(db.clone());
    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let service = Service::new(sec, events, audit, ServiceConfig::default());

    let ctx = ctx_allow_tenants(&[tenant_id]);

    let cities_page = service
        .list_cities_page(&ctx, &ODataQuery::default().with_limit(3))
        .await
        .expect("Cities query should succeed");

    let languages_page = service
        .list_languages_page(&ctx, &ODataQuery::default().with_limit(3))
        .await
        .expect("Languages query should succeed");

    assert_eq!(cities_page.items.len(), 3);
    assert_eq!(languages_page.items.len(), 3);
    assert!(cities_page.page_info.next_cursor.is_some());
    assert!(languages_page.page_info.next_cursor.is_some());
}
