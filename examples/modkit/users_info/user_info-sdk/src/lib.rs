//! User Info SDK
//!
//! This crate provides the public API for the user_info module:
//! - `UsersInfoApi` trait
//! - Model types (`User`, `NewUser`, `UserPatch`, `UpdateUserRequest`)
//! - Error type (`UsersInfoError`)
//!
//! ## Usage
//!
//! Consumers obtain the client from `ClientHub`:
//! ```ignore
//! use user_info_sdk::UsersInfoApi;
//!
//! // Get the client from ClientHub
//! let client = hub.get::<dyn UsersInfoApi>()?;
//!
//! // Use the API
//! let user = client.get_user(&ctx, user_id).await?;
//! let users = client.list_users(&ctx, query).await?;
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod api;
pub mod errors;
pub mod models;

// Re-export main types at crate root for convenience
pub use api::UsersInfoApi;
pub use errors::UsersInfoError;
pub use models::{NewUser, UpdateUserRequest, User, UserPatch};
