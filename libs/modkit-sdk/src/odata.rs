//! Typed `OData` query builder
//!
//! This module provides a generic, reusable typed query builder for `OData` that produces
//! `modkit_odata::ODataQuery` with correct filter hashing.
//!
//! # Design
//!
//! - **Schema trait**: Defines field enums and their string mappings
//! - **`FieldRef`**: Type-safe field references with schema and Rust type markers
//! - **Filter constructors**: Typed comparison and string operations returning AST expressions
//! - **`QueryBuilder`**: Fluent API for building queries with filter/order/select/limit
//!
//! # Example
//!
//! ```rust,ignore
//! use modkit_sdk::odata::{Schema, FieldRef, QueryBuilder};
//! use modkit_odata::SortDir;
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

use modkit_odata::{
    ast::{CompareOperator, Expr, Value},
    pagination::short_filter_hash,
    ODataOrderBy, ODataQuery, OrderKey, SortDir,
};
use std::marker::PhantomData;

/// Schema trait defining field enums and their string mappings.
///
/// Implement this trait for your entity schemas to enable type-safe query building.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Copy, Clone, Eq, PartialEq)]
/// enum UserField {
///     Id,
///     Name,
/// }
///
/// struct UserSchema;
///
/// impl Schema for UserSchema {
///     type Field = UserField;
///
///     fn field_name(field: Self::Field) -> &'static str {
///         match field {
///             UserField::Id => "id",
///             UserField::Name => "name",
///         }
///     }
/// }
/// ```
pub trait Schema {
    /// The field enum type (must be Copy + Eq)
    type Field: Copy + Eq;

    /// Map a field enum to its string name
    fn field_name(field: Self::Field) -> &'static str;
}

/// Type-safe field reference holding schema and Rust type information.
///
/// This struct binds a field to both its schema and expected Rust type,
/// enabling compile-time type checking for filter operations.
/// NOTE:
///  `FieldRef` equality and hashing are based solely on the underlying schema field.
///  The generic type parameter `T` is a phantom type used only for compile-time
///  validation of operations and is not part of the field identity.
/// # Type Parameters
///
/// * `S` - The schema type implementing `Schema`
/// * `T` - The Rust type this field represents (e.g., `String`, `uuid::Uuid`)
pub struct FieldRef<S: Schema, T> {
    field: S::Field,
    _phantom: PhantomData<(S, T)>,
}

impl<S: Schema, T> FieldRef<S, T> {
    /// Create a new typed field reference.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// const NAME: FieldRef<UserSchema, String> = FieldRef::new(UserField::Name);
    /// ```
    #[must_use]
    pub const fn new(field: S::Field) -> Self {
        Self {
            field,
            _phantom: PhantomData,
        }
    }

    /// Get the field name as a string.
    #[must_use]
    pub fn name(&self) -> &'static str {
        S::field_name(self.field)
    }

    /// Create an identifier expression for this field.
    #[must_use]
    fn identifier(&self) -> Expr {
        Expr::Identifier(self.name().to_owned())
    }
}

impl<S: Schema, T> Clone for FieldRef<S, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: Schema, T> Copy for FieldRef<S, T> {}

impl<S: Schema, T> std::fmt::Debug for FieldRef<S, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldRef")
            .field("field", &self.name())
            .finish()
    }
}

impl<S: Schema, T> PartialEq for FieldRef<S, T> {
    fn eq(&self, other: &Self) -> bool {
        self.field == other.field
    }
}

impl<S: Schema, T> Eq for FieldRef<S, T> {}

impl<S: Schema, T> std::hash::Hash for FieldRef<S, T>
where
    S::Field: std::hash::Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.field.hash(state);
    }
}

/// Trait for extracting field names from field references.
///
/// This allows the `select` method to accept heterogeneous field arrays
/// with different type parameters.
#[doc(hidden)]
pub trait AsFieldName {
    /// Get the field name as a string.
    fn as_field_name(&self) -> &'static str;
}

