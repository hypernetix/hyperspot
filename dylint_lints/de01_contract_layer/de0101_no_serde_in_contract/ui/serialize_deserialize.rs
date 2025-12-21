// simulated_dir=/hyperspot/modules/some_module/contract/
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct Product {
    pub id: String,
    pub price: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    pub id: String,
    pub total: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Invoice {
    pub id: String,
    pub amount: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
}

fn main() {}
