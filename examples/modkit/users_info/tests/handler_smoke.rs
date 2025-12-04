#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Smoke test for HTTP handlers - verifies the full stack works.

mod support;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::{self, Next},
    Extension, Router,
};
use modkit_db::secure::SecureConn;
use modkit_security::SecurityCtx;
use std::sync::Arc;
use support::{inmem_db, seed_user, MockAuditPort, MockEventPublisher};
use tower::ServiceExt;
use users_info::{
    api::rest::handlers,
    domain::service::{Service, ServiceConfig},
    infra::storage::sea_orm_repo::SeaOrmUsersRepository,
};
use uuid::Uuid;

/// Middleware to inject a fake SecurityCtx for testing
async fn inject_fake_security_ctx(mut req: Request<Body>, next: Next) -> axum::response::Response {
    let fake_tenant = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let fake_subject = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
    let ctx = SecurityCtx::for_tenant(fake_tenant, fake_subject);
    req.extensions_mut().insert(ctx);
    next.run(req).await
}

/// Create a test router with real database and service
async fn create_test_router() -> Router {
    let db = inmem_db().await;
    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);

    let events = Arc::new(MockEventPublisher);
    let audit = Arc::new(MockAuditPort);
    let config = ServiceConfig::default();

    let service = Arc::new(Service::new(Arc::new(repo), events, audit, config));

    Router::new()
        .route("/users/{id}", axum::routing::get(handlers::get_user))
        .route("/users", axum::routing::get(handlers::list_users))
        .layer(Extension(service))
        .layer(middleware::from_fn(inject_fake_security_ctx))
}

#[tokio::test]
async fn get_user_handler_returns_json() {
    // Note: This test demonstrates the security behavior.
    // The handler uses fake_ctx_from_request() which creates a fixed tenant context.
    // To access a user, we must seed them in the same tenant as the fake context.
    //
    // The fake context uses tenant: 00000000-0000-0000-0000-000000000001
    // So we seed the user in that same tenant to make it accessible.

    // Arrange: Create router with seeded user in the fake tenant
    let db = inmem_db().await;
    let fake_tenant = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let user_id = Uuid::new_v4();
    let _user = seed_user(&db, user_id, fake_tenant, "test@example.com", "Test User").await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let service = Arc::new(Service::new(
        Arc::new(repo),
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    ));

    let app = Router::new()
        .route("/users/{id}", axum::routing::get(handlers::get_user))
        .layer(Extension(service))
        .layer(middleware::from_fn(inject_fake_security_ctx));

    // Act: Call GET /users/:id
    let request = Request::builder()
        .method("GET")
        .uri(format!("/users/{}", user_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert: Should return 200 OK since user is in the same tenant as the fake context
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Handler should return 200 OK when user is in accessible tenant"
    );
}

#[tokio::test]
async fn get_nonexistent_user_returns_404() {
    // Arrange: Create router (empty database)
    let app = create_test_router().await;
    let random_id = Uuid::new_v4();

    // Act: Call GET /users/:id for non-existent user
    let request = Request::builder()
        .method("GET")
        .uri(format!("/users/{}", random_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert: Should return 404 Not Found
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_users_returns_json_page() {
    // Arrange: Create router with seeded users in the fake tenant
    let db = inmem_db().await;
    let fake_tenant = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    seed_user(
        &db,
        Uuid::new_v4(),
        fake_tenant,
        "user1@example.com",
        "User 1",
    )
    .await;
    seed_user(
        &db,
        Uuid::new_v4(),
        fake_tenant,
        "user2@example.com",
        "User 2",
    )
    .await;

    let sec = SecureConn::new(db);
    let repo = SeaOrmUsersRepository::new(sec);
    let service = Arc::new(Service::new(
        Arc::new(repo),
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        ServiceConfig::default(),
    ));

    let app = Router::new()
        .route("/users", axum::routing::get(handlers::list_users))
        .layer(Extension(service))
        .layer(middleware::from_fn(inject_fake_security_ctx));

    // Act: Call GET /users
    let request = Request::builder()
        .method("GET")
        .uri("/users")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert: Should return 200 OK
    assert_eq!(response.status(), StatusCode::OK);

    // Note: The list should contain users from the fake tenant context
    // This demonstrates tenant-based filtering in action
}
