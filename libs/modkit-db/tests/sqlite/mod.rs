//! SQLite-specific tests
//!
//! This module organizes all SQLite-specific tests.
//! All tests in this module require the `sqlite` feature flag.

#![cfg(feature = "sqlite")]

#[cfg_attr(coverage_nightly, coverage(off))]

mod sqlite_tests;
mod transaction;
mod concurrency_tests;
mod pooling_tests;
