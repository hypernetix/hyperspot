//! Tests for the `OData` module
//!
//! This module is organized into:
//! - `core_unit`: Unit tests for core `OData` functions (legacy `FieldMap`-based)
//! - `core_unit_legacy`: Unit tests for cursor encoding/decoding (legacy)
//! - `filter_unit`: Unit tests for filter conversion (FilterNode-based)
//! - `sea_orm_filter_unit`: Unit tests for SeaORM filter mapping
//! - `sqlite`: SQLite integration tests for various `OData` components

#![cfg(feature = "sea-orm")]
#[cfg_attr(coverage_nightly, coverage(off))]

// Unit tests for core functions
mod core_unit;
mod core_unit_legacy;
mod filter_unit;
mod sea_orm_filter_unit;

// SQLite integration tests
#[cfg(feature = "sqlite")]
mod sqlite;
