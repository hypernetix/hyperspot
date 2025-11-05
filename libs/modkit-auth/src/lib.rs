#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]
#![warn(warnings)]

pub mod claims;
pub mod errors;
pub mod traits;
pub mod types;

pub mod authorizer;
pub mod jwks;
pub mod scope_builder;

#[cfg(feature = "axum-ext")]
pub mod axum_ext;

pub use claims::Claims;
pub use errors::AuthError;
pub use types::{AuthRequirement, RoutePolicy, SecRequirement};
