// simulated_dir=/hyperspot/modules/some_module/contract/
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Order {
    pub id: String,
    pub total: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

fn main() {}
