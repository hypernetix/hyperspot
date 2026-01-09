//! Settings Module Implementation
//!
//! The public API is defined in `settings-sdk` and re-exported here.

pub use settings_sdk::{Settings, SettingsApi, SettingsError, SettingsPatch};

pub mod module;
pub use module::SettingsModule;

pub mod local_client;

#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
#[doc(hidden)]
pub mod infra;
