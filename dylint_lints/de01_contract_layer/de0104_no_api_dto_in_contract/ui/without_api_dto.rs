// simulated_dir=/hyperspot/modules/some_module/api/rest/
#![allow(dead_code)]

// Should not trigger DE0104 - api_dto in contract
#[modkit_macros::api_dto(request, response)]
pub struct UserDto {
    pub id: String,
    pub name: String,
}

// Should not trigger DE0104 - api_dto in contract
pub struct PlainStruct {
    pub field: String,
}

fn main() {}
