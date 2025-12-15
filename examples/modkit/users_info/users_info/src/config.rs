use serde::{Deserialize, Serialize};

/// Configuration for the `users_info` module
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsersInfoConfig {
    #[serde(default = "default_page_size")]
    pub default_page_size: u32,
    #[serde(default = "default_max_page_size")]
    pub max_page_size: u32,
    #[serde(default = "default_audit_base_url")]
    pub audit_base_url: String,
    #[serde(default = "default_notifications_base_url")]
    pub notifications_base_url: String,
}

impl Default for UsersInfoConfig {
    fn default() -> Self {
        Self {
            default_page_size: default_page_size(),
            max_page_size: default_max_page_size(),
            audit_base_url: default_audit_base_url(),
            notifications_base_url: default_notifications_base_url(),
        }
    }
}

fn default_page_size() -> u32 {
    50
}

fn default_max_page_size() -> u32 {
    1000
}

fn default_audit_base_url() -> String {
    "http://audit.local".to_string()
}

fn default_notifications_base_url() -> String {
    "http://notifications.local".to_string()
}
