//! Type-safe API operation builder with compile-time guarantees
//!
//! This module provides a type-state builder pattern that enforces at compile time
//! that API operations cannot be registered unless both a handler and at least one
//! response are specified.

pub mod api_dto;
pub mod error_layer;
pub mod odata;
pub mod openapi_registry;
pub mod operation_builder;
pub mod problem;
pub mod response;
pub mod select;
pub mod trace_layer;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod odata_policy_tests;

pub use error_layer::{
    IntoProblem, error_mapping_middleware, extract_trace_id, map_error_to_problem,
};
pub use openapi_registry::{OpenApiInfo, OpenApiRegistry, OpenApiRegistryImpl, ensure_schema};
pub use operation_builder::{
    Missing, OperationBuilder, OperationSpec, ParamLocation, ParamSpec, Present, RateLimitSpec,
    ResponseSpec, state,
};
pub use problem::{
    APPLICATION_PROBLEM_JSON, Problem, ValidationError, bad_request, conflict, internal_error,
    not_found,
};
pub use select::{apply_select, page_to_projected_json, project_json};
pub use trace_layer::{WithRequestContext, WithTraceContext};

/// Prelude module that re-exports common API types and utilities for module authors
pub mod prelude {
    // Result type (Problem-only)
    pub use crate::result::ApiResult;

    // Problem type for error construction
    pub use super::problem::Problem;

    // Response sugar
    pub use super::response::{JsonBody, JsonPage, created_json, no_content, ok_json};

    // OData and field projection
    pub use super::select::apply_select;

    // Useful axum bits (common in handlers)
    pub use axum::{Json, http::StatusCode, response::IntoResponse};
}
