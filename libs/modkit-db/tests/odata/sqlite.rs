//! SQLite integration tests for OData functionality
//!
//! This module organizes all SQLite-specific OData tests.
//! All tests in this module require the `sqlite` feature flag.

// Shared test support utilities
pub(crate) mod support;

// SQLite integration tests
mod expr_to_condition;
mod paginate_with_odata;
mod apply_ext_methods;
mod convert_expr_to_filter_node;
mod pager;
