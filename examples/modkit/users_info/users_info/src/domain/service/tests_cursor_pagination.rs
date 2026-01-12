#![allow(clippy::unwrap_used, clippy::expect_used)]

use modkit_db::secure::SecureConn;
use modkit_odata::{CursorV1, ODataQuery};
use uuid::Uuid;

use crate::domain::service::ServiceConfig;
use crate::test_support::{build_services, ctx_allow_tenants, ctx_deny_all, inmem_db, seed_user};

async fn seed_users_sequential(
    db: &sea_orm::DatabaseConnection,
    count: usize,
    tenant_id: Uuid,
) -> Vec<Uuid> {
    let mut ids = Vec::with_capacity(count);
    for i in 0..count {
        let id = Uuid::new_v4();
        seed_user(
            db,
            id,
            tenant_id,
            &format!("user{i}@example.com"),
            &format!("User {i}"),
        )
        .await;
        ids.push(id);

        // ensure different timestamps for deterministic ordering
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    }
    ids
}

#[tokio::test]
async fn forward_pagination_over_multiple_pages() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let seeded = seed_users_sequential(&db, 25, tenant_id).await;

    let services = build_services(SecureConn::new(db), ServiceConfig::default());
    let ctx = ctx_allow_tenants(&[tenant_id]);

    let mut query = ODataQuery::default().with_limit(10);
    let mut fetched = Vec::new();

    loop {
        let page = services.users.list_users_page(&ctx, &query).await.unwrap();
        fetched.extend(page.items.iter().map(|u| u.id));
        match page.page_info.next_cursor.clone() {
            Some(c) => {
                let decoded = CursorV1::decode(&c).expect("cursor must decode");
                query = query.clone().with_cursor(decoded);
            }
            None => break,
        }
    }

    assert_eq!(fetched.len(), 25);
    // sanity: must be a permutation of seeded ids
    for id in seeded {
        assert!(fetched.contains(&id));
    }
}

#[tokio::test]
async fn deny_all_returns_empty_page() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    seed_user(&db, Uuid::new_v4(), tenant_id, "u@example.com", "U").await;

    let services = build_services(SecureConn::new(db), ServiceConfig::default());
    let ctx = ctx_deny_all();

    let page = services
        .users
        .list_users_page(&ctx, &ODataQuery::default().with_limit(10))
        .await
        .unwrap();
    assert!(page.items.is_empty());
}
