// simulated_dir=/hyperspot/modules/some_module/api/rest/
use serde::Serialize;

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct OnlySerializeDto {
    pub id: String,
}

fn main() {}
