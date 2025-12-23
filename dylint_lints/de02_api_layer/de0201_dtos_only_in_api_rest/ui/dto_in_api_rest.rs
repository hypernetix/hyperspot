// simulated_dir=/hyperspot/modules/some_module/api/rest/
#![allow(dead_code)]

// Should not trigger DE0201 - DTOs only in api/rest
pub struct UserDto {
    pub id: String,
}

fn main() {}
