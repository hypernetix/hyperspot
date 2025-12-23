// simulated_dir=/hyperspot/modules/some_module/api/rest/
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
// Should trigger DE0204 - DTOs must have ToSchema derive
pub struct UserDto {
    pub id: String,
}

fn main() {}
