// simulated_dir=/hyperspot/modules/some_module/contract/
// Should not trigger DE0103 - HTTP types in contract
use std::collections::HashMap;
// Should trigger DE0103 - HTTP types in contract
use http::StatusCode;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum OrderStatus {
    Pending,
    Confirmed,
}

#[allow(dead_code)]
pub struct OrderResult {
    pub status: StatusCode,
    pub metadata: HashMap<String, String>,
}

fn main() {}
