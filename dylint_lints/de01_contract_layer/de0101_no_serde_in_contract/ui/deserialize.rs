// simulated_dir=/hyperspot/modules/some_module/contract/
use serde::Deserialize;

#[allow(dead_code)]
// Should trigger DE0101 - Serde in contract
#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    pub id: String,
    pub total: f64,
}

#[allow(dead_code)]
// Should trigger DE0101 - Serde in contract
#[derive(Debug, Clone, Deserialize)]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

fn main() {}
