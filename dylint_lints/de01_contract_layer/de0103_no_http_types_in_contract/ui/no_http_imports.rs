// simulated_dir=/hyperspot/modules/some_module/contract/
#[derive(Debug, Clone)]
#[allow(dead_code)]
// Should not trigger DE0103 - HTTP types in contract
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
// Should not trigger DE0103 - HTTP types in contract
pub struct OrderResult {
    pub status: OrderStatus,
}

fn main() {}
