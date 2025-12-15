// Test DE0201: DTOs Only in API Rest Folder
// This file is CORRECTLY in api/rest/, so DTOs here are OK
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Should NOT trigger - DTO in correct location with all derives
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserDto {
    pub id: String,
    pub name: String,
}

// Should NOT trigger - DTO in correct location with all derives
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProductDto {
    pub id: String,
    pub price: f64,
}
