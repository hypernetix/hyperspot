use modkit_sdk_macros::ODataSchema;

#[derive(ODataSchema)]
struct User(uuid::Uuid, String);

fn main() {}
