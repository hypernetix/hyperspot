// simulated_dir=/hyperspot/modules/some_module/api/rest/dto.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct BadUppercaseDto {
    pub id: String,
}

fn main() {}
