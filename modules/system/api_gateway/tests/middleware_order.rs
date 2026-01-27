#![allow(clippy::expect_used)]

//! Validates the *actual* middleware execution order of `ApiGateway::apply_middleware_stack`.
//!
//! The intended order is documented in `modules/api_gateway/src/lib.rs`:
//! set request id -> propagate request id -> trace -> push request id to extensions
//! -> timeout -> body limit -> CORS -> MIME validation -> rate limit -> error mapping -> auth -> router
//!
use anyhow::Result;
use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    extract::{Extension, Json},
    http::{Request, StatusCode},
    response::IntoResponse,
};
use modkit::{
    Module, api::OperationBuilder, config::ConfigProvider, context::ModuleCtx,
    contracts::ApiGatewayCapability,
};
use tenant_resolver_sdk::{
    AccessOptions, TenantFilter, TenantId, TenantInfo, TenantResolverError,
    TenantResolverGatewayClient, TenantStatus,
};

use modkit_security::SecurityContext;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

use api_gateway::middleware::request_id::XRequestId;

struct MockTenantResolver;

#[async_trait]
impl TenantResolverGatewayClient for MockTenantResolver {
    async fn get_tenant(
        &self,
        _ctx: &SecurityContext,
        id: TenantId,
    ) -> std::result::Result<TenantInfo, TenantResolverError> {
        Ok(TenantInfo {
            id,
            name: format!("Tenant {id}"),
            status: TenantStatus::Active,
            tenant_type: None,
        })
    }

    async fn can_access(
        &self,
        _ctx: &SecurityContext,
        _target: TenantId,
        _options: Option<&AccessOptions>,
    ) -> std::result::Result<bool, TenantResolverError> {
        Ok(true)
    }

    async fn get_accessible_tenants(
        &self,
        _ctx: &SecurityContext,
        _filter: Option<&TenantFilter>,
        _options: Option<&AccessOptions>,
    ) -> std::result::Result<Vec<TenantInfo>, TenantResolverError> {
        Ok(vec![])
    }
}

struct TestConfigProvider {
    config: serde_json::Value,
}

impl ConfigProvider for TestConfigProvider {
    fn get_module_config(&self, module: &str) -> Option<&serde_json::Value> {
        self.config.get(module)
    }
}

fn create_api_gateway_ctx(config: serde_json::Value) -> ModuleCtx {
    let hub = Arc::new(modkit::ClientHub::new());
    hub.register::<dyn TenantResolverGatewayClient>(Arc::new(MockTenantResolver));

    ModuleCtx::new(
        "api_gateway",
        Uuid::new_v4(),
        Arc::new(TestConfigProvider { config }),
        hub,
        tokio_util::sync::CancellationToken::new(),
        None,
    )
}

async fn handler(Extension(XRequestId(rid)): Extension<XRequestId>) -> impl IntoResponse {
    Json(json!({ "request_id": rid }))
}

#[tokio::test]
async fn real_middlewares_observe_documented_order() -> Result<()> {
    // Configure strict + deterministic rate limiting for the test route.
    let cfg = json!({
        "api_gateway": {
            "config": {
                "bind_addr": "127.0.0.1:0",
                "cors_enabled": true,
                "auth_disabled": true,
                "defaults": {
                    "rate_limit": { "rps": 1, "burst": 1, "in_flight": 64 }
                }
            }
        }
    });
    let ctx = create_api_gateway_ctx(cfg);

    let api = api_gateway::ApiGateway::default();
    api.init(&ctx).await?;

    // Register an endpoint that enables both MIME validation and rate limiting.
    let router = Router::new();
    let mut builder = OperationBuilder::post("/tests/v1/middleware-order");
    builder.require_rate_limit(1, 1, 64);
    let router = builder
        .operation_id("test:middleware-order")
        .summary("Middleware order test endpoint")
        .public()
        .allow_content_types(&["application/json"]) // turns on MIME validation
        .json_response(StatusCode::OK, "OK")
        .handler(axum::routing::post(handler))
        .register(router, &api);

    // Apply the real gateway middleware stack.
    let app = api.rest_finalize(&ctx, router)?;

    // --------------------
    // Req1: invalid Content-Type -> should be rejected by MIME validation (415),
    // but MUST still have CORS headers (CORS wraps MIME).
    // Also: x-request-id must be echoed (request-id is outermost).
    // --------------------
    let res1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tests/v1/middleware-order")
                .header("origin", "https://example.com")
                .header("x-request-id", "fixed-req-1")
                .header("content-type", "text/plain")
                .body(Body::from("hi"))?,
        )
        .await?;
    assert_eq!(res1.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(
        res1.headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok()),
        Some("fixed-req-1")
    );
    assert!(
        res1.headers().get("access-control-allow-origin").is_some(),
        "CORS header must be present on 415 => CORS wraps MIME validation"
    );

    // --------------------
    // Req2: valid Content-Type -> should pass MIME + consume rate-limit token.
    // Also handler must see request-id via extensions.
    // --------------------
    let res2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tests/v1/middleware-order")
                .header("origin", "https://example.com")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"ok":true}"#))?,
        )
        .await?;
    assert_eq!(res2.status(), StatusCode::OK);
    let res2_rid = res2
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .expect("x-request-id must be set on success")
        .to_owned();
    let body2 = axum::body::to_bytes(res2.into_body(), usize::MAX).await?;
    let json2: serde_json::Value = serde_json::from_slice(&body2)?;
    assert_eq!(
        json2.get("request_id").and_then(|v| v.as_str()),
        Some(res2_rid.as_str()),
        "handler must receive request_id via extensions (push_req_id_to_extensions)"
    );

    // --------------------
    // Req3: another valid request immediately -> must be rate-limited (429),
    // proving Req1 didn't consume token (MIME before rate limit), while Req2 did.
    // --------------------
    let res3 = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tests/v1/middleware-order")
                .header("origin", "https://example.com")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"ok":true}"#))?,
        )
        .await?;
    assert_eq!(
        res3.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "Second valid request should be rate-limited (token consumed by Req2, not by Req1)"
    );

    Ok(())
}
