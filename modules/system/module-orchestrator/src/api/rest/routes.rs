use axum::http;
use axum::{Extension, Router};
use modkit::api::{
    OpenApiRegistry, OperationBuilder,
    operation_builder::{AuthReqAction, AuthReqResource},
};
use std::sync::Arc;

use super::dto::ModuleDto;
use super::handlers;
use crate::domain::service::ModulesService;

enum Resource {
    Modules,
}

enum Action {
    Read,
}

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &'static str {
        match self {
            Resource::Modules => "module_orchestrator",
        }
    }
}

impl AuthReqResource for Resource {}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &'static str {
        match self {
            Action::Read => "read",
        }
    }
}

impl AuthReqAction for Action {}

/// Register all REST routes for the module orchestrator
#[allow(clippy::needless_pass_by_value)]
pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<ModulesService>,
) -> Router {
    // GET /module-orchestrator/v1/modules - List all registered modules
    router = OperationBuilder::get("/module-orchestrator/v1/modules")
        .operation_id("module_orchestrator.list_modules")
        .summary("List all registered modules")
        .description(
            "Returns a list of all compiled-in and out-of-process modules with their \
         capabilities, dependencies, running instances, and deployment mode.",
        )
        .tag("module-orchestrator")
        .require_auth(&Resource::Modules, &Action::Read)
        .no_license_required()
        .handler(handlers::list_modules)
        .json_response_with_schema::<Vec<ModuleDto>>(
            openapi,
            http::StatusCode::OK,
            "List of registered modules",
        )
        .standard_errors(openapi)
        .register(router, openapi);

    router = router.layer(Extension(service));

    router
}
