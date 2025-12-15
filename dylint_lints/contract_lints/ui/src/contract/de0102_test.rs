// Test DE0102: No ToSchema in Contract Models
use utoipa::ToSchema;

// Should trigger DE0102 - ToSchema in contract
#[derive(Debug, Clone, ToSchema)]
pub struct User {
    pub id: String,
    pub name: String,
}

// Should trigger DE0102 - ToSchema with other derives
#[derive(Debug, Clone, PartialEq, ToSchema)]
pub struct Product {
    pub id: String,
    pub price: f64,
}

// Should NOT trigger - no ToSchema
#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    pub id: String,
    pub total: f64,
}

// Should NOT trigger - no ToSchema
#[derive(Debug, Clone)]
pub struct Invoice {
    pub id: String,
    pub amount: i64,
}

// Should trigger DE0102 - enum with ToSchema
#[derive(Debug, Clone, ToSchema)]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

// Should NOT trigger - enum without ToSchema
#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
}
