// simulated_dir=/hyperspot/modules/some_module/api/rest/dto.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BadNestedFieldRenameDto {
    // Should trigger DE0803 - DTO fields must not use non-snake_case in serde rename/rename_all
    #[serde(rename(serialize = "userName"))]
    pub user_name_serialize: String,
    // Should trigger DE0803 - DTO fields must not use non-snake_case in serde rename/rename_all
    #[serde(rename(deserialize = "userName"))]
    pub user_name_deserialize: String,
}

fn main() {}
