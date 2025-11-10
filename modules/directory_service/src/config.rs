//! Configuration for the directory service module

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct DirectoryServiceConfig {
    pub bind_addr: String,
}

impl Default for DirectoryServiceConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:7444".to_string(),
        }
    }
}
