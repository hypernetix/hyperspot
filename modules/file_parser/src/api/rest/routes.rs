use crate::api::rest::handlers;
use crate::domain::service::FileParserService;
use axum::{Extension, Router};
use modkit::api::{OpenApiRegistry, OperationBuilder};
use std::sync::Arc;

#[allow(clippy::needless_pass_by_value)] // Arc is intentionally passed by value for Extension layer
pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<FileParserService>,
) -> Router {
    // GET /file-parser/info - Get information about available file parsers
    router = OperationBuilder::get("/file-parser/info")
        .operation_id("file_parser.get_parser_info")
        .summary("Get information about available file parsers")
        .tag("File Parser")
        .require_auth("file_parser", "read")
        .handler(handlers::get_parser_info)
        .json_response_with_schema::<crate::api::rest::dto::FileParserInfoDto>(
            openapi,
            http::StatusCode::OK,
            "Information about available parsers",
        )
        .standard_errors(openapi)
        .register(router, openapi);

    // POST /file-parser/parse-local - Parse a file from a local path
    router = OperationBuilder::post("/file-parser/parse-local")
        .operation_id("file_parser.parse_local")
        .summary("Parse a file from a local path")
        .tag("File Parser")
        .require_auth("file_parser", "read")
        .query_param_typed(
            "render_markdown",
            false,
            "Render Markdown output if true (optional, default false)",
            "boolean",
        )
        .json_request::<crate::api::rest::dto::ParseLocalFileRequest>(openapi, "Local file path")
        .allow_content_types(&["application/json"])
        .handler(handlers::parse_local)
        .json_response_with_schema::<crate::api::rest::dto::ParsedDocResponseDto>(
            openapi,
            http::StatusCode::OK,
            "Parsed document with optional markdown",
        )
        .standard_errors(openapi)
        .error_415(openapi)
        .register(router, openapi);

    // POST /file-parser/upload - Upload and parse a file
    router = OperationBuilder::post("/file-parser/upload")
        .operation_id("file_parser.upload")
        .summary("Upload and parse a file")
        .tag("File Parser")
        .require_auth("file_parser", "read")
        .query_param_typed(
            "render_markdown",
            false,
            "Render Markdown output if true (optional, default false)",
            "boolean",
        )
        .query_param_typed(
            "filename",
            false,
            "Optional original filename (used to determine file type if Content-Type is ambiguous)",
            "string",
        )
        .octet_stream_request(Some("Raw file bytes to parse"))
        .handler(handlers::upload_and_parse)
        .json_response_with_schema::<crate::api::rest::dto::ParsedDocResponseDto>(
            openapi,
            http::StatusCode::OK,
            "Parsed document with optional markdown",
        )
        .standard_errors(openapi)
        .error_415(openapi)
        .register(router, openapi);

    // POST /file-parser/parse-url - Parse a file from a URL
    router = OperationBuilder::post("/file-parser/parse-url")
        .operation_id("file_parser.parse_url")
        .summary("Parse a file from a URL")
        .tag("File Parser")
        .require_auth("file_parser", "read")
        .query_param_typed(
            "render_markdown",
            false,
            "Render Markdown output if true (optional, default false)",
            "boolean",
        )
        .json_request::<crate::api::rest::dto::ParseUrlRequest>(openapi, "URL to file")
        .allow_content_types(&["application/json"])
        .handler(handlers::parse_url)
        .json_response_with_schema::<crate::api::rest::dto::ParsedDocResponseDto>(
            openapi,
            http::StatusCode::OK,
            "Parsed document with optional markdown",
        )
        .standard_errors(openapi)
        .error_415(openapi)
        .register(router, openapi);

    // POST /file-parser/parse-local/markdown - Parse a local file and stream Markdown
    router = OperationBuilder::post("/file-parser/parse-local/markdown")
        .operation_id("file_parser.parse_local_markdown")
        .summary("Parse a local file and stream Markdown")
        .tag("File Parser")
        .require_auth("file_parser", "read")
        .json_request::<crate::api::rest::dto::ParseLocalFileRequest>(openapi, "Local file path")
        .allow_content_types(&["application/json"])
        .handler(handlers::parse_local_markdown)
        .text_response(http::StatusCode::OK, "Markdown stream", "text/markdown")
        .standard_errors(openapi)
        .register(router, openapi);

    // POST /file-parser/upload/markdown - Upload and parse a file, streaming Markdown
    router = OperationBuilder::post("/file-parser/upload/markdown")
        .operation_id("file_parser.upload_markdown")
        .summary("Upload and parse a file, streaming Markdown")
        .tag("File Parser")
        .require_auth("file_parser", "read")
        .multipart_file_request("file", Some("File to parse and stream as Markdown"))
        .handler(handlers::upload_and_parse_markdown)
        .text_response(http::StatusCode::OK, "Markdown stream", "text/markdown")
        .standard_errors(openapi)
        .error_415(openapi)
        .register(router, openapi);

    // POST /file-parser/parse-url/markdown - Parse a file from a URL and stream Markdown
    router = OperationBuilder::post("/file-parser/parse-url/markdown")
        .operation_id("file_parser.parse_url_markdown")
        .summary("Parse a file from a URL and stream Markdown")
        .tag("File Parser")
        .require_auth("file_parser", "read")
        .json_request::<crate::api::rest::dto::ParseUrlRequest>(openapi, "URL to file")
        .allow_content_types(&["application/json"])
        .handler(handlers::parse_url_markdown)
        .text_response(http::StatusCode::OK, "Markdown stream", "text/markdown")
        .standard_errors(openapi)
        .error_415(openapi)
        .register(router, openapi);

    router = router.layer(Extension(service.clone()));

    router
}
