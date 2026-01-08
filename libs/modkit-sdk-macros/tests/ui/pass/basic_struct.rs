use modkit_sdk_macros::ODataSchema;

#[derive(ODataSchema)]
struct User {
    id: uuid::Uuid,
    email: String,
    name: String,
}

fn main() {
    let _field = UserField::Id;
    let _schema = UserSchema;
    let _id = user::id();
}
