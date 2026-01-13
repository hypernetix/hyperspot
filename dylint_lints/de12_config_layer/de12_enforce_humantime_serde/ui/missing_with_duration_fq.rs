use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    // Should trigger DE0301 - Config duration fields must use humantime serde
    pub timeout: std::time::Duration,
}

fn main() {}
