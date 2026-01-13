// @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types
//! GTS type definitions for Analytics module
//!
//! This module contains Rust struct definitions that generate JSON Schema files
//! for the Analytics module's GTS types using the `struct_to_gts_schema` macro.

pub mod schema;

pub use schema::*;
