// simulated_dir=/hyperspot/modules/some_module/other/structs.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
// Should not trigger DE0803 - DTOs must not use non-snake_case in serde rename_all (DE0803)
#[serde(rename_all = "PascalCase")]
pub struct OutsideApiDto {
    pub id: String,
}

fn main() {}
