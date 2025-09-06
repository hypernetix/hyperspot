//! Example demonstrating the type-safe API operation builder
//!
//! This example shows how to use the type-safe OperationBuilder to create
//! API operations with compile-time guarantees.

use axum::{response::IntoResponse, Json, Router};
use modkit::api::{Missing, OpenApiRegistry, OperationBuilder, OperationSpec};
use serde_json::Value;

// Example OpenAPI registry implementation
struct ExampleRegistry {
    operations: Vec<OperationSpec>,
}

impl ExampleRegistry {
    fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    fn print_operations(&self) {
        for op in &self.operations {
            println!(
                "Registered: {} {} - {}",
                op.method.as_str(),
                op.path,
                op.summary.as_deref().unwrap_or("No summary")
            );
        }
    }
}

impl OpenApiRegistry for ExampleRegistry {
    fn register_operation(&self, spec: &OperationSpec) {
        // Mut interior not needed in example; for simplicity clone push requires interior mutability.
        // For this example, we ignore storing operations.
        let _ = spec;
    }

    fn ensure_schema_raw(
        &self,
        root_name: &str,
        _schemas: Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) -> String {
        // Example implementation - schemas not tracked, just return the name
        root_name.to_string()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Example handlers
async fn get_users() -> Json<Value> {
    Json(serde_json::json!({
        "users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]
    }))
}

async fn create_user() -> impl IntoResponse {
    Json(serde_json::json!({
        "id": 3,
        "name": "Charlie",
        "created": true
    }))
}

async fn get_user() -> Json<Value> {
    Json(serde_json::json!({
        "id": 1,
        "name": "Alice",
        "email": "alice@example.com"
    }))
}

#[tokio::main]
async fn main() {
    println!("Type-Safe API Operation Builder Example");
    println!("=======================================");

    let registry = ExampleRegistry::new();
    let mut router = Router::new();

    // Build a complete REST API using the type-safe builder
    // Each operation MUST have both a handler and at least one response

    println!("Building GET /users endpoint...");
    router = OperationBuilder::<Missing, Missing, ()>::get("/users")
        .operation_id("users.list")
        .summary("List all users")
        .description("Returns a paginated list of all users in the system")
        .tag("users")
        .query_param("limit", false, "Maximum number of users to return")
        .query_param("offset", false, "Number of users to skip for pagination")
        .json_response(200, "Successfully retrieved user list")
        .json_response(500, "Internal server error")
        .handler(get_users) // <- Required: handler must be set
        .register(router, &registry); // <- Only works when both handler and response are present

    println!("Building POST /users endpoint...");
    router = OperationBuilder::<Missing, Missing, ()>::post("/users")
        .operation_id("users.create")
        .summary("Create a new user")
        .description("Creates a new user account in the system")
        .tag("users")
        .json_response(201, "User created successfully") // <- Required: at least one response
        .json_response(400, "Invalid user data")
        .json_response(409, "User already exists")
        .json_response(500, "Internal server error")
        .handler(create_user) // <- Required: handler must be set
        .register(router, &registry);

    println!("Building GET /users/{{id}} endpoint...");
    let _router = OperationBuilder::<Missing, Missing, ()>::get("/users/{id}")
        .operation_id("users.get")
        .summary("Get user by ID")
        .description("Retrieves detailed information about a specific user")
        .tag("users")
        .path_param("id", "Unique identifier for the user")
        .json_response(200, "User details retrieved successfully")
        .json_response(404, "User not found")
        .json_response(500, "Internal server error")
        .handler(get_user)
        .register(router, &registry);

    println!("\nAPI Operations registered:");
    println!("=========================");
    registry.print_operations();

    println!("\nType Safety Demonstrations:");
    println!("===========================");

    println!("✅ VALID: Both handler and response are provided");
    println!("   OperationBuilder::get(\"/example\")");
    println!("     .json_response(200, \"OK\")");
    println!("     .handler(some_handler)");
    println!("     .register(router, registry) // ← This compiles!");

    println!();
    println!("❌ INVALID: Missing handler (compile-time error)");
    println!("   OperationBuilder::get(\"/example\")");
    println!("     .json_response(200, \"OK\")");
    println!("     .register(router, registry) // ← Compilation error!");

    println!();
    println!("❌ INVALID: Missing response (compile-time error)");
    println!("   OperationBuilder::get(\"/example\")");
    println!("     .handler(some_handler)");
    println!("     .register(router, registry) // ← Compilation error!");

    println!();
    println!("✅ FLEXIBLE: Descriptive methods can be called in any order");
    println!("   OperationBuilder::get(\"/example\")");
    println!("     .summary(\"Example\")        // ← Can be anywhere");
    println!("     .handler(some_handler)      // ← Can be anywhere");
    println!("     .description(\"Details\")     // ← Can be anywhere");
    println!("     .json_response(200, \"OK\")   // ← Can be anywhere");
    println!("     .tag(\"example\")             // ← Can be anywhere");
    println!("     .register(router, registry) // ← Always at the end");

    println!("\n🎉 All operations built successfully with compile-time type safety!");
}
