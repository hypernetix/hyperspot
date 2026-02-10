//! REST route registration for the Types Registry module.

use std::sync::Arc;

use axum::{Extension, Router};
use modkit::api::OpenApiRegistry;
use modkit::api::operation_builder::{
    AuthReqAction, AuthReqResource, LicenseFeature, OperationBuilder,
};
use modkit::api::prelude::StatusCode;

use super::dto::{
    GtsEntityDto, ListEntitiesResponse, RegisterEntitiesRequest, RegisterEntitiesResponse,
};
use super::handlers;
use crate::domain::service::TypesRegistryService;

const TAG: &str = "types-registry";

enum Resource {
    TypesRegistry,
}

enum Action {
    Read,
    Write,
}

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &'static str {
        match self {
            Resource::TypesRegistry => "types_registry",
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

/// Registers all REST routes for the Types Registry module.
#[allow(clippy::needless_pass_by_value)]
pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<TypesRegistryService>,
) -> Router {
    // POST /types-registry/v1/entities - Register GTS entities
    router = OperationBuilder::post("/types-registry/v1/entities")
        .operation_id("types-registry.register")
        .summary("Register GTS entities")
        .description(
            "Register one or more GTS entities (types or instances) in batch. Returns per-item results.",
        )
        .tag(TAG)
        .require_auth(&Resource::TypesRegistry, &Action::Write)
        .require_license_features::<License>([])
        .json_request::<RegisterEntitiesRequest>(openapi, "GTS entities to register")
        .handler(handlers::register_entities)
        .json_response_with_schema::<RegisterEntitiesResponse>(
            openapi,
            StatusCode::OK,
            "Registration results",
        )
        .standard_errors(openapi)
        .register(router, openapi);

    // GET /types-registry/v1/entities - List GTS entities
    router = OperationBuilder::get("/types-registry/v1/entities")
        .operation_id("types-registry.list")
        .summary("List GTS entities")
        .description(
            "List registered GTS entities with optional filtering by pattern, kind, vendor, package, or namespace.",
        )
        .tag(TAG)
        .require_auth(&Resource::TypesRegistry, &Action::Read)
        .require_license_features::<License>([])
        .query_param("pattern", false, "Wildcard pattern for GTS ID matching (e.g., gts.acme.*)")
        .query_param("kind", false, "Filter by entity kind: 'type' or 'instance'")
        .query_param("vendor", false, "Filter by vendor")
        .query_param("package", false, "Filter by package")
        .query_param("namespace", false, "Filter by namespace")
        .query_param("segmentScope", false, "Segment match scope: 'primary' or 'any' (default)")
        .handler(handlers::list_entities)
        .json_response_with_schema::<ListEntitiesResponse>(
            openapi,
            StatusCode::OK,
            "List of entities",
        )
        .standard_errors(openapi)
        .register(router, openapi);

    // GET /types-registry/v1/entities/{gts_id} - Get GTS entity by ID
    router = OperationBuilder::get("/types-registry/v1/entities/{gts_id}")
        .operation_id("types-registry.get")
        .summary("Get GTS entity by ID")
        .description("Retrieve a single GTS entity by its identifier.")
        .tag(TAG)
        .require_auth(&Resource::TypesRegistry, &Action::Read)
        .require_license_features::<License>([])
        .path_param(
            "gts_id",
            "The GTS identifier (e.g., gts.acme.core.events.user_created.v1~)",
        )
        .handler(handlers::get_entity)
        .json_response_with_schema::<GtsEntityDto>(openapi, StatusCode::OK, "The requested entity")
        .problem_response(openapi, StatusCode::NOT_FOUND, "Entity not found")
        .standard_errors(openapi)
        .register(router, openapi);

    router.layer(Extension(service))
}
