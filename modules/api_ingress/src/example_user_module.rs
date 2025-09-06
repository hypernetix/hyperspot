//! Example User Module (ModKit)
//! Type-safe OperationBuilder + local typed client via macro helpers.

use anyhow::Result;
use async_trait::async_trait;
use axum::{
    extract::{Json, Path},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use modkit::contracts::{Module, OpenApiRegistry, RestfulModule};

// --- Models ---

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[schema(title = "User")]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub created_at: String, // ISO 8601
    pub is_active: bool,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[schema(title = "UserListResponse")]
pub struct UserListResponse {
    pub users: Vec<User>,
    pub total: u32,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[schema(title = "CreateUserRequest")]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

// --- Typed client API (object-safe) ---

pub trait UserApi: Send + Sync {
    fn total_users(&self) -> u64;
    fn get_user(&self, id: u32) -> Option<User>;
}

// Simple local implementation (stub/demo)
#[derive(Default)]
struct LocalUserClient;

impl UserApi for LocalUserClient {
    fn total_users(&self) -> u64 {
        2
    }
    fn get_user(&self, id: u32) -> Option<User> {
        Some(User {
            id,
            name: "Example User".into(),
            email: "user@example.com".into(),
            created_at: chrono::Utc::now().to_rfc3339(),
            is_active: true,
        })
    }
}

// --- Module ---

#[modkit::module(name = "user_example", caps = [rest])]
#[derive(Default)]
pub struct UserModule;

#[async_trait]
impl Module for UserModule {
    async fn init(&self, _ctx: &modkit::context::ModuleCtx) -> Result<()> {
        // Publish a local client instance into the ClientHub.
        // Macro generated: `expose_user_example_api(ctx, &Arc<dyn UserApi>)`.
        tracing::info!("UserModule initialized and client published");
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl RestfulModule for UserModule {
    fn register_rest(
        &self,
        _ctx: &modkit::context::ModuleCtx,
        router: Router,
        openapi: &dyn OpenApiRegistry,
    ) -> Result<Router> {
        use modkit::api::OperationBuilder;

        // Schemas
        // Schemas are now auto-registered via utoipa ToSchema when used in operations

        // GET /users — list
        let router = OperationBuilder::get("/users")
            .operation_id("user.list")
            .summary("List all users")
            .description("Retrieve a paginated list of all users in the system")
            .tag("Users")
            .query_param("limit", false, "Max users to return (default: 10)")
            .query_param("offset", false, "Users to skip (default: 0)")
            .query_param("search", false, "Filter by name or email")
            .json_response(200, "List of users")
            .handler(get(list_users))
            .register(router, openapi);

        // GET /users/{id}
        let router = OperationBuilder::get("/users/{id}")
            .operation_id("user.get")
            .summary("Get user by ID")
            .description("Retrieve a specific user by their unique identifier")
            .tag("Users")
            .path_param("id", "User identifier")
            .json_response(200, "User found")
            .json_response(404, "User not found")
            .handler(get(get_user))
            .register(router, openapi);

        // POST /users
        let router = OperationBuilder::post("/users")
            .operation_id("user.create")
            .summary("Create a new user")
            .description("Create a new user with the provided information")
            .tag("Users")
            .json_response(201, "User created")
            .json_response(400, "Invalid input data")
            .handler(axum::routing::post(create_user))
            .register(router, openapi);

        tracing::debug!("UserModule routes registered");
        Ok(router)
    }
}

// --- Handlers (demo stubs) ---

async fn list_users() -> Json<UserListResponse> {
    Json(UserListResponse {
        users: vec![
            User {
                id: 1,
                name: "Alice Smith".into(),
                email: "alice@example.com".into(),
                created_at: chrono::Utc::now().to_rfc3339(),
                is_active: true,
            },
            User {
                id: 2,
                name: "Bob Jones".into(),
                email: "bob@example.com".into(),
                created_at: chrono::Utc::now().to_rfc3339(),
                is_active: true,
            },
        ],
        total: 2,
        limit: 10,
        offset: 0,
    })
}

async fn get_user(Path(id): Path<u32>) -> Json<User> {
    Json(User {
        id,
        name: "Example User".into(),
        email: "user@example.com".into(),
        created_at: chrono::Utc::now().to_rfc3339(),
        is_active: true,
    })
}

async fn create_user(Json(req): Json<CreateUserRequest>) -> Json<User> {
    Json(User {
        id: 3,
        name: req.name,
        email: req.email,
        created_at: chrono::Utc::now().to_rfc3339(),
        is_active: true,
    })
}

/*
Client call sample:

...
let api: Arc<dyn user_module::UserApi> =
    user_module::user_example_client(&ctx.client_hub()).await;
let total = api.total_users();
...

*/
