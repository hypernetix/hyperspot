//! Configuration for the directory service module

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct DirectoryServiceConfig {
    // No transport config needed - gRPC hub handles the bind
    // Future: could add service-level config here (timeouts, etc.)
}
