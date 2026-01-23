#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use modkit_db::secure::SecureConn;
use modkit_security::SecurityContext;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use tenant_resolver_sdk::{
    TenantFilter, TenantResolverError, TenantResolverGatewayClient, TenantStatus,
};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::events::UserDomainEvent;
use crate::domain::ports::{AuditPort, EventPublisher};
use crate::domain::service::ServiceConfig;
use crate::infra::storage::{OrmAddressesRepository, OrmCitiesRepository, OrmUsersRepository};
use crate::module::ConcreteAppServices;

#[must_use]
pub fn ctx_allow_tenants(tenants: &[Uuid]) -> SecurityContext {
    let tenant_id = tenants.first().copied().unwrap_or_else(Uuid::new_v4);
    SecurityContext::builder()
        .tenant_id(tenant_id)
        .subject_id(Uuid::new_v4())
        .build()
}

#[must_use]
pub fn ctx_deny_all() -> SecurityContext {
    SecurityContext::anonymous()
}

pub async fn inmem_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory database");

    crate::infra::storage::migrations::Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    db
}

pub async fn seed_user(
    db: &DatabaseConnection,
    id: Uuid,
    tenant_id: Uuid,
    email: &str,
    display_name: &str,
) {
    use crate::infra::storage::entity::user::ActiveModel;
    use sea_orm::{ActiveModelTrait, Set};

    let now = OffsetDateTime::now_utc();
    let user = ActiveModel {
        id: Set(id),
        tenant_id: Set(tenant_id),
        email: Set(email.to_owned()),
        display_name: Set(display_name.to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
    };

    user.insert(db).await.expect("Failed to seed user");
}

pub struct MockEventPublisher;
pub struct MockAuditPort;

impl EventPublisher<UserDomainEvent> for MockEventPublisher {
    fn publish(&self, _event: &UserDomainEvent) {}
}

#[async_trait::async_trait]
impl AuditPort for MockAuditPort {
    async fn get_user_access(&self, _id: Uuid) -> Result<(), crate::domain::error::DomainError> {
        Ok(())
    }

    async fn notify_user_created(&self) -> Result<(), crate::domain::error::DomainError> {
        Ok(())
    }
}

/// Mock tenant resolver that returns the context's tenant as an accessible tenant.
pub struct MockTenantResolver;

#[async_trait::async_trait]
impl TenantResolverGatewayClient for MockTenantResolver {
    async fn get_tenant(
        &self,
        _ctx: &SecurityContext,
        id: tenant_resolver_sdk::TenantId,
    ) -> Result<tenant_resolver_sdk::TenantInfo, TenantResolverError> {
        Ok(tenant_resolver_sdk::TenantInfo {
            id,
            name: format!("Tenant {id}"),
            status: TenantStatus::Active,
            tenant_type: None,
        })
    }

    async fn can_access(
        &self,
        ctx: &SecurityContext,
        target: tenant_resolver_sdk::TenantId,
        _options: Option<&tenant_resolver_sdk::AccessOptions>,
    ) -> Result<bool, TenantResolverError> {
        // Allow access if target matches context tenant
        Ok(ctx.tenant_id() == target)
    }

    async fn get_accessible_tenants(
        &self,
        ctx: &SecurityContext,
        _filter: Option<&TenantFilter>,
        _options: Option<&tenant_resolver_sdk::AccessOptions>,
    ) -> Result<Vec<tenant_resolver_sdk::TenantInfo>, TenantResolverError> {
        // Return the context's tenant as the only accessible tenant
        let tenant_id = ctx.tenant_id();
        if tenant_id == Uuid::default() {
            // Anonymous context - no accessible tenants
            return Ok(vec![]);
        }
        Ok(vec![tenant_resolver_sdk::TenantInfo {
            id: tenant_id,
            name: format!("Tenant {tenant_id}"),
            status: TenantStatus::Active,
            tenant_type: None,
        }])
    }
}

pub fn build_services(sec: SecureConn, config: ServiceConfig) -> Arc<ConcreteAppServices> {
    let limit_cfg = config.limit_cfg();

    let users_repo = OrmUsersRepository::new(limit_cfg);
    let cities_repo = OrmCitiesRepository::new(limit_cfg);
    let addresses_repo = OrmAddressesRepository::new(limit_cfg);

    Arc::new(ConcreteAppServices::new(
        users_repo,
        cities_repo,
        addresses_repo,
        sec,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        Arc::new(MockTenantResolver),
        config,
    ))
}
