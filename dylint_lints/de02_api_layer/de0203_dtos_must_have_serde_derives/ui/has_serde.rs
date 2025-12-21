// simulated_dir=/hyperspot/modules/some_module/api/rest/
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    pub id: String,
}

#[allow(dead_code)]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ProductDto {
    pub name: String,
}

fn main() {}
