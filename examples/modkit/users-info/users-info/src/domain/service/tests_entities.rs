#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::str_to_string)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::default_trait_access)]

use sea_orm::Set;
use time::OffsetDateTime;
use uuid::Uuid;

use users_info_sdk::{CityPatch, NewAddress, NewCity};

use crate::domain::service::ServiceConfig;
use crate::infra::storage::entity::city::ActiveModel as CityAM;
use crate::infra::storage::entity::city::Entity as CityEntity;
use crate::test_support::{build_services, ctx_allow_tenants, inmem_db, seed_user};
use modkit_db::secure::{AccessScope, secure_insert};

#[tokio::test]
async fn create_city_success() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();

    let services = build_services(db.clone(), ServiceConfig::default());
    let ctx = ctx_allow_tenants(&[tenant_id]);

    let new_city = NewCity {
        id: None,
        tenant_id,
        name: "San Francisco".to_string(),
        country: "USA".to_string(),
    };

    let city = services.cities.create_city(&ctx, new_city).await.unwrap();
    assert_eq!(city.name, "San Francisco");
    assert_eq!(city.country, "USA");
    assert_eq!(city.tenant_id, tenant_id);
}

#[tokio::test]
async fn get_city_respects_tenant_scope() {
    let db = inmem_db().await;
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();
    let city_id = Uuid::new_v4();

    let now = OffsetDateTime::now_utc();
    let city_am = CityAM {
        id: Set(city_id),
        tenant_id: Set(tenant1),
        name: Set("Paris".to_string()),
        country: Set("France".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let scope = AccessScope::tenants_only(vec![tenant1]);
    let conn = db.conn().unwrap();
    let _ = secure_insert::<CityEntity>(city_am, &scope, &conn)
        .await
        .expect("Failed to seed city");

    let services = build_services(db.clone(), ServiceConfig::default());
    let ctx_ok = ctx_allow_tenants(&[tenant1]);
    let ctx_deny = ctx_allow_tenants(&[tenant2]);

    assert_eq!(
        services
            .cities
            .get_city(&ctx_ok, city_id)
            .await
            .unwrap()
            .name,
        "Paris"
    );
    assert!(services.cities.get_city(&ctx_deny, city_id).await.is_err());
}

#[tokio::test]
async fn update_city_success() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let city_id = Uuid::new_v4();

    let now = OffsetDateTime::now_utc();
    let city_am = CityAM {
        id: Set(city_id),
        tenant_id: Set(tenant_id),
        name: Set("Old Name".to_string()),
        country: Set("Old Country".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let scope = AccessScope::tenants_only(vec![tenant_id]);
    let conn = db.conn().unwrap();
    let _ = secure_insert::<CityEntity>(city_am, &scope, &conn)
        .await
        .expect("Failed to seed city");

    let services = build_services(db.clone(), ServiceConfig::default());
    let ctx = ctx_allow_tenants(&[tenant_id]);
    let patch = CityPatch {
        name: Some("New Name".to_string()),
        country: Some("New Country".to_string()),
    };

    let updated = services
        .cities
        .update_city(&ctx, city_id, patch)
        .await
        .unwrap();
    assert_eq!(updated.name, "New Name");
    assert_eq!(updated.country, "New Country");
}

#[tokio::test]
async fn address_crud_and_scope() {
    let db = inmem_db().await;
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let conn = db.conn().unwrap();
    seed_user(&conn, user_id, tenant1, "u@example.com", "U").await;

    let services = build_services(db.clone(), ServiceConfig::default());
    let ctx1 = ctx_allow_tenants(&[tenant1]);
    let ctx2 = ctx_allow_tenants(&[tenant2]);

    // Address requires an existing city_id in this tenant
    let city = services
        .cities
        .create_city(
            &ctx1,
            NewCity {
                id: None,
                tenant_id: tenant1,
                name: "C".to_string(),
                country: "K".to_string(),
            },
        )
        .await
        .unwrap();

    let created = services
        .addresses
        .create_address(
            &ctx1,
            NewAddress {
                id: None,
                tenant_id: tenant1,
                user_id,
                city_id: city.id,
                street: "Main St".to_string(),
                postal_code: "12345".to_string(),
            },
        )
        .await
        .unwrap();

    assert!(
        services
            .addresses
            .get_address(&ctx2, created.id)
            .await
            .is_err()
    );
    services
        .addresses
        .delete_address(&ctx1, created.id)
        .await
        .unwrap();
    assert!(
        services
            .addresses
            .get_address(&ctx1, created.id)
            .await
            .is_err()
    );
}
