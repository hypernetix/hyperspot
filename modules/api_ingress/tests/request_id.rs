use axum::{
    body::Body,
    extract::Extension,
    http::{Request, StatusCode},
    response::Json,
    routing::get,
    Router,
};
use serde_json::json;
use tower::util::ServiceExt; // for `oneshot`

use api_ingress::request_id::{header as request_id_header, MakeReqId, XRequestId};

#[tokio::test]
async fn generates_request_id_when_missing() {
    let app = test_app();

    let response = app
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let request_id = response
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok());
    assert!(request_id.is_some(), "x-request-id should be generated");
    assert!(
        !request_id.unwrap().is_empty(),
        "request_id should not be empty"
    );
}

#[tokio::test]
async fn preserves_incoming_request_id() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("x-request-id", "abc-123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let request_id = response
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok());
    assert_eq!(request_id, Some("abc-123"));
}

#[tokio::test]
async fn includes_request_id_in_error_json() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/error")
                .header("x-request-id", "error-test-123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // Check header
    let request_id = response
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok());
    assert_eq!(request_id, Some("error-test-123"));

    // Check JSON body
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["error"], "Test error");
    assert_eq!(json["code"], 500);
    assert_eq!(json["request_id"], "error-test-123");
}

// Test app with success and error routes
fn test_app() -> Router {
    use axum::middleware::from_fn;
    use tower_http::request_id::{PropagateRequestIdLayer, SetRequestIdLayer};

    let x_request_id = request_id_header();

    let routes = Router::new()
        .route("/test", get(success_handler))
        .route("/error", get(error_handler));

    Router::new()
        .merge(routes)
        .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MakeReqId::default(),
        ))
        .layer(from_fn(api_ingress::request_id::push_req_id_to_extensions))
        .layer(api_ingress::request_id::create_trace_layer())
}

async fn success_handler(
    Extension(XRequestId(request_id)): Extension<XRequestId>,
) -> Json<serde_json::Value> {
    Json(json!({"status": "ok", "request_id": request_id}))
}

async fn error_handler(
    Extension(XRequestId(request_id)): Extension<XRequestId>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": "Test error",
            "code": 500,
            "request_id": request_id
        })),
    )
}
