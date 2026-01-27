// simulated_dir=/hyperspot/modules/some_module/api/rest/dto.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum GoodEnumVariantRenameDto {
    // Should not trigger DE0803 - DTO fields must not use non-snake_case in serde rename/rename_all
    #[serde(rename = "first_variant")]
    FirstVariant,
    // Should not trigger DE0803 - DTO fields must not use non-snake_case in serde rename/rename_all
    #[serde(rename = "second_variant")]
    SecondVariant,
}

fn main() {}
