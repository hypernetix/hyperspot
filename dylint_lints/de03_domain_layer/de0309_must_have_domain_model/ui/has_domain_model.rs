// simulated_dir=/hyperspot/modules/example/src/domain/

// Test: Domain structs WITH #[domain_model] should NOT trigger lint

// For testing purposes, we define a dummy domain_model attribute
// In real code, this comes from modkit_macros
#[allow(dead_code)]
mod modkit {
    pub use modkit_macros::domain_model;
}

use modkit::domain_model;

// Should not trigger DE0309 - domain_model attribute
#[domain_model]
pub struct User {
    pub id: i64,
    pub email: String,
}

// Should not trigger DE0309 - domain_model attribute
#[domain_model]
pub enum UserStatus {
    Active,
    Inactive,
}

// Should not trigger DE0309 - domain_model attribute
#[domain_model]
pub struct ServiceConfig {
    pub timeout_ms: u64,
}

fn main() {}
