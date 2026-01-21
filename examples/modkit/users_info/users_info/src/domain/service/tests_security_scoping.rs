#![allow(clippy::unwrap_used, clippy::expect_used)]

use uuid::Uuid;

use crate::domain::service::ServiceConfig;
use crate::test_support::{build_services, ctx_allow_tenants, ctx_deny_all, inmem_db, seed_user};
use modkit_db::secure::SecureConn;
use user_info_sdk::NewUser;

#[tokio::test]
async fn tenant_scope_only_sees_its_tenant() {
    let db = inmem_db().await;
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();

    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();
    seed_user(&db, user1, tenant1, "u1@example.com", "U1").await;
    seed_user(&db, user2, tenant2, "u2@example.com", "U2").await;

    let services = build_services(SecureConn::new(db), ServiceConfig::default());
    let ctx_t1 = ctx_allow_tenants(&[tenant1]);

    let page = services
        .users
        .list_users_page(&ctx_t1, &modkit_odata::ODataQuery::default())
        .await
        .unwrap();
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].tenant_id, tenant1);
}

#[tokio::test]
async fn deny_all_sees_nothing() {
    let db = inmem_db().await;
    let tenant = Uuid::new_v4();
    seed_user(&db, Uuid::new_v4(), tenant, "u@example.com", "U").await;

    let services = build_services(SecureConn::new(db), ServiceConfig::default());
    let ctx = ctx_deny_all();

    let page = services
        .users
        .list_users_page(&ctx, &modkit_odata::ODataQuery::default())
        .await
        .unwrap();
    assert!(page.items.is_empty());
}

#[tokio::test]
async fn create_user_with_transaction() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();

    let services = build_services(SecureConn::new(db), ServiceConfig::default());
    // Use a context with tenants, not root, because insert requires tenant scope
    let ctx = ctx_allow_tenants(&[tenant_id]);

    let new_user = NewUser {
        id: None,
        tenant_id,
        email: "test@example.com".to_owned(),
        display_name: "Test User".to_owned(),
    };

    let result = services.users.create_user(&ctx, new_user).await;
    assert!(result.is_ok(), "create_user failed: {:?}", result.err());

    let created = result.unwrap();
    assert_eq!(created.email, "test@example.com");
    assert_eq!(created.display_name, "Test User");
    assert_eq!(created.tenant_id, tenant_id);
}
