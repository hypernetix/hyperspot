use axum::{
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    Json,
};

/// Short aliases for JSON responses
pub type JsonBody<T> = Json<T>;
pub type JsonPage<T> = Json<modkit_odata::Page<T>>;

/// 200 OK + JSON
pub fn ok_json<T: serde::Serialize>(value: T) -> impl IntoResponse {
    (StatusCode::OK, Json(value))
}

/// 201 Created + JSON with Location header
pub fn created_json<T: serde::Serialize>(value: T, uri: &Uri, new_id: &str) -> impl IntoResponse {
    let location = [uri.path().trim_end_matches('/'), new_id].join("/");
    (
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(value),
    )
}

/// 204 No Content
#[must_use]
pub fn no_content() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

/// Convert any `IntoResponse` into a concrete Response (useful for unified signatures)
pub fn to_response<R: IntoResponse>(r: R) -> Response {
    r.into_response()
}
