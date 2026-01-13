use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    // Should trigger DE0301 - Config duration fields must use humantime serde
    pub timeout: Duration,
}

fn main() {}
