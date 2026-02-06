// simulated_dir=/hyperspot/modules/example/src/domain/

// Test: Domain structs without #[domain_model] should trigger lint

// Should trigger DE0309 - domain_model attribute
pub struct User {
    pub id: i64,
    pub email: String,
}

// Should trigger DE0309 - domain_model attribute
pub enum UserStatus {
    Active,
    Inactive,
}

// Should trigger DE0309 - domain_model attribute
pub struct ServiceConfig {
    pub timeout_ms: u64,
}

fn main() {}
