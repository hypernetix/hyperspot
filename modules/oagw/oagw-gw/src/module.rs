//! OAGW Gateway module definition.

use std::sync::Arc;

use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::{DbModule, Module, ModuleCtx, RestfulModule};
use modkit_security::SecurityCtx;
use oagw_sdk::{get_oagw_base_schemas, get_oagw_well_known_instances, OagwApi, OagwPluginSpecV1};
use sea_orm_migration::MigratorTrait;
use tracing::info;
use types_registry_sdk::TypesRegistryApi;

use crate::api::rest::routes;
use crate::config::OagwConfig;
use crate::domain::ports::StubSecretResolver;
use crate::domain::repo::{LinkRepository, RouteRepository};
use crate::domain::service::{Service, ServiceConfig};
use crate::infra::storage::link_repo::SeaOrmLinkRepository;
use crate::infra::storage::route_repo::SeaOrmRouteRepository;
use crate::local_client::OagwLocalClient;

/// OAGW Gateway module.
///
/// This module provides:
/// - Route and link configuration management
/// - Plugin-based protocol and authentication handling
/// - REST API for outbound API invocation
#[modkit::module(
    name = "oagw_gw",
    deps = ["types_registry", "oagw_default_plugin"],
    capabilities = [db, rest]
)]
pub struct OagwGateway {
    service: arc_swap::ArcSwapOption<Service>,
}

impl Default for OagwGateway {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
        }
    }
}

#[async_trait]
impl Module for OagwGateway {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing OAGW gateway module");

        // Load module configuration
        let cfg: OagwConfig = ctx.config()?;

        // Acquire DB with SecureConn for security-aware queries
        let db = ctx.db_required()?;
        let sec_conn = db.sea_secure();

        // === SCHEMA REGISTRATION ===
        // Gateway registers the plugin SCHEMA; plugins register their INSTANCES
        let registry = ctx.client_hub().get::<dyn TypesRegistryApi>()?;

        // Register OAGW base schemas (proto, stream_proto, auth_type, strategy)
        let base_schemas = get_oagw_base_schemas()?;
        let _ = registry
            .register(&SecurityCtx::root_ctx(), base_schemas)
            .await?;
        info!("Registered OAGW base schemas: proto, stream_proto, auth_type, strategy");

        // Register OAGW plugin schema
        let schema_str = OagwPluginSpecV1::gts_schema_with_refs_as_string();
        let schema_json: serde_json::Value = serde_json::from_str(&schema_str)?;
        let _ = registry
            .register(&SecurityCtx::root_ctx(), vec![schema_json])
            .await?;
        info!(
            schema_id = %OagwPluginSpecV1::gts_schema_id(),
            "Registered OAGW plugin schema in types-registry"
        );

        // Register well-known instances for protocols, auth types, and strategies
        let well_known_instances = get_oagw_well_known_instances();
        let _ = registry
            .register(&SecurityCtx::root_ctx(), well_known_instances)
            .await?;
        info!(
            "Registered OAGW well-known instances (protocols, stream_protocols, auth_types, strategies)"
        );

        // Wire repositories with SecureConn
        let route_repo: Arc<dyn RouteRepository> =
            Arc::new(SeaOrmRouteRepository::new(sec_conn.clone()));
        let link_repo: Arc<dyn LinkRepository> = Arc::new(SeaOrmLinkRepository::new(sec_conn));

        // Wire secret resolver
        // TODO(v2): Replace with actual cred_store integration
        let secret_resolver = Arc::new(StubSecretResolver);

        // Create service with dependencies
        let service_config = ServiceConfig::from(&cfg);
        let domain_service = Arc::new(Service::new(
            route_repo,
            link_repo,
            secret_resolver,
            ctx.client_hub(),
            service_config,
        ));

        // Store service for REST handlers
        self.service.store(Some(domain_service.clone()));

        // === EXPLICIT CLIENT REGISTRATION ===
        // Create local client adapter that implements the SDK API trait
        let local_client = OagwLocalClient::new(domain_service);
        let api: Arc<dyn OagwApi> = Arc::new(local_client);

        // Register directly in ClientHub
        ctx.client_hub().register::<dyn OagwApi>(api);
        info!("OAGW API registered in ClientHub via local adapter");

        Ok(())
    }
}

#[async_trait]
impl DbModule for OagwGateway {
    async fn migrate(&self, db: &modkit_db::DbHandle) -> anyhow::Result<()> {
        info!("Running OAGW database migrations");
        let conn = db.sea();
        crate::infra::storage::migrations::Migrator::up(&conn, None).await?;
        Ok(())
    }
}

impl RestfulModule for OagwGateway {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        info!("Registering OAGW REST routes");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("OAGW service not initialized"))?
            .clone();

        let router = routes::register_routes(router, openapi, service)?;

        Ok(router)
    }
}