/// Trait for extracting schema field keys from field references.
///
/// `QueryBuilder::select()` stores schema keys (`S::Field`) instead of field names so we can
/// avoid allocating `String`s during the builder phase and only allocate during `build()`.
#[doc(hidden)]
pub trait AsFieldKey<S: Schema> {
    /// Get the schema field key.
    fn as_field_key(&self) -> S::Field;
}

impl<S: Schema, T> AsFieldName for FieldRef<S, T> {
    fn as_field_name(&self) -> &'static str {
        self.name()
    }
}

impl<S: Schema, T> AsFieldKey<S> for FieldRef<S, T> {
    fn as_field_key(&self) -> S::Field {
        self.field
    }
}

impl<T: AsFieldName + ?Sized> AsFieldName for &T {
    fn as_field_name(&self) -> &'static str {
        (*self).as_field_name()
    }
}

impl<S: Schema, T: AsFieldKey<S> + ?Sized> AsFieldKey<S> for &T {
    fn as_field_key(&self) -> S::Field {
        (*self).as_field_key()
    }
}

/// Trait for types that can be converted to `OData` AST values.
pub trait IntoODataValue {
    /// Convert this value into an `OData` AST value.
    fn into_odata_value(self) -> Value;
}

impl IntoODataValue for bool {
    fn into_odata_value(self) -> Value {
        Value::Bool(self)
    }
}

#[cfg(feature = "uuid")]
impl IntoODataValue for uuid::Uuid {
    fn into_odata_value(self) -> Value {
        Value::Uuid(self)
    }
}

impl IntoODataValue for String {
    fn into_odata_value(self) -> Value {
        Value::String(self)
    }
}

impl IntoODataValue for &str {
    fn into_odata_value(self) -> Value {
        Value::String(self.to_owned())
    }
}

impl IntoODataValue for i32 {
    fn into_odata_value(self) -> Value {
        Value::Number(self.into())
    }
}

impl IntoODataValue for i64 {
    fn into_odata_value(self) -> Value {
        Value::Number(self.into())
    }
}

impl IntoODataValue for u32 {
    fn into_odata_value(self) -> Value {
        Value::Number(self.into())
    }
}

impl IntoODataValue for u64 {
    fn into_odata_value(self) -> Value {
        Value::Number(self.into())
    }
}

#[cfg(feature = "chrono")]
impl IntoODataValue for chrono::DateTime<chrono::Utc> {
    fn into_odata_value(self) -> Value {
        Value::DateTime(self)
    }
}

#[cfg(feature = "chrono")]
impl IntoODataValue for chrono::NaiveDate {
    fn into_odata_value(self) -> Value {
        Value::Date(self)
    }
}

#[cfg(feature = "chrono")]
impl IntoODataValue for chrono::NaiveTime {
    fn into_odata_value(self) -> Value {
        Value::Time(self)
    }
}

/// Comparison operations for any field type.
impl<S: Schema, T> FieldRef<S, T> {
    /// Create an equality comparison: `field eq value`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let filter = ID.eq(user_id);
    /// ```
    #[must_use]
    pub fn eq<V: IntoODataValue>(self, value: V) -> Expr {
        Expr::Compare(
            Box::new(self.identifier()),
            CompareOperator::Eq,
            Box::new(Expr::Value(value.into_odata_value())),
        )
    }

    /// Create a not-equal comparison: `field ne value`
    #[must_use]
    pub fn ne<V: IntoODataValue>(self, value: V) -> Expr {
        Expr::Compare(
            Box::new(self.identifier()),
            CompareOperator::Ne,
            Box::new(Expr::Value(value.into_odata_value())),
        )
    }

    /// Create a greater-than comparison: `field gt value`
    #[must_use]
    pub fn gt<V: IntoODataValue>(self, value: V) -> Expr {
        Expr::Compare(
            Box::new(self.identifier()),
            CompareOperator::Gt,
            Box::new(Expr::Value(value.into_odata_value())),
        )
    }

