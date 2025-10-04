//! Type-safe API operation builder with compile-time guarantees
//!
//! This module provides a type-state builder pattern that enforces at compile time
//! that API operations cannot be registered unless both a handler and at least one
//! response are specified.

pub mod error;
pub mod error_layer;
pub mod odata;
pub mod odata_policy_tests;
pub mod operation_builder;
pub mod pagination;
pub mod problem;
pub mod response;

pub use error::ApiError;
pub use error_layer::{
    error_mapping_middleware, extract_trace_id, map_error_to_problem, IntoProblemResponse,
};
pub use operation_builder::{
    ensure_schema, state, Missing, OpenApiRegistry, OperationBuilder, OperationSpec, ParamLocation,
    ParamSpec, Present, ResponseSpec,
};
pub use pagination::{normalize_filter_for_hash, short_filter_hash};
pub use problem::{
    bad_request, conflict, internal_error, not_found, Problem, ProblemResponse, ValidationError,
    APPLICATION_PROBLEM_JSON,
};

/// Prelude module that re-exports common API types and utilities for module authors
pub mod prelude {
    // Errors + Result
    pub use super::error::{ApiError, ApiResult};

    // Response sugar
    pub use super::response::{created_json, no_content, ok_json, to_response, JsonBody, JsonPage};

    // Useful axum bits (common in handlers)
    pub use axum::{http::StatusCode, response::IntoResponse, Json};
}
