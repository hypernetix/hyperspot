use crate::api::rest::{dto, handlers};
use crate::domain::service::Service;
use axum::http::StatusCode;
use axum::{Extension, Router};
use modkit::api::operation_builder::{AuthReqAction, AuthReqResource, LicenseFeature};
use modkit::api::{OpenApiRegistry, OperationBuilder};
use std::sync::Arc;

enum Resource {
    Settings,
}

enum Action {
    Read,
    Write,
}

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &'static str {
        match self {
            Resource::Settings => "simple-user-settings",
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

struct License;

impl AsRef<str> for License {
    fn as_ref(&self) -> &'static str {
        "gts.x.core.lic.feat.v1~x.core.global.base.v1"
    }
}

impl LicenseFeature for License {}

pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<Service>,
) -> Router {
    router = OperationBuilder::get("/simple-user-settings/v1/settings")
        .operation_id("settings.get_settings")
        .summary("Get user settings")
        .description("Retrieve settings for the authenticated user")
        .tag("Settings")
        .require_auth(&Resource::Settings, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_settings)
        .json_response_with_schema::<dto::SimpleUserSettingsDto>(
            openapi,
            StatusCode::OK,
            "Settings retrieved",
        )
        .error_401(openapi)
        .error_403(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::post("/simple-user-settings/v1/settings")
        .operation_id("settings.update_settings")
        .summary("Update user settings")
        .description("Full update of user settings (POST semantics)")
        .tag("Settings")
        .require_auth(&Resource::Settings, &Action::Write)
        .require_license_features::<License>([])
        .json_request::<dto::UpdateSimpleUserSettingsRequest>(openapi, "Settings update data")
        .handler(handlers::update_settings)
        .json_response_with_schema::<dto::SimpleUserSettingsDto>(
            openapi,
            StatusCode::OK,
            "Settings updated",
        )
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_422(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::patch("/simple-user-settings/v1/settings")
        .operation_id("settings.patch_settings")
        .summary("Partially update user settings")
        .description("Partial update of user settings (PATCH semantics)")
        .tag("Settings")
        .require_auth(&Resource::Settings, &Action::Write)
        .require_license_features::<License>([])
        .json_request::<dto::PatchSimpleUserSettingsRequest>(openapi, "Settings patch data")
        .handler(handlers::patch_settings)
        .json_response_with_schema::<dto::SimpleUserSettingsDto>(
            openapi,
            StatusCode::OK,
            "Settings patched",
        )
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_422(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router = router.layer(Extension(service));

    router
}
