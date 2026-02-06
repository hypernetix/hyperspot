//! Domain layer for license enforcer gateway.

pub mod error;
pub mod local_client;
pub mod service;

pub use error::DomainError;
pub use local_client::LocalClient;
pub use service::Service;
