// simulated_dir=/hyperspot/modules/some_module/api/rest/dto.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BadFieldUppercaseDto {
    // Should trigger DE0803 - DTO fields must not use non-snake_case in serde rename/rename_all
    #[serde(rename = "UPPERCASE_FIELD")]
    pub id: String,
}

fn main() {}
