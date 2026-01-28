//! SQLite-specific tests
//!
//! This module organizes all SQLite-specific tests.
//! All tests in this module require the `sqlite` feature flag.

#![cfg(feature = "sqlite")]

mod concurrency_tests;
mod manager;
mod options;
mod pooling_tests;
mod secure_insert_tenant_validation;
mod secure_update_tenant_safety;
#[cfg_attr(coverage_nightly, coverage(off))]
mod sqlite_tests;
mod transaction;
