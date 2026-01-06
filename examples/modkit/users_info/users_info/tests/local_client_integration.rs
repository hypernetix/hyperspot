#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::str_to_string)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::default_trait_access)]

//! Integration tests for `UsersInfoLocalClient`.
//!
//! These tests verify that the local client correctly delegates to the domain service
//! and properly converts errors from `DomainError` to `UsersInfoError`.

mod support;

use modkit_odata::ODataQuery;
use std::sync::Arc;
use support::{ctx_root, TestContext};
use user_info_sdk::{
    AddressPatch, CityPatch, LanguagePatch, NewAddress, NewCity, NewLanguage, NewUser,
    UpdateAddressRequest, UpdateCityRequest, UpdateLanguageRequest, UpdateUserRequest, UserPatch,
    UsersInfoClient, UsersInfoError,
};
use users_info::local_client::UsersInfoLocalClient;
use uuid::Uuid;

/// Helper to create a local client from test context
fn create_client(ctx: &TestContext) -> UsersInfoLocalClient {
    UsersInfoLocalClient::new(ctx.service.clone())
}

#[tokio::test]
async fn test_user_crud_operations() {
    let test_ctx = TestContext::new().await;
    let client = create_client(&test_ctx);
    let ctx = ctx_root();
    let tenant_id = Uuid::new_v4();

    let new_user = NewUser {
        id: None,
        tenant_id,
        email: "test@example.com".to_string(),
        display_name: "Test User".to_string(),
    };

    let created = client.create_user(&ctx, new_user).await.unwrap();
    assert_eq!(created.email, "test@example.com");
    assert_eq!(created.display_name, "Test User");
    assert_eq!(created.tenant_id, tenant_id);

    let fetched = client.get_user(&ctx, created.id).await.unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.email, created.email);

    let update_req = UpdateUserRequest {
        id: created.id,
        patch: UserPatch {
            email: Some("updated@example.com".to_string()),
            display_name: Some("Updated User".to_string()),
        },
    };
    let updated = client.update_user(&ctx, update_req).await.unwrap();
    assert_eq!(updated.email, "updated@example.com");
    assert_eq!(updated.display_name, "Updated User");

    client.delete_user(&ctx, created.id).await.unwrap();

    let result = client.get_user(&ctx, created.id).await;
    assert!(matches!(result, Err(UsersInfoError::NotFound { .. })));
}

