// Test file to verify DE0101, DE0102, DE0103 lints work
// This simulates a contract module with violations

// Simulating being in src/contract/model.rs
mod contract {
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    // DE0101: Should trigger - serde in contract
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BadUser {
        pub id: String,
        pub email: String,
    }

    // DE0102: Should trigger - ToSchema in contract
    #[derive(Debug, Clone, ToSchema)]
    pub struct BadDto {
        pub name: String,
    }

    // DE0103: Should trigger - HTTP type import
    // use http::StatusCode; // This would trigger if uncommented
}

fn main() {}
