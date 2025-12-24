#![allow(clippy::unwrap_used, clippy::expect_used)]

use anyhow::Result;
use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::IntoResponse,
    Router,
};
use modkit::{
    api::operation_builder::{AuthReqAction, AuthReqResource, LicenseFeature},
    api::OperationBuilder,
    config::ConfigProvider,
    context::ModuleCtx,
    contracts::{OpenApiRegistry, RestHostModule, RestfulModule},
    ClientHub, Module,
};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

struct TestConfigProvider {
    config: serde_json::Value,
}

impl ConfigProvider for TestConfigProvider {
    fn get_module_config(&self, module: &str) -> Option<&serde_json::Value> {
        self.config.get(module)
    }
}

fn create_api_ingress_ctx(config: serde_json::Value) -> ModuleCtx {
    ModuleCtx::new(
        "api_ingress",
        Uuid::new_v4(),
        Arc::new(TestConfigProvider { config }),
        Arc::new(ClientHub::new()),
        tokio_util::sync::CancellationToken::new(),
        None,
    )
}

fn create_test_module_ctx() -> ModuleCtx {
    ModuleCtx::new(
        "test_module",
        Uuid::new_v4(),
        Arc::new(TestConfigProvider { config: json!({}) }),
        Arc::new(ClientHub::new()),
        tokio_util::sync::CancellationToken::new(),
        None,
    )
}

async fn ok_handler() -> impl IntoResponse {
    StatusCode::OK
}

pub struct TestLicenseModule;

#[async_trait]
impl Module for TestLicenseModule {
    async fn init(&self, _ctx: &ModuleCtx) -> Result<()> {
        Ok(())
    }
}

enum TestResource {
    Test,
}

impl AsRef<str> for TestResource {
    fn as_ref(&self) -> &'static str {
        match self {
            TestResource::Test => "test",
        }
    }
}

impl AuthReqResource for TestResource {}

enum TestAction {
    Read,
}

impl AsRef<str> for TestAction {
    fn as_ref(&self) -> &'static str {
        match self {
            TestAction::Read => "read",
        }
    }
}

impl AuthReqAction for TestAction {}

struct NonBaseFeature;

impl AsRef<str> for NonBaseFeature {
    fn as_ref(&self) -> &'static str {
        "some_other_feature"
    }
}

impl LicenseFeature for NonBaseFeature {}

struct BaseFeature;

impl AsRef<str> for BaseFeature {
    fn as_ref(&self) -> &'static str {
        "gts.x.core.lic.feat.v1~x.core.global.base.v1"
    }
}

impl LicenseFeature for BaseFeature {}

impl RestfulModule for TestLicenseModule {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: Router,
        openapi: &dyn OpenApiRegistry,
    ) -> Result<Router> {
        let feature = NonBaseFeature;

        let router = OperationBuilder::get("/tests/v1/license/bad")
            .operation_id("test.license.bad")
            .require_auth(&TestResource::Test, &TestAction::Read)
            .require_license_features([&feature])
            .handler(ok_handler)
            .json_response(http::StatusCode::OK, "OK")
            .register(router, openapi);

        let base_feature = BaseFeature;

        let router = OperationBuilder::get("/tests/v1/license/good")
            .operation_id("test.license.good")
            .require_auth(&TestResource::Test, &TestAction::Read)
            .require_license_features([&base_feature])
            .handler(ok_handler)
            .json_response(http::StatusCode::OK, "OK")
            .register(router, openapi);

        let router = OperationBuilder::get("/tests/v1/license/none")
            .operation_id("test.license.none")
            .require_auth(&TestResource::Test, &TestAction::Read)
            .require_license_features::<BaseFeature>([])
            .handler(ok_handler)
            .json_response(http::StatusCode::OK, "OK")
            .register(router, openapi);

        Ok(router)
    }
}

#[tokio::test]
async fn rejects_non_base_feature_requirement() {
    let config = json!({
        "api_ingress": {
            "config": {
                "bind_addr": "0.0.0.0:8080",
                "enable_docs": false,
                "cors_enabled": false,
                "auth_disabled": true
            }
        }
    });

    let api_ctx = create_api_ingress_ctx(config);
    let test_ctx = create_test_module_ctx();

    let api_ingress = api_ingress::ApiIngress::default();
    api_ingress.init(&api_ctx).await.expect("Failed to init");

    let router = Router::new();
    let test_module = TestLicenseModule;
    let router = test_module
        .register_rest(&test_ctx, router, &api_ingress)
        .expect("Failed to register routes");

    let router = api_ingress
        .rest_finalize(&api_ctx, router)
        .expect("Failed to finalize");

    let response = router
        .oneshot(
            Request::builder()
                .uri("/tests/v1/license/bad")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn allows_base_feature_requirement() {
    let config = json!({
        "api_ingress": {
            "config": {
                "bind_addr": "0.0.0.0:8080",
                "enable_docs": false,
                "cors_enabled": false,
                "auth_disabled": true
            }
        }
    });

    let api_ctx = create_api_ingress_ctx(config);
    let test_ctx = create_test_module_ctx();

    let api_ingress = api_ingress::ApiIngress::default();
    api_ingress.init(&api_ctx).await.expect("Failed to init");

    let router = Router::new();
    let test_module = TestLicenseModule;
    let router = test_module
        .register_rest(&test_ctx, router, &api_ingress)
        .expect("Failed to register routes");

    let router = api_ingress
        .rest_finalize(&api_ctx, router)
        .expect("Failed to finalize");

    let response = router
        .oneshot(
            Request::builder()
                .uri("/tests/v1/license/good")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn allows_no_license_requirement() {
    let config = json!({
        "api_ingress": {
            "config": {
                "bind_addr": "0.0.0.0:8080",
                "enable_docs": false,
                "cors_enabled": false,
                "auth_disabled": true
            }
        }
    });

    let api_ctx = create_api_ingress_ctx(config);
    let test_ctx = create_test_module_ctx();

    let api_ingress = api_ingress::ApiIngress::default();
    api_ingress.init(&api_ctx).await.expect("Failed to init");

    let router = Router::new();
    let test_module = TestLicenseModule;
    let router = test_module
        .register_rest(&test_ctx, router, &api_ingress)
        .expect("Failed to register routes");

    let router = api_ingress
        .rest_finalize(&api_ctx, router)
        .expect("Failed to finalize");

    let response = router
        .oneshot(
            Request::builder()
                .uri("/tests/v1/license/none")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);
}
