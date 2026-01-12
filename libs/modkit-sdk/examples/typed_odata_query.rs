//! Example demonstrating the typed `OData` query builder
//!
//! This example shows how to define a schema, create typed field references,
//! and build type-safe `OData` queries with filters, ordering, and field selection.

#![allow(clippy::use_debug)]

fn main() {
    use modkit_odata::SortDir;
    use modkit_sdk::odata::{FieldRef, QueryBuilder, Schema};
    use uuid::Uuid;

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    enum UserField {
        Id,
        Name,
        Email,
        Age,
        IsActive,
    }

    struct UserSchema;

    impl Schema for UserSchema {
        type Field = UserField;

        fn field_name(field: Self::Field) -> &'static str {
            match field {
                UserField::Id => "id",
                UserField::Name => "name",
                UserField::Email => "email",
                UserField::Age => "age",
                UserField::IsActive => "is_active",
            }
        }
    }

    const ID: FieldRef<UserSchema, Uuid> = FieldRef::new(UserField::Id);
    const NAME: FieldRef<UserSchema, String> = FieldRef::new(UserField::Name);
    const EMAIL: FieldRef<UserSchema, String> = FieldRef::new(UserField::Email);
    const AGE: FieldRef<UserSchema, i32> = FieldRef::new(UserField::Age);
    const IS_ACTIVE: FieldRef<UserSchema, bool> = FieldRef::new(UserField::IsActive);

    println!("=== Typed OData Query Builder Examples ===\n");

    // Example 1: Simple equality filter
    println!("1. Simple equality filter:");
    let user_id = Uuid::new_v4();
    let query = QueryBuilder::<UserSchema>::new()
        .filter(ID.eq(user_id))
        .build();
    println!("   Filter: id eq {user_id}");
    println!("   Filter hash: {:#?}\n", query.filter_hash);

    // Example 2: String contains filter
    println!("2. String contains filter:");
    let query = QueryBuilder::<UserSchema>::new()
        .filter(NAME.contains("john"))
        .build();
    println!("   Filter: contains(name, 'john')");
    println!("   Filter hash: {:#?}\n", query.filter_hash);

    // Example 3: Complex filter with AND/OR
    println!("3. Complex filter with AND/OR:");
    let query = QueryBuilder::<UserSchema>::new()
        .filter(
            IS_ACTIVE
                .eq(true)
                .and(AGE.ge(18))
                .and(AGE.le(65))
                .and(EMAIL.endswith("@example.com")),
        )
        .build();
    println!("   Filter: is_active eq true AND age ge 18 AND age le 65 AND endswith(email, '@example.com')");
    println!("   Filter hash: {:#?}\n", query.filter_hash);

    // Example 4: Filter with OR combinator
    println!("4. Filter with OR combinator:");
    let query = QueryBuilder::<UserSchema>::new()
        .filter(AGE.lt(18).or(AGE.gt(65)))
        .build();
    println!("   Filter: age lt 18 OR age gt 65");
    println!("   Filter hash: {:#?}\n", query.filter_hash);

    // Example 5: Filter with NOT
    println!("5. Filter with NOT:");
    let query = QueryBuilder::<UserSchema>::new()
        .filter(NAME.startswith("test").not())
        .build();
    println!("   Filter: NOT startswith(name, 'test')");
    println!("   Filter hash: {:#?}\n", query.filter_hash);

    // Example 6: Ordering
    println!("6. Query with ordering:");
    let _query = QueryBuilder::<UserSchema>::new()
        .order_by(NAME, SortDir::Asc)
        .order_by(AGE, SortDir::Desc)
        .build();
    println!("   Order: name asc, age desc\n");

    // Example 7: Field selection (projection)
    println!("7. Query with field selection:");
    let _query = QueryBuilder::<UserSchema>::new()
        .select([NAME, EMAIL])
        .build();
    println!("   Select: id, name, email\n");

    // Example 8: Page size limit
    println!("8. Query with page size:");
    let _query = QueryBuilder::<UserSchema>::new().page_size(50).build();
    println!("   Limit: Some(50)\n");

    // Example 9: Full query with all features
    println!("9. Full query with all features:");
    let user_id = Uuid::new_v4();
    let query = QueryBuilder::<UserSchema>::new()
        .filter(
            ID.ne(user_id)
                .and(IS_ACTIVE.eq(true))
                .and(AGE.ge(18))
                .and(NAME.contains("smith")),
        )
        .order_by(NAME, SortDir::Asc)
        .order_by(AGE, SortDir::Desc)
        .select([NAME, EMAIL])
        .page_size(25)
        .build();
    println!(
        "   Filter: id ne {user_id} AND is_active eq true AND age ge 18 AND contains(name, 'smith')"
    );
    println!("   Order: name asc, age desc");
    if let Some(fields) = query.selected_fields() {
        println!("   Select: {}", fields.join(", "));
    }
    println!("   Limit: {:#?}", query.limit);
    println!("   Filter hash: {:#?}\n", query.filter_hash);

    // Example 10: Demonstrating filter hash stability
    println!("10. Filter hash stability:");
    let id1 = Uuid::new_v4();
    let query1 = QueryBuilder::<UserSchema>::new().filter(ID.eq(id1)).build();
    let query2 = QueryBuilder::<UserSchema>::new().filter(ID.eq(id1)).build();
    println!("   Query 1 hash: {:#?}", query1.filter_hash);
    println!("   Query 2 hash: {:#?}", query2.filter_hash);
    println!(
        "   Hashes match: {}\n",
        query1.filter_hash == query2.filter_hash
    );

    println!("=== All examples completed successfully ===");
}
