//! Typed `OData` query builder
//!
//! This module provides a generic, reusable typed query builder for `OData` that produces
//! `ODataQuery` with correct filter hashing.
//!
//! # Design
//!
//! - **Schema trait**: Defines field enums and their string mappings (from `schema` module)
//! - **`FieldRef`**: Type-safe field references with schema and Rust type markers
//! - **Filter constructors**: Typed comparison and string operations returning AST expressions
//! - **`QueryBuilder`**: Fluent API for building queries with filter/order/select/limit
//!
//! # Example
//!
//! ```rust,ignore
//! use modkit_odata::{Schema, FieldRef, QueryBuilder, SortDir};
//!
//! #[derive(Copy, Clone, Eq, PartialEq)]
//! enum UserField {
//!     Id,
//!     Name,
//!     Email,
//! }
//!
//! struct UserSchema;
//!
//! impl Schema for UserSchema {
//!     type Field = UserField;
//!
//!     fn field_name(field: Self::Field) -> &'static str {
//!         match field {
//!             UserField::Id => "id",
//!             UserField::Name => "name",
//!             UserField::Email => "email",
//!         }
//!     }
//! }
//!
//! // Define typed field references
//! const ID: FieldRef<UserSchema, uuid::Uuid> = FieldRef::new(UserField::Id);
//! const NAME: FieldRef<UserSchema, String> = FieldRef::new(UserField::Name);
//!
//! // Build a query
//! let user_id = uuid::Uuid::new_v4();
//! let query = QueryBuilder::<UserSchema>::new()
//!     .filter(ID.eq(user_id).and(NAME.contains("john")))
//!     .order_by(NAME, SortDir::Asc)
//!     .page_size(50)
//!     .build();
//! ```

use crate::schema::{AsFieldKey, AsFieldName, FieldRef, Schema};
use crate::{
    ODataOrderBy, ODataQuery, OrderKey, SortDir, ast::Expr, pagination::short_filter_hash,
};
use std::marker::PhantomData;

/// Typed query builder for `OData` queries.
///
/// This builder provides a fluent API for constructing `ODataQuery` instances
/// with type-safe field references and automatic filter hashing.
///
/// # Example
///
/// ```rust,ignore
/// let query = QueryBuilder::<UserSchema>::new()
///     .filter(NAME.contains("john"))
///     .order_by(NAME, SortDir::Asc)
///     .select([NAME, EMAIL])
///     .page_size(50)
///     .build();
/// ```
pub struct QueryBuilder<S: Schema> {
    filter: Option<Expr>,
    order: Vec<OrderKey>,
    select: Option<Vec<S::Field>>,
    limit: Option<u64>,
    _phantom: PhantomData<S>,
}

