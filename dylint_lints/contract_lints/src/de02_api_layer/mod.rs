//! DE02: API Layer Checks
//!
//! Linters to validate REST API implementation follows guidelines
//! and maintains proper separation from domain logic.

pub mod de0201_dtos_only_in_api_rest;
pub mod de0202_dtos_not_referenced_outside_api;
pub mod de0203_dtos_must_have_serde_derives;
pub mod de0204_dtos_must_have_toschema_derive;

pub use de0201_dtos_only_in_api_rest::DE0201_DTOS_ONLY_IN_API_REST;
pub use de0202_dtos_not_referenced_outside_api::DE0202_DTOS_NOT_REFERENCED_OUTSIDE_API;
pub use de0203_dtos_must_have_serde_derives::DE0203_DTOS_MUST_HAVE_SERDE_DERIVES;
pub use de0204_dtos_must_have_toschema_derive::DE0204_DTOS_MUST_HAVE_TOSCHEMA_DERIVE;
