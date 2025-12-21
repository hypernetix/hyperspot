#![allow(unused)]

mod api {
    pub mod rest {
        pub mod dto {
            pub struct UserDto;
        }
    }
}

mod contract {
    use crate::api::rest::dto::UserDto;

    pub fn get_user() -> UserDto {
        UserDto
    }
}

fn main() {}
