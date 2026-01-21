// simulated_dir=/hyperspot/modules/some_module/contract/
#![allow(dead_code)]

// Should trigger DE0104 - api_dto in contract
#[modkit_macros::api_dto(request, response)]
pub struct User {
    pub id: String,
    pub name: String,
}

// Should trigger DE0104 - api_dto in contract
#[modkit_macros::api_dto(response)]
pub struct Product {
    pub id: String,
    pub price: f64,
}

// Should trigger DE0104 - api_dto in contract
#[modkit_macros::api_dto(request)]
pub enum OrderStatus {
    Pending,
    Completed,
}

// Should not trigger DE0104 - api_dto in contract
pub struct ValidContract {
    pub field: String,
}

fn main() {}
