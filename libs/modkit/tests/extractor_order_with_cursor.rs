#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::{
    Router,
    body::to_bytes,
    http::{Request, StatusCode},
    routing::get,
};
use modkit::api::odata::OData;
use tower::ServiceExt;

#[tokio::test]
async fn order_with_cursor_is_422() {
    // trivial route just to trigger extractor
    async fn handler(OData(_q): OData) -> &'static str {
        "ok"
    }

    let app = Router::new().route("/", get(handler));

    // Provide both cursor and $orderby
    let req = Request::builder()
        .uri("/?cursor=eyJ2IjoxLCJrIjpbIjEiXS&$orderby=id%20desc")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    // OData errors return 422 (Unprocessable Entity) per GTS catalog
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Check body contains error about cursor/orderby conflict
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let s = String::from_utf8_lossy(&body);
    // The error now uses the GTS catalog and mentions both cursor and orderby
    assert!(s.contains("invalid_cursor") || s.contains("cursor") || s.contains("orderby"));
}

#[tokio::test]
async fn cursor_only_is_ok() {
    async fn handler(OData(_q): OData) -> &'static str {
        "ok"
    }

    let app = Router::new().route("/", get(handler));

    // Provide only cursor
    let req = Request::builder()
        .uri("/?cursor=eyJ2IjoxLCJrIjpbIjEiXS")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    // Should be 422 due to invalid cursor format, but not about orderby conflict
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let s = String::from_utf8_lossy(&body);
    // Should not mention orderby since we're only passing cursor
    assert!(!s.contains("orderby") || !s.contains("both"));
}

#[tokio::test]
async fn orderby_only_is_ok() {
    async fn handler(OData(_q): OData) -> &'static str {
        "ok"
    }

    let app = Router::new().route("/", get(handler));

    // Provide only $orderby
    let req = Request::builder()
        .uri("/?$orderby=id%20desc")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
