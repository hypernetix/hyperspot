// simulated_dir=/hyperspot/modules/some_module/api/rest/
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserDto {
    pub id: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProductDto {
    pub name: String,
}

fn main() {}
