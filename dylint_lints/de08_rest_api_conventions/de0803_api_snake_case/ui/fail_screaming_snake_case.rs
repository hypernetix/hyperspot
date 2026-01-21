// simulated_dir=/hyperspot/modules/some_module/api/rest/dto.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
// Should trigger DE0803 - DTO fields must not use non-snake_case in serde rename/rename_all
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct BadScreamingSnakeCaseDto {
    pub id: String,
}

fn main() {}
