use axum::{Extension, Router};
use modkit::api::{OpenApiRegistry, OperationBuilder};
use std::sync::Arc;

use crate::api::rest::{dto, handlers};
use crate::domain::service::Service;

pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<Service>,
) -> anyhow::Result<Router> {
    // Schemas should be auto-registered via ToSchema when used in operations

    // GET /settings - Get user's settings
    router = OperationBuilder::<modkit::api::Missing, modkit::api::Missing, ()>::get("/settings")
        .operation_id("settings.get_settings")
        .summary("Get user settings")
        .description("Retrieve user's settings")
        .tag("settings")
        .handler(handlers::get_settings)
        .json_response_with_schema::<dto::SettingsDTO>(openapi, 200, "User's settings")
        .problem_response(openapi, 400, "Bad Request")
        .problem_response(openapi, 500, "Internal Server Error")
        .register(router, openapi);

    // POST /settings - Update user's settings
    router = OperationBuilder::<modkit::api::Missing, modkit::api::Missing, ()>::post("/settings")
        .operation_id("settings.update")
        .summary("Update user settings")
        .description("Update settings for a specific user")
        .tag("settings")
        .json_request::<dto::UpdateSettingsReq>(openapi, "Setting update data")
        .handler(handlers::update_settings)
        .json_response_with_schema::<dto::SettingsDTO>(openapi, 200, "User found")
        .problem_response(openapi, 404, "Not Found")
        .problem_response(openapi, 500, "Internal Server Error")
        .register(router, openapi);

    // PATCH /settings - Update user's settings
    router = OperationBuilder::<modkit::api::Missing, modkit::api::Missing, ()>::patch("/settings")
        .operation_id("settings.patch")
        .summary("Partially update user settings")
        .description("Partially update settings for a specific user")
        .tag("settings")
        .json_request::<dto::UpdateSettingsReq>(openapi, "Setting update data")
        .handler(handlers::update_settings)
        .json_response_with_schema::<dto::SettingsDTO>(openapi, 200, "User found")
        .problem_response(openapi, 404, "Not Found")
        .problem_response(openapi, 500, "Internal Server Error")
        .register(router, openapi);

    router = router.layer(Extension(service.clone()));

    Ok(router)
}
