// simulated_dir=/hyperspot/modules/some_module/api/rest/dto.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
// Should trigger DE0803 - DTO fields must not use non-snake_case in serde rename/rename_all
#[serde(rename_all(serialize = "camelCase"))]
pub struct BadNestedRenameAllSerializeDto {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
// Should trigger DE0803 - DTO fields must not use non-snake_case in serde rename/rename_all
#[serde(rename_all(deserialize = "camelCase"))]
pub struct BadNestedRenameAllDeserializeDto {
    pub id: String,
}

fn main() {}
