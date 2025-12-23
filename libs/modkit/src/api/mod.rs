//! Type-safe API operation builder with compile-time guarantees
//!
//! This module provides a type-state builder pattern that enforces at compile time
//! that API operations cannot be registered unless both a handler and at least one
//! response are specified.

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
    error_mapping_middleware, extract_trace_id, map_error_to_problem, IntoProblem,
};
pub use openapi_registry::{ensure_schema, OpenApiInfo, OpenApiRegistry, OpenApiRegistryImpl};
pub use operation_builder::{
    state, Missing, OperationBuilder, OperationSpec, ParamLocation, ParamSpec, Present,
    RateLimitSpec, ResponseSpec,
};
pub use problem::{
    bad_request, conflict, internal_error, not_found, Problem, ValidationError,
    APPLICATION_PROBLEM_JSON,
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
    pub use super::response::{created_json, no_content, ok_json, JsonBody, JsonPage};

    // OData and field projection
    pub use super::select::apply_select;

    // Useful axum bits (common in handlers)
    pub use axum::{http::StatusCode, response::IntoResponse, Json};
}
