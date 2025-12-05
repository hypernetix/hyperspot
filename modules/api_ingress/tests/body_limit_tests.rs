#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Integration tests for request body size limits and compatibility with CORS

use anyhow::Result;
use async_trait::async_trait;
use axum::{extract::Json, routing::post, Router};
use modkit::{
    api::OperationBuilder,
    config::ConfigProvider,
    contracts::{OpenApiRegistry, RestHostModule},
    Module, ModuleCtx, RestfulModule,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

struct TestConfigProvider {
    config: serde_json::Value,
}

impl ConfigProvider for TestConfigProvider {
    fn get_module_config(&self, module: &str) -> Option<&serde_json::Value> {
        if module == "api_ingress" {
            Some(&self.config)
        } else {
            None
        }
    }
}

fn wrap_config(config: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "config": config
    })
}

fn create_test_module_ctx_with_body_limit(limit_bytes: usize) -> ModuleCtx {
    let config = wrap_config(serde_json::json!({
        "bind_addr": "127.0.0.1:0",
        "cors_enabled": true,
        "auth_disabled": true,
        "defaults": {
            "body_limit_bytes": limit_bytes,
            "rate_limit": {
                "rps": 100,
                "burst": 200,
                "in_flight": 64
            }
        }
    }));

    ModuleCtx::new(
        "api_ingress",
        Arc::new(TestConfigProvider { config }),
        Arc::new(modkit::ClientHub::new()),
        tokio_util::sync::CancellationToken::new(),
        None,
    )
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
struct LargePayload {
    data: String,
}

pub struct BodyLimitTestModule;

#[async_trait]
impl Module for BodyLimitTestModule {
    async fn init(&self, _ctx: &modkit::ModuleCtx) -> Result<()> {
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl RestfulModule for BodyLimitTestModule {
    fn register_rest(
        &self,
        _ctx: &modkit::ModuleCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> Result<axum::Router> {
        let router = OperationBuilder::post("/upload")
            .operation_id("test:upload")
            .summary("Upload endpoint with body limit")
            .json_request::<LargePayload>(openapi, "Large payload")
            .public()
            .json_response(http::StatusCode::OK, "Success")
            .json_response(http::StatusCode::PAYLOAD_TOO_LARGE, "Payload too large")
            .handler(post(upload_handler))
            .register(router, openapi);

        Ok(router)
    }
}

async fn upload_handler(Json(payload): Json<LargePayload>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "received": payload.data.len()
    }))
}

#[tokio::test]
async fn test_body_limit_configured() {
    let limit = 1024; // 1KB limit
    let api_ingress = api_ingress::ApiIngress::default();
    let ctx = create_test_module_ctx_with_body_limit(limit);
    api_ingress.init(&ctx).await.expect("Failed to init");

    let module = BodyLimitTestModule;
    let router = Router::new();
    let router = module
        .register_rest(&ctx, router, &api_ingress)
        .expect("Failed to register routes");

    let _final_router = api_ingress
        .rest_finalize(&ctx, router)
        .expect("Failed to finalize router");

    // Verify router builds with custom body limit
    let config = api_ingress.get_config();
    assert_eq!(
        config.defaults.body_limit_bytes, limit,
        "Body limit should match config"
    );
}

#[tokio::test]
async fn test_body_limit_with_cors() {
    // Verify body limit and CORS layers coexist
    let api_ingress = api_ingress::ApiIngress::default();
    let ctx = create_test_module_ctx_with_body_limit(16 * 1024 * 1024);
    api_ingress.init(&ctx).await.expect("Failed to init");

    let module = BodyLimitTestModule;
    let router = Router::new();
    let router = module
        .register_rest(&ctx, router, &api_ingress)
        .expect("Failed to register routes");

    let _final_router = api_ingress
        .rest_finalize(&ctx, router)
        .expect("Failed to finalize router");

    // Both CORS and body limit should be active
    let config = api_ingress.get_config();
    assert!(config.cors_enabled, "CORS should be enabled");
    assert!(
        config.defaults.body_limit_bytes > 0,
        "Body limit should be set"
    );
}

#[tokio::test]
async fn test_default_body_limit() {
    let config = wrap_config(serde_json::json!({
        "bind_addr": "127.0.0.1:0",
        "auth_disabled": true
    }));

    let ctx = ModuleCtx::new(
        "api_ingress",
        Arc::new(TestConfigProvider { config }),
        Arc::new(modkit::ClientHub::new()),
        tokio_util::sync::CancellationToken::new(),
        None,
    );

    let api_ingress = api_ingress::ApiIngress::default();
    api_ingress.init(&ctx).await.expect("Failed to init");

    let module = BodyLimitTestModule;
    let router = Router::new();
    let router = module
        .register_rest(&ctx, router, &api_ingress)
        .expect("Failed to register routes");

    let _final_router = api_ingress
        .rest_finalize(&ctx, router)
        .expect("Failed to finalize router");

    // Verify default body limit is applied (16MB)
    let config = api_ingress.get_config();
    assert_eq!(
        config.defaults.body_limit_bytes,
        16 * 1024 * 1024,
        "Default body limit should be 16MB"
    );
}

#[tokio::test]
async fn test_openapi_includes_413_response() {
    let api_ingress = api_ingress::ApiIngress::default();
    let ctx = create_test_module_ctx_with_body_limit(1024);
    api_ingress.init(&ctx).await.expect("Failed to init");

    let module = BodyLimitTestModule;
    let router = Router::new();
    let _router = module
        .register_rest(&ctx, router, &api_ingress)
        .expect("Failed to register routes");

    let openapi = api_ingress
        .build_openapi()
        .expect("Failed to build OpenAPI");
    let json = serde_json::to_value(&openapi).expect("Failed to serialize");

    // Verify 413 response is documented
    let upload_op = json
        .pointer("/paths/~1upload/post")
        .expect("Upload endpoint not found");
    let responses = upload_op.get("responses").expect("Responses not found");
    let response_413 = responses.get("413");
    assert!(
        response_413.is_some(),
        "413 Payload Too Large response should be documented"
    );
}
