//! OData filter field definitions for `user_info` resources.
//!
//! This module defines the filterable fields for each resource exposed by the
//! `user_info` module. These field enums are used for type-safe OData filter
//! construction and validation.
//!
//! ## Usage
//!
//! These types are primarily used by:
//! - REST API layer for OpenAPI schema generation
//! - Infrastructure layer for mapping to database columns
//! - Client code for type-safe filter construction
//!
//! ## Feature Gate
//!
//! This module requires the `odata` feature to be enabled.

mod addresses;
mod cities;
mod languages;
mod users;

pub use addresses::*;
pub use cities::*;
pub use languages::*;
pub use users::*;
