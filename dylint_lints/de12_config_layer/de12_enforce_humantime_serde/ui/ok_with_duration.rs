use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    // Should not trigger DE0301 - Config duration fields must use humantime serde
    #[serde(with = "modkit_utils::humantime_serde")]
    pub timeout: std::time::Duration,
}

fn main() {}
