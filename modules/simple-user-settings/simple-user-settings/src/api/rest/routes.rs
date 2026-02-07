use crate::api::rest::{dto, handlers};
use crate::domain::service::Service;
use crate::infra::storage::sea_orm_repo::SeaOrmSettingsRepository;
use axum::http::StatusCode;
use axum::{Extension, Router};
use modkit::api::operation_builder::{AuthReqAction, AuthReqResource, LicenseFeature};
use modkit::api::{OpenApiRegistry, OperationBuilder};
use std::sync::Arc;

/// Type alias for the concrete service type.
pub type ConcreteService = Service<SeaOrmSettingsRepository>;

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
    service: Arc<ConcreteService>,
) -> Router {
    router = OperationBuilder::get("/simple-user-settings/v1/settings")
        .operation_id("simple-user-settings.get_settings")
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
        .operation_id("simple-user-settings.update_settings")
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
        .operation_id("simple-user-settings.patch_settings")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_settings_as_ref() {
        let resource = Resource::Settings;
        assert_eq!(resource.as_ref(), "simple-user-settings");
    }

    #[test]
    fn test_action_read_as_ref() {
        let action = Action::Read;
        assert_eq!(action.as_ref(), "read");
    }

    #[test]
    fn test_action_write_as_ref() {
        let action = Action::Write;
        assert_eq!(action.as_ref(), "write");
    }

    #[test]
    fn test_license_as_ref() {
        let license = License;
        assert_eq!(
            license.as_ref(),
            "gts.x.core.lic.feat.v1~x.core.global.base.v1"
        );
    }

    #[test]
    fn test_resource_implements_auth_req_resource() {
        fn assert_auth_req_resource<T: AuthReqResource>(_: &T) {}
        let resource = Resource::Settings;
        assert_auth_req_resource(&resource);
    }

    #[test]
    fn test_action_implements_auth_req_action() {
        fn assert_auth_req_action<T: AuthReqAction>(_: &T) {}
        let action = Action::Read;
        assert_auth_req_action(&action);
    }

    #[test]
    fn test_license_implements_license_feature() {
        fn assert_license_feature<T: LicenseFeature>(_: &T) {}
        let license = License;
        assert_license_feature(&license);
    }
}
