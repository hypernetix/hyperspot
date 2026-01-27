// simulated_dir=/hyperspot/modules/some_module/api/rest/dto.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum BadEnumVariantRenameDto {
    // Should trigger DE0803 - DTO fields must not use non-snake_case in serde rename/rename_all
    #[serde(rename = "firstVariant")]
    FirstVariant,
    // Should trigger DE0803 - DTO fields must not use non-snake_case in serde rename/rename_all
    #[serde(rename = "secondVariant")]
    SecondVariant,
}

fn main() {}
