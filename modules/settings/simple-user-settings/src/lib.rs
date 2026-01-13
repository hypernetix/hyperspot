//! Settings Module Implementation
//!
//! The public API is defined in `simple-user-settings-sdk` and re-exported here.

pub use simple_user_settings_sdk::{
    SettingsError, SimpleUserSettings, SimpleUserSettingsApi, SimpleUserSettingsPatch,
};

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
pub mod errors;
#[doc(hidden)]
pub mod infra;
