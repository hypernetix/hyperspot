// @fdd-change:change-rust-gts-types
//! TEST FILE: Attempting to generate schemas using struct_to_gts_schema macro
//!
//! This file tests whether the macro can generate schemas identical to
//! the existing hand-written ones.

use gts_macros::struct_to_gts_schema;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Test: Base schema with macro generation
/// Target: gts/types/schema/v1/base.schema.json
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[struct_to_gts_schema(
    dir_path = "../../../gts/types/schema/v1/test",
    base = true,
    schema_id = "gts://gts.hypernetix.hyperspot.ax.schema.v1~",
    description = "Base schema type for defining data structures. All specialized schema types (query_params, query_returns, template_config, values) must inherit from this. Schemas define the structure, validation rules, and mock data generation for their respective data types. The x-gts-mock field provides example data for testing and documentation.",
    properties = ""
)]
pub struct TestSchemaV1;

/// Test: Query returns with detailed structure
/// Target: gts/types/schema/v1/query_returns.schema.json
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[struct_to_gts_schema(
    dir_path = "../../../gts/types/schema/v1/test",
    base = TestSchemaV1,
    schema_id = "gts://gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~",
    description = "OData v4 response format for query results. Follows standard OData conventions with @odata.* metadata fields and 'value' array for data items. All items must have flat structure (scalar values only) for consistent widget rendering.",
    properties = "@odata.context,@odata.count,@odata.nextLink,value"
)]
pub struct TestQueryReturnsSchemaV1 {
    /// OData context URL describing the payload structure
    #[serde(rename = "@odata.context")]
    #[schemars(description = "OData context URL describing the payload structure. Example: '$metadata#EntitySet'")]
    pub odata_context: Option<String>,
    
    /// Total count of items matching the query
    #[serde(rename = "@odata.count")]
    #[schemars(description = "Total count of items matching the query. Only present when $count=true is requested. May be expensive for large datasets.")]
    pub odata_count: Option<u64>,
    
    /// URL to fetch the next page of results
    #[serde(rename = "@odata.nextLink")]
    #[schemars(description = "URL to fetch the next page of results. Omitted if this is the last page. Contains full URL with query parameters including skip token.")]
    pub odata_next_link: Option<String>,
    
    /// Array of result items. Each item must be flat object with scalar values only.
    #[schemars(description = "Array of result items. Each item must be a flat object with scalar values only (string, number, boolean, null) - no nested objects or arrays. This ensures consistent rendering across all widget types.")]
    pub value: Vec<serde_json::Value>,
}
