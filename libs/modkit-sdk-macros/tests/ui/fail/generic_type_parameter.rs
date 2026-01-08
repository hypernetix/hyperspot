use modkit_sdk_macros::ODataSchema;

#[derive(ODataSchema)]
struct User {
    id: uuid::Uuid,
    age: i32,
}

fn main() {
    // This should fail: field constructors are not generic
    let _ = user::age::<String>();
}
