//! `OData` integration for `SeaORM` with security-scoped pagination.
//!
//! This module provides `SeaORM`-specific adapters for `OData` queries:
//! - `OData` filter compilation to `SeaORM` conditions (legacy `FieldMap` and new `FilterNode`)
//! - Cursor-based pagination with `OData` ordering
//! - Security-scoped pagination via `OPager` builder
//!
//! # Filter DSL
//!
//! The core filter types (`FilterField`, `FilterNode`, `FilterOp`, `FieldKind`) are defined
//! in `modkit-odata` as part of the `OData` protocol contract. Import them from:
//! ```ignore
//! use modkit_odata::filter::{FilterField, FilterNode, FilterOp, FieldKind};
//! ```
//!
//! # Modules
//!
//! - `core`: Core `OData` to `SeaORM` translation (filters, cursors, ordering) - legacy `FieldMap` based
//! - `sea_orm_filter`: Type-safe mapping from `FilterNode<F>` to `SeaORM` conditions
//! - `pager`: Fluent builder for secure + `OData` pagination

// Core OData functionality (legacy FieldMap-based)
mod core;

// SeaORM-specific filter mapping
pub mod sea_orm_filter;

// Fluent pagination builder
pub mod pager;

// Re-export all public items from core (legacy API)
pub use core::*;

// Re-export SeaORM filter mapping and pagination
pub use sea_orm_filter::{
    FieldToColumn, LimitCfg, ODataFieldMapping, encode_cursor_value, filter_node_to_condition,
    paginate_odata, parse_cursor_value,
};
