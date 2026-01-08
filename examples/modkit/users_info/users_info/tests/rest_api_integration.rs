//! Integration tests for the complete REST API
//!
//! Tests all endpoints: Users, Cities, Languages, Addresses, and User-Language relationships

#![allow(clippy::str_to_string)]
#![allow(clippy::uninlined_format_args)]

use modkit_security::SecurityContext;
use uuid::Uuid;

mod support;
use support::TestContext;

// ==================== City Tests ====================

#[tokio::test]
async fn test_city_crud_operations() {
    let ctx = TestContext::new().await;
    let tenant_id = Uuid::new_v4();
    let sec_ctx = SecurityContext::root();

    // Create a city
    let city_id = Uuid::new_v4();
    let new_city = user_info_sdk::NewCity {
        id: Some(city_id),
        tenant_id,
        name: "San Francisco".to_string(),
        country: "USA".to_string(),
    };

    let created = ctx
        .service
        .create_city(&sec_ctx, new_city)
        .await
        .expect("Failed to create city");
    assert_eq!(created.id, city_id);
    assert_eq!(created.name, "San Francisco");
    assert_eq!(created.country, "USA");

    // Get the city
    let fetched = ctx
        .service
        .get_city(&sec_ctx, city_id)
        .await
        .expect("Failed to get city");
    assert_eq!(fetched.id, city_id);
    assert_eq!(fetched.name, "San Francisco");

    // Update the city
    let patch = user_info_sdk::CityPatch {
        name: Some("San Jose".to_string()),
        country: None,
    };
    let updated = ctx
        .service
        .update_city(&sec_ctx, city_id, patch)
        .await
        .expect("Failed to update city");
    assert_eq!(updated.name, "San Jose");
    assert_eq!(updated.country, "USA");

    // List cities
    let query = modkit_odata::ODataQuery::default();
    let page = ctx
        .service
        .list_cities_page(&sec_ctx, &query)
        .await
        .expect("Failed to list cities");
    assert!(!page.items.is_empty());

    // Delete the city
    ctx.service
        .delete_city(&sec_ctx, city_id)
        .await
        .expect("Failed to delete city");

    // Verify deletion
    let result = ctx.service.get_city(&sec_ctx, city_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_city_not_found() {
    let ctx = TestContext::new().await;
    let sec_ctx = SecurityContext::root();
    let non_existent_id = Uuid::new_v4();

    let result = ctx.service.get_city(&sec_ctx, non_existent_id).await;
    assert!(result.is_err());
}

// ==================== Language Tests ====================

#[tokio::test]
async fn test_language_crud_operations() {
    let ctx = TestContext::new().await;
    let tenant_id = Uuid::new_v4();
    let sec_ctx = SecurityContext::root();

    // Create a language
    let language_id = Uuid::new_v4();
    let new_language = user_info_sdk::NewLanguage {
        id: Some(language_id),
        tenant_id,
        code: "en".to_string(),
        name: "English".to_string(),
    };

    let created = ctx
        .service
        .create_language(&sec_ctx, new_language)
        .await
        .expect("Failed to create language");
    assert_eq!(created.id, language_id);
    assert_eq!(created.code, "en");
    assert_eq!(created.name, "English");

    // Get the language
    let fetched = ctx
        .service
        .get_language(&sec_ctx, language_id)
        .await
        .expect("Failed to get language");
    assert_eq!(fetched.id, language_id);
    assert_eq!(fetched.code, "en");

    // Update the language
    let patch = user_info_sdk::LanguagePatch {
        code: Some("en-US".to_string()),
        name: Some("English (US)".to_string()),
    };
    let updated = ctx
        .service
        .update_language(&sec_ctx, language_id, patch)
        .await
        .expect("Failed to update language");
    assert_eq!(updated.code, "en-US");
    assert_eq!(updated.name, "English (US)");

    // List languages
    let query = modkit_odata::ODataQuery::default();
    let page = ctx
        .service
        .list_languages_page(&sec_ctx, &query)
        .await
        .expect("Failed to list languages");
    assert!(!page.items.is_empty());

    // Delete the language
    ctx.service
        .delete_language(&sec_ctx, language_id)
        .await
        .expect("Failed to delete language");

    // Verify deletion
    let result = ctx.service.get_language(&sec_ctx, language_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_language_not_found() {
    let ctx = TestContext::new().await;
    let sec_ctx = SecurityContext::root();
    let non_existent_id = Uuid::new_v4();

    let result = ctx.service.get_language(&sec_ctx, non_existent_id).await;
    assert!(result.is_err());
}

// ==================== Address Tests ====================

#[tokio::test]
async fn test_address_operations() {
    let ctx = TestContext::new().await;
    let tenant_id = Uuid::new_v4();
    let sec_ctx = SecurityContext::root();

    // Create a user first
    let user_id = Uuid::new_v4();
    let new_user = user_info_sdk::NewUser {
        id: Some(user_id),
        tenant_id,
        email: "user@example.com".to_string(),
        display_name: "Test User".to_string(),
    };
    ctx.service
        .create_user(&sec_ctx, new_user)
        .await
        .expect("Failed to create user");

    // Create a city for the address
    let city_id = Uuid::new_v4();
    let new_city = user_info_sdk::NewCity {
        id: Some(city_id),
        tenant_id,
        name: "New York".to_string(),
        country: "USA".to_string(),
    };
    ctx.service
        .create_city(&sec_ctx, new_city)
        .await
        .expect("Failed to create city");

    // Initially, user should have no address
    let initial_address = ctx
        .service
        .get_user_address(&sec_ctx, user_id)
        .await
        .expect("Failed to get user address");
    assert!(initial_address.is_none());

    // Create an address for the user (PUT = upsert)
    let new_address = user_info_sdk::NewAddress {
        id: None,
        tenant_id,
        user_id,
        city_id,
        street: "123 Main St".to_string(),
        postal_code: "10001".to_string(),
    };
    let created = ctx
        .service
        .put_user_address(&sec_ctx, user_id, new_address)
        .await
        .expect("Failed to create address");
    assert_eq!(created.user_id, user_id);
    assert_eq!(created.city_id, city_id);
    assert_eq!(created.street, "123 Main St");

    // Get the address
    let fetched = ctx
        .service
        .get_user_address(&sec_ctx, user_id)
        .await
        .expect("Failed to get address")
        .expect("Address should exist");
    assert_eq!(fetched.user_id, user_id);
    assert_eq!(fetched.street, "123 Main St");

    // Update the address (PUT again = replace)
    let updated_address = user_info_sdk::NewAddress {
        id: None,
        tenant_id,
        user_id,
        city_id,
        street: "456 Oak Ave".to_string(),
        postal_code: "10002".to_string(),
    };
    let updated = ctx
        .service
        .put_user_address(&sec_ctx, user_id, updated_address)
        .await
        .expect("Failed to update address");
    assert_eq!(updated.street, "456 Oak Ave");
    assert_eq!(updated.postal_code, "10002");

    // Delete the address
    ctx.service
        .delete_user_address(&sec_ctx, user_id)
        .await
        .expect("Failed to delete address");

    // Verify deletion
    let after_delete = ctx
        .service
        .get_user_address(&sec_ctx, user_id)
        .await
        .expect("Failed to get address after delete");
    assert!(after_delete.is_none());
}

#[tokio::test]
async fn test_address_for_nonexistent_user() {
    let ctx = TestContext::new().await;
    let sec_ctx = SecurityContext::root();
    let non_existent_user = Uuid::new_v4();
    let city_id = Uuid::new_v4();

    let new_address = user_info_sdk::NewAddress {
        id: None,
        tenant_id: Uuid::new_v4(),
        user_id: non_existent_user,
        city_id,
        street: "123 Main St".to_string(),
        postal_code: "10001".to_string(),
    };

    let result = ctx
        .service
        .put_user_address(&sec_ctx, non_existent_user, new_address)
        .await;
    assert!(result.is_err());
}

// ==================== User-Language Relationship Tests ====================

#[tokio::test]
async fn test_user_language_operations() {
    let ctx = TestContext::new().await;
    let tenant_id = Uuid::new_v4();
    let sec_ctx = SecurityContext::root();

    // Create a user
    let user_id = Uuid::new_v4();
    let new_user = user_info_sdk::NewUser {
        id: Some(user_id),
        tenant_id,
        email: "polyglot@example.com".to_string(),
        display_name: "Polyglot User".to_string(),
    };
    ctx.service
        .create_user(&sec_ctx, new_user)
        .await
        .expect("Failed to create user");

    // Create languages
    let lang1_id = Uuid::new_v4();
    let lang1 = user_info_sdk::NewLanguage {
        id: Some(lang1_id),
        tenant_id,
        code: "en".to_string(),
        name: "English".to_string(),
    };
    ctx.service
        .create_language(&sec_ctx, lang1)
        .await
        .expect("Failed to create language 1");

    let lang2_id = Uuid::new_v4();
    let lang2 = user_info_sdk::NewLanguage {
        id: Some(lang2_id),
        tenant_id,
        code: "es".to_string(),
        name: "Spanish".to_string(),
    };
    ctx.service
        .create_language(&sec_ctx, lang2)
        .await
        .expect("Failed to create language 2");

    // Initially, user should have no languages
    let initial_languages = ctx
        .service
        .list_user_languages(&sec_ctx, user_id)
        .await
        .expect("Failed to list user languages");
    assert!(initial_languages.is_empty());

    // Assign first language to user
    ctx.service
        .assign_language_to_user(&sec_ctx, user_id, lang1_id)
        .await
        .expect("Failed to assign language 1");

    // Verify assignment
    let languages = ctx
        .service
        .list_user_languages(&sec_ctx, user_id)
        .await
        .expect("Failed to list user languages");
    assert_eq!(languages.len(), 1);
    assert_eq!(languages[0].id, lang1_id);

    // Assign second language to user
    ctx.service
        .assign_language_to_user(&sec_ctx, user_id, lang2_id)
        .await
        .expect("Failed to assign language 2");

    // Verify both languages
    let languages = ctx
        .service
        .list_user_languages(&sec_ctx, user_id)
        .await
        .expect("Failed to list user languages");
    assert_eq!(languages.len(), 2);

    // Idempotent assignment (should not fail)
    ctx.service
        .assign_language_to_user(&sec_ctx, user_id, lang1_id)
        .await
        .expect("Failed to assign language 1 again (idempotent)");

    // Still should have 2 languages
    let languages = ctx
        .service
        .list_user_languages(&sec_ctx, user_id)
        .await
        .expect("Failed to list user languages");
    assert_eq!(languages.len(), 2);

    // Remove first language
    ctx.service
        .remove_language_from_user(&sec_ctx, user_id, lang1_id)
        .await
        .expect("Failed to remove language 1");

    // Verify removal
    let languages = ctx
        .service
        .list_user_languages(&sec_ctx, user_id)
        .await
        .expect("Failed to list user languages");
    assert_eq!(languages.len(), 1);
    assert_eq!(languages[0].id, lang2_id);

    // Idempotent removal (should not fail)
    ctx.service
        .remove_language_from_user(&sec_ctx, user_id, lang1_id)
        .await
        .expect("Failed to remove language 1 again (idempotent)");

    // Still should have 1 language
    let languages = ctx
        .service
        .list_user_languages(&sec_ctx, user_id)
        .await
        .expect("Failed to list user languages");
    assert_eq!(languages.len(), 1);

    // Remove second language
    ctx.service
        .remove_language_from_user(&sec_ctx, user_id, lang2_id)
        .await
        .expect("Failed to remove language 2");

    // Verify all removed
    let languages = ctx
        .service
        .list_user_languages(&sec_ctx, user_id)
        .await
        .expect("Failed to list user languages");
    assert!(languages.is_empty());
}

#[tokio::test]
async fn test_assign_language_to_nonexistent_user() {
    let ctx = TestContext::new().await;
    let sec_ctx = SecurityContext::root();
    let non_existent_user = Uuid::new_v4();
    let language_id = Uuid::new_v4();

    let result = ctx
        .service
        .assign_language_to_user(&sec_ctx, non_existent_user, language_id)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_assign_nonexistent_language_to_user() {
    let ctx = TestContext::new().await;
    let tenant_id = Uuid::new_v4();
    let sec_ctx = SecurityContext::root();

    // Create a user
    let user_id = Uuid::new_v4();
    let new_user = user_info_sdk::NewUser {
        id: Some(user_id),
        tenant_id,
        email: "user@example.com".to_string(),
        display_name: "Test User".to_string(),
    };
    ctx.service
        .create_user(&sec_ctx, new_user)
        .await
        .expect("Failed to create user");

    let non_existent_language = Uuid::new_v4();
    let result = ctx
        .service
        .assign_language_to_user(&sec_ctx, user_id, non_existent_language)
        .await;
    assert!(result.is_err());
}

// ==================== Combined Workflow Tests ====================

#[tokio::test]
async fn test_complete_user_profile_workflow() {
    let ctx = TestContext::new().await;
    let tenant_id = Uuid::new_v4();
    let sec_ctx = SecurityContext::root();

    // 1. Create a city
    let city_id = Uuid::new_v4();
    let new_city = user_info_sdk::NewCity {
        id: Some(city_id),
        tenant_id,
        name: "Paris".to_string(),
        country: "France".to_string(),
    };
    ctx.service
        .create_city(&sec_ctx, new_city)
        .await
        .expect("Failed to create city");

    // 2. Create languages
    let french_id = Uuid::new_v4();
    let french = user_info_sdk::NewLanguage {
        id: Some(french_id),
        tenant_id,
        code: "fr".to_string(),
        name: "French".to_string(),
    };
    ctx.service
        .create_language(&sec_ctx, french)
        .await
        .expect("Failed to create French language");

    let english_id = Uuid::new_v4();
    let english = user_info_sdk::NewLanguage {
        id: Some(english_id),
        tenant_id,
        code: "en".to_string(),
        name: "English".to_string(),
    };
    ctx.service
        .create_language(&sec_ctx, english)
        .await
        .expect("Failed to create English language");

    // 3. Create a user
    let user_id = Uuid::new_v4();
    let new_user = user_info_sdk::NewUser {
        id: Some(user_id),
        tenant_id,
        email: "jean@example.com".to_string(),
        display_name: "Jean Dupont".to_string(),
    };
    let user = ctx
        .service
        .create_user(&sec_ctx, new_user)
        .await
        .expect("Failed to create user");
    assert_eq!(user.email, "jean@example.com");

    // 4. Add address to user
    let new_address = user_info_sdk::NewAddress {
        id: None,
        tenant_id,
        user_id,
        city_id,
        street: "10 Rue de la Paix".to_string(),
        postal_code: "75002".to_string(),
    };
    let address = ctx
        .service
        .put_user_address(&sec_ctx, user_id, new_address)
        .await
        .expect("Failed to create address");
    assert_eq!(address.street, "10 Rue de la Paix");

    // 5. Assign languages to user
    ctx.service
        .assign_language_to_user(&sec_ctx, user_id, french_id)
        .await
        .expect("Failed to assign French");
    ctx.service
        .assign_language_to_user(&sec_ctx, user_id, english_id)
        .await
        .expect("Failed to assign English");

    // 6. Verify complete profile
    let fetched_user = ctx
        .service
        .get_user(&sec_ctx, user_id)
        .await
        .expect("Failed to get user");
    assert_eq!(fetched_user.email, "jean@example.com");

    let fetched_address = ctx
        .service
        .get_user_address(&sec_ctx, user_id)
        .await
        .expect("Failed to get address")
        .expect("Address should exist");
    assert_eq!(fetched_address.street, "10 Rue de la Paix");

    let user_languages = ctx
        .service
        .list_user_languages(&sec_ctx, user_id)
        .await
        .expect("Failed to list languages");
    assert_eq!(user_languages.len(), 2);

    // 7. Cleanup - delete user (cascading should handle relationships)
    ctx.service
        .delete_user(&sec_ctx, user_id)
        .await
        .expect("Failed to delete user");

    // Verify user is deleted
    let result = ctx.service.get_user(&sec_ctx, user_id).await;
    assert!(result.is_err());
}
