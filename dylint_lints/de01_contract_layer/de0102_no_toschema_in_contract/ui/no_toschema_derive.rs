// simulated_dir=/hyperspot/modules/some_module/contract/
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: f64,
}

#[allow(dead_code)]
#[derive(Clone, PartialEq)]
pub enum Status {
    Active,
    Inactive,
    Pending,
}

fn main() {}
