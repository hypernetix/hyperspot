use super::{dto, handlers, Action, License, Resource};
use axum::Router;
use modkit::api::operation_builder::{OperationBuilder, OperationBuilderODataExt};
use modkit::api::OpenApiRegistry;
use user_info_sdk::odata::CityFilterField;

pub(super) fn register_city_routes(mut router: Router, openapi: &dyn OpenApiRegistry) -> Router {
    // GET /users-info/v1/cities - List cities with cursor-based pagination
    router = OperationBuilder::get("/users-info/v1/cities")
        .operation_id("users_info.list_cities")
        .summary("List cities with cursor pagination")
        .description("Retrieve a paginated list of cities using cursor-based pagination")
        .tag("cities")
        .require_auth(&Resource::Cities, &Action::Read)
        .require_license_features::<License>([])
        .query_param_typed(
            "limit",
            false,
            "Maximum number of cities to return",
            "integer",
        )
        .query_param("cursor", false, "Cursor for pagination")
        .handler(handlers::list_cities)
        .json_response_with_schema::<modkit_odata::Page<dto::CityDto>>(
            openapi,
            http::StatusCode::OK,
            "Paginated list of cities",
        )
        .with_odata_filter::<CityFilterField>()
        .with_odata_select()
        .with_odata_orderby::<CityFilterField>()
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /users-info/v1/cities/{id} - Get a specific city
    router = OperationBuilder::get("/users-info/v1/cities/{id}")
        .operation_id("users_info.get_city")
        .require_auth(&Resource::Cities, &Action::Read)
        .require_license_features::<License>([])
        .summary("Get city by ID")
        .description("Retrieve a specific city by UUID")
        .tag("cities")
        .path_param("id", "City UUID")
        .handler(handlers::get_city)
        .with_odata_select()
        .json_response_with_schema::<dto::CityDto>(openapi, http::StatusCode::OK, "City found")
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // POST /users-info/v1/cities - Create a new city
    router = OperationBuilder::post("/users-info/v1/cities")
        .operation_id("users_info.create_city")
        .require_auth(&Resource::Cities, &Action::Create)
        .require_license_features::<License>([])
        .summary("Create a new city")
        .description("Create a new city with the provided information")
        .tag("cities")
        .json_request::<dto::CreateCityReq>(openapi, "City creation data")
        .handler(handlers::create_city)
        .json_response_with_schema::<dto::CityDto>(
            openapi,
            http::StatusCode::CREATED,
            "Created city",
        )
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_409(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // PATCH /users-info/v1/cities/{id} - Update a city
    router = OperationBuilder::patch("/users-info/v1/cities/{id}")
        .operation_id("users_info.update_city")
        .require_auth(&Resource::Cities, &Action::Update)
        .require_license_features::<License>([])
        .summary("Update city")
        .description("Partially update a city with the provided fields")
        .tag("cities")
        .path_param("id", "City UUID")
        .json_request::<dto::UpdateCityReq>(openapi, "City update data")
        .handler(handlers::update_city)
        .json_response_with_schema::<dto::CityDto>(openapi, http::StatusCode::OK, "Updated city")
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_409(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // DELETE /users-info/v1/cities/{id} - Delete a city
    router = OperationBuilder::delete("/users-info/v1/cities/{id}")
        .operation_id("users_info.delete_city")
        .require_auth(&Resource::Cities, &Action::Delete)
        .require_license_features::<License>([])
        .summary("Delete city")
        .description("Delete a city by UUID")
        .tag("cities")
        .path_param("id", "City UUID")
        .handler(handlers::delete_city)
        .json_response(http::StatusCode::NO_CONTENT, "City deleted successfully")
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router
}
