// simulated_dir=/hyperspot/modules/some_module/api/rest/
#![allow(unused)]

mod api {
    pub mod rest {
        pub mod dto {
            pub struct UserDto;
        }

        // Should not trigger DE0202 - DTOs not referenced outside api
        use crate::api::rest::dto::UserDto;
    }
}

fn main() {}
