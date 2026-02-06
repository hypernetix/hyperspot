use super::{Action, Resource, dto, handlers};
use axum::Router;
use modkit::api::OpenApiRegistry;
use modkit::api::operation_builder::{OperationBuilder, OperationBuilderODataExt};
use user_info_sdk::odata::UserFilterField;

pub(super) fn register_user_routes(mut router: Router, openapi: &dyn OpenApiRegistry) -> Router {
    // GET /users-info/v1/users - List users with cursor-based pagination
    router = OperationBuilder::get("/users-info/v1/users")
        .operation_id("users_info.list_users")
        .summary("List users with cursor pagination")
        .description("Retrieve a paginated list of users using cursor-based pagination")
        .tag("users")
        .require_auth(&Resource::Users, &Action::Read)
        .query_param_typed(
            "limit",
            false,
            "Maximum number of users to return",
            "integer",
        )
        .require_no_license_features()
        .query_param("cursor", false, "Cursor for pagination")
        .handler(handlers::list_users)
        .json_response_with_schema::<modkit_odata::Page<dto::UserDto>>(
            openapi,
            http::StatusCode::OK,
            "Paginated list of users",
        )
        .with_odata_filter::<UserFilterField>()
        .with_odata_select()
        .with_odata_orderby::<UserFilterField>()
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /users-info/v1/users/{id} - Get a specific user
    router = OperationBuilder::get("/users-info/v1/users/{id}")
        .operation_id("users_info.get_user")
        .require_auth(&Resource::Users, &Action::Read)
        .require_no_license_features()
        .summary("Get user by ID")
        .description("Retrieve a specific user by their UUID")
        .tag("users")
        .path_param("id", "User UUID")
        .handler(handlers::get_user)
        .with_odata_select()
        .json_response_with_schema::<dto::UserDto>(openapi, http::StatusCode::OK, "User found")
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // POST /users-info/v1/users - Create a new user
    router = OperationBuilder::post("/users-info/v1/users")
        .operation_id("users_info.create_user")
        .require_auth(&Resource::Users, &Action::Create)
        .require_no_license_features()
        .summary("Create a new user")
        .description("Create a new user with the provided information")
        .tag("users")
        .json_request::<dto::CreateUserReq>(openapi, "User creation data")
        .handler(handlers::create_user)
        .json_response_with_schema::<dto::UserDto>(
            openapi,
            http::StatusCode::CREATED,
            "Created user",
        )
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_409(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // PATCH /users-info/v1/users/{id} - Partially update a user
    router = OperationBuilder::patch("/users-info/v1/users/{id}")
        .operation_id("users_info.update_user")
        .require_auth(&Resource::Users, &Action::Update)
        .require_no_license_features()
        .summary("Update user")
        .description("Partially update a user with the provided fields")
        .tag("users")
        .path_param("id", "User UUID")
        .json_request::<dto::UpdateUserReq>(openapi, "User update data")
        .handler(handlers::update_user)
        .json_response_with_schema::<dto::UserDto>(openapi, http::StatusCode::OK, "Updated user")
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_409(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // DELETE /users-info/v1/users/{id} - Delete a user
    router = OperationBuilder::delete("/users-info/v1/users/{id}")
        .operation_id("users_info.delete_user")
        .require_auth(&Resource::Users, &Action::Delete)
        .require_no_license_features()
        .summary("Delete user")
        .description("Delete a user by their UUID")
        .tag("users")
        .path_param("id", "User UUID")
        .handler(handlers::delete_user)
        .json_response(http::StatusCode::NO_CONTENT, "User deleted successfully")
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router
}
