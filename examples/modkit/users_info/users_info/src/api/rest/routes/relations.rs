use super::{Action, License, Resource, dto, handlers};
use axum::Router;
use modkit::api::operation_builder::OperationBuilder;
use modkit::api::OpenApiRegistry;

pub(super) fn register_relation_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
) -> Router {
    // GET /users-info/v1/users/{id}/languages - List user's languages
    router = OperationBuilder::get("/users-info/v1/users/{id}/languages")
        .operation_id("users_info.list_user_languages")
        .require_auth(&Resource::UserLanguages, &Action::Read)
        .require_license_features::<License>([])
        .summary("List user languages")
        .description("Retrieve all languages assigned to a user")
        .tag("user-languages")
        .path_param("id", "User UUID")
        .handler(handlers::list_user_languages)
        .json_response_with_schema::<Vec<dto::LanguageDto>>(
            openapi,
            http::StatusCode::OK,
            "List of user's languages",
        )
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // PUT /users-info/v1/users/{id}/languages/{langId} - Assign language to user
    router = OperationBuilder::put("/users-info/v1/users/{id}/languages/{langId}")
        .operation_id("users_info.assign_language_to_user")
        .require_auth(&Resource::UserLanguages, &Action::Update)
        .require_license_features::<License>([])
        .summary("Assign language to user")
        .description("Assign a language to a user (idempotent)")
        .tag("user-languages")
        .path_param("id", "User UUID")
        .path_param("langId", "Language UUID")
        .handler(handlers::assign_language_to_user)
        .json_response(
            http::StatusCode::NO_CONTENT,
            "Language assigned successfully",
        )
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // DELETE /users-info/v1/users/{id}/languages/{langId} - Remove language from user
    router = OperationBuilder::delete("/users-info/v1/users/{id}/languages/{langId}")
        .operation_id("users_info.remove_language_from_user")
        .require_auth(&Resource::UserLanguages, &Action::Delete)
        .require_license_features::<License>([])
        .summary("Remove language from user")
        .description("Remove a language from a user (idempotent)")
        .tag("user-languages")
        .path_param("id", "User UUID")
        .path_param("langId", "Language UUID")
        .handler(handlers::remove_language_from_user)
        .json_response(
            http::StatusCode::NO_CONTENT,
            "Language removed successfully",
        )
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router
}
