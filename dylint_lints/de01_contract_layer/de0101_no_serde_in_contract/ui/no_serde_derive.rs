// simulated_dir=/hyperspot/modules/some_module/contract/
#[allow(dead_code)]
// Should not trigger DE0101 - Serde in contract
#[derive(Debug, Clone, PartialEq)]
pub struct Invoice {
    pub id: String,
    pub amount: i64,
}

#[allow(dead_code)]
// Should not trigger DE0101 - Serde in contract
#[derive(Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
}

fn main() {}
