use axum::Router;
use std::sync::Arc;

use crate::domain::gts_core::GtsCoreRouter;
use super::handlers::handle_gts_request;

pub fn create_router(gts_router: Arc<GtsCoreRouter>) -> Router {
    Router::new()
        .route("/gts/{id}", axum::routing::get(handle_gts_request))
        .route("/gts/{id}", axum::routing::post(handle_gts_request))
        .route("/gts/{id}", axum::routing::put(handle_gts_request))
        .route("/gts/{id}", axum::routing::patch(handle_gts_request))
        .route("/gts/{id}", axum::routing::delete(handle_gts_request))
        .with_state(gts_router)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::gts_core::RoutingTable;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_router_handles_get_request() {
        let mut table = RoutingTable::new();
        table.register("gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.test.v1", "feature-one").unwrap();
        let router = Arc::new(GtsCoreRouter::new(table));
        
        let app = create_router(router);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/gts/gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.instance.v1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_router_handles_post_request() {
        let mut table = RoutingTable::new();
        table.register("gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.test.v1", "feature-one").unwrap();
        let router = Arc::new(GtsCoreRouter::new(table));
        
        let app = create_router(router);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/gts/gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.instance.v1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
