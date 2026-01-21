// simulated_dir=/hyperspot/modules/some_module/api/rest/dto.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
// Should trigger DE0803 - DTO fields must not use non-snake_case in serde rename/rename_all
#[serde(rename_all = "UPPERCASE")]
pub struct BadUppercaseDto {
    pub id: String,
}

fn main() {}
