use crate::api::rest::{dto, handlers};
use crate::domain::service::Service;
use axum::{Extension, Router};
use modkit::api::operation_builder::{AuthReqAction, AuthReqResource, OperationBuilderODataExt};
use modkit::api::{OpenApiRegistry, OperationBuilder};
use std::sync::Arc;
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;

enum Resource {
    Users,
}

enum Action {
    Read,
    Delete,
    Update,
    Create,
}

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &'static str {
        match self {
            Resource::Users => "users",
        }
    }
}

impl AuthReqResource for Resource {}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &'static str {
        match self {
            Action::Read => "read",
            Action::Delete => "delete",
            Action::Update => "update",
            Action::Create => "create",
        }
    }
}

impl AuthReqAction for Action {}

#[allow(clippy::needless_pass_by_value)] // Arc is intentionally passed by value for Extension layer
pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<Service>,
) -> Router {
    // Schemas should be auto-registered via ToSchema when used in operations

    // GET /users-info/v1/users - List users with cursor-based pagination
    router = OperationBuilder::get("/users-info/v1/users")
        .operation_id("users_info.list_users")
        .summary("List users with cursor pagination")
        .description("Retrieve a paginated list of users using cursor-based pagination")
        .tag("users")
        .require_auth(&Resource::Users, &Action::Read)
        .query_param_typed(
            "limit",
            false,
            "Maximum number of users to return",
            "integer",
        )
        .query_param("cursor", false, "Cursor for pagination")
        .handler(handlers::list_users)
        .json_response_with_schema::<modkit_odata::Page<dto::UserDto>>(
            openapi,
            http::StatusCode::OK,
            "Paginated list of users",
        )
        .with_odata_filter::<dto::UserDtoFilterField>()
        .with_odata_select()
        .with_odata_orderby::<dto::UserDtoFilterField>()
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /users-info/v1/users/{id} - Get a specific user
    router = OperationBuilder::get("/users-info/v1/users/{id}")
        .operation_id("users_info.get_user")
        .require_auth(&Resource::Users, &Action::Read)
        .summary("Get user by ID")
        .description("Retrieve a specific user by their UUID")
        .tag("users")
        .path_param("id", "User UUID")
        .handler(handlers::get_user)
        .with_odata_select()
        .json_response_with_schema::<dto::UserDto>(openapi, http::StatusCode::OK, "User found")
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // POST /users-info/v1/users - Create a new user
    router = OperationBuilder::post("/users-info/v1/users")
        .operation_id("users_info.create_user")
        .require_auth(&Resource::Users, &Action::Create)
        .summary("Create a new user")
        .description("Create a new user with the provided information")
        .tag("users")
        .json_request::<dto::CreateUserReq>(openapi, "User creation data")
        .handler(handlers::create_user)
        .json_response_with_schema::<dto::UserDto>(
            openapi,
            http::StatusCode::CREATED,
            "Created user",
        )
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_409(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // PATCH /users-info/v1/users/{id} - Partially update a user
    router = OperationBuilder::patch("/users-info/v1/users/{id}")
        .operation_id("users_info.update_user")
        .require_auth(&Resource::Users, &Action::Update)
        .summary("Update user")
        .description("Partially update a user with the provided fields")
        .tag("users")
        .path_param("id", "User UUID")
        .json_request::<dto::UpdateUserReq>(openapi, "User update data")
        .handler(handlers::update_user)
        .json_response_with_schema::<dto::UserDto>(openapi, http::StatusCode::OK, "Updated user")
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_409(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // DELETE /users-info/v1/users/{id} - Delete a user
    router = OperationBuilder::delete("/users-info/v1/users/{id}")
        .operation_id("users_info.delete_user")
        .require_auth(&Resource::Users, &Action::Delete) // ← Explicit auth requirement
        .summary("Delete user")
        .description("Delete a user by their UUID")
        .tag("users")
        .path_param("id", "User UUID")
        .handler(handlers::delete_user)
        .json_response(http::StatusCode::NO_CONTENT, "User deleted successfully")
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router = router.layer(Extension(service));

    router
}

/// Register SSE route for user events. The broadcaster is injected per-route via `Extension`.
pub fn register_users_sse_route<S>(
    router: axum::Router<S>,
    openapi: &dyn modkit::api::OpenApiRegistry,
    sse: modkit::SseBroadcaster<dto::UserEvent>,
) -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    // First register the route, then add layers
    let router = OperationBuilder::get("/users-info/v1/users/events")
        .operation_id("users_info.events")
        .require_auth(&Resource::Users, &Action::Read) // ← Explicit auth requirement for event stream
        .summary("User events stream (SSE)")
        .description("Real-time stream of user events as Server-Sent Events")
        .tag("users")
        .handler(handlers::users_events)
        .sse_json::<dto::UserEvent>(openapi, "SSE stream of UserEvent")
        .register(router, openapi);

    // Apply layers to the specific route using Router::layer
    router
        .layer(axum::Extension(sse))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::GATEWAY_TIMEOUT,
            Duration::from_secs(60 * 60),
        ))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod sse_tests {
    use super::*;
    use crate::api::rest::sse_adapter::SseUserEventPublisher;
    use crate::domain::events::UserDomainEvent;
    use crate::domain::ports::EventPublisher;
    use futures::StreamExt;
    use modkit::SseBroadcaster;
    use time::OffsetDateTime;
    use tokio::time::{timeout, Duration};
    use uuid::Uuid;

    #[tokio::test]
    async fn openapi_has_users_sse_content() {
        // Create a mock OpenAPI registry (using api_ingress)
        let api = api_ingress::ApiIngress::default();
        let router: axum::Router<()> = axum::Router::new();
        let sse_broadcaster = SseBroadcaster::<dto::UserEvent>::new(4);

        let _router = register_users_sse_route(router, &api, sse_broadcaster);

        let doc = api.build_openapi().expect("openapi");
        let v = serde_json::to_value(&doc).expect("json");

        // UserEvent schema is materialized
        let schema = v
            .pointer("/components/schemas/UserEvent")
            .expect("UserEvent missing");
        assert!(schema.get("$ref").is_none());

        // content is text/event-stream with $ref to our schema
        // Path is /users-info/v1/users/events, JSON pointer escapes / as ~1
        let refp = v
            .pointer(
                "/paths/~1users-info~1v1~1users~1events/get/responses/200/content/text~1event-stream/schema/$ref",
            )
            .and_then(|x| x.as_str())
            .unwrap_or_default();
        assert_eq!(refp, "#/components/schemas/UserEvent");
    }

    #[tokio::test]
    async fn sse_broadcaster_delivers_events() {
        let broadcaster = SseBroadcaster::<dto::UserEvent>::new(10);
        let mut stream = Box::pin(broadcaster.subscribe_stream());

        let test_event = dto::UserEvent {
            kind: "created".to_owned(),
            id: Uuid::new_v4(),
            at: OffsetDateTime::now_utc(),
        };

        // Send event
        broadcaster.send(test_event.clone());

        // Receive event
        let received = timeout(Duration::from_millis(100), stream.next())
            .await
            .expect("timeout")
            .expect("event received");

        assert_eq!(received.kind, test_event.kind);
        assert_eq!(received.id, test_event.id);
        assert_eq!(received.at, test_event.at);
    }

    #[tokio::test]
    async fn sse_adapter_publishes_domain_events() {
        let broadcaster = SseBroadcaster::<dto::UserEvent>::new(10);
        let adapter = SseUserEventPublisher::new(broadcaster.clone());
        let mut stream = Box::pin(broadcaster.subscribe_stream());

        let user_id = Uuid::new_v4();
        let timestamp = OffsetDateTime::now_utc();
        let domain_event = UserDomainEvent::Created {
            id: user_id,
            at: timestamp,
        };

        // Publish domain event through adapter
        adapter.publish(&domain_event);

        // Receive converted event
        let received = timeout(Duration::from_millis(100), stream.next())
            .await
            .expect("timeout")
            .expect("event received");

        assert_eq!(received.kind, "created");
        assert_eq!(received.id, user_id);
        assert_eq!(received.at, timestamp);
    }

    #[tokio::test]
    async fn sse_adapter_handles_all_event_types() {
        let broadcaster = SseBroadcaster::<dto::UserEvent>::new(10);
        let adapter = SseUserEventPublisher::new(broadcaster.clone());
        let mut stream = Box::pin(broadcaster.subscribe_stream());

        let user_id = Uuid::new_v4();
        let timestamp = OffsetDateTime::now_utc();

        // Test Created event
        adapter.publish(&UserDomainEvent::Created {
            id: user_id,
            at: timestamp,
        });
        let event = timeout(Duration::from_millis(100), stream.next())
            .await
            .expect("timeout")
            .expect("event received");
        assert_eq!(event.kind, "created");

        // Test Updated event
        adapter.publish(&UserDomainEvent::Updated {
            id: user_id,
            at: timestamp,
        });
        let event = timeout(Duration::from_millis(100), stream.next())
            .await
            .expect("timeout")
            .expect("event received");
        assert_eq!(event.kind, "updated");

        // Test Deleted event
        adapter.publish(&UserDomainEvent::Deleted {
            id: user_id,
            at: timestamp,
        });
        let event = timeout(Duration::from_millis(100), stream.next())
            .await
            .expect("timeout")
            .expect("event received");
        assert_eq!(event.kind, "deleted");
    }

    #[tokio::test]
    async fn sse_broadcaster_handles_multiple_subscribers() {
        let broadcaster = SseBroadcaster::<dto::UserEvent>::new(10);
        let mut stream1 = Box::pin(broadcaster.subscribe_stream());
        let mut stream2 = Box::pin(broadcaster.subscribe_stream());

        let test_event = dto::UserEvent {
            kind: "created".to_owned(),
            id: Uuid::new_v4(),
            at: OffsetDateTime::now_utc(),
        };

        // Send event
        broadcaster.send(test_event.clone());

        // Both subscribers should receive the event
        let received1 = timeout(Duration::from_millis(100), stream1.next())
            .await
            .expect("timeout")
            .expect("event received");
        let received2 = timeout(Duration::from_millis(100), stream2.next())
            .await
            .expect("timeout")
            .expect("event received");

        assert_eq!(received1.kind, test_event.kind);
        assert_eq!(received2.kind, test_event.kind);
        assert_eq!(received1.id, received2.id);
    }

    #[tokio::test]
    async fn sse_response_stream_works() {
        let broadcaster = SseBroadcaster::<dto::UserEvent>::new(10);
        let sse_response = broadcaster.sse_response();

        // The response should be created successfully
        // This test mainly ensures the type system works correctly
        drop(sse_response);
    }
}
