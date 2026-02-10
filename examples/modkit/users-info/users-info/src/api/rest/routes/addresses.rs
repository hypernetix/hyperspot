use super::{Action, License, Resource, dto, handlers};
use axum::Router;
use modkit::api::OpenApiRegistry;
use modkit::api::operation_builder::OperationBuilder;

pub(super) fn register_address_routes(mut router: Router, openapi: &dyn OpenApiRegistry) -> Router {
    // GET /users-info/v1/users/{id}/address - Get user's address
    router = OperationBuilder::get("/users-info/v1/users/{id}/address")
        .operation_id("users_info.get_user_address")
        .require_auth(&Resource::Addresses, &Action::Read)
        .require_license_features::<License>([])
        .summary("Get user address")
        .description("Retrieve the address for a specific user")
        .tag("addresses")
        .path_param("id", "User UUID")
        .handler(handlers::get_user_address)
        .json_response_with_schema::<dto::AddressDto>(openapi, http::StatusCode::OK, "User address")
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // PUT /users-info/v1/users/{id}/address - Upsert user's address
    router = OperationBuilder::put("/users-info/v1/users/{id}/address")
        .operation_id("users_info.put_user_address")
        .require_auth(&Resource::Addresses, &Action::Update)
        .require_license_features::<License>([])
        .summary("Upsert user address")
        .description("Create or replace the address for a specific user")
        .tag("addresses")
        .path_param("id", "User UUID")
        .json_request::<dto::PutAddressReq>(openapi, "Address data")
        .handler(handlers::put_user_address)
        .json_response_with_schema::<dto::AddressDto>(
            openapi,
            http::StatusCode::OK,
            "Address created or updated",
        )
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // DELETE /users-info/v1/users/{id}/address - Delete user's address
    router = OperationBuilder::delete("/users-info/v1/users/{id}/address")
        .operation_id("users_info.delete_user_address")
        .require_auth(&Resource::Addresses, &Action::Delete)
        .require_license_features::<License>([])
        .summary("Delete user address")
        .description("Delete the address for a specific user")
        .tag("addresses")
        .path_param("id", "User UUID")
        .handler(handlers::delete_user_address)
        .json_response(http::StatusCode::NO_CONTENT, "Address deleted successfully")
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router
}
