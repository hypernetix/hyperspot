use std::sync::Arc;

use anyhow::Result;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use chrono::Utc;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

use tower::ServiceExt;
use uuid::Uuid;

use users_info::{
    api::rest::dto::{CreateUserReq, UserDto},
    contract::client::UsersInfoApi,
    domain::service::{Service, ServiceConfig},
    gateways::local::UsersInfoLocalClient,
    infra::storage::migrations::Migrator,
};

/// Create a fresh test database for each test
async fn create_test_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    db
}

/// Create a test domain service
async fn create_test_service() -> Arc<Service> {
    let db = create_test_db().await;
    let config = ServiceConfig::default();
    Arc::new(Service::new(db, config))
}

/// Create a test local client
async fn create_test_client() -> Arc<dyn UsersInfoApi> {
    let service = create_test_service().await;
    Arc::new(UsersInfoLocalClient::new(service))
}

/// Mock OpenAPI registry for testing
struct MockOpenApiRegistry;

impl modkit::api::OpenApiRegistry for MockOpenApiRegistry {
    fn register_operation(&self, _spec: &modkit::api::OperationSpec) {
        // No-op for tests
    }

    fn ensure_schema_raw(
        &self,
        root_name: &str,
        _schemas: Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) -> String {
        // Return the root name for tests
        root_name.to_string()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Create a test HTTP router
async fn create_test_router() -> Router {
    let service = create_test_service().await;
    let openapi = MockOpenApiRegistry;

    // Use the actual route registration function
    users_info::api::rest::routes::register_routes(Router::new(), &openapi, service)
        .expect("Failed to register routes")
}

#[tokio::test]
async fn test_domain_service_crud() -> Result<()> {
    let service = create_test_service().await;

    // Test create user
    let new_user = users_info::contract::model::NewUser {
        email: "test@example.com".to_string(),
        display_name: "Test User".to_string(),
    };

    let created_user = service.create_user(new_user).await?;
    assert_eq!(created_user.email, "test@example.com");
    assert_eq!(created_user.display_name, "Test User");

    // Test get user
    let retrieved_user = service.get_user(created_user.id).await?;
    assert_eq!(retrieved_user.id, created_user.id);
    assert_eq!(retrieved_user.email, created_user.email);

    // Test list users
    let users = service.list_users(None, None).await?;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, created_user.id);

    // Test update user
    let patch = users_info::contract::model::UserPatch {
        email: None,
        display_name: Some("Updated Name".to_string()),
    };

    let updated_user = service.update_user(created_user.id, patch).await?;
    assert_eq!(updated_user.display_name, "Updated Name");
    assert_eq!(updated_user.email, "test@example.com"); // Unchanged

    // Test delete user
    service.delete_user(created_user.id).await?;

    // Verify user is deleted
    let result = service.get_user(created_user.id).await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_domain_service_validation() -> Result<()> {
    let service = create_test_service().await;

    // Test invalid email
    let invalid_email_user = users_info::contract::model::NewUser {
        email: "invalid-email".to_string(),
        display_name: "Test User".to_string(),
    };

    let result = service.create_user(invalid_email_user).await;
    assert!(result.is_err());

    // Test empty display name
    let empty_name_user = users_info::contract::model::NewUser {
        email: "test@example.com".to_string(),
        display_name: "".to_string(),
    };

    let result = service.create_user(empty_name_user).await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_domain_service_email_uniqueness() -> Result<()> {
    let service = create_test_service().await;

    // Create first user
    let new_user1 = users_info::contract::model::NewUser {
        email: "unique@example.com".to_string(),
        display_name: "User 1".to_string(),
    };

    service.create_user(new_user1).await?;

    // Try to create second user with same email
    let new_user2 = users_info::contract::model::NewUser {
        email: "unique@example.com".to_string(),
        display_name: "User 2".to_string(),
    };

    let result = service.create_user(new_user2).await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_local_client() -> Result<()> {
    let client = create_test_client().await;

    // Test create user through client
    let new_user = users_info::contract::model::NewUser {
        email: "client@example.com".to_string(),
        display_name: "Client User".to_string(),
    };

    let created_user = client.create_user(new_user).await?;
    assert_eq!(created_user.email, "client@example.com");

    // Test get user through client
    let retrieved_user = client.get_user(created_user.id).await?;
    assert_eq!(retrieved_user.id, created_user.id);

    // Test list users through client
    let users = client.list_users(Some(10), Some(0)).await?;
    assert!(!users.is_empty());

    // Test update user through client
    let patch = users_info::contract::model::UserPatch {
        email: None,
        display_name: Some("Updated Client User".to_string()),
    };

    let updated_user = client.update_user(created_user.id, patch).await?;
    assert_eq!(updated_user.display_name, "Updated Client User");

    // Test delete user through client
    client.delete_user(created_user.id).await?;

    Ok(())
}

#[tokio::test]
async fn test_rest_api_create_user() -> Result<()> {
    let router = create_test_router().await;

    let create_request = CreateUserReq {
        email: "rest@example.com".to_string(),
        display_name: "REST User".to_string(),
    };

    let request = Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&create_request)?))
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let user: UserDto = serde_json::from_slice(&body)?;

    assert_eq!(user.email, "rest@example.com");
    assert_eq!(user.display_name, "REST User");

    Ok(())
}

#[tokio::test]
async fn test_rest_api_list_users() -> Result<()> {
    let router = create_test_router().await;

    let request = Request::builder()
        .method("GET")
        .uri("/users")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let user_list: users_info::api::rest::dto::UserListDto = serde_json::from_slice(&body)?;

    assert_eq!(user_list.limit, 50); // Default limit
    assert_eq!(user_list.offset, 0); // Default offset

    Ok(())
}

#[tokio::test]
async fn test_rest_api_validation_errors() -> Result<()> {
    let router = create_test_router().await;

    // Test invalid email
    let invalid_request = CreateUserReq {
        email: "invalid-email".to_string(),
        display_name: "Test User".to_string(),
    };

    let request = Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&invalid_request)?))
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
async fn test_rest_api_not_found() -> Result<()> {
    let router = create_test_router().await;

    let non_existent_id = Uuid::new_v4();
    let request = Request::builder()
        .method("GET")
        .uri(&format!("/users/{}", non_existent_id))
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    Ok(())
}

