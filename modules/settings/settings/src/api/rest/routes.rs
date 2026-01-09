use crate::api::rest::{dto, handlers};
use crate::domain::service::Service;
use axum::http::StatusCode;
use axum::{Extension, Router};
use modkit::api::operation_builder::{AuthReqAction, AuthReqResource};
use modkit::api::{OpenApiRegistry, OperationBuilder};
use std::sync::Arc;

#[allow(dead_code)]
enum Resource {
    Settings,
}

#[allow(dead_code)]
enum Action {
    Read,
    Write,
}

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &'static str {
        match self {
            Resource::Settings => "settings",
        }
    }
}

impl AuthReqResource for Resource {}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &'static str {
        match self {
            Action::Read => "read",
            Action::Write => "write",
        }
    }
}

impl AuthReqAction for Action {}

#[allow(clippy::needless_pass_by_value)]
pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<Service>,
) -> Router {
    router = OperationBuilder::get("/settings/v1/settings")
        .operation_id("settings.get_settings")
        .summary("Get user settings")
        .description("Retrieve settings for the authenticated user")
        .tag("Settings")
        .public()
        .handler(handlers::get_settings)
        .json_response_with_schema::<dto::SettingsDto>(
            openapi,
            StatusCode::OK,
            "Settings retrieved",
        )
        .error_401(openapi)
        .error_403(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::post("/settings/v1/settings")
        .operation_id("settings.update_settings")
        .summary("Update user settings")
        .description("Full update of user settings (POST semantics)")
        .tag("Settings")
        .public()
        .json_request::<dto::UpdateSettingsRequest>(openapi, "Settings update data")
        .handler(handlers::update_settings)
        .json_response_with_schema::<dto::SettingsDto>(openapi, StatusCode::OK, "Settings updated")
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::patch("/settings/v1/settings")
        .operation_id("settings.patch_settings")
        .summary("Partially update user settings")
        .description("Partial update of user settings (PATCH semantics)")
        .tag("Settings")
        .public()
        .json_request::<dto::PatchSettingsRequest>(openapi, "Settings patch data")
        .handler(handlers::patch_settings)
        .json_response_with_schema::<dto::SettingsDto>(openapi, StatusCode::OK, "Settings patched")
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router = router.layer(Extension(service));

    router
}
