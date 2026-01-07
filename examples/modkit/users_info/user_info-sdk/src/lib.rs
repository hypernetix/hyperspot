//! User Info SDK
//!
//! This crate provides the public API for the `user_info` module:
//! - `UsersInfoClient` trait
//! - Model types for users, addresses and cities
//! - Error type (`UsersInfoError`)
//! - `OData` filter field definitions (behind `odata` feature)
//!
//! ## Usage
//!
//! Consumers obtain the client from `ClientHub`:
//! ```ignore
//! use user_info_sdk::UsersInfoClient;
//!
//! // Get the client from ClientHub
//! let client = hub.get::<dyn UsersInfoClient>()?;
//!
//! // Use the API
//! let user = client.get_user(&ctx, user_id).await?;
//! let users = client.list_users(&ctx, query).await?;
//! ```
//!
//! ## `OData` Support
//!
//! Enable the `odata` feature to access filter field definitions:
//! ```ignore
//! use user_info_sdk::odata::{UserFilterField, CityFilterField};
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod client;
pub mod errors;
pub mod models;

// OData filter field definitions (feature-gated)
#[cfg(feature = "odata")]
pub mod odata;

// Re-export main types at crate root for convenience
pub use client::UsersInfoClient;
pub use errors::UsersInfoError;
pub use models::{
    Address, AddressPatch, City, CityPatch, NewAddress, NewCity, NewUser, UpdateAddressRequest,
    UpdateCityRequest, UpdateUserRequest, User, UserFull, UserPatch,
};
