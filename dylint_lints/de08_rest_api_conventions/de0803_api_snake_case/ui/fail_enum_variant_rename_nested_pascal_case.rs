// simulated_dir=/hyperspot/modules/some_module/api/rest/dto.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum BadNestedVariantRenameEnum {
    // Should trigger DE0803 - DTO fields must not use non-snake_case in serde rename/rename_all
    #[serde(rename(serialize = "FirstVariant"))]
    FirstVariantSerialize,
    // Should trigger DE0803 - DTO fields must not use non-snake_case in serde rename/rename_all
    #[serde(rename(deserialize = "FirstVariant"))]
    FirstVariantDeserialize,
}

fn main() {}
