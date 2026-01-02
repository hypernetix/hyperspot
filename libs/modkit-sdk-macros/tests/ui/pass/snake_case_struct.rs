use modkit_sdk_macros::ODataSchema;

#[derive(ODataSchema)]
struct UserProfile {
    user_id: uuid::Uuid,
    first_name: String,
    last_name: String,
}

fn main() {
    let _id = user_profile::user_id();
    let _first = user_profile::first_name();
    let _last = user_profile::last_name();
}
