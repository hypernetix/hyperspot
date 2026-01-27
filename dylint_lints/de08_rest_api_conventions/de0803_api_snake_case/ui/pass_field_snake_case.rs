// simulated_dir=/hyperspot/modules/some_module/api/rest/dto.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GoodFieldSnakeCaseDto {
    // Should not trigger DE0803 - DTO fields must not use non-snake_case in serde rename/rename_all
    #[serde(rename = "snake_case_field")]
    pub id: String,
}

fn main() {}
