// @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types
//! GTS Schema Type Definitions
//!
//! Rust structs matching JSON Schema definitions at `modules/analytics/gts/types/schema/v1/`.
//! These types are used for runtime data handling and match the GTS schema structure.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Base schema type for defining data structures.
///
/// This is an open type (additionalProperties: true) that accepts any fields.
/// All specialized schema types inherit from this base.
///
/// # GTS ID
/// `gts://gts.hypernetix.hyperspot.ax.schema.v1~`
///
/// # Schema Location
/// `modules/analytics/gts/types/schema/v1/base.schema.json`
// @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaV1 {
    /// Dynamic fields - base schema accepts any additional properties
    #[serde(flatten)]
    pub fields: std::collections::HashMap<String, JsonValue>,
}

/// Query returns schema with OData v4 response format.
///
/// Defines the structure for query results following OData conventions.
/// All result items must have flat structure (scalar values only).
///
/// # GTS ID
/// `gts://gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~`
///
/// # Schema Location
/// `modules/analytics/gts/types/schema/v1/query_returns.schema.json`
// @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryReturnsSchemaV1 {
    /// OData context URL describing the payload structure
    /// Example: "$metadata#EntitySet"
    #[serde(rename = "@odata.context", skip_serializing_if = "Option::is_none")]
    pub odata_context: Option<String>,

    /// Total count of items matching the query
    /// Only present when $count=true is requested
    #[serde(rename = "@odata.count", skip_serializing_if = "Option::is_none")]
    pub odata_count: Option<u64>,

    /// URL to fetch the next page of results
    /// Omitted if this is the last page
    #[serde(rename = "@odata.nextLink", skip_serializing_if = "Option::is_none")]
    pub odata_next_link: Option<String>,

    /// Array of result items
    /// Each item must be a flat object with scalar values only
    pub value: Vec<std::collections::HashMap<String, JsonValue>>,
}

// @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types
impl SchemaV1 {
    /// Creates a new empty base schema instance
    pub fn new() -> Self {
        Self {
            fields: std::collections::HashMap::new(),
        }
    }

    /// Creates a schema from a JSON value
    pub fn from_json(value: JsonValue) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }

    /// Converts the schema to a JSON value
    pub fn to_json(&self) -> Result<JsonValue, serde_json::Error> {
        serde_json::to_value(self)
    }
}

// @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types
impl Default for SchemaV1 {
    fn default() -> Self {
        Self::new()
    }
}

// @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types
impl QueryReturnsSchemaV1 {
    /// Creates a new query returns schema with empty results
    pub fn new() -> Self {
        Self {
            odata_context: None,
            odata_count: None,
            odata_next_link: None,
            value: Vec::new(),
        }
    }

    /// Creates a query returns schema with results
    pub fn with_results(value: Vec<std::collections::HashMap<String, JsonValue>>) -> Self {
        Self {
            odata_context: None,
            odata_count: None,
            odata_next_link: None,
            value,
        }
    }

    /// Validates that all field values in result items are scalar types
    ///
    /// Checks each item in the value array to ensure all fields contain
    /// only scalar values (string, number, boolean, null) and no nested
    /// objects or arrays.
    ///
    /// # Returns
    /// - `Ok(())` if all fields are scalar
    /// - `Err(String)` with description of first non-scalar field found
    // @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types
    pub fn validate_scalar_fields(&self) -> Result<(), String> {
        for (idx, item) in self.value.iter().enumerate() {
            for (key, value) in item.iter() {
                if !Self::is_scalar_value(value) {
                    return Err(format!(
                        "Non-scalar value found in item {} at field '{}': nested objects/arrays not allowed",
                        idx, key
                    ));
                }
            }
        }
        Ok(())
    }

    /// Checks if a JSON value is scalar (not object or array)
    fn is_scalar_value(value: &JsonValue) -> bool {
        !value.is_object() && !value.is_array()
    }

