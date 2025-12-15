// Test DE0203: DTOs Must Have Serde Derives
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Should trigger DE0203 - missing Serialize
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct MissingSerializeDto {
    pub id: String,
    pub name: String,
}

// Should trigger DE0203 - missing Deserialize
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MissingDeserializeDto {
    pub id: String,
    pub name: String,
}

// Should trigger DE0203 - missing both serde derives
#[derive(Debug, Clone, ToSchema)]
pub struct MissingBothDto {
    pub id: String,
    pub name: String,
}

// Should NOT trigger - has both serde derives
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CorrectDto {
    pub id: String,
    pub name: String,
}

// Should NOT trigger - not a DTO (no Dto suffix)
#[derive(Debug, Clone)]
pub struct RegularStruct {
    pub id: String,
}

// Should trigger DE0203 - enum missing Serialize
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub enum StatusDto {
    Active,
    Inactive,
}

// Should trigger DE0203 - enum missing Deserialize
#[derive(Debug, Clone, Serialize, ToSchema)]
pub enum RoleDto {
    Admin,
    User,
}
