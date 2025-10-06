//! Integration-style tests for the users_info module.
//!
//! Key points:
//! - Each test runs on a fresh in-memory SQLite DB and applies migrations.
//! - Service is constructed with a SeaORM-backed repository (Domain Port + Adapter).
//! - Local client is tested against the same Service.
//! - REST layer is exercised via an Axum Router registered through real routes.

use std::sync::Arc;

use anyhow::Result;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use chrono::Utc;
use modkit::SseBroadcaster;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use tower::ServiceExt;
use uuid::Uuid;

use users_info::{
    api::rest::dto::{CreateUserReq, UserDto, UserEvent},
    api::rest::sse_adapter::SseUserEventPublisher,
    contract::client::UsersInfoApi,
    domain::{
        error::DomainError,
        events::UserDomainEvent,
        ports::{AuditPort, EventPublisher},
        service::{Service, ServiceConfig},
    },
    gateways::local::UsersInfoLocalClient,
    infra::storage::{
        migrations::Migrator,
        sea_orm_repo::SeaOrmUsersRepository, // <-- SeaORM adapter (implements UsersRepository)
    },
};

/// Create a fresh test database for each test (in-memory SQLite) and run migrations.
async fn create_test_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to test database");
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");
    db
}

/// Build the domain Service with a SeaORM-backed repository.
async fn create_test_service() -> Arc<Service> {
    let db = create_test_db().await;
    let repo = SeaOrmUsersRepository::new(db);
    let events: Arc<dyn EventPublisher<UserDomainEvent>> = Arc::new(MockEventPublisher);
    let audit: Arc<dyn AuditPort> = Arc::new(MockAuditPort);
    let config = ServiceConfig::default();
    Arc::new(Service::new(Arc::new(repo), events, audit, config))
}

/// Build a local in-process client on top of the Service.
async fn create_test_client() -> Arc<dyn UsersInfoApi> {
    let service = create_test_service().await;
    Arc::new(UsersInfoLocalClient::new(service))
}

/// Mock audit port for tests - always succeeds
struct MockAuditPort;

#[async_trait::async_trait]
impl AuditPort for MockAuditPort {
    async fn get_user_access(&self, _id: Uuid) -> Result<(), DomainError> {
        Ok(())
    }

    async fn notify_user_created(&self) -> Result<(), DomainError> {
        Ok(())
    }
}

/// Mock event publisher for tests - just ignores events
struct MockEventPublisher;

impl EventPublisher<UserDomainEvent> for MockEventPublisher {
    fn publish(&self, _event: &UserDomainEvent) {
        // no-op in tests
    }
}

/// Minimal OpenAPI registry stub for tests.
///
/// We only need it to satisfy the route registration path. It records nothing.
struct MockOpenApiRegistry;

impl modkit::api::OpenApiRegistry for MockOpenApiRegistry {
    fn register_operation(&self, _spec: &modkit::api::OperationSpec) {
        // no-op in tests
    }

    // Some versions of the trait include this convenience helper;
    // keep it to stay source-compatible with your current codebase.
    fn ensure_schema_raw(
        &self,
        root_name: &str,
        _schemas: Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) -> String {
        root_name.to_string()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Build an Axum router by calling the real route registration.
async fn create_test_router() -> Router {
    let service = create_test_service().await;
    let openapi = MockOpenApiRegistry;
    users_info::api::rest::routes::register_routes(Router::new(), &openapi, service)
        .expect("Failed to register routes")
}

#[tokio::test]
async fn test_domain_service_crud() -> Result<()> {
    let service = create_test_service().await;

    // create
    let new_user = users_info::contract::model::NewUser {
        email: "test@example.com".to_string(),
        display_name: "Test User".to_string(),
    };
    let created_user = service.create_user(new_user).await?;
    assert_eq!(created_user.email, "test@example.com");
    assert_eq!(created_user.display_name, "Test User");

    // get
    let retrieved_user = service.get_user(created_user.id).await?;
    assert_eq!(retrieved_user.id, created_user.id);
    assert_eq!(retrieved_user.email, created_user.email);

    // list
    let page = service
        .list_users_page(modkit::api::odata::ODataQuery::default())
        .await?;
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].id, created_user.id);

    // update
    let patch = users_info::contract::model::UserPatch {
        email: None,
        display_name: Some("Updated Name".to_string()),
    };
    let updated_user = service.update_user(created_user.id, patch).await?;
    assert_eq!(updated_user.display_name, "Updated Name");
    assert_eq!(updated_user.email, "test@example.com");

    // delete
    service.delete_user(created_user.id).await?;
    let result = service.get_user(created_user.id).await;
    assert!(result.is_err(), "user should be gone");

    Ok(())
}

