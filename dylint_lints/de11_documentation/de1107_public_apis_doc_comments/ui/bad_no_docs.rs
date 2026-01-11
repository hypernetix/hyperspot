// simulated_dir=/hyperspot/modules/some_module/contract/types.rs
// Test file for DE1107: Public items without doc comments
#![allow(dead_code)]

// Should trigger DE1107 - doc comments
pub struct User {
    pub id: u64,
    pub name: String,
}

// Should trigger DE1107 - doc comments
pub enum Status {
    Active,
    Inactive,
}

// Should trigger DE1107 - doc comments
pub fn create_user() {}

fn main() {}
