//! Domain layer for the tenant resolver gateway.

pub mod error;
pub mod local_client;
pub mod service;

pub use error::DomainError;
pub use local_client::TenantResolverGwLocalClient;
pub use service::Service;
