mod contract {
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub enum OrderStatus {
        Pending,
        Confirmed,
        Shipped,
    }

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub struct OrderResult {
        pub status: OrderStatus,
    }
}

fn main() {}
