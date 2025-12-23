// simulated_dir=/hyperspot/modules/some_module/api/rest/
use serde::Serialize;

#[allow(dead_code)]
#[derive(Debug, Serialize)]
// Should trigger DE0203 - DTOs must have serde derives
pub struct OnlySerializeDto {
    pub id: String,
}

fn main() {}
