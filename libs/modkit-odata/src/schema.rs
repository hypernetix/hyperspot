//! `OData` schema types for type-safe query building.
//!
//! This module defines the core schema abstraction for `OData` queries:
//! - `Schema` trait: Maps field enums to string names
//! - `FieldRef`: Type-safe field references with compile-time type checking
//! - Helper traits for field operations and value conversion
//!
//! These types are protocol-level abstractions independent of SDK implementation.

use crate::ast::{CompareOperator, Expr, Value};
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
///
/// **NOTE:** `FieldRef` equality and hashing are based solely on the underlying
/// schema field. The generic type parameter `T` is a phantom type used only for
/// compile-time validation of operations and is not part of the field identity.
///
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
/// `QueryBuilder::select()` stores schema keys (`S::Field`) instead of field names
/// so we can avoid allocating `String`s during the builder phase and only allocate
/// during `build()`.
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

impl IntoODataValue for chrono::DateTime<chrono::Utc> {
    fn into_odata_value(self) -> Value {
        Value::DateTime(self)
    }
}

impl IntoODataValue for chrono::NaiveDate {
    fn into_odata_value(self) -> Value {
        Value::Date(self)
    }
}

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
    pub fn contains(self, substring: &str) -> Expr {
        Expr::Function(
            "contains".to_owned(),
            vec![
                self.identifier(),
                Expr::Value(Value::String(substring.to_owned())),
            ],
        )
    }

    /// Create a startswith function call: `startswith(field, 'prefix')`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let filter = NAME.startswith("Dr");
    /// ```
    #[must_use]
    pub fn startswith(self, prefix: &str) -> Expr {
        Expr::Function(
            "startswith".to_owned(),
            vec![
                self.identifier(),
                Expr::Value(Value::String(prefix.to_owned())),
            ],
        )
    }

    /// Create an endswith function call: `endswith(field, 'suffix')`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let filter = EMAIL.endswith("@example.com");
    /// ```
    #[must_use]
    pub fn endswith(self, suffix: &str) -> Expr {
        Expr::Function(
            "endswith".to_owned(),
            vec![
                self.identifier(),
                Expr::Value(Value::String(suffix.to_owned())),
            ],
        )
    }
}
