use crate::config::AnalyticsConfig;
use crate::domain::gts_core::{GtsCoreRouter, RoutingTable};
use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::{DbModule, Module, ModuleCtx, RestfulModule};
use std::sync::Arc;

#[modkit::module(name = "analytics", capabilities = [rest, db])]
#[derive(Clone)]
pub struct AnalyticsModule {
    config: AnalyticsConfig,
    gts_router: Arc<GtsCoreRouter>,
}

impl Default for AnalyticsModule {
    fn default() -> Self {
        let table = RoutingTable::new();
        Self {
            config: AnalyticsConfig::default(),
            gts_router: Arc::new(GtsCoreRouter::new(table)),
        }
    }
}

#[async_trait]
impl Module for AnalyticsModule {
    async fn init(&self, _ctx: &ModuleCtx) -> anyhow::Result<()> {
        let _ = &self.config;
        tracing::info!(module = "analytics", "Analytics module initialized");
        Ok(())
    }
}

#[async_trait]
impl DbModule for AnalyticsModule {
    async fn migrate(&self, _db: &modkit_db::DbHandle) -> anyhow::Result<()> {
        // Migrations will be added by business features
        Ok(())
    }
}

/// REST API integration via ModKit's RestfulModule pattern.
///
/// This implementation registers GTS Core routes using OperationBuilder for type-safe
/// OpenAPI generation. All endpoints automatically receive:
/// - JWT validation via api_gateway
/// - SecurityCtx injection with tenant isolation
/// - Request tracing and correlation IDs
/// - RFC 7807 Problem Details error handling
impl RestfulModule for AnalyticsModule {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        // Register GTS Core routes with service instance
        let router =
            crate::api::rest::gts_core::register_routes(router, openapi, self.gts_router.clone());

        Ok(router)
    }
}
