//! `OData` integration for `SeaORM` with security-scoped pagination.
//!
//! This module provides:
//! - Type-safe `OData` filter representation via `FilterField` trait and `FilterNode<F>` AST
//! - `OData` filter compilation to `SeaORM` conditions (legacy `FieldMap` and new `FilterNode`)
//! - Cursor-based pagination with `OData` ordering
//! - Security-scoped pagination via `OPager` builder
//!
//! # Modules
//!
//! - `core`: Core `OData` to `SeaORM` translation (filters, cursors, ordering) - legacy `FieldMap` based
//! - `filter`: Type-safe filter representation using `FilterField` trait and `FilterNode<F>` AST
//! - `pager`: Fluent builder for secure + `OData` pagination

// Shared FieldKind enum for both legacy and new code
pub mod kind;

// Core OData functionality (legacy FieldMap-based)
mod core;

// Type-safe filter representation
pub mod filter;

// SeaORM-specific filter mapping
pub mod sea_orm_filter;

// Fluent pagination builder
pub mod pager;

// Re-export shared FieldKind
pub use kind::FieldKind;

// Re-export all public items from core (legacy API)
pub use core::*;

// Re-export new filter types for convenience
pub use filter::{
    convert_expr_to_filter_node, parse_odata_filter, FilterError, FilterField, FilterNode,
    FilterOp, FilterResult, ODataValue,
};

// Re-export SeaORM filter mapping and pagination
pub use sea_orm_filter::{
    encode_cursor_value, filter_node_to_condition, paginate_odata, parse_cursor_value,
    FieldToColumn, LimitCfg, ODataFieldMapping,
};
