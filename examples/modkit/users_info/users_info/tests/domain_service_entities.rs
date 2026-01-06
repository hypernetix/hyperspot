#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::str_to_string)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::default_trait_access)]

//! Integration tests for domain service operations on cities, languages, addresses, and user-language relations.
//!
//! These tests verify:
//! - CRUD operations for cities and languages
//! - Address operations (1:1 relationship with users)
//! - User-language relationship operations (N:N)
//! - Security scoping for all operations
//! - Idempotency of assign/remove language operations

mod support;

use modkit_db::secure::SecureConn;
use modkit_odata::ODataQuery;
use sea_orm::{ActiveModelTrait, Set};
use std::sync::Arc;
use support::{
    ctx_allow_tenants, ctx_deny_all, inmem_db, seed_user, MockAuditPort, MockEventPublisher,
};
use time::OffsetDateTime;
use user_info_sdk::{CityPatch, LanguagePatch, NewAddress, NewCity, NewLanguage};
use users_info::domain::service::{Service, ServiceConfig};
use users_info::infra::storage::entity::{
    city::ActiveModel as CityAM, language::ActiveModel as LanguageAM,
};
use uuid::Uuid;

// ==================== City Tests ====================

#[tokio::test]
async fn create_city_success() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);
    let new_city = NewCity {
        id: None,
        tenant_id,
        name: "San Francisco".to_string(),
        country: "USA".to_string(),
    };

    let result = service.create_city(&ctx, new_city).await;
    assert!(result.is_ok());
    let city = result.unwrap();
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
    city_am.insert(&db).await.expect("Failed to seed city");

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx_ok = ctx_allow_tenants(&[tenant1]);
    let ctx_deny = ctx_allow_tenants(&[tenant2]);

    let result_ok = service.get_city(&ctx_ok, city_id).await;
    assert!(result_ok.is_ok());
    assert_eq!(result_ok.unwrap().name, "Paris");

    let result_deny = service.get_city(&ctx_deny, city_id).await;
    assert!(
        result_deny.is_err(),
        "Should not access city in different tenant"
    );
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
    city_am.insert(&db).await.expect("Failed to seed city");

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);
    let patch = CityPatch {
        name: Some("New Name".to_string()),
        country: Some("New Country".to_string()),
    };

    let result = service.update_city(&ctx, city_id, patch).await;
    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.name, "New Name");
    assert_eq!(updated.country, "New Country");
}

#[tokio::test]
async fn delete_city_success() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let city_id = Uuid::new_v4();

    let now = OffsetDateTime::now_utc();
    let city_am = CityAM {
        id: Set(city_id),
        tenant_id: Set(tenant_id),
        name: Set("To Delete".to_string()),
        country: Set("Test".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    city_am.insert(&db).await.expect("Failed to seed city");

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);
    let result = service.delete_city(&ctx, city_id).await;
    assert!(result.is_ok());

    let get_result = service.get_city(&ctx, city_id).await;
    assert!(get_result.is_err(), "City should be deleted");
}

#[tokio::test]
async fn list_cities_with_pagination() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();

    let now = OffsetDateTime::now_utc();
    for i in 0..5 {
        let city_am = CityAM {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            name: Set(format!("City {}", i)),
            country: Set("Test".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
        };
        city_am.insert(&db).await.expect("Failed to seed city");
    }

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);
    let query = ODataQuery {
        filter: None,
        order: Default::default(),
        limit: Some(10),
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let result = service.list_cities_page(&ctx, &query).await;
    assert!(result.is_ok());
    let page = result.unwrap();
    assert_eq!(page.items.len(), 5);
}

// ==================== Language Tests ====================

#[tokio::test]
async fn create_language_success() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);
    let new_language = NewLanguage {
        id: None,
        tenant_id,
        code: "en".to_string(),
        name: "English".to_string(),
    };

    let result = service.create_language(&ctx, new_language).await;
    assert!(result.is_ok());
    let language = result.unwrap();
    assert_eq!(language.code, "en");
    assert_eq!(language.name, "English");
}

