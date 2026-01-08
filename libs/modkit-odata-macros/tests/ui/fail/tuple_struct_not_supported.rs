use modkit_odata_macros::ODataSchema;

#[derive(ODataSchema)]
struct User(uuid::Uuid, String);

fn main() {}
