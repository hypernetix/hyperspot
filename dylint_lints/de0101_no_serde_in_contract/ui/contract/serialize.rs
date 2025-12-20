mod contract {
    use serde::Serialize;

    #[allow(dead_code)]
    #[derive(Debug, Clone, Serialize)]
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
    #[derive(Debug, Clone, Serialize)]
    pub enum UserRole {
        Admin,
        User,
        Guest,
    }
}

fn main() {}