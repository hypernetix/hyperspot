//! Domain layer for the tenant resolver.

pub mod error;
pub mod local_client;
pub mod service;

pub use error::DomainError;
pub use local_client::TenantResolverLocalClient;
pub use service::Service;
