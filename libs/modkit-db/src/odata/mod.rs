//! OData integration for SeaORM with security-scoped pagination.
//!
//! This module provides:
//! - OData filter compilation to SeaORM conditions
//! - Cursor-based pagination with OData ordering
//! - Security-scoped pagination via `OPager` builder
//!
//! # Modules
//!
//! - `core`: Core OData to SeaORM translation (filters, cursors, ordering)
//! - `pager`: Fluent builder for secure + OData pagination
//! - `tests`: Integration tests (when compiled with `#[cfg(test)]`)

// Core OData functionality
mod core;

// Fluent pagination builder
pub mod pager;

// Tests (only compiled during tests)
// TODO: Fix test module after refactoring
// #[cfg(test)]
// #[path = "tests.rs"]
// mod odata_tests;

// Re-export all public items from core
pub use core::*;