    /// Create a greater-than-or-equal comparison: `field ge value`
    #[must_use]
    pub fn ge<V: IntoODataValue>(self, value: V) -> Expr {
        Expr::Compare(
            Box::new(self.identifier()),
            CompareOperator::Ge,
            Box::new(Expr::Value(value.into_odata_value())),
        )
    }

    /// Create a less-than comparison: `field lt value`
    #[must_use]
    pub fn lt<V: IntoODataValue>(self, value: V) -> Expr {
        Expr::Compare(
            Box::new(self.identifier()),
            CompareOperator::Lt,
            Box::new(Expr::Value(value.into_odata_value())),
        )
    }

    /// Create a less-than-or-equal comparison: `field le value`
    #[must_use]
    pub fn le<V: IntoODataValue>(self, value: V) -> Expr {
        Expr::Compare(
            Box::new(self.identifier()),
            CompareOperator::Le,
            Box::new(Expr::Value(value.into_odata_value())),
        )
    }

    /// Create a null check: `field eq null`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let filter = OPTIONAL_FIELD.is_null();
    /// ```
    #[must_use]
    pub fn is_null(self) -> Expr {
        Expr::Compare(
            Box::new(self.identifier()),
            CompareOperator::Eq,
            Box::new(Expr::Value(Value::Null)),
        )
    }

    /// Create a not-null check: `field ne null`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let filter = OPTIONAL_FIELD.is_not_null();
    /// ```
    #[must_use]
    pub fn is_not_null(self) -> Expr {
        Expr::Compare(
            Box::new(self.identifier()),
            CompareOperator::Ne,
            Box::new(Expr::Value(Value::Null)),
        )
    }
}

/// String-specific operations (only available for String fields).
impl<S: Schema> FieldRef<S, String> {
    /// Create a contains function call: `contains(field, 'value')`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let filter = NAME.contains("john");
    /// ```
    #[must_use]
    pub fn contains(self, value: impl Into<String>) -> Expr {
        Expr::Function(
            "contains".to_owned(),
            vec![self.identifier(), Expr::Value(Value::String(value.into()))],
        )
    }

    /// Create a startswith function call: `startswith(field, 'value')`
    #[must_use]
    pub fn startswith(self, value: impl Into<String>) -> Expr {
        Expr::Function(
            "startswith".to_owned(),
            vec![self.identifier(), Expr::Value(Value::String(value.into()))],
        )
    }

    /// Create an endswith function call: `endswith(field, 'value')`
    #[must_use]
    pub fn endswith(self, value: impl Into<String>) -> Expr {
        Expr::Function(
            "endswith".to_owned(),
            vec![self.identifier(), Expr::Value(Value::String(value.into()))],
        )
    }
}

/// Extension trait for combining filter expressions.
pub trait FilterExpr: Sized {
    /// Combine two expressions with AND: `expr1 and expr2`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let filter = ID.eq(user_id).and(NAME.contains("john"));
    /// ```
    #[must_use]
    fn and(self, other: Expr) -> Expr;

    /// Combine two expressions with OR: `expr1 or expr2`
    #[must_use]
    fn or(self, other: Expr) -> Expr;

    /// Negate an expression: `not expr`
    #[must_use]
    fn not(self) -> Expr;
}

impl FilterExpr for Expr {
    fn and(self, other: Expr) -> Expr {
        Expr::And(Box::new(self), Box::new(other))
    }

    fn or(self, other: Expr) -> Expr {
        Expr::Or(Box::new(self), Box::new(other))
    }

