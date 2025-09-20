//! Type-safe API operation builder with compile-time guarantees
//!
//! This module provides a type-state builder pattern that enforces at compile time
//! that API operations cannot be registered unless both a handler and at least one
//! response are specified.

pub mod odata;
pub mod odata_policy_tests;
pub mod operation_builder;
pub mod pagination;
pub mod problem;

pub use operation_builder::{
    ensure_schema, state, Missing, OpenApiRegistry, OperationBuilder, OperationSpec, ParamLocation,
    ParamSpec, Present, ResponseSpec,
};
pub use pagination::{normalize_filter_for_hash, odata_page_error_to_problem, short_filter_hash};
pub use problem::{
    bad_request, conflict, internal_error, not_found, Problem, ProblemResponse, ValidationError,
    APPLICATION_PROBLEM_JSON,
};
