use super::{Action, License, Resource, dto, handlers};
use axum::Router;
use modkit::api::operation_builder::{OperationBuilder, OperationBuilderODataExt};
use modkit::api::OpenApiRegistry;
use user_info_sdk::odata::LanguageFilterField;

pub(super) fn register_language_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
) -> Router {
    // GET /users-info/v1/languages - List languages with cursor-based pagination
    router = OperationBuilder::get("/users-info/v1/languages")
        .operation_id("users_info.list_languages")
        .summary("List languages with cursor pagination")
        .description("Retrieve a paginated list of languages using cursor-based pagination")
        .tag("languages")
        .require_auth(&Resource::Languages, &Action::Read)
        .require_license_features::<License>([])
        .query_param_typed(
            "limit",
            false,
            "Maximum number of languages to return",
            "integer",
        )
        .query_param("cursor", false, "Cursor for pagination")
        .handler(handlers::list_languages)
        .json_response_with_schema::<modkit_odata::Page<dto::LanguageDto>>(
            openapi,
            http::StatusCode::OK,
            "Paginated list of languages",
        )
        .with_odata_filter::<LanguageFilterField>()
        .with_odata_select()
        .with_odata_orderby::<LanguageFilterField>()
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /users-info/v1/languages/{id} - Get a specific language
    router = OperationBuilder::get("/users-info/v1/languages/{id}")
        .operation_id("users_info.get_language")
        .require_auth(&Resource::Languages, &Action::Read)
        .require_license_features::<License>([])
        .summary("Get language by ID")
        .description("Retrieve a specific language by UUID")
        .tag("languages")
        .path_param("id", "Language UUID")
        .handler(handlers::get_language)
        .with_odata_select()
        .json_response_with_schema::<dto::LanguageDto>(
            openapi,
            http::StatusCode::OK,
            "Language found",
        )
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // POST /users-info/v1/languages - Create a new language
    router = OperationBuilder::post("/users-info/v1/languages")
        .operation_id("users_info.create_language")
        .require_auth(&Resource::Languages, &Action::Create)
        .require_license_features::<License>([])
        .summary("Create a new language")
        .description("Create a new language with the provided information")
        .tag("languages")
        .json_request::<dto::CreateLanguageReq>(openapi, "Language creation data")
        .handler(handlers::create_language)
        .json_response_with_schema::<dto::LanguageDto>(
            openapi,
            http::StatusCode::CREATED,
            "Created language",
        )
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_409(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // PATCH /users-info/v1/languages/{id} - Update a language
    router = OperationBuilder::patch("/users-info/v1/languages/{id}")
        .operation_id("users_info.update_language")
        .require_auth(&Resource::Languages, &Action::Update)
        .require_license_features::<License>([])
        .summary("Update language")
        .description("Partially update a language with the provided fields")
        .tag("languages")
        .path_param("id", "Language UUID")
        .json_request::<dto::UpdateLanguageReq>(openapi, "Language update data")
        .handler(handlers::update_language)
        .json_response_with_schema::<dto::LanguageDto>(
            openapi,
            http::StatusCode::OK,
            "Updated language",
        )
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_409(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // DELETE /users-info/v1/languages/{id} - Delete a language
    router = OperationBuilder::delete("/users-info/v1/languages/{id}")
        .operation_id("users_info.delete_language")
        .require_auth(&Resource::Languages, &Action::Delete)
        .require_license_features::<License>([])
        .summary("Delete language")
        .description("Delete a language by UUID")
        .tag("languages")
        .path_param("id", "Language UUID")
        .handler(handlers::delete_language)
        .json_response(
            http::StatusCode::NO_CONTENT,
            "Language deleted successfully",
        )
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router
}
