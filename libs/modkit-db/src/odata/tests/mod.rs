//! Tests for the `OData` module
//!
//! This module is organized into:
//! - `core_unit`: Unit tests for core `OData` functions (legacy `FieldMap`-based)
//! - `support`: Shared test utilities for `SQLite` tests
//! - `SQLite` integration tests for various `OData` components

#![cfg(feature = "sea-orm")]

// Unit tests for core functions
mod core_unit;

// Shared test support utilities
#[cfg(feature = "sqlite")]
mod support;

// SQLite integration tests
#[cfg(feature = "sqlite")]
mod sqlite_pagination;

#[cfg(feature = "sqlite")]
mod expr_to_condition_sqlite;

#[cfg(feature = "sqlite")]
mod paginate_with_odata_sqlite;

#[cfg(feature = "sqlite")]
mod apply_ext_methods_sqlite;

#[cfg(feature = "sqlite")]
mod convert_expr_to_filter_node_sqlite;

#[cfg(feature = "sqlite")]
mod pager_sqlite;
