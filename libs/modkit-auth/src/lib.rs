#![warn(warnings)]

// Core modules
pub mod claims;
pub mod errors;
pub mod traits;
pub mod types;

pub mod authorizer;
pub mod jwks;
pub mod scope_builder;

// Plugin system modules
pub mod auth_mode;
pub mod claims_error;
pub mod config;
pub mod config_error;
pub mod dispatcher;
pub mod metrics;
pub mod plugin_traits;
pub mod plugins;
pub mod providers;
pub mod validation;

#[cfg(feature = "axum-ext")]
pub mod axum_ext;

// Core exports
pub use claims::Claims;
pub use errors::AuthError;
pub use traits::TokenValidator;
pub use types::{AuthRequirement, RoutePolicy, SecRequirement};

// Plugin system exports
pub use auth_mode::{AuthModeConfig, PluginRegistry};
pub use claims_error::ClaimsError;
pub use config::{build_auth_dispatcher, AuthConfig, JwksConfig, PluginConfig};
pub use config_error::ConfigError;
pub use dispatcher::AuthDispatcher;
pub use metrics::{AuthEvent, AuthMetricLabels, AuthMetrics, LoggingMetrics, NoOpMetrics};
pub use plugin_traits::{ClaimsPlugin, IntrospectionProvider, KeyProvider};
pub use validation::ValidationConfig;
