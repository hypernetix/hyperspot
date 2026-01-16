//! Settings SDK
//!
//! This crate provides the public API for the settings module:
//! - `SimpleUserSettingsClient` trait for inter-module communication
//! - Model types (`SimpleUserSettings`, `SimpleUserSettingsPatch`)
//! - Error type (`SettingsError`)
//!
//! Consumers obtain the client from `ClientHub`:
//! ```ignore
//! let client = hub.get::<dyn SimpleUserSettingsClient>()?;
//! let settings = client.get_settings(&ctx).await?;
//! ```

#![forbid(unsafe_code)]

pub mod api;
pub mod errors;
pub mod models;

pub use api::SimpleUserSettingsClient;
pub use errors::SettingsError;
pub use models::{SimpleUserSettings, SimpleUserSettingsPatch, SimpleUserSettingsUpdate};
