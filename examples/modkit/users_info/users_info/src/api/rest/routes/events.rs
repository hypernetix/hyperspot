use super::{Action, License, Resource, dto, handlers};
use axum::Router;
use modkit::api::operation_builder::OperationBuilder;
use modkit::api::OpenApiRegistry;
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;

pub(super) fn register_sse_route<S>(
    router: Router<S>,
    openapi: &dyn OpenApiRegistry,
    sse: modkit::SseBroadcaster<dto::UserEvent>,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    // Register the SSE route
    let router = OperationBuilder::get("/users-info/v1/users/events")
        .operation_id("users_info.events")
        .require_auth(&Resource::Users, &Action::Read)
        .require_license_features::<License>([])
        .summary("User events stream (SSE)")
        .description("Real-time stream of user events as Server-Sent Events")
        .tag("users")
        .handler(handlers::users_events)
        .sse_json::<dto::UserEvent>(openapi, "SSE stream of UserEvent")
        .register(router, openapi);

    // Apply layers for the specific route
    router
        .layer(axum::Extension(sse))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::GATEWAY_TIMEOUT,
            Duration::from_secs(60 * 60),
        ))
}
