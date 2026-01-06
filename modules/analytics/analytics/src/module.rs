use crate::config::AnalyticsConfig;
use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::{DbModule, Module, ModuleCtx, RestfulModule};

#[modkit::module(
    name = "analytics",
    capabilities = [db, rest]
)]
#[derive(Clone, Default)]
pub struct AnalyticsModule {
    config: AnalyticsConfig,
}

#[async_trait]
impl Module for AnalyticsModule {
    async fn init(&self, _ctx: &ModuleCtx) -> anyhow::Result<()> {
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

impl RestfulModule for AnalyticsModule {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        _openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        // REST routes will be added by business features
        Ok(router)
    }
}
