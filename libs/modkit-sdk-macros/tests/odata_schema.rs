use modkit_odata::SortDir;
use modkit_sdk::odata::{AsFieldKey, FieldRef, FilterExpr, QueryBuilder, Schema};
use modkit_sdk_macros::ODataSchema;

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

#[test]
fn test_query_builder_integration() {
    let user_id = uuid::Uuid::new_v4();

    let query = QueryBuilder::<UserSchema>::new()
        .filter(user::id().eq(user_id))
        .order_by(user::name(), SortDir::Asc)
        .page_size(50)
        .build();

    assert!(query.has_filter());
    assert!(query.filter_hash.is_some());
    assert_eq!(query.order.0.len(), 1);
    assert_eq!(query.limit, Some(50));
}

#[test]
fn test_string_operations() {
    let query = QueryBuilder::<UserSchema>::new()
        .filter(user::email().contains("@example.com"))
        .build();

    assert!(query.has_filter());
}

#[test]
fn test_complex_filter_with_generated_fields() {
    let user_id = uuid::Uuid::new_v4();

    let query = QueryBuilder::<UserSchema>::new()
        .filter(
            user::id()
                .eq(user_id)
                .and(user::name().contains("john"))
                .and(user::age().ge(18).and(user::age().le(65))),
        )
        .order_by(user::name(), SortDir::Asc)
        .order_by(user::age(), SortDir::Desc)
        .build();

    assert!(query.has_filter());
    assert_eq!(query.order.0.len(), 2);
}

#[test]
fn test_select_with_generated_fields() {
    let id = user::id();
    let email = user::email();
    let name = user::name();

    let query = QueryBuilder::<UserSchema>::new()
        .select(&[
            &id as &dyn AsFieldKey<UserSchema>,
            &email as &dyn AsFieldKey<UserSchema>,
            &name as &dyn AsFieldKey<UserSchema>,
        ])
        .build();

    assert!(query.has_select());
    let fields = query.selected_fields().expect("select fields");
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0], "id");
    assert_eq!(fields[1], "email");
    assert_eq!(fields[2], "name");
}

#[test]
fn test_field_ref_copy_clone() {
    let field1 = user::id();
    let field2 = field1;
    let field3 = field1;

    assert_eq!(field1.name(), field2.name());
    assert_eq!(field1.name(), field3.name());
}

#[test]
fn test_comparison_operators() {
    let _query = QueryBuilder::<UserSchema>::new()
        .filter(user::age().gt(18))
        .build();

    let _query = QueryBuilder::<UserSchema>::new()
        .filter(user::age().ge(18))
        .build();

    let _query = QueryBuilder::<UserSchema>::new()
        .filter(user::age().lt(65))
        .build();

    let _query = QueryBuilder::<UserSchema>::new()
        .filter(user::age().le(65))
        .build();

    let _query = QueryBuilder::<UserSchema>::new()
        .filter(user::age().ne(0))
        .build();
}
