//! Type-safe API operation builder with compile-time guarantees
//!
//! This module provides a type-state builder pattern that enforces at compile time
//! that API operations cannot be registered unless both a handler and at least one
//! response are specified.

pub mod operation_builder;

pub use operation_builder::{
    ensure_schema, state, Missing, OpenApiRegistry, OperationBuilder, OperationSpec, ParamLocation,
    ParamSpec, Present, ResponseSpec,
};
