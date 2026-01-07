//! Infrastructure layer for the Types Registry module.
//!
//! Contains storage implementations and adapters.

pub mod storage;

pub use storage::InMemoryGtsRepository;
