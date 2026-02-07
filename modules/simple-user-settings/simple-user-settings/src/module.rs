use std::sync::Arc;

use async_trait::async_trait;
use axum::Router;
use modkit::api::OpenApiRegistry;
use modkit::{Module, ModuleCtx};
use modkit_db::DBProvider;
use modkit_db::DbError;
use tracing::info;

use simple_user_settings_sdk::SimpleUserSettingsClient;

use crate::api::rest::routes;
use crate::config::SettingsConfig;
use crate::domain::local_client::LocalClient;
use crate::domain::service::{Service, ServiceConfig};
use crate::infra::storage::sea_orm_repo::SeaOrmSettingsRepository;

/// Type alias for the concrete service type with ORM repository.
type ConcreteService = Service<SeaOrmSettingsRepository>;

#[modkit::module(
    name = "simple-user-settings",
    capabilities = [rest, db]
)]
pub struct SettingsModule {
    service: arc_swap::ArcSwapOption<ConcreteService>,
}

impl Default for SettingsModule {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
        }
    }
}

impl Clone for SettingsModule {
    fn clone(&self) -> Self {
        Self {
            service: arc_swap::ArcSwapOption::new(self.service.load().as_ref().map(Clone::clone)),
        }
    }
}

impl modkit::contracts::DatabaseCapability for SettingsModule {
    fn migrations(&self) -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        use sea_orm_migration::MigratorTrait;
        info!("Providing settings database migrations");
        crate::infra::storage::migrations::Migrator::migrations()
    }
}

#[async_trait]
impl Module for SettingsModule {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing settings module");

        let cfg: SettingsConfig = ctx.config()?;

        let db: Arc<DBProvider<DbError>> = Arc::new(ctx.db_required()?);

        // Repository no longer stores connection - uses &impl DBRunner per-method
        let repo = Arc::new(SeaOrmSettingsRepository::new());

        let service_config = ServiceConfig {
            max_field_length: cfg.max_field_length,
        };
        let service = Arc::new(Service::new(db, repo, service_config));

        let local_client: Arc<dyn SimpleUserSettingsClient> =
            Arc::new(LocalClient::new(service.clone()));
        ctx.client_hub().register(local_client);

        self.service.store(Some(service));

        Ok(())
    }
}

#[async_trait]
impl modkit::contracts::RestApiCapability for SettingsModule {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<Router> {
        info!("Settings module: register_rest called");
        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        let router = routes::register_routes(router, openapi, service);
        info!("Settings module: REST routes registered successfully");
        Ok(router)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_module_default() {
        let module = SettingsModule::default();
        assert!(module.service.load().is_none());
    }

    #[test]
    fn test_settings_module_clone_empty_service() {
        let module = SettingsModule::default();
        let cloned = module.clone();
        assert!(cloned.service.load().is_none());
        assert!(module.service.load().is_none());
    }
}
