//! Settings SDK
//!
//! This crate provides the public API for the settings module:
//! - `SettingsApi` trait for inter-module communication
//! - Model types (`Settings`, `SettingsPatch`)
//! - Error type (`SettingsError`)
//!
//! Consumers obtain the client from `ClientHub`:
//! ```ignore
//! let client = hub.get::<dyn SettingsApi>()?;
//! let settings = client.get_settings(&ctx).await?;
//! ```

#![forbid(unsafe_code)]

pub mod api;
pub mod errors;
pub mod models;

pub use api::SettingsApi;
pub use errors::SettingsError;
pub use models::{Settings, SettingsPatch};
