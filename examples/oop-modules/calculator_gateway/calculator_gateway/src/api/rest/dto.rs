//! REST DTOs for calculator_gateway module
//!
//! These types are transport-specific (serde + utoipa for REST/OpenAPI).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request to add two numbers.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AddRequest {
    /// First operand
    pub a: i64,
    /// Second operand
    pub b: i64,
}

/// Response containing the sum.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AddResponse {
    /// The sum of a and b
    pub sum: i64,
}