#[tokio::test]
async fn get_language_respects_tenant_scope() {
    let db = inmem_db().await;
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();
    let language_id = Uuid::new_v4();

    let now = OffsetDateTime::now_utc();
    let lang_am = LanguageAM {
        id: Set(language_id),
        tenant_id: Set(tenant1),
        code: Set("fr".to_string()),
        name: Set("French".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    lang_am.insert(&db).await.expect("Failed to seed language");

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx_ok = ctx_allow_tenants(&[tenant1]);
    let ctx_deny = ctx_allow_tenants(&[tenant2]);

    let result_ok = service.get_language(&ctx_ok, language_id).await;
    assert!(result_ok.is_ok());
    assert_eq!(result_ok.unwrap().code, "fr");

    let result_deny = service.get_language(&ctx_deny, language_id).await;
    assert!(
        result_deny.is_err(),
        "Should not access language in different tenant"
    );
}

#[tokio::test]
async fn update_language_success() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let language_id = Uuid::new_v4();

    let now = OffsetDateTime::now_utc();
    let lang_am = LanguageAM {
        id: Set(language_id),
        tenant_id: Set(tenant_id),
        code: Set("old".to_string()),
        name: Set("Old Name".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    lang_am.insert(&db).await.expect("Failed to seed language");

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);
    let patch = LanguagePatch {
        code: Some("new".to_string()),
        name: Some("New Name".to_string()),
    };

    let result = service.update_language(&ctx, language_id, patch).await;
    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.code, "new");
    assert_eq!(updated.name, "New Name");
}

#[tokio::test]
async fn delete_language_success() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let language_id = Uuid::new_v4();

    let now = OffsetDateTime::now_utc();
    let lang_am = LanguageAM {
        id: Set(language_id),
        tenant_id: Set(tenant_id),
        code: Set("del".to_string()),
        name: Set("To Delete".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    lang_am.insert(&db).await.expect("Failed to seed language");

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);
    let result = service.delete_language(&ctx, language_id).await;
    assert!(result.is_ok());

    let get_result = service.get_language(&ctx, language_id).await;
    assert!(get_result.is_err(), "Language should be deleted");
}

#[tokio::test]
async fn list_languages_with_pagination() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();

    let now = OffsetDateTime::now_utc();
    for i in 0..5 {
        let lang_am = LanguageAM {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            code: Set(format!("l{}", i)),
            name: Set(format!("Language {}", i)),
            created_at: Set(now),
            updated_at: Set(now),
        };
        lang_am.insert(&db).await.expect("Failed to seed language");
    }

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);
    let query = ODataQuery {
        filter: None,
        order: Default::default(),
        limit: Some(10),
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let result = service.list_languages_page(&ctx, &query).await;
    assert!(result.is_ok());
    let page = result.unwrap();
    assert_eq!(page.items.len(), 5);
}

// ==================== Address Tests ====================

#[tokio::test]
async fn get_user_address_returns_none_when_not_exists() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);
    let result = service.get_user_address(&ctx, user_id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn put_user_address_creates_new_address() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let city_id = Uuid::new_v4();
    let now = OffsetDateTime::now_utc();
    let city_am = CityAM {
        id: Set(city_id),
        tenant_id: Set(tenant_id),
        name: Set("Test City".to_string()),
        country: Set("Test".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    city_am.insert(&db).await.expect("Failed to seed city");

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);
    let new_address = NewAddress {
        id: None,
        tenant_id,
        user_id,
        city_id,
        street: "123 Main St".to_string(),
        postal_code: "12345".to_string(),
    };

    let result = service.put_user_address(&ctx, user_id, new_address).await;
    assert!(result.is_ok());
    let address = result.unwrap();
    assert_eq!(address.user_id, user_id);
    assert_eq!(address.street, "123 Main St");
    assert_eq!(address.postal_code, "12345");
}

#[tokio::test]
async fn put_user_address_updates_existing_address() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let city_id = Uuid::new_v4();
    let now = OffsetDateTime::now_utc();
    let city_am = CityAM {
        id: Set(city_id),
        tenant_id: Set(tenant_id),
        name: Set("Test City".to_string()),
        country: Set("Test".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    city_am.insert(&db).await.expect("Failed to seed city");

    let sec = SecureConn::new(db.clone());
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);

    let first_address = NewAddress {
        id: None,
        tenant_id,
        user_id,
        city_id,
        street: "Old Street".to_string(),
        postal_code: "00000".to_string(),
    };
    let created = service
        .put_user_address(&ctx, user_id, first_address)
        .await
        .unwrap();

    let updated_address = NewAddress {
        id: Some(created.id),
        tenant_id,
        user_id,
        city_id,
        street: "New Street".to_string(),
        postal_code: "99999".to_string(),
    };
    let result = service
        .put_user_address(&ctx, user_id, updated_address)
        .await;
    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.street, "New Street");
    assert_eq!(updated.postal_code, "99999");
}

