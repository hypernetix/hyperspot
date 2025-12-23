// simulated_dir=/hyperspot/modules/some_module/domain/
#![allow(unused)]

mod api {
    pub mod rest {
        pub mod dto {
            pub struct UserDto;
        }
    }
}

// Should trigger DE0202 - DTOs not referenced outside api
use crate::api::rest::dto::UserDto;

pub struct UserService;

fn main() {}