#[tokio::test]
async fn test_domain_service_validation() -> Result<()> {
    let service = create_test_service().await;

    // invalid email
    let invalid_email_user = users_info::contract::model::NewUser {
        email: "invalid-email".to_string(),
        display_name: "Test User".to_string(),
    };
    let result = service.create_user(invalid_email_user).await;
    assert!(result.is_err());

    // empty display name
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

    // first
    let new_user1 = users_info::contract::model::NewUser {
        email: "unique@example.com".to_string(),
        display_name: "User 1".to_string(),
    };
    service.create_user(new_user1).await?;

    // second with same email -> error
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

    // create
    let new_user = users_info::contract::model::NewUser {
        email: "client@example.com".to_string(),
        display_name: "Client User".to_string(),
    };
    let created_user = client.create_user(new_user).await?;
    assert_eq!(created_user.email, "client@example.com");

    // get
    let retrieved_user = client.get_user(created_user.id).await?;
    assert_eq!(retrieved_user.id, created_user.id);

    // list
    let page = client
        .list_users(modkit::api::odata::ODataQuery::default())
        .await?;
    assert!(!page.items.is_empty());

    // update
    let patch = users_info::contract::model::UserPatch {
        email: None,
        display_name: Some("Updated Client User".to_string()),
    };
    let updated_user = client.update_user(created_user.id, patch).await?;
    assert_eq!(updated_user.display_name, "Updated Client User");

    // delete
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
    let user_page: modkit_odata::Page<users_info::api::rest::dto::UserDto> =
        serde_json::from_slice(&body)?;

    // default paging is defined by pagination LimitCfg::default()
    assert_eq!(user_page.page_info.limit, 25);
    assert!(user_page.page_info.next_cursor.is_none()); // No more pages

    Ok(())
}

#[tokio::test]
async fn test_rest_api_validation_errors() -> Result<()> {
    let router = create_test_router().await;

    // invalid email
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
async fn test_rest_api_invalid_odata_filter() -> Result<()> {
    let router = create_test_router().await;

    // Create a user first to have some data
    let create_request = CreateUserReq {
        email: "test@example.com".to_string(),
        display_name: "Test User".to_string(),
    };

    let request = Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&create_request)?))
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Now test with invalid OData filter - unknown field
    let request = Request::builder()
        .method("GET")
        .uri("/users?%24filter=unknown_field%20eq%20%27test%27")
        .body(Body::empty())
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    // OData errors return 422 (Unprocessable Entity) per GTS catalog
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let problem: serde_json::Value = serde_json::from_slice(&body)?;

    // Verify it's the correct error type for invalid filter (GTS catalog format)
    assert_eq!(
        problem["type"],
        "https://errors.example.com/gts.hx.core.errors.err.v1~hx.odata.errors.invalid_filter.v1"
    );
    assert_eq!(problem["title"], "Invalid Filter");
    assert!(problem["detail"]
        .as_str()
        .unwrap()
        .contains("Invalid $filter"));

    // Test with another type of invalid filter - type mismatch
    let request = Request::builder()
        .method("GET")
        .uri("/users?%24filter=id%20eq%20%27not-a-uuid%27")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    // OData errors return 422 (Unprocessable Entity) per GTS catalog
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let problem: serde_json::Value = serde_json::from_slice(&body)?;

    // Should still be the same error type (GTS catalog format)
    assert_eq!(
        problem["type"],
        "https://errors.example.com/gts.hx.core.errors.err.v1~hx.odata.errors.invalid_filter.v1"
    );
    assert_eq!(problem["title"], "Invalid Filter");

    Ok(())
}

