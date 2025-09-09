// === PUBLIC CONTRACT ===
// Only the contract module should be public for other modules to consume
pub mod contract;

// === MODULE DEFINITION ===
// ModKit needs access to the module struct for instantiation
pub mod module;
pub use module::Settings;

// === INTERNAL MODULES ===
// WARNING: These modules are internal implementation details!
// They are exposed only for compub(crate)prehensive testing and should NOT be used by external consumers.
// Only use the `contract` module for stable public APIs.
#[doc(hidden)]
pub mod api;
// #[doc(hidden)]
// pub mod config;
#[doc(hidden)]
pub mod domain;
// #[doc(hidden)]
// pub mod gateways;
#[doc(hidden)]
pub mod infra;
