// simulated_dir=/hyperspot/modules/some_module/contract/types.rs
// Test file for DE1107: Public items with proper doc comments
#![allow(dead_code)]

// Should not trigger DE1107 - doc comments
/// A user entity representing a registered user.
pub struct User {
    pub id: u64,
    pub name: String,
}

// Should not trigger DE1107 - doc comments
/// Status of an entity.
pub enum Status {
    Active,
    Inactive,
}

// Should not trigger DE1107 - doc comments
/// Creates a new user.
pub fn create_user() {}

// Private items don't need docs
struct InternalHelper {
    value: i32,
}

fn main() {}