#[tokio::test]
async fn test_list_users_with_pagination() {
    let test_ctx = TestContext::new().await;
    let client = create_client(&test_ctx);
    let ctx = ctx_root();
    let tenant_id = Uuid::new_v4();

    for i in 0..5 {
        let new_user = NewUser {
            id: None,
            tenant_id,
            email: format!("user{}@example.com", i),
            display_name: format!("User {}", i),
        };
        client.create_user(&ctx, new_user).await.unwrap();
    }

    let query = ODataQuery {
        filter: None,
        order: Default::default(),
        limit: Some(10),
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let page = client.list_users(&ctx, query).await.unwrap();
    assert_eq!(page.items.len(), 5);
}

#[tokio::test]
async fn test_city_crud_operations() {
    let test_ctx = TestContext::new().await;
    let client = create_client(&test_ctx);
    let ctx = ctx_root();
    let tenant_id = Uuid::new_v4();

    let new_city = NewCity {
        id: None,
        tenant_id,
        name: "San Francisco".to_string(),
        country: "USA".to_string(),
    };

    let created = client.create_city(&ctx, new_city).await.unwrap();
    assert_eq!(created.name, "San Francisco");
    assert_eq!(created.country, "USA");

    let fetched = client.get_city(&ctx, created.id).await.unwrap();
    assert_eq!(fetched.id, created.id);

    let update_req = UpdateCityRequest {
        id: created.id,
        patch: CityPatch {
            name: Some("Los Angeles".to_string()),
            country: Some("United States".to_string()),
        },
    };
    let updated = client.update_city(&ctx, update_req).await.unwrap();
    assert_eq!(updated.name, "Los Angeles");
    assert_eq!(updated.country, "United States");

    client.delete_city(&ctx, created.id).await.unwrap();

    let result = client.get_city(&ctx, created.id).await;
    assert!(matches!(result, Err(UsersInfoError::NotFound { .. })));
}

#[tokio::test]
async fn test_list_cities() {
    let test_ctx = TestContext::new().await;
    let client = create_client(&test_ctx);
    let ctx = ctx_root();
    let tenant_id = Uuid::new_v4();

    for i in 0..3 {
        let new_city = NewCity {
            id: None,
            tenant_id,
            name: format!("City {}", i),
            country: "Country".to_string(),
        };
        client.create_city(&ctx, new_city).await.unwrap();
    }

    let query = ODataQuery {
        filter: None,
        order: Default::default(),
        limit: Some(10),
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let page = client.list_cities(&ctx, query).await.unwrap();
    assert_eq!(page.items.len(), 3);
}

#[tokio::test]
async fn test_language_crud_operations() {
    let test_ctx = TestContext::new().await;
    let client = create_client(&test_ctx);
    let ctx = ctx_root();
    let tenant_id = Uuid::new_v4();

    let new_language = NewLanguage {
        id: None,
        tenant_id,
        code: "en".to_string(),
        name: "English".to_string(),
    };

    let created = client.create_language(&ctx, new_language).await.unwrap();
    assert_eq!(created.code, "en");
    assert_eq!(created.name, "English");

    let fetched = client.get_language(&ctx, created.id).await.unwrap();
    assert_eq!(fetched.id, created.id);

    let update_req = UpdateLanguageRequest {
        id: created.id,
        patch: LanguagePatch {
            code: Some("en-US".to_string()),
            name: Some("English (US)".to_string()),
        },
    };
    let updated = client.update_language(&ctx, update_req).await.unwrap();
    assert_eq!(updated.code, "en-US");
    assert_eq!(updated.name, "English (US)");

    client.delete_language(&ctx, created.id).await.unwrap();

    let result = client.get_language(&ctx, created.id).await;
    assert!(matches!(result, Err(UsersInfoError::NotFound { .. })));
}

#[tokio::test]
async fn test_list_languages() {
    let test_ctx = TestContext::new().await;
    let client = create_client(&test_ctx);
    let ctx = ctx_root();
    let tenant_id = Uuid::new_v4();

    for (code, name) in [("en", "English"), ("es", "Spanish"), ("fr", "French")] {
        let new_language = NewLanguage {
            id: None,
            tenant_id,
            code: code.to_string(),
            name: name.to_string(),
        };
        client.create_language(&ctx, new_language).await.unwrap();
    }

    let query = ODataQuery {
        filter: None,
        order: Default::default(),
        limit: Some(10),
        cursor: None,
        filter_hash: None,
        select: None,
    };

    let page = client.list_languages(&ctx, query).await.unwrap();
    assert_eq!(page.items.len(), 3);
}

#[tokio::test]
async fn test_address_crud_operations() {
    let test_ctx = TestContext::new().await;
    let client = create_client(&test_ctx);
    let ctx = ctx_root();
    let tenant_id = Uuid::new_v4();

    let user = client
        .create_user(
            &ctx,
            NewUser {
                id: None,
                tenant_id,
                email: "user@example.com".to_string(),
                display_name: "User".to_string(),
            },
        )
        .await
        .unwrap();

    let city = client
        .create_city(
            &ctx,
            NewCity {
                id: None,
                tenant_id,
                name: "New York".to_string(),
                country: "USA".to_string(),
            },
        )
        .await
        .unwrap();

    let new_address = NewAddress {
        id: None,
        tenant_id,
        user_id: user.id,
        city_id: city.id,
        street: "123 Main St".to_string(),
        postal_code: "10001".to_string(),
    };

    let created = client.create_address(&ctx, new_address).await.unwrap();
    assert_eq!(created.user_id, user.id);
    assert_eq!(created.city_id, city.id);
    assert_eq!(created.street, "123 Main St");
    assert_eq!(created.postal_code, "10001");

    let fetched = client.get_address(&ctx, created.id).await.unwrap();
    assert_eq!(fetched.id, created.id);

    let by_user = client
        .get_address_by_user(&ctx, user.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(by_user.id, created.id);

    let update_req = UpdateAddressRequest {
        id: created.id,
        patch: AddressPatch {
            city_id: None,
            street: Some("456 Oak Ave".to_string()),
            postal_code: Some("10002".to_string()),
        },
    };
    let updated = client.update_address(&ctx, update_req).await.unwrap();
    assert_eq!(updated.street, "456 Oak Ave");
    assert_eq!(updated.postal_code, "10002");

    client.delete_address(&ctx, created.id).await.unwrap();

    let result = client.get_address(&ctx, created.id).await;
    assert!(matches!(result, Err(UsersInfoError::NotFound { .. })));
}

#[tokio::test]
async fn test_get_address_by_user_not_found() {
    let test_ctx = TestContext::new().await;
    let client = create_client(&test_ctx);
    let ctx = ctx_root();
    let tenant_id = Uuid::new_v4();

    let user = client
        .create_user(
            &ctx,
            NewUser {
                id: None,
                tenant_id,
                email: "user@example.com".to_string(),
                display_name: "User".to_string(),
            },
        )
        .await
        .unwrap();

    let result = client.get_address_by_user(&ctx, user.id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_user_language_relationships() {
    let test_ctx = TestContext::new().await;
    let client = create_client(&test_ctx);
    let ctx = ctx_root();
    let tenant_id = Uuid::new_v4();

    let user = client
        .create_user(
            &ctx,
            NewUser {
                id: None,
                tenant_id,
                email: "user@example.com".to_string(),
                display_name: "User".to_string(),
            },
        )
        .await
        .unwrap();

    let lang1 = client
        .create_language(
            &ctx,
            NewLanguage {
                id: None,
                tenant_id,
                code: "en".to_string(),
                name: "English".to_string(),
            },
        )
        .await
        .unwrap();

    let lang2 = client
        .create_language(
            &ctx,
            NewLanguage {
                id: None,
                tenant_id,
                code: "es".to_string(),
                name: "Spanish".to_string(),
            },
        )
        .await
        .unwrap();

    let languages = client.list_user_languages(&ctx, user.id).await.unwrap();
    assert_eq!(languages.len(), 0);

    client
        .assign_language_to_user(&ctx, user.id, lang1.id)
        .await
        .unwrap();
    client
        .assign_language_to_user(&ctx, user.id, lang2.id)
        .await
        .unwrap();

    let languages = client.list_user_languages(&ctx, user.id).await.unwrap();
    assert_eq!(languages.len(), 2);
    let codes: Vec<String> = languages.iter().map(|l| l.code.clone()).collect();
    assert!(codes.contains(&"en".to_string()));
    assert!(codes.contains(&"es".to_string()));

    client
        .assign_language_to_user(&ctx, user.id, lang1.id)
        .await
        .unwrap();
    let languages = client.list_user_languages(&ctx, user.id).await.unwrap();
    assert_eq!(languages.len(), 2);

    client
        .remove_language_from_user(&ctx, user.id, lang1.id)
        .await
        .unwrap();
    let languages = client.list_user_languages(&ctx, user.id).await.unwrap();
    assert_eq!(languages.len(), 1);
    assert_eq!(languages[0].code, "es");

    client
        .remove_language_from_user(&ctx, user.id, lang1.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_error_conversion_not_found() {
    let test_ctx = TestContext::new().await;
    let client = create_client(&test_ctx);
    let ctx = ctx_root();
    let non_existent_id = Uuid::new_v4();

    let result = client.get_user(&ctx, non_existent_id).await;
    assert!(matches!(result, Err(UsersInfoError::NotFound { id }) if id == non_existent_id));
}

#[tokio::test]
async fn test_error_conversion_validation() {
    let test_ctx = TestContext::new().await;
    let client = create_client(&test_ctx);
    let ctx = ctx_root();
    let tenant_id = Uuid::new_v4();

    let new_user = NewUser {
        id: None,
        tenant_id,
        email: "invalid-email".to_string(),
        display_name: "Test User".to_string(),
    };

    let result = client.create_user(&ctx, new_user).await;
    assert!(matches!(result, Err(UsersInfoError::Validation { .. })));
}

#[tokio::test]
async fn test_error_conversion_conflict() {
    let test_ctx = TestContext::new().await;
    let client = create_client(&test_ctx);
    let ctx = ctx_root();
    let tenant_id = Uuid::new_v4();

    let new_user = NewUser {
        id: None,
        tenant_id,
        email: "duplicate@example.com".to_string(),
        display_name: "User 1".to_string(),
    };
    client.create_user(&ctx, new_user.clone()).await.unwrap();

    let duplicate_user = NewUser {
        id: None,
        tenant_id,
        email: "duplicate@example.com".to_string(),
        display_name: "User 2".to_string(),
    };
    let result = client.create_user(&ctx, duplicate_user).await;
    assert!(matches!(result, Err(UsersInfoError::Conflict { .. })));
}

#[tokio::test]
async fn test_client_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<UsersInfoLocalClient>();
}

#[tokio::test]
async fn test_client_can_be_wrapped_in_arc() {
    let test_ctx = TestContext::new().await;
    let client = create_client(&test_ctx);
    let arc_client: Arc<dyn UsersInfoClient> = Arc::new(client);

    let ctx = ctx_root();
    let tenant_id = Uuid::new_v4();

    let new_user = NewUser {
        id: None,
        tenant_id,
        email: "arc@example.com".to_string(),
        display_name: "Arc User".to_string(),
    };

    let created = arc_client.create_user(&ctx, new_user).await.unwrap();
    assert_eq!(created.email, "arc@example.com");
}
