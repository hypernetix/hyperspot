use std::sync::Arc;

use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::{DbModule, Module, ModuleCtx, RestfulModule, SseBroadcaster, TracedClient};
use sea_orm_migration::MigratorTrait;
use tracing::{debug, info};
use url::Url;

// Import the client trait from SDK
#[allow(unused_imports)]
use user_info_sdk::UsersInfoClient;

use crate::api::rest::dto::UserEvent;
use crate::api::rest::routes;
use crate::api::rest::sse_adapter::SseUserEventPublisher;
use crate::config::UsersInfoConfig;
use crate::domain::events::UserDomainEvent;
use crate::domain::ports::{AuditPort, EventPublisher};
use crate::domain::service::{AppServices, ServiceConfig};
use crate::infra::audit::HttpAuditClient;
use crate::infra::local_client::client::UsersInfoLocalClient;
use crate::infra::storage::{OrmAddressesRepository, OrmCitiesRepository, OrmUsersRepository};

/// Type alias for the concrete `AppServices` type used with ORM repositories.
/// This lives in the composition root (module.rs) to avoid infra dependencies in domain.
/// May be converted to `AppState` if we need additional fields like metrics, config and etc
pub(crate) type ConcreteAppServices =
    AppServices<OrmUsersRepository, OrmCitiesRepository, OrmAddressesRepository>;

/// Main module struct with DDD-light layout and proper `ClientHub` integration
#[modkit::module(
    name = "users_info",
    capabilities = [db, rest]
)]
pub struct UsersInfo {
    // Keep the domain service behind ArcSwap for cheap read-mostly access.
    service: arc_swap::ArcSwapOption<ConcreteAppServices>,
    // SSE broadcaster for user events
    sse: SseBroadcaster<UserEvent>,
}

impl Default for UsersInfo {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
            sse: SseBroadcaster::new(1024),
        }
    }
}

impl Clone for UsersInfo {
    fn clone(&self) -> Self {
        Self {
            service: arc_swap::ArcSwapOption::new(self.service.load().as_ref().map(Clone::clone)),
            sse: self.sse.clone(),
        }
    }
}

#[async_trait]
impl Module for UsersInfo {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing users_info module");

        // Load module configuration using new API
        let cfg: UsersInfoConfig = ctx.config()?;
        debug!(
            "Loaded users_info config: default_page_size={}, max_page_size={}",
            cfg.default_page_size, cfg.max_page_size
        );

        // Acquire DB (SeaORM connection handle with security enforcement)
        let db = ctx.db_required()?;
        let sec_conn = db.sea_secure(); // SecureConn - enforces access control on all queries

        // Create event publisher adapter that bridges domain events to SSE
        let publisher: Arc<dyn EventPublisher<UserDomainEvent>> =
            Arc::new(SseUserEventPublisher::new(self.sse.clone()));

        // Build traced HTTP client
        let traced_client = TracedClient::default();

        // Parse audit service URLs from config
        let audit_base = Url::parse(&cfg.audit_base_url)
            .map_err(|e| anyhow::anyhow!("invalid audit_base_url: {e}"))?;
        let notify_base = Url::parse(&cfg.notifications_base_url)
            .map_err(|e| anyhow::anyhow!("invalid notifications_base_url: {e}"))?;

        // Create audit adapter
        let audit_adapter: Arc<dyn AuditPort> =
            Arc::new(HttpAuditClient::new(traced_client, audit_base, notify_base));

        let service_config = ServiceConfig {
            max_display_name_length: 100,
            default_page_size: cfg.default_page_size,
            max_page_size: cfg.max_page_size,
        };

        // Create repository implementations
        let limit_cfg = service_config.limit_cfg();
        let users_repo = OrmUsersRepository::new(limit_cfg);
        let cities_repo = OrmCitiesRepository::new(limit_cfg);
        let addresses_repo = OrmAddressesRepository::new(limit_cfg);

        // Create services with repository dependencies
        let services = Arc::new(AppServices::new(
            users_repo,
            cities_repo,
            addresses_repo,
            sec_conn,
            publisher,
            audit_adapter,
            service_config,
        ));

        // Store service for REST and internal usage
        self.service.store(Some(services.clone()));

        // Create local client adapter that implements object-safe UsersInfoClient
        let local = UsersInfoLocalClient::new(services);

        // Register under the SDK trait for transport-agnostic consumption
        ctx.client_hub()
            .register::<dyn UsersInfoClient>(Arc::new(local));
        info!("UsersInfo client registered into ClientHub as dyn UsersInfoClient");
        Ok(())
    }
}

#[async_trait]
impl DbModule for UsersInfo {
    async fn migrate(&self, db: &modkit_db::DbHandle) -> anyhow::Result<()> {
        info!("Running users_info database migrations");
        let conn = db.sea();
        crate::infra::storage::migrations::Migrator::up(&conn, None).await?;
        info!("Users database migrations completed successfully");
        Ok(())
    }
}

impl RestfulModule for UsersInfo {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        info!("Registering users_info REST routes");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        let router = routes::register_routes(router, openapi, service);

        // Register SSE route with per-route Extension
        let router = routes::register_users_sse_route(router, openapi, self.sse.clone());

        info!("Users REST routes registered successfully");
        Ok(router)
    }
}
