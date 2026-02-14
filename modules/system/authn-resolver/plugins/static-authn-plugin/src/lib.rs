#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! Static `AuthN` Resolver Plugin
//!
//! This plugin provides static token-to-identity mapping for development and testing.
//!
//! ## Modes
//!
//! - **`accept_all`** (default): Accepts any non-empty token, returns configured default identity.
//!   Replaces the `auth_disabled` use case for scenarios that still need a `SecurityContext`.
//!
//! - **`static_tokens`**: Maps specific tokens to specific identities. Useful for E2E tests
//!   with distinct users.
//!
//! ## Configuration
//!
//! ```yaml
//! modules:
//!   static_authn_plugin:
//!     config:
//!       vendor: "hyperspot"
//!       priority: 100
//!       mode: accept_all
//!       default_identity:
//!         subject_id: "11111111-6a88-4768-9dfc-6bcd5187d9ed"
//!         subject_tenant_id: "00000000-df51-5b42-9538-d2b56b7ee953"
//!         token_scopes: ["*"]
//!       tokens: []
//! ```

pub mod config;
pub mod domain;
pub mod module;

pub use module::StaticAuthNPlugin;
