use axum::{Extension, Router};
use modkit::api::{OpenApiRegistry, OperationBuilder};
use std::sync::Arc;

use crate::api::rest::{dto, handlers};
use crate::domain::service::Service;

pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<Service>,
) -> anyhow::Result<Router> {
    // Schemas should be auto-registered via ToSchema when used in operations

    // GET /users - List all users
    router = OperationBuilder::<modkit::api::Missing, modkit::api::Missing, ()>::get("/users")
        .operation_id("users_info.list_users")
        .summary("List all users")
        .description("Retrieve a paginated list of all users in the system")
        .tag("users")
        .query_param("limit", false, "Maximum number of users to return")
        .query_param("offset", false, "Number of users to skip")
        .handler(handlers::list_users)
        .json_response_with_schema::<dto::UserListDto>(openapi, 200, "List of users")
        .problem_response(openapi, 400, "Bad Request")
        .problem_response(openapi, 500, "Internal Server Error")
        .register(router, openapi);

    // GET /users/{id} - Get a specific user
    router = OperationBuilder::<modkit::api::Missing, modkit::api::Missing, ()>::get("/users/{id}")
        .operation_id("users_info.get_user")
        .summary("Get user by ID")
        .description("Retrieve a specific user by their UUID")
        .tag("users")
        .path_param("id", "User UUID")
        .handler(handlers::get_user)
        .json_response_with_schema::<dto::UserDto>(openapi, 200, "User found")
        .problem_response(openapi, 404, "Not Found")
        .problem_response(openapi, 500, "Internal Server Error")
        .register(router, openapi);

    // POST /users - Create a new user
    router = OperationBuilder::<modkit::api::Missing, modkit::api::Missing, ()>::post("/users")
        .operation_id("users_info.create_user")
        .summary("Create a new user")
        .description("Create a new user with the provided information")
        .tag("users")
        .json_request::<dto::CreateUserReq>(openapi, "User creation data")
        .handler(handlers::create_user)
        .json_response_with_schema::<dto::UserDto>(openapi, 201, "Created user")
        .problem_response(openapi, 400, "Bad Request")
        .problem_response(openapi, 409, "Conflict")
        .problem_response(openapi, 500, "Internal Server Error")
        .register(router, openapi);

    // PUT /users/{id} - Update a user
    router = OperationBuilder::<modkit::api::Missing, modkit::api::Missing, ()>::put("/users/{id}")
        .operation_id("users_info.update_user")
        .summary("Update user")
        .description("Update a user with partial data")
        .tag("users")
        .path_param("id", "User UUID")
        .json_request::<dto::UpdateUserReq>(openapi, "User update data")
        .handler(handlers::update_user)
        .json_response_with_schema::<dto::UserDto>(openapi, 200, "Updated user")
        .problem_response(openapi, 400, "Bad Request")
        .problem_response(openapi, 404, "Not Found")
        .problem_response(openapi, 409, "Conflict")
        .problem_response(openapi, 500, "Internal Server Error")
        .register(router, openapi);

    // DELETE /users/{id} - Delete a user
    router =
        OperationBuilder::<modkit::api::Missing, modkit::api::Missing, ()>::delete("/users/{id}")
            .operation_id("users_info.delete_user")
            .summary("Delete user")
            .description("Delete a user by their UUID")
            .tag("users")
            .path_param("id", "User UUID")
            .handler(handlers::delete_user)
            .json_response(204, "User deleted successfully")
            .problem_response(openapi, 404, "Not Found")
            .problem_response(openapi, 500, "Internal Server Error")
            .register(router, openapi);

    router = router.layer(Extension(service.clone()));

    Ok(router)
}
