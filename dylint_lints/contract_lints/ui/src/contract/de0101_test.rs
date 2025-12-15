// Test DE0101: No Serde in Contract Models
use serde::{Deserialize, Serialize};

// Should trigger DE0101 - Serialize
// Should trigger DE0101 - Deserialize
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
}

// Should trigger DE0101 - Serialize
#[derive(Debug, Clone, Serialize)]
pub struct Product {
    pub id: String,
    pub price: f64,
}

// Should trigger DE0101 - Deserialize
#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    pub id: String,
    pub total: f64,
}

// Should NOT trigger - no serde derives
#[derive(Debug, Clone, PartialEq)]
pub struct Invoice {
    pub id: String,
    pub amount: i64,
}

// Should trigger DE0101 - Serialize
// Should trigger DE0101 - Deserialize
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

// Should NOT trigger DE0101 - enum without serde
#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
}
