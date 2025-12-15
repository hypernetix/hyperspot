// Test DE0204: DTOs Must Have ToSchema Derive
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Should trigger DE0204 - missing ToSchema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingSchemaDto {
    pub id: String,
    pub name: String,
}

// Should NOT trigger - has ToSchema
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CorrectDto {
    pub id: String,
    pub name: String,
}

// Should trigger DE0204 - missing ToSchema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnotherBadDto {
    pub id: String,
    pub value: i32,
}

// Should trigger DE0204 - has serde but missing ToSchema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YetAnotherBadDto {
    pub code: String,
}

// Should NOT trigger - not a DTO (no Dto suffix)
#[derive(Debug, Clone)]
pub struct RegularStruct {
    pub id: String,
}

// Should NOT trigger - has all required derives
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompleteDto {
    pub id: String,
    pub name: String,
    pub value: i32,
}

// Should trigger DE0204 - enum missing ToSchema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusDto {
    Active,
    Inactive,
}