#[tokio::test]
async fn test_rest_api_not_found() -> Result<()> {
    let router = create_test_router().await;

    let non_existent_id = Uuid::new_v4();
    let request = Request::builder()
        .method("GET")
        .uri(format!("/users/{non_existent_id}"))
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

    // User -> UserDto
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

    // CreateUserReq -> NewUser
    let create_req = CreateUserReq {
        email: "new@example.com".to_string(),
        display_name: "New User".to_string(),
    };
    let new_user = NewUser::from(create_req.clone());
    assert_eq!(new_user.email, create_req.email);
    assert_eq!(new_user.display_name, create_req.display_name);

    // UpdateUserReq -> UserPatch
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
    // Contract models should not have serde derives.
    use users_info::contract::model::User;

    let user = User {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        display_name: "Test User".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // This wouldn't compile if User had Serialize:
    // let _ = serde_json::to_string(&user);

    // But REST DTOs *do* have serde derives:
    let dto = users_info::api::rest::dto::UserDto::from(user);
    let serialized = serde_json::to_string(&dto);
    assert!(serialized.is_ok());
}

#[tokio::test]
async fn test_end_to_end_sse_events() -> Result<()> {
    use futures::StreamExt;
    use tokio::time::{timeout, Duration};

    // Create SSE broadcaster and adapter
    let sse_broadcaster = SseBroadcaster::<UserEvent>::new(10);
    let event_publisher: Arc<dyn EventPublisher<UserDomainEvent>> =
        Arc::new(SseUserEventPublisher::new(sse_broadcaster.clone()));

    // Create service with SSE event publisher
    let db = create_test_db().await;
    let repo = SeaOrmUsersRepository::new(db);
    let audit: Arc<dyn AuditPort> = Arc::new(MockAuditPort);
    let config = ServiceConfig::default();
    let service = Arc::new(Service::new(Arc::new(repo), event_publisher, audit, config));

    // Subscribe to SSE stream
    let mut event_stream = Box::pin(sse_broadcaster.subscribe_stream());

    // Create a user - should trigger Created event
    let new_user = users_info::contract::model::NewUser {
        email: "sse-test@example.com".to_string(),
        display_name: "SSE Test User".to_string(),
    };
    let created_user = service.create_user(new_user).await?;

    // Wait for Created event
    let created_event = timeout(Duration::from_millis(200), event_stream.next())
        .await
        .expect("timeout waiting for created event")
        .expect("should receive created event");

    assert_eq!(created_event.kind, "created");
    assert_eq!(created_event.id, created_user.id);

    // Update the user - should trigger Updated event
    let patch = users_info::contract::model::UserPatch {
        email: None,
        display_name: Some("Updated SSE User".to_string()),
    };
    let updated_user = service.update_user(created_user.id, patch).await?;

    // Wait for Updated event
    let updated_event = timeout(Duration::from_millis(200), event_stream.next())
        .await
        .expect("timeout waiting for updated event")
        .expect("should receive updated event");

    assert_eq!(updated_event.kind, "updated");
    assert_eq!(updated_event.id, updated_user.id);

    // Delete the user - should trigger Deleted event
    service.delete_user(created_user.id).await?;

    // Wait for Deleted event
    let deleted_event = timeout(Duration::from_millis(200), event_stream.next())
        .await
        .expect("timeout waiting for deleted event")
        .expect("should receive deleted event");

    assert_eq!(deleted_event.kind, "deleted");
    assert_eq!(deleted_event.id, created_user.id);

    Ok(())
}

#[tokio::test]
async fn test_sse_adapter_integration() -> Result<()> {
    use futures::StreamExt;
    use tokio::time::{timeout, Duration};

    // Test the SSE adapter in isolation
    let sse_broadcaster = SseBroadcaster::<UserEvent>::new(10);
    let adapter = SseUserEventPublisher::new(sse_broadcaster.clone());
    let mut event_stream = Box::pin(sse_broadcaster.subscribe_stream());

    let test_user_id = Uuid::new_v4();
    let test_timestamp = Utc::now();

    // Test all domain event types
    let domain_events = vec![
        UserDomainEvent::Created {
            id: test_user_id,
            at: test_timestamp,
        },
        UserDomainEvent::Updated {
            id: test_user_id,
            at: test_timestamp,
        },
        UserDomainEvent::Deleted {
            id: test_user_id,
            at: test_timestamp,
        },
    ];

    let expected_kinds = ["created", "updated", "deleted"];

    // Publish all events
    for event in &domain_events {
        adapter.publish(event);
    }

    // Verify all events are received in order
    for expected_kind in &expected_kinds {
        let received_event = timeout(Duration::from_millis(200), event_stream.next())
            .await
            .expect("timeout waiting for event")
            .expect("should receive event");

        assert_eq!(received_event.kind, *expected_kind);
        assert_eq!(received_event.id, test_user_id);
        assert_eq!(received_event.at, test_timestamp);
    }

    Ok(())
}

#[tokio::test]
async fn test_multiple_sse_subscribers() -> Result<()> {
    use futures::StreamExt;
    use tokio::time::{timeout, Duration};

    // Create broadcaster with multiple subscribers
    let sse_broadcaster = SseBroadcaster::<UserEvent>::new(10);
    let adapter = SseUserEventPublisher::new(sse_broadcaster.clone());

    // Create multiple subscribers
    let mut stream1 = Box::pin(sse_broadcaster.subscribe_stream());
    let mut stream2 = Box::pin(sse_broadcaster.subscribe_stream());
    let mut stream3 = Box::pin(sse_broadcaster.subscribe_stream());

    let test_user_id = Uuid::new_v4();
    let test_timestamp = Utc::now();

    // Publish a single event
    adapter.publish(&UserDomainEvent::Created {
        id: test_user_id,
        at: test_timestamp,
    });

    // All subscribers should receive the same event
    let event1 = timeout(Duration::from_millis(200), stream1.next())
        .await
        .expect("timeout waiting for event")
        .expect("should receive event");

    let event2 = timeout(Duration::from_millis(200), stream2.next())
        .await
        .expect("timeout waiting for event")
        .expect("should receive event");

    let event3 = timeout(Duration::from_millis(200), stream3.next())
        .await
        .expect("timeout waiting for event")
        .expect("should receive event");

    // Verify all received the same event
    assert_eq!(event1.kind, "created");
    assert_eq!(event2.kind, "created");
    assert_eq!(event3.kind, "created");
    assert_eq!(event1.id, event2.id);
    assert_eq!(event2.id, event3.id);
    assert_eq!(event1.at, event2.at);
    assert_eq!(event2.at, event3.at);

    Ok(())
}
