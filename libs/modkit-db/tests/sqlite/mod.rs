//! SQLite-specific tests
//!
//! This module organizes all SQLite-specific tests.
//! All tests in this module require the `sqlite` feature flag.

#![cfg(feature = "sqlite")]

mod concurrency_tests;
mod manager;
mod options;
mod pooling_tests;
#[cfg_attr(coverage_nightly, coverage(off))]
mod sqlite_tests;
mod transaction;
