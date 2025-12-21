// simulated_dir=/hyperspot/modules/some_module/api/rest/
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct UserDto {
    pub id: String,
}

fn main() {}
