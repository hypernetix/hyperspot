use modkit_odata_macros::ODataSchema;

#[derive(ODataSchema)]
struct User {
    id: uuid::Uuid,
    age: i32,
}

struct NotODataValue;

fn main() {
    // This should fail: comparing with a type that doesn't implement IntoODataValue
    let _ = user::age().eq(NotODataValue);
}
