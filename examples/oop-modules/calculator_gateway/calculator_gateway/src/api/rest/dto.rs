//! REST DTOs for calculator_gateway module
//!
//! These types are transport-specific (serde + utoipa for REST/OpenAPI).

/// Request to add two numbers.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(request)]
pub struct AddRequest {
    /// First operand
    pub a: i64,
    /// Second operand
    pub b: i64,
}

/// Response containing the sum.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AddResponse {
    /// The sum of a and b
    pub sum: i64,
}
