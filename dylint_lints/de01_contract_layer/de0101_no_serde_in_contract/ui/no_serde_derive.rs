mod contract {
    #[allow(dead_code)]
    #[derive(Debug, Clone, PartialEq)]
    pub struct Invoice {
        pub id: String,
        pub amount: i64,
    }

    #[allow(dead_code)]
    #[derive(Clone, PartialEq)]
    pub enum OrderStatus {
        Pending,
        Confirmed,
        Shipped,
    }
}

fn main() {}
