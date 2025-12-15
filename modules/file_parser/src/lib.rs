// === MODULE DEFINITION ===
// ModKit needs access to the module struct for instantiation
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
pub mod module;
pub use module::FileParserModule;

// === INTERNAL MODULES ===
// WARNING: These modules are internal implementation details!
// They are exposed only for comprehensive testing and should NOT be used by external consumers.
#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