impl<S: Schema> QueryBuilder<S> {
    /// Create a new empty query builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            filter: None,
            order: Vec::new(),
            select: None,
            limit: None,
            _phantom: PhantomData,
        }
    }

    /// Set the filter expression.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// builder.filter(ID.eq(user_id).and(NAME.contains("john")))
    /// ```
    #[must_use]
    pub fn filter(mut self, expr: Expr) -> Self {
        self.filter = Some(expr);
        self
    }

    /// Add an order-by clause.
    ///
    /// Can be called multiple times to add multiple sort keys.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// builder
    ///     .order_by(NAME, SortDir::Asc)
    ///     .order_by(ID, SortDir::Desc)
    /// ```
    #[must_use]
    pub fn order_by<F>(mut self, field: F, dir: SortDir) -> Self
    where
        F: AsFieldName,
    {
        self.order.push(OrderKey {
            field: field.as_field_name().to_owned(),
            dir,
        });
        self
    }

    /// Set the select fields (field projection).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// builder.select([NAME, EMAIL])
    /// builder.select(vec![NAME, EMAIL])
    ///
    /// // Backwards-compatible (still supported)
    /// builder.select(&[&ID, &NAME, &EMAIL])
    /// ```
    #[must_use]
    pub fn select<I>(mut self, fields: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsFieldKey<S>,
    {
        let iter = fields.into_iter();
        let (lower, _) = iter.size_hint();
        let mut out = Vec::with_capacity(lower);
        for f in iter {
            out.push(f.as_field_key());
        }
        self.select = Some(out);
        self
    }

    /// Set the page size limit.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// builder.page_size(50)
    /// ```
    #[must_use]
    pub fn page_size(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Build the final `ODataQuery` with computed filter hash.
    ///
    /// The filter hash is computed using the stable hashing algorithm from
    /// `pagination::short_filter_hash`.
    pub fn build(self) -> ODataQuery {
        let filter_hash = short_filter_hash(self.filter.as_ref());

        let mut query = ODataQuery::new();

        if let Some(expr) = self.filter {
            query = query.with_filter(expr);
        }

        if !self.order.is_empty() {
            query = query.with_order(ODataOrderBy(self.order));
        }

        if let Some(limit) = self.limit {
            query = query.with_limit(limit);
        }

        if let Some(hash) = filter_hash {
            query = query.with_filter_hash(hash);
        }

        if let Some(fields) = self.select {
            let names: Vec<String> = fields
                .into_iter()
                .map(|k| FieldRef::<S, ()>::new(k).name().to_owned())
                .collect();
            query = query.with_select(names);
        }

        query
    }
}

impl<S: Schema> Default for QueryBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::ast::{CompareOperator, Value};
    use crate::schema::FieldRef;

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    enum UserField {
        Id,
        Name,
        Email,
        Age,
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
            }
        }
    }

    const NAME: FieldRef<UserSchema, String> = FieldRef::new(UserField::Name);
    const EMAIL: FieldRef<UserSchema, String> = FieldRef::new(UserField::Email);
    const AGE: FieldRef<UserSchema, i32> = FieldRef::new(UserField::Age);
    const ID: FieldRef<UserSchema, uuid::Uuid> = FieldRef::new(UserField::Id);

    #[test]
    fn test_field_name_mapping() {
        assert_eq!(NAME.name(), "name");
        assert_eq!(EMAIL.name(), "email");
        assert_eq!(AGE.name(), "age");
    }

    #[test]
    fn test_simple_eq_filter() {
        let user_id = uuid::Uuid::new_v4();
        let query = QueryBuilder::<UserSchema>::new()
            .filter(ID.eq(user_id))
            .build();

        assert!(query.has_filter());
        assert!(query.filter_hash.is_some());
    }

    #[test]
    fn test_string_contains() {
        let query = QueryBuilder::<UserSchema>::new()
            .filter(NAME.contains("john"))
            .build();

        assert!(query.has_filter());
        if let Some(filter) = query.filter() {
            if let Expr::Function(name, args) = filter {
                assert_eq!(name, "contains");
                assert_eq!(args.len(), 2);
            } else {
                panic!("Expected Function expression");
            }
        }
    }

    #[test]
    fn test_string_startswith() {
        let query = QueryBuilder::<UserSchema>::new()
            .filter(NAME.startswith("jo"))
            .build();

        assert!(query.has_filter());
        if let Some(filter) = query.filter() {
            if let Expr::Function(name, _) = filter {
                assert_eq!(name, "startswith");
            } else {
                panic!("Expected Function expression");
            }
        }
    }

    #[test]
    fn test_string_endswith() {
        let query = QueryBuilder::<UserSchema>::new()
            .filter(EMAIL.endswith("@example.com"))
            .build();

        assert!(query.has_filter());
        if let Some(filter) = query.filter() {
            if let Expr::Function(name, _) = filter {
                assert_eq!(name, "endswith");
            } else {
                panic!("Expected Function expression");
            }
        }
    }

    #[test]
    fn test_comparison_operators() {
        let query = QueryBuilder::<UserSchema>::new().filter(AGE.gt(18)).build();
        assert!(query.has_filter());

        let query = QueryBuilder::<UserSchema>::new().filter(AGE.ge(18)).build();
        assert!(query.has_filter());

        let query = QueryBuilder::<UserSchema>::new().filter(AGE.lt(65)).build();
        assert!(query.has_filter());

        let query = QueryBuilder::<UserSchema>::new().filter(AGE.le(65)).build();
        assert!(query.has_filter());

        let query = QueryBuilder::<UserSchema>::new().filter(AGE.ne(0)).build();
        assert!(query.has_filter());
    }

    #[test]
    fn test_and_combinator() {
        let user_id = uuid::Uuid::new_v4();
        let query = QueryBuilder::<UserSchema>::new()
            .filter(ID.eq(user_id).and(AGE.gt(18)))
            .build();

        assert!(query.has_filter());
        if let Some(filter) = query.filter() {
            if let Expr::And(_, _) = filter {
            } else {
                panic!("Expected And expression");
            }
        }
    }

    #[test]
    fn test_or_combinator() {
        let query = QueryBuilder::<UserSchema>::new()
            .filter(AGE.lt(18).or(AGE.gt(65)))
            .build();

        assert!(query.has_filter());
        if let Some(filter) = query.filter() {
            if let Expr::Or(_, _) = filter {
            } else {
                panic!("Expected Or expression");
            }
        }
    }

    #[test]
    fn test_not_combinator() {
        let query = QueryBuilder::<UserSchema>::new()
            .filter(NAME.contains("test").not())
            .build();

        assert!(query.has_filter());
        if let Some(filter) = query.filter() {
            if let Expr::Not(_) = filter {
            } else {
                panic!("Expected Not expression");
            }
        }
    }

    #[test]
    fn test_complex_filter() {
        let user_id = uuid::Uuid::new_v4();
        let query = QueryBuilder::<UserSchema>::new()
            .filter(
                ID.eq(user_id)
                    .and(NAME.contains("john"))
                    .and(AGE.ge(18).and(AGE.le(65))),
            )
            .build();

        assert!(query.has_filter());
        assert!(query.filter_hash.is_some());
    }

    #[test]
    fn test_order_by_single() {
        let query = QueryBuilder::<UserSchema>::new()
            .order_by(NAME, SortDir::Asc)
            .build();

        assert_eq!(query.order.0.len(), 1);
        assert_eq!(query.order.0[0].field, "name");
        assert_eq!(query.order.0[0].dir, SortDir::Asc);
    }

    #[test]
    fn test_order_by_multiple() {
        let query = QueryBuilder::<UserSchema>::new()
            .order_by(NAME, SortDir::Asc)
            .order_by(AGE, SortDir::Desc)
            .build();

        assert_eq!(query.order.0.len(), 2);
        assert_eq!(query.order.0[0].field, "name");
        assert_eq!(query.order.0[0].dir, SortDir::Asc);
        assert_eq!(query.order.0[1].field, "age");
        assert_eq!(query.order.0[1].dir, SortDir::Desc);
    }

    #[test]
    fn test_select_fields() {
        let query = QueryBuilder::<UserSchema>::new()
            .select([NAME, EMAIL])
            .build();

        assert!(query.has_select());
        let fields = query.selected_fields().unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0], "name");
        assert_eq!(fields[1], "email");
    }

    #[test]
    fn test_select_fields_vec() {
        let query = QueryBuilder::<UserSchema>::new()
            .select(vec![NAME, EMAIL])
            .build();

        assert!(query.has_select());
        let fields = query.selected_fields().unwrap();
        assert_eq!(fields, &["name", "email"]);
    }

    #[test]
    fn test_select_fields_legacy_slice_syntax() {
        let query = QueryBuilder::<UserSchema>::new()
            .select(&[&NAME, &EMAIL])
            .build();

        assert!(query.has_select());
        let fields = query.selected_fields().unwrap();
        assert_eq!(fields, &["name", "email"]);
    }

    #[test]
    fn test_page_size() {
        let query = QueryBuilder::<UserSchema>::new().page_size(50).build();

        assert_eq!(query.limit, Some(50));
    }

    #[test]
    fn test_full_query_build() {
        let user_id = uuid::Uuid::new_v4();
        let query = QueryBuilder::<UserSchema>::new()
            .filter(ID.eq(user_id).and(AGE.gt(18)))
            .order_by(NAME, SortDir::Asc)
            .select([NAME, EMAIL])
            .page_size(25)
            .build();

        assert!(query.has_filter());
        assert!(query.filter_hash.is_some());
        assert_eq!(query.order.0.len(), 1);
        assert!(query.has_select());
        assert_eq!(query.limit, Some(25));
    }

    #[test]
    fn test_filter_hash_stability() {
        let user_id = uuid::Uuid::new_v4();

        let query1 = QueryBuilder::<UserSchema>::new()
            .filter(ID.eq(user_id))
            .build();

        let query2 = QueryBuilder::<UserSchema>::new()
            .filter(ID.eq(user_id))
            .build();

        assert_eq!(query1.filter_hash, query2.filter_hash);
        assert!(query1.filter_hash.is_some());
    }

    #[test]
    fn test_filter_hash_different_for_different_filters() {
        let query1 = QueryBuilder::<UserSchema>::new()
            .filter(NAME.eq("alice"))
            .build();

        let query2 = QueryBuilder::<UserSchema>::new().filter(AGE.gt(18)).build();

        assert_ne!(query1.filter_hash, query2.filter_hash);
    }

    #[test]
    fn test_no_filter_no_hash() {
        let query = QueryBuilder::<UserSchema>::new()
            .order_by(NAME, SortDir::Asc)
            .build();

        assert!(!query.has_filter());
        assert!(query.filter_hash.is_none());
    }

    #[test]
    fn test_empty_query() {
        let query = QueryBuilder::<UserSchema>::new().build();

        assert!(!query.has_filter());
        assert!(query.filter_hash.is_none());
        assert!(query.order.is_empty());
        assert!(!query.has_select());
        assert_eq!(query.limit, None);
    }

    #[test]
    fn test_normalized_filter_consistency() {
        use crate::pagination::normalize_filter_for_hash;

        let expr1 = NAME.eq("test");
        let expr2 = NAME.eq("test");

        let norm1 = normalize_filter_for_hash(&expr1);
        let norm2 = normalize_filter_for_hash(&expr2);

        assert_eq!(norm1, norm2);
    }

    #[test]
    fn test_is_null() {
        let query = QueryBuilder::<UserSchema>::new()
            .filter(NAME.is_null())
            .build();

        assert!(query.has_filter());
        if let Some(filter) = query.filter() {
            if let Expr::Compare(_, op, value) = filter {
                assert_eq!(*op, CompareOperator::Eq);
                if let Expr::Value(Value::Null) = **value {
                } else {
                    panic!("Expected Value::Null");
                }
            } else {
                panic!("Expected Compare expression");
            }
        }
    }

    #[test]
    fn test_is_not_null() {
        let query = QueryBuilder::<UserSchema>::new()
            .filter(EMAIL.is_not_null())
            .build();

        assert!(query.has_filter());
        if let Some(filter) = query.filter() {
            if let Expr::Compare(_, op, value) = filter {
                assert_eq!(*op, CompareOperator::Ne);
                if let Expr::Value(Value::Null) = **value {
                } else {
                    panic!("Expected Value::Null");
                }
            } else {
                panic!("Expected Compare expression");
            }
        }
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_chrono_datetime_conversion() {
        use chrono::Utc;

        const CREATED_AT: FieldRef<UserSchema, chrono::DateTime<Utc>> =
            FieldRef::new(UserField::Age);

        let now = Utc::now();
        let query = QueryBuilder::<UserSchema>::new()
            .filter(CREATED_AT.eq(now))
            .build();

        assert!(query.has_filter());
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_chrono_naive_date_conversion() {
        use chrono::NaiveDate;

        const DATE_FIELD: FieldRef<UserSchema, NaiveDate> = FieldRef::new(UserField::Age);

        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let query = QueryBuilder::<UserSchema>::new()
            .filter(DATE_FIELD.eq(date))
            .build();

        assert!(query.has_filter());
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_chrono_naive_time_conversion() {
        use chrono::NaiveTime;

        const TIME_FIELD: FieldRef<UserSchema, NaiveTime> = FieldRef::new(UserField::Age);

        let time = NaiveTime::from_hms_opt(12, 30, 0).unwrap();
        let query = QueryBuilder::<UserSchema>::new()
            .filter(TIME_FIELD.eq(time))
            .build();

        assert!(query.has_filter());
    }
}
