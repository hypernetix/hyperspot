//! Domain layer for the Types Registry module.
//!
//! Contains business logic, error types, and repository traits.

pub mod error;
pub mod repo;
pub mod service;

pub use error::DomainError;
pub use repo::GtsRepository;
pub use service::TypesRegistryService;
