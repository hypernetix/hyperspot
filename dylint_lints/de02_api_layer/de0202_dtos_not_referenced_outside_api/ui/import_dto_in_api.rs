// simulated_dir=/hyperspot/modules/some_module/api/rest/
#![allow(unused)]

mod api {
    pub mod rest {
        pub mod dto {
            pub struct UserDto;
        }

        mod handlers {
            // This is allowed
            use crate::api::rest::dto::UserDto;
        }
    }
}

fn main() {}
