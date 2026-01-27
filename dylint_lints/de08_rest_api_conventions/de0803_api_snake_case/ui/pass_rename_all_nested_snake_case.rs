// simulated_dir=/hyperspot/modules/some_module/api/rest/dto.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
// Should not trigger DE0803 - DTOs must not use non-snake_case in serde rename/rename_all
#[serde(rename_all(serialize = "snake_case", deserialize = "snake_case"))]
pub struct GoodNestedRenameAllDto {
    pub id: String,
}

fn main() {}
