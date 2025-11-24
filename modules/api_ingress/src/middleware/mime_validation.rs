//! MIME type validation middleware for enforcing per-operation allowed Content-Type headers
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use dashmap::DashMap;
use http::Method;
use std::sync::Arc;

use modkit::api::{OperationSpec, Problem};

/// Map from (method, path) to allowed content types
pub type MimeValidationMap = Arc<DashMap<(Method, String), Vec<&'static str>>>;

/// Build MIME validation map from operation specs
pub fn build_mime_validation_map(specs: &[OperationSpec]) -> MimeValidationMap {
    let map = DashMap::new();

    for spec in specs {
        if let Some(ref allowed) = spec.allowed_request_content_types {
            let key = (spec.method.clone(), spec.path.clone());
            map.insert(key, allowed.clone());
        }
    }

    Arc::new(map)
}

/// MIME validation middleware
///
/// Checks the Content-Type header against the allowed types configured
/// for the operation. Returns 415 Unsupported Media Type if the content
/// type is not allowed.
pub async fn mime_validation_middleware(
    validation_map: MimeValidationMap,
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    // Use MatchedPath extension (set by Axum router) for accurate route matching
    let path = req
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());

    // Check if this operation has MIME validation configured
    if let Some(allowed_types) = validation_map.get(&(method.clone(), path.clone())) {
        // Extract Content-Type header
        if let Some(ct_header) = req.headers().get(http::header::CONTENT_TYPE) {
            match ct_header.to_str() {
                Ok(ct_str) => {
                    // Strip parameters: "type/subtype; charset=utf-8" -> "type/subtype"
                    let ct_main = ct_str.split(';').next().map(str::trim).unwrap_or(ct_str);

                    // Check if content type is allowed
                    let allowed_match = allowed_types.contains(&ct_main);

                    if !allowed_match {
                        tracing::warn!(
                            method = %method,
                            path = %path,
                            content_type = ct_main,
                            allowed_types = ?allowed_types.value(),
                            "MIME type not allowed for this endpoint"
                        );

                        let problem = Problem::new(
                            StatusCode::UNSUPPORTED_MEDIA_TYPE,
                            "Unsupported Media Type",
                            format!(
                                "Content-Type '{}' is not allowed for this endpoint. Allowed types: {}",
                                ct_main,
                                allowed_types.join(", ")
                            ),
                        );
                        return problem.into_response();
                    }
                }
                Err(_) => {
                    // Invalid header value - treat as unsupported
                    tracing::warn!(
                        method = %method,
                        path = %path,
                        "Invalid Content-Type header value"
                    );

                    let problem = Problem::new(
                        StatusCode::UNSUPPORTED_MEDIA_TYPE,
                        "Unsupported Media Type",
                        "Invalid Content-Type header",
                    );
                    return problem.into_response();
                }
            }
        } else {
            // No Content-Type header but validation is configured
            tracing::warn!(
                method = %method,
                path = %path,
                allowed_types = ?allowed_types.value(),
                "Missing Content-Type header for endpoint with MIME validation"
            );

            let problem = Problem::new(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Unsupported Media Type",
                format!(
                    "Missing Content-Type header. Allowed types: {}",
                    allowed_types.join(", ")
                ),
            );
            return problem.into_response();
        }
    }

    // No validation configured or validation passed - proceed
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_mime_validation_map() {
        use modkit::api::operation_builder::{RequestBodySchema, RequestBodySpec};

        let specs = vec![OperationSpec {
            method: Method::POST,
            path: "/upload".to_string(),
            operation_id: None,
            summary: None,
            description: None,
            tags: vec![],
            params: vec![],
            request_body: Some(RequestBodySpec {
                content_type: "multipart/form-data",
                description: None,
                schema: RequestBodySchema::MultipartFile {
                    field_name: "file".to_string(),
                },
                required: true,
            }),
            responses: vec![],
            handler_id: "test".to_string(),
            sec_requirement: None,
            is_public: false,
            rate_limit: None,
            allowed_request_content_types: Some(vec!["multipart/form-data", "application/pdf"]),
        }];

        let map = build_mime_validation_map(&specs);

        assert!(map.contains_key(&(Method::POST, "/upload".to_string())));
        let allowed = map.get(&(Method::POST, "/upload".to_string())).unwrap();
        assert_eq!(allowed.len(), 2);
        assert!(allowed.contains(&"multipart/form-data"));
        assert!(allowed.contains(&"application/pdf"));
    }

    #[test]
    fn test_content_type_parameter_stripping() {
        // Test the logic for stripping parameters from Content-Type
        let ct_with_charset = "application/json; charset=utf-8";
        let ct_main = ct_with_charset
            .split(';')
            .next()
            .map(str::trim)
            .unwrap_or(ct_with_charset);

        assert_eq!(ct_main, "application/json");

        // Test with multiple parameters
        let ct_complex = "multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW";
        let ct_main2 = ct_complex
            .split(';')
            .next()
            .map(str::trim)
            .unwrap_or(ct_complex);

        assert_eq!(ct_main2, "multipart/form-data");

        // Test without parameters
        let ct_simple = "application/pdf";
        let ct_main3 = ct_simple
            .split(';')
            .next()
            .map(str::trim)
            .unwrap_or(ct_simple);

        assert_eq!(ct_main3, "application/pdf");
    }
}
