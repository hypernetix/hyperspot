// simulated_dir=/hyperspot/modules/some_module/api/rest/
#[allow(dead_code)]
#[derive(Debug, Clone)]
// Should trigger DE0203 - DTOs must have serde derives
pub struct UserDto {
    pub id: String,
}

#[allow(dead_code)]
#[derive(Debug)]
// Should trigger DE0203 - DTOs must have serde derives
pub struct ProductDto {
    pub name: String,
}

fn main() {}
