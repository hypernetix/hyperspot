//! Compile-time type safety verification tests
//!
//! These tests demonstrate that the typed `OData` builder enforces type safety
//! at compile time. Uncomment the failing tests to see compile errors.

use modkit_odata::SortDir;
use modkit_sdk::odata::{items_stream, pages_stream, FieldRef, QueryBuilder, Schema};

#[derive(Copy, Clone, Eq, PartialEq)]
enum TestField {
    Id,
    Name,
    Age,
}

struct TestSchema;

impl Schema for TestSchema {
    type Field = TestField;

    fn field_name(field: Self::Field) -> &'static str {
        match field {
            TestField::Id => "id",
            TestField::Name => "name",
            TestField::Age => "age",
        }
    }
}

const ID: FieldRef<TestSchema, i32> = FieldRef::new(TestField::Id);
const NAME: FieldRef<TestSchema, String> = FieldRef::new(TestField::Name);
const AGE: FieldRef<TestSchema, i32> = FieldRef::new(TestField::Age);

#[test]
fn test_string_ops_only_on_string_fields() {
    // ✅ Valid: String field with string operations
    let _query = QueryBuilder::<TestSchema>::new()
        .filter(NAME.contains("test"))
        .build();

    let _query = QueryBuilder::<TestSchema>::new()
        .filter(NAME.startswith("test"))
        .build();

    let _query = QueryBuilder::<TestSchema>::new()
        .filter(NAME.endswith("test"))
        .build();

    // ❌ COMPILE ERROR: Uncomment to verify type safety
    // let _query = QueryBuilder::<TestSchema>::new()
    //     .filter(AGE.contains("test"))  // ERROR: no method `contains` for i32 field
    //     .build();

    // ❌ COMPILE ERROR: Uncomment to verify type safety
    // let _query = QueryBuilder::<TestSchema>::new()
    //     .filter(ID.startswith("test"))  // ERROR: no method `startswith` for Uuid field
    //     .build();
}

#[test]
fn test_comparison_ops_require_into_odata_value() {
    // ✅ Valid: Supported types
    let _query = QueryBuilder::<TestSchema>::new()
        .filter(AGE.eq(42)) // i32 implements IntoODataValue
        .build();

    let _query = QueryBuilder::<TestSchema>::new()
        .filter(NAME.eq("test")) // &str implements IntoODataValue
        .build();

    let _query = QueryBuilder::<TestSchema>::new()
        .filter(ID.eq(123)) // i32 implements IntoODataValue
        .build();

    // ❌ COMPILE ERROR: Uncomment to verify type safety
    // struct CustomType;
    // let _query = QueryBuilder::<TestSchema>::new()
    //     .filter(AGE.eq(CustomType))  // ERROR: CustomType doesn't implement IntoODataValue
    //     .build();
}

#[test]
fn test_no_manual_odata_query_construction() {
    // ✅ Valid: User only interacts with QueryBuilder
    let query = QueryBuilder::<TestSchema>::new()
        .filter(AGE.gt(18))
        .order_by(NAME, SortDir::Asc)
        .page_size(50)
        .build();

    // Query is built with filter_hash computed automatically
    assert!(query.filter_hash.is_some());
    assert!(query.has_filter());
    assert_eq!(query.limit, Some(50));

    // User never needs to:
    // - Manually create ODataQuery
    // - Set cursor
    // - Compute filter_hash
    // - Touch internal OData structures
}

#[test]
fn test_filter_hash_determinism() {
    let user_id = 123;

    // Build same query twice
    let query1 = QueryBuilder::<TestSchema>::new()
        .filter(ID.eq(user_id).and(AGE.gt(18)))
        .build();

    let query2 = QueryBuilder::<TestSchema>::new()
        .filter(ID.eq(user_id).and(AGE.gt(18)))
        .build();

    // Filter hashes must be identical
    assert_eq!(query1.filter_hash, query2.filter_hash);
    assert!(query1.filter_hash.is_some());

    // Different filter produces different hash
    let query3 = QueryBuilder::<TestSchema>::new().filter(AGE.lt(65)).build();

    assert_ne!(query1.filter_hash, query3.filter_hash);
}

#[test]
fn test_heterogeneous_field_selection() {
    // ✅ Valid: Homogeneous selection (all i32 fields)
    let query = QueryBuilder::<TestSchema>::new().select([ID, AGE]).build();

    assert!(query.has_select());
    let fields = query.selected_fields().unwrap();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0], "id");
    assert_eq!(fields[1], "age");
}

#[test]
fn test_complex_type_safe_query() {
    let user_id = 123;

    // Complex query with multiple type-safe operations
    let query = QueryBuilder::<TestSchema>::new()
        .filter(
            ID.ne(user_id)
                .and(NAME.contains("smith").or(NAME.startswith("john")))
                .and(AGE.ge(18).and(AGE.le(65))),
        )
        .order_by(NAME, SortDir::Asc)
        .order_by(AGE, SortDir::Desc)
        .select([ID, AGE])
        .page_size(25)
        .build();

    // Verify all components set correctly
    assert!(query.has_filter());
    assert!(query.filter_hash.is_some());
    assert_eq!(query.order.0.len(), 2);
    assert!(query.has_select());
    assert_eq!(query.limit, Some(25));
}

#[test]
fn test_pager_helpers_typecheck() {
    let _pages = pages_stream(
        QueryBuilder::<TestSchema>::new().page_size(1),
        |_| async move { Ok::<modkit_odata::Page<i32>, ()>(modkit_odata::Page::empty(1)) },
    );

    let _items = items_stream(
        QueryBuilder::<TestSchema>::new().page_size(1),
        |_| async move { Ok::<modkit_odata::Page<i32>, ()>(modkit_odata::Page::empty(1)) },
    );
}
