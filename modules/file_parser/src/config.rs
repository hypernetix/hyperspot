use serde::{Deserialize, Serialize};

/// Configuration for the file_parser module
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileParserConfig {
    #[serde(default = "default_max_file_size_mb")]
    pub max_file_size_mb: u64,
    #[serde(default = "default_download_timeout_secs")]
    pub download_timeout_secs: u64,
}

impl Default for FileParserConfig {
    fn default() -> Self {
        Self {
            max_file_size_mb: default_max_file_size_mb(),
            download_timeout_secs: default_download_timeout_secs(),
        }
    }
}

fn default_max_file_size_mb() -> u64 {
    100
}

fn default_download_timeout_secs() -> u64 {
    60
}
