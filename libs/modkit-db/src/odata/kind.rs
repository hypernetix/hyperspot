//! Shared `FieldKind` enum for `OData` type system.
//!
//! This enum represents the logical types of fields that can be used in
//! `OData` filters, orderby, and pagination. It is used by both the legacy
//! FieldMap-based code and the new type-safe FilterField-based code.

use std::fmt;

/// Logical field types supported in `OData` operations.
///
/// This enum describes the data type of a field for the purpose of:
/// - Value coercion in filters (converting `OData` values to `SeaORM` values)
/// - Type validation in the `FilterField` trait
/// - Cursor value encoding/decoding
///
/// # Example
///
/// ```ignore
/// use modkit_db::odata::FieldKind;
///
/// let kind = FieldKind::String;
/// assert_eq!(kind.to_string(), "String");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldKind {
    String,
    I64,
    F64,
    Bool,
    Uuid,
    DateTimeUtc,
    Date,
    Time,
    Decimal,
}

impl fmt::Display for FieldKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldKind::String => write!(f, "String"),
            FieldKind::I64 => write!(f, "I64"),
            FieldKind::F64 => write!(f, "F64"),
            FieldKind::Bool => write!(f, "Bool"),
            FieldKind::Uuid => write!(f, "Uuid"),
            FieldKind::DateTimeUtc => write!(f, "DateTimeUtc"),
            FieldKind::Date => write!(f, "Date"),
            FieldKind::Time => write!(f, "Time"),
            FieldKind::Decimal => write!(f, "Decimal"),
        }
    }
}