    /// Creates from a JSON value
    pub fn from_json(value: JsonValue) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }

    /// Converts to a JSON value
    pub fn to_json(&self) -> Result<JsonValue, serde_json::Error> {
        serde_json::to_value(self)
    }
}

// @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types
impl Default for QueryReturnsSchemaV1 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types
    #[test]
    fn test_schema_v1_creation() {
        let schema = SchemaV1::new();
        assert_eq!(schema.fields.len(), 0);
    }

    // @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types
    #[test]
    fn test_schema_v1_serialization() {
        let mut schema = SchemaV1::new();
        schema.fields.insert("test_field".to_string(), json!("test_value"));
        
        let json = schema.to_json().unwrap();
        assert_eq!(json["test_field"], "test_value");
        
        let deserialized = SchemaV1::from_json(json).unwrap();
        assert_eq!(deserialized.fields.get("test_field"), Some(&json!("test_value")));
    }

    // @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types
    #[test]
    fn test_query_returns_schema_v1_creation() {
        let schema = QueryReturnsSchemaV1::new();
        assert_eq!(schema.value.len(), 0);
        assert!(schema.odata_context.is_none());
        assert!(schema.odata_count.is_none());
        assert!(schema.odata_next_link.is_none());
    }

    // @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types
    #[test]
    fn test_query_returns_with_results() {
        let mut item = std::collections::HashMap::new();
        item.insert("id".to_string(), json!("ord-001"));
        item.insert("customer".to_string(), json!("Acme Corp"));
        item.insert("revenue".to_string(), json!(15000));
        
        let schema = QueryReturnsSchemaV1::with_results(vec![item]);
        assert_eq!(schema.value.len(), 1);
        assert_eq!(schema.value[0].get("id"), Some(&json!("ord-001")));
    }

    // @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types
    #[test]
    fn test_scalar_field_validation_valid() {
        let mut item = std::collections::HashMap::new();
        item.insert("string_field".to_string(), json!("test"));
        item.insert("number_field".to_string(), json!(42));
        item.insert("bool_field".to_string(), json!(true));
        item.insert("null_field".to_string(), json!(null));
        
        let schema = QueryReturnsSchemaV1::with_results(vec![item]);
        assert!(schema.validate_scalar_fields().is_ok());
    }

    // @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types
    #[test]
    fn test_scalar_field_validation_invalid_nested_object() {
        let mut item = std::collections::HashMap::new();
        item.insert("valid_field".to_string(), json!("test"));
        item.insert("nested_object".to_string(), json!({"inner": "value"}));
        
        let schema = QueryReturnsSchemaV1::with_results(vec![item]);
        let result = schema.validate_scalar_fields();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nested objects/arrays not allowed"));
    }

    // @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types
    #[test]
    fn test_scalar_field_validation_invalid_array() {
        let mut item = std::collections::HashMap::new();
        item.insert("valid_field".to_string(), json!("test"));
        item.insert("array_field".to_string(), json!([1, 2, 3]));
        
        let schema = QueryReturnsSchemaV1::with_results(vec![item]);
        let result = schema.validate_scalar_fields();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nested objects/arrays not allowed"));
    }

    // @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types
    #[test]
    fn test_query_returns_serialization() {
        let mut item = std::collections::HashMap::new();
        item.insert("id".to_string(), json!("test-id"));
        item.insert("name".to_string(), json!("Test Name"));
        
        let mut schema = QueryReturnsSchemaV1::with_results(vec![item]);
        schema.odata_context = Some("$metadata#TestEntity".to_string());
        schema.odata_count = Some(100);
        
        let json = schema.to_json().unwrap();
        assert_eq!(json["@odata.context"], "$metadata#TestEntity");
        assert_eq!(json["@odata.count"], 100);
        assert!(json["value"].is_array());
        
        let deserialized = QueryReturnsSchemaV1::from_json(json).unwrap();
        assert_eq!(deserialized.odata_context, Some("$metadata#TestEntity".to_string()));
        assert_eq!(deserialized.odata_count, Some(100));
        assert_eq!(deserialized.value.len(), 1);
    }
}
