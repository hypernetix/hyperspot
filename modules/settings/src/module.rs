use crate::api::rest::routes;
use crate::contract::client::SettingsApi;
use crate::domain::service::{Service, ServiceConfig};
use async_trait::async_trait;
use axum::Router;
use modkit::{DbModule, Module, ModuleCtx, OpenApiRegistry, RestfulModule};
use sea_orm_migration::MigratorTrait;
use std::any::Any;
use std::sync::Arc;
use tracing::{debug, info};

#[modkit::module(
    name = "settings",
    caps = [db, rest],
    client = crate::contract::client::SettingsApi
)]
#[derive(Default)]
pub struct Settings {
    service: arc_swap::ArcSwapOption<Service>,
}

#[async_trait]
impl Module for Settings {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing Settings module");

        // Load module configuration
        // todo: we don't have one

        // Get database connection
        let db = ctx.db().ok_or_else(|| anyhow::anyhow!("DB required"))?;
        let db_conn = db.seaorm();

        // Create domain service with configuration
        let service_config = ServiceConfig::default();

        let service = Service::new(db_conn.clone(), service_config);
        self.service.store(Some(Arc::new(service.clone())));

        // Create and register the local client implementation
        // TODO: no gateways yet.
        // let api: Arc<dyn SettingsApi> = Arc::new(UsersInfoLocalClient::new(Arc::new(service)));
        // expose_users_info_client(ctx, &api)?;
        //info!("Settings API exposed to ClientHub");

        info!("Settings module initialized");
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl DbModule for Settings {
    async fn migrate(&self, db: &db::DbHandle) -> anyhow::Result<()> {
        info!("Running Settings module database migrations");

        let conn = db.seaorm();
        crate::infra::storage::migrations::Migrator::up(conn, None).await?;

        info!("Settings module database migrations completed successfully");
        Ok(())
    }
}

impl RestfulModule for Settings {
    fn register_rest(
        &self,
        ctx: &ModuleCtx,
        router: Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<Router> {
        info!("Registering settings REST routes");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        let router = routes::register_routes(router, openapi, service)?;

        info!("Settings REST routes registered successfully");
        Ok(router)
    }
}
