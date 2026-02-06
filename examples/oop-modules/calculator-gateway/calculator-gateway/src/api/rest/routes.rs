//! Route registration for calculator_gateway module

use std::sync::Arc;

use axum::{Extension, Router};
use http::StatusCode;

use modkit::api::{OpenApiRegistry, OperationBuilder};

use crate::domain::Service;

use super::dto::{AddRequest, AddResponse};
use super::handlers;

/// Register all REST routes for calculator_gateway module.
///
/// # Arguments
/// * `router` - Axum router to add routes to
/// * `openapi` - OpenAPI registry for documentation
/// * `service` - Domain Service
pub fn register_routes(
    router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<Service>,
) -> anyhow::Result<Router> {
    // POST /calculator-gateway/v1/calculator/add - Add two numbers
    let router = OperationBuilder::post("/calculator-gateway/v1/calculator/add")
        .operation_id("calculator_gateway.add")
        .summary("Add two numbers")
        .description(
            "Accepts a JSON body with `a` and `b`, returns their sum via calculator service",
        )
        .tag("calculator")
        .public() // No auth required for this example
        .json_request::<AddRequest>(openapi, "Addition request with a and b operands")
        .handler(handlers::handle_add)
        .json_response_with_schema::<AddResponse>(openapi, StatusCode::OK, "Sum of the two numbers")
        .error_500(openapi)
        .register(router, openapi);

    // Add Service as Extension
    let router = router.layer(Extension(service));

    Ok(router)
}