    fn not(self) -> Expr {
        Expr::Not(Box::new(self))
    }
}

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
    pub fn order_by<T>(mut self, field: FieldRef<S, T>, dir: SortDir) -> Self {
        self.order.push(OrderKey {
            field: field.name().to_owned(),
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
            // Store field keys (identity) rather than names to avoid allocations during
            // query builder chaining. Names are resolved once in `build()`.
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
    /// `modkit_odata::pagination::short_filter_hash`.
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
            // Allocate `String`s only once per field at final query construction time.
            let names: Vec<String> = fields
                .into_iter()
                // Resolve field names through FieldRef so the underlying schema mapping logic
                // stays centralized and doesn't require Schema::field_name to be a static fn.
                .map(|k| FieldRef::<S, ()>::new(k).name().to_owned())
                .collect();
            query = query.with_select(names);
        }

        query
    }

    /// Create a stream of pages using the given fetcher function.
    ///
    /// This method consumes the builder and returns a `PagesPager` that implements
    /// `Stream<Item = Result<Page<T>, E>>`, automatically managing cursor pagination.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use futures_util::StreamExt;
    ///
    /// let pages = QueryBuilder::<UserSchema>::new()
    ///     .filter(NAME.contains("john"))
    ///     .page_size(50)
    ///     .pages_stream(|query| async move {
    ///         client.list_users(query).await
    ///     });
    ///
    /// while let Some(result) = pages.next().await {
    ///     match result {
    ///         Ok(page) => println!("Got {} items", page.items.len()),
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// }
    /// ```
    pub fn pages_stream<T, E, F, Fut>(self, fetcher: F) -> crate::pager::PagesPager<T, E, F, Fut>
    where
        F: FnMut(ODataQuery) -> Fut,
        Fut: std::future::Future<Output = Result<modkit_odata::Page<T>, E>>,
    {
        let query = self.build();
        crate::pager::PagesPager::new(query, fetcher)
    }

    /// Create a stream of individual items using the given fetcher function.
    ///
    /// This method consumes the builder and returns a `CursorPager` that implements
    /// `Stream<Item = Result<T, E>>`, automatically managing cursor pagination and
    /// buffering items from each page.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use futures_util::StreamExt;
    ///
    /// let items = QueryBuilder::<UserSchema>::new()
    ///     .filter(NAME.contains("john"))
    ///     .page_size(50)
    ///     .items_stream(|query| async move {
    ///         client.list_users(query).await
    ///     });
    ///
    /// while let Some(result) = items.next().await {
    ///     match result {
    ///         Ok(user) => println!("User: {:?}", user),
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// }
    /// ```
    pub fn items_stream<T, E, F, Fut>(self, fetcher: F) -> crate::pager::CursorPager<T, E, F, Fut>
    where
        F: FnMut(ODataQuery) -> Fut,
        Fut: std::future::Future<Output = Result<modkit_odata::Page<T>, E>>,
    {
        let query = self.build();
        crate::pager::CursorPager::new(query, fetcher)
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

    #[cfg(feature = "uuid")]
    const ID: FieldRef<UserSchema, uuid::Uuid> = FieldRef::new(UserField::Id);

    #[test]
    fn test_field_name_mapping() {
        assert_eq!(NAME.name(), "name");
        assert_eq!(EMAIL.name(), "email");
        assert_eq!(AGE.name(), "age");
    }

    #[cfg(feature = "uuid")]
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

    #[cfg(feature = "uuid")]
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

    #[cfg(feature = "uuid")]
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

    #[cfg(feature = "uuid")]
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

    #[cfg(feature = "uuid")]
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
        use modkit_odata::pagination::normalize_filter_for_hash;

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
                    // Expected
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
                    // Expected
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
            FieldRef::new(UserField::Age); // Reusing Age field for test

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

        const DATE_FIELD: FieldRef<UserSchema, NaiveDate> = FieldRef::new(UserField::Age); // Reusing Age field for test

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

        const TIME_FIELD: FieldRef<UserSchema, NaiveTime> = FieldRef::new(UserField::Age); // Reusing Age field for test

        let time = NaiveTime::from_hms_opt(12, 30, 0).unwrap();
        let query = QueryBuilder::<UserSchema>::new()
            .filter(TIME_FIELD.eq(time))
            .build();

        assert!(query.has_filter());
    }
}
