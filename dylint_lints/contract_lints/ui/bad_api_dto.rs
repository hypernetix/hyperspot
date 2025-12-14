// Test file to verify DE0203, DE0204 lints work
// This simulates an API rest module with DTO violations

// Simulating being in src/api/rest/dto.rs
mod api {
    pub mod rest {
        // DTO missing Serialize derive - should trigger DE0203
        #[derive(Debug, Clone, Deserialize)]
        pub struct CreateUserRequest {
            pub name: String,
            pub email: String,
        }

        // DTO missing Deserialize derive - should trigger DE0203
        #[derive(Debug, Clone, Serialize)]
        pub struct UserResponse {
            pub id: String,
            pub name: String,
            pub email: String,
        }

        // DTO missing both serde derives - should trigger DE0203
        #[derive(Debug, Clone)]
        pub struct UpdateUserRequest {
            pub name: Option<String>,
            pub email: Option<String>,
        }

        // DTO missing ToSchema derive - should trigger DE0204
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct UserDto {
            pub id: String,
            pub name: String,
        }

        // DTO missing both ToSchema and one serde derive - should trigger DE0203 and DE0204
        #[derive(Debug, Clone, Serialize)]
        pub struct DeleteUserResponse {
            pub success: bool,
        }

        // Correct DTO - should not trigger any lints
        #[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
        pub struct CorrectUserDto {
            pub id: String,
            pub name: String,
            pub email: String,
        }
    }
}

fn main() {}
