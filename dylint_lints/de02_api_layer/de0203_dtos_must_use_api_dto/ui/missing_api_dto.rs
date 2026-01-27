// simulated_dir=/hyperspot/modules/some_module/api/rest/
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
// Should trigger DE0203 - DTOs must use api_dto
pub struct UserDto {
    pub id: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
// Should trigger DE0203 - DTOs must use api_dto
pub struct ProductDto {
    pub name: String,
}

fn main() {}
