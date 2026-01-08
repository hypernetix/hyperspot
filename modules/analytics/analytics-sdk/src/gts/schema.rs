// @fdd-change:change-rust-gts-types
//! GTS Schema Type Definitions
//!
//! Provides Rust type markers for GTS schema types. The actual JSON Schema
//! files are maintained manually at `modules/analytics/gts/types/schema/v1/`.
//!
//! These types serve as compile-time type markers and do not generate schemas
//! since the schema definitions already exist and are carefully crafted.

/// Base schema type for defining data structures.
///
/// All schemas inherit from this base type. Schemas define the structure,
/// validation rules, and mock data generation for their respective data types.
///
/// # GTS ID
/// `gts://gts.hypernetix.hyperspot.ax.schema.v1~`
///
/// # Schema Location
/// `modules/analytics/gts/types/schema/v1/base.schema.json`
///
/// # Note
/// This is a marker type. The actual JSON Schema is maintained separately.
// @fdd-change:change-rust-gts-types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SchemaV1;

/// Query returns schema with scalar-only field enforcement.
///
/// Defines the OData v4 response format for query results. Follows standard
/// OData conventions with @odata.* metadata fields and 'value' array for data items.
/// All items must have flat structure (scalar values only) for consistent widget rendering.
///
/// # Constraints
/// - Field values must be scalar types only: string, number, integer, boolean, null
/// - Nested objects and arrays are not allowed
/// - Ensures consistent rendering across all widget types
///
/// # GTS ID
/// `gts://gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~`
///
/// # Schema Location
/// `modules/analytics/gts/types/schema/v1/query_returns.schema.json`
///
/// # Note
/// This is a marker type. The actual JSON Schema is maintained separately.
// @fdd-change:change-rust-gts-types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QueryReturnsSchemaV1;

impl SchemaV1 {
    /// Creates a new base schema instance.
    // @fdd-change:change-rust-gts-types
    pub fn new() -> Self {
        Self
    }
}

impl Default for SchemaV1 {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryReturnsSchemaV1 {
    /// Creates a new query returns schema instance.
    // @fdd-change:change-rust-gts-types
    pub fn new() -> Self {
        Self
    }

    /// Validates that all field types are scalar (no nested objects).
    ///
    /// This is a placeholder for the validation logic that will be implemented
    /// in the service layer. The actual validation will check field definitions
    /// to ensure only scalar types are used.
    ///
    /// # Scalar Types
    /// - string
    /// - number
    /// - integer  
    /// - boolean
    /// - null
    // @fdd-change:change-rust-gts-types
    pub fn validate_scalar_fields() -> bool {
        // Placeholder - actual implementation in schema service layer
        true
    }
}

impl Default for QueryReturnsSchemaV1 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_v1_creation() {
        let schema = SchemaV1::new();
        assert_eq!(schema, SchemaV1);
    }

    #[test]
    fn test_query_returns_schema_v1_creation() {
        let schema = QueryReturnsSchemaV1::new();
        assert_eq!(schema, QueryReturnsSchemaV1);
    }

    #[test]
    fn test_scalar_field_validation_placeholder() {
        assert!(QueryReturnsSchemaV1::validate_scalar_fields());
    }
}
