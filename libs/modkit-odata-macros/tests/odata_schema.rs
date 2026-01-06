use modkit_odata::schema::{FieldRef, Schema};
use modkit_odata_macros::ODataSchema;

#[derive(ODataSchema)]
#[allow(dead_code)]
struct User {
    id: uuid::Uuid,
    email: String,
    name: String,
    age: i32,
}

#[derive(ODataSchema)]
#[allow(dead_code)]
struct Product {
    #[odata(name = "product_id")]
    id: uuid::Uuid,
    #[odata(name = "product_name")]
    name: String,
    price: i32,
    #[odata(name = "created_at")]
    created_at: String,
}

#[test]
fn test_user_field_enum_generated() {
    let _ = UserField::Id;
    let _ = UserField::Email;
    let _ = UserField::Name;
    let _ = UserField::Age;
}

#[test]
fn test_user_schema_impl() {
    assert_eq!(UserSchema::field_name(UserField::Id), "id");
    assert_eq!(UserSchema::field_name(UserField::Email), "email");
    assert_eq!(UserSchema::field_name(UserField::Name), "name");
    assert_eq!(UserSchema::field_name(UserField::Age), "age");
}

#[test]
fn test_user_field_constructors() {
    let id_field: FieldRef<UserSchema, uuid::Uuid> = user::id();
    let email_field: FieldRef<UserSchema, String> = user::email();
    let name_field: FieldRef<UserSchema, String> = user::name();
    let age_field: FieldRef<UserSchema, i32> = user::age();

    assert_eq!(id_field.name(), "id");
    assert_eq!(email_field.name(), "email");
    assert_eq!(name_field.name(), "name");
    assert_eq!(age_field.name(), "age");
}

#[test]
fn test_product_custom_names() {
    assert_eq!(ProductSchema::field_name(ProductField::Id), "product_id");
    assert_eq!(
        ProductSchema::field_name(ProductField::Name),
        "product_name"
    );
    assert_eq!(ProductSchema::field_name(ProductField::Price), "price");
    assert_eq!(
        ProductSchema::field_name(ProductField::CreatedAt),
        "created_at"
    );
}

#[test]
fn test_product_field_constructors() {
    let id_field = product::id();
    let name_field = product::name();
    let price_field = product::price();
    let created_at_field = product::created_at();

    assert_eq!(id_field.name(), "product_id");
    assert_eq!(name_field.name(), "product_name");
    assert_eq!(price_field.name(), "price");
    assert_eq!(created_at_field.name(), "created_at");
}

// Note: QueryBuilder integration tests moved to modkit-sdk
// since QueryBuilder is SDK-level functionality, not protocol-level.
// These macro tests focus on verifying correct Schema trait generation.

// Test removed - QueryBuilder functionality belongs in modkit-sdk tests

#[test]
fn test_field_ref_copy_clone() {
    let field1 = user::id();
    let field2 = field1;
    let field3 = field1;

    assert_eq!(field1.name(), field2.name());
    assert_eq!(field1.name(), field3.name());
}

// Comparison operator tests moved to modkit-sdk where QueryBuilder lives
