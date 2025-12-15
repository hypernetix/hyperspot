// Test DE0103: No HTTP Types in Contract
// This lint checks for HTTP type IMPORTS in contract modules

// Should trigger DE0103 - HTTP imports in contract
use http::StatusCode;

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

pub fn process() -> u16 {
    200
}
