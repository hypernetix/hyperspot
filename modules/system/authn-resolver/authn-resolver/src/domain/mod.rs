//! Domain layer for the `AuthN` resolver.

pub mod error;
pub mod local_client;
pub mod service;

pub use error::DomainError;
pub use local_client::AuthNResolverLocalClient;
pub use service::Service;
