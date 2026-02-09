//! Domain layer for the `AuthZ` resolver.

pub mod error;
pub mod local_client;
pub mod service;

pub use error::DomainError;
pub use local_client::AuthZResolverLocalClient;
pub use service::Service;