#[tokio::test]
async fn delete_user_address_success() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let city_id = Uuid::new_v4();
    let now = OffsetDateTime::now_utc();
    let city_am = CityAM {
        id: Set(city_id),
        tenant_id: Set(tenant_id),
        name: Set("Test City".to_string()),
        country: Set("Test".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    city_am.insert(&db).await.expect("Failed to seed city");

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);
    let new_address = NewAddress {
        id: None,
        tenant_id,
        user_id,
        city_id,
        street: "123 Main St".to_string(),
        postal_code: "12345".to_string(),
    };
    service
        .put_user_address(&ctx, user_id, new_address)
        .await
        .unwrap();

    let result = service.delete_user_address(&ctx, user_id).await;
    assert!(result.is_ok());

    let get_result = service.get_user_address(&ctx, user_id).await;
    assert!(get_result.is_ok());
    assert!(get_result.unwrap().is_none());
}

#[tokio::test]
async fn delete_user_address_returns_error_when_not_exists() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);
    let result = service.delete_user_address(&ctx, user_id).await;
    assert!(result.is_err());
}

// ==================== User-Language Relationship Tests ====================

#[tokio::test]
async fn assign_language_to_user_success() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let language_id = Uuid::new_v4();

    seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let now = OffsetDateTime::now_utc();
    let lang_am = LanguageAM {
        id: Set(language_id),
        tenant_id: Set(tenant_id),
        code: Set("en".to_string()),
        name: Set("English".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    lang_am.insert(&db).await.expect("Failed to seed language");

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);
    let result = service
        .assign_language_to_user(&ctx, user_id, language_id)
        .await;
    assert!(result.is_ok());

    let languages = service.list_user_languages(&ctx, user_id).await.unwrap();
    assert_eq!(languages.len(), 1);
    assert_eq!(languages[0].id, language_id);
}

#[tokio::test]
async fn assign_language_to_user_is_idempotent() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let language_id = Uuid::new_v4();

    seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let now = OffsetDateTime::now_utc();
    let lang_am = LanguageAM {
        id: Set(language_id),
        tenant_id: Set(tenant_id),
        code: Set("en".to_string()),
        name: Set("English".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    lang_am.insert(&db).await.expect("Failed to seed language");

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);

    let result1 = service
        .assign_language_to_user(&ctx, user_id, language_id)
        .await;
    assert!(result1.is_ok());

    let result2 = service
        .assign_language_to_user(&ctx, user_id, language_id)
        .await;
    assert!(
        result2.is_ok(),
        "Second assignment should succeed (idempotent)"
    );

    let languages = service.list_user_languages(&ctx, user_id).await.unwrap();
    assert_eq!(
        languages.len(),
        1,
        "Should only have one language assignment"
    );
}

