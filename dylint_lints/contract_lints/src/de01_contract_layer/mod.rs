//! DE01: Contract Layer Checks
//!
//! Linters to ensure contract modules remain transport-agnostic
//! and properly isolated from implementation details.

pub mod de0101_no_serde_in_contract;
pub mod de0102_no_toschema_in_contract;
pub mod de0103_no_http_types_in_contract;

pub use de0101_no_serde_in_contract::DE0101_NO_SERDE_IN_CONTRACT;
pub use de0102_no_toschema_in_contract::DE0102_NO_TOSCHEMA_IN_CONTRACT;
pub use de0103_no_http_types_in_contract::DE0103_NO_HTTP_TYPES_IN_CONTRACT;