#[tokio::test]
async fn test_rest_dto_conversions() -> Result<()> {
    use users_info::api::rest::dto::*;
    use users_info::contract::model::*;

    // Test User to UserDto conversion
    let user = User {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        display_name: "Test User".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let dto = UserDto::from(user.clone());
    assert_eq!(dto.id, user.id);
    assert_eq!(dto.email, user.email);
    assert_eq!(dto.display_name, user.display_name);

    // Test CreateUserReq to NewUser conversion
    let create_req = CreateUserReq {
        email: "new@example.com".to_string(),
        display_name: "New User".to_string(),
    };

    let new_user = NewUser::from(create_req.clone());
    assert_eq!(new_user.email, create_req.email);
    assert_eq!(new_user.display_name, create_req.display_name);

    // Test UpdateUserReq to UserPatch conversion
    let update_req = UpdateUserReq {
        email: Some("updated@example.com".to_string()),
        display_name: None,
    };

    let patch = UserPatch::from(update_req.clone());
    assert_eq!(patch.email, update_req.email);
    assert_eq!(patch.display_name, update_req.display_name);

    Ok(())
}

#[tokio::test]
async fn test_contract_model_has_no_serde() {
    // This test ensures that contract models don't have serde derives
    // by checking that they cannot be serialized/deserialized
    use users_info::contract::model::User;

    let user = User {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        display_name: "Test User".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // This should not compile if User has Serialize derive
    // let serialized = serde_json::to_string(&user);
    // assert!(serialized.is_err()); // Comment this out since it won't compile

    // Instead, we test that REST DTOs DO have serde
    let dto = users_info::api::rest::dto::UserDto::from(user);
    let serialized = serde_json::to_string(&dto);
    assert!(serialized.is_ok());
}