#[tokio::test]
async fn remove_language_from_user_success() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let language_id = Uuid::new_v4();

    seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let now = OffsetDateTime::now_utc();
    let lang_am = LanguageAM {
        id: Set(language_id),
        tenant_id: Set(tenant_id),
        code: Set("en".to_string()),
        name: Set("English".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    lang_am.insert(&db).await.expect("Failed to seed language");

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);
    service
        .assign_language_to_user(&ctx, user_id, language_id)
        .await
        .unwrap();

    let result = service
        .remove_language_from_user(&ctx, user_id, language_id)
        .await;
    assert!(result.is_ok());

    let languages = service.list_user_languages(&ctx, user_id).await.unwrap();
    assert_eq!(languages.len(), 0);
}

#[tokio::test]
async fn remove_language_from_user_is_idempotent() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let language_id = Uuid::new_v4();

    seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let now = OffsetDateTime::now_utc();
    let lang_am = LanguageAM {
        id: Set(language_id),
        tenant_id: Set(tenant_id),
        code: Set("en".to_string()),
        name: Set("English".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    lang_am.insert(&db).await.expect("Failed to seed language");

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);
    service
        .assign_language_to_user(&ctx, user_id, language_id)
        .await
        .unwrap();

    let result1 = service
        .remove_language_from_user(&ctx, user_id, language_id)
        .await;
    assert!(result1.is_ok());

    let result2 = service
        .remove_language_from_user(&ctx, user_id, language_id)
        .await;
    assert!(
        result2.is_ok(),
        "Second removal should succeed (idempotent)"
    );

    let languages = service.list_user_languages(&ctx, user_id).await.unwrap();
    assert_eq!(languages.len(), 0);
}

#[tokio::test]
async fn list_user_languages_returns_empty_when_none() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);
    let result = service.list_user_languages(&ctx, user_id).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn list_user_languages_returns_multiple() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let now = OffsetDateTime::now_utc();
    let mut lang_ids = Vec::new();
    for i in 0..3 {
        let lang_id = Uuid::new_v4();
        let lang_am = LanguageAM {
            id: Set(lang_id),
            tenant_id: Set(tenant_id),
            code: Set(format!("l{}", i)),
            name: Set(format!("Language {}", i)),
            created_at: Set(now),
            updated_at: Set(now),
        };
        lang_am.insert(&db).await.expect("Failed to seed language");
        lang_ids.push(lang_id);
    }

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_allow_tenants(&[tenant_id]);
    for lang_id in &lang_ids {
        service
            .assign_language_to_user(&ctx, user_id, *lang_id)
            .await
            .unwrap();
    }

    let result = service.list_user_languages(&ctx, user_id).await;
    assert!(result.is_ok());
    let languages = result.unwrap();
    assert_eq!(languages.len(), 3);
}

// ==================== Security Scope Tests ====================

#[tokio::test]
async fn deny_all_context_blocks_all_operations() {
    let db = inmem_db().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let city_id = Uuid::new_v4();
    let language_id = Uuid::new_v4();

    seed_user(&db, user_id, tenant_id, "test@example.com", "Test User").await;

    let now = OffsetDateTime::now_utc();
    let city_am = CityAM {
        id: Set(city_id),
        tenant_id: Set(tenant_id),
        name: Set("Test City".to_string()),
        country: Set("Test".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    city_am.insert(&db).await.expect("Failed to seed city");

    let lang_am = LanguageAM {
        id: Set(language_id),
        tenant_id: Set(tenant_id),
        code: Set("en".to_string()),
        name: Set("English".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    lang_am.insert(&db).await.expect("Failed to seed language");

    let sec = SecureConn::new(db);
    let service = Service::new(
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    );

    let ctx = ctx_deny_all();

    let city_result = service.get_city(&ctx, city_id).await;
    assert!(
        city_result.is_err(),
        "Deny-all context should block city access"
    );

    let lang_result = service.get_language(&ctx, language_id).await;
    assert!(
        lang_result.is_err(),
        "Deny-all context should block language access"
    );

    let user_result = service.get_user(&ctx, user_id).await;
    assert!(
        user_result.is_err(),
        "Deny-all context should block user access"
    );
}
