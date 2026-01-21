//! Local client adapters split by resource.
//!
//! This module houses the object-safe streaming facades that wrap the
//! domain service. Type erasure happens here, at the SDK boundary.

pub mod addresses;
pub mod cities;
pub mod client;
pub mod users;
