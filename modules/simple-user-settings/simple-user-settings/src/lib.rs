//! Settings Module Implementation
//!
//! The public API is defined in `simple_user_settings-sdk` and re-exported here.

pub use simple_user_settings_sdk::{
    SettingsError, SimpleUserSettings, SimpleUserSettingsClient, SimpleUserSettingsPatch,
    SimpleUserSettingsUpdate,
};

pub mod module;
pub use module::SettingsModule;

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
