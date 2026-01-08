use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response DTO for GTS entity operations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GtsEntityDto {
    /// GTS entity ID
    #[schema(example = "gts.hypernetix.hyperspot.ax.query.v1~instance.v1")]
    pub id: String,
    
    /// GTS type identifier
    #[schema(example = "gts.hypernetix.hyperspot.ax.query.v1~")]
    pub type_id: String,
    
    /// Entity data
    pub entity: serde_json::Value,
    
    /// Tenant ID
    pub tenant: String,
    
    /// Registration timestamp
    #[schema(example = "2026-01-08T10:00:00Z")]
    pub registered_at: String,
}

/// Request DTO for creating/updating GTS entities
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GtsEntityRequestDto {
    /// GTS entity ID (for updates)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    
    /// Entity data
    pub entity: serde_json::Value,
}

/// Response DTO for GTS entity list
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GtsEntityListDto {
    /// OData context
    #[serde(rename = "@odata.context")]
    pub odata_context: String,
    
    /// Total count (if $count=true)
    #[serde(rename = "@odata.count", skip_serializing_if = "Option::is_none")]
    pub odata_count: Option<i64>,
    
    /// Next link for pagination
    #[serde(rename = "@odata.nextLink", skip_serializing_if = "Option::is_none")]
    pub odata_next_link: Option<String>,
    
    /// List of entities
    pub value: Vec<GtsEntityDto>,
}
