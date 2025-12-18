#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! Feature Flags Gateway SDK.
//!
//! This crate defines **transport-agnostic** types used by the `feature_flags_gateway` module.
//!
//! # Public API
//!
//! - [`FeatureFlag`]: well-known feature flag constants.

pub mod api;
pub mod errors;
pub mod models;

pub use api::FeatureFlagsApi;
pub use errors::FeatureFlagsError;
pub use models::FeatureFlag;

#[must_use]
pub fn sdk_version() -> &'static str {
    "0.1.0"
}
