// simulated_dir=/hyperspot/modules/some_module/api/rest/dto.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GoodFieldScreamingSnakeCaseDto {
    #[serde(rename = "SCREAMING_SNAKE_FIELD")]
    pub id: String,
}

fn main() {}
