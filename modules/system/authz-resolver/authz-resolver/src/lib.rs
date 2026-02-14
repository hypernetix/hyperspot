//! `AuthZ` Resolver Module
//!
//! This module discovers `AuthZ` resolver plugins via types-registry
//! and routes evaluation calls to the selected plugin based on vendor configuration.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod config;
pub mod domain;
pub mod module;
