// Test DE0201: DTOs Only in API Rest Folder
// This file is in domain/ which is WRONG for DTOs
use serde::{Deserialize, Serialize};

// Should trigger DE0201 - DTO in domain layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    pub id: String,
    pub name: String,
}

// Should trigger DE0201 - DTO in domain layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDto {
    pub id: String,
    pub price: f64,
}

// Should NOT trigger - not a DTO (no dto suffix)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
}
