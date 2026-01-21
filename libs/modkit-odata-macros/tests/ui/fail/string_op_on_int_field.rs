use modkit_odata_macros::ODataSchema;

#[derive(ODataSchema)]
struct User {
    id: uuid::Uuid,
    age: i32,
}

fn main() {
    // This should fail: contains() is only available for String fields
    let _ = user::age().contains("test");
}
