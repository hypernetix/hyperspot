//! Outbound `OAuth2` client credentials flow.
//!
//! This module implements token acquisition, caching, and automatic injection
//! for outbound HTTP requests to vendor services secured with `OAuth2`.

pub mod builder_ext;
pub mod config;
pub(crate) mod discovery;
pub mod error;
pub mod layer;
pub(crate) mod source;
pub mod token;
pub mod types;

pub use builder_ext::HttpClientBuilderExt;
pub use config::OAuthClientConfig;
pub use error::TokenError;
pub use layer::BearerAuthLayer;
pub use token::Token;
pub use types::{ClientAuthMethod, SecretString};
