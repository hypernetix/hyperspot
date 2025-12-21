// simulated_dir=/hyperspot/modules/some_module/contract/
use utoipa::ToSchema;

#[allow(dead_code)]
#[derive(Debug, Clone, ToSchema)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, ToSchema)]
pub struct Order {
    pub id: String,
    pub total: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, ToSchema)]
pub enum Status {
    Active,
    Inactive,
    Pending,
}

fn main() {}
