#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use modkit_db::migration_runner::run_migrations_for_testing;
use modkit_db::secure::DBRunner;
use modkit_db::secure::{AccessScope, secure_insert};
use modkit_db::{ConnectOpts, DBProvider, Db, DbError, connect_db};
use modkit_security::SecurityContext;
use sea_orm_migration::MigratorTrait;
use tenant_resolver_sdk::{
    GetAncestorsOptions, GetAncestorsResponse, GetDescendantsOptions, GetDescendantsResponse,
    GetTenantsOptions, IsAncestorOptions, TenantRef, TenantResolverError,
    TenantResolverGatewayClient, TenantStatus,
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

/// Create an in-memory database for testing.
pub async fn inmem_db() -> Db {
    let opts = ConnectOpts {
        max_conns: Some(1),
        min_conns: Some(1),
        ..Default::default()
    };
    let db = connect_db("sqlite::memory:", opts)
        .await
        .expect("Failed to connect to in-memory database");

    run_migrations_for_testing(
        &db,
        crate::infra::storage::migrations::Migrator::migrations(),
    )
    .await
    .map_err(|e| e.to_string())
    .expect("Failed to run migrations");

    db
}

pub async fn seed_user(
    db: &impl DBRunner,
    id: Uuid,
    tenant_id: Uuid,
    email: &str,
    display_name: &str,
) {
    use crate::infra::storage::entity::user::ActiveModel;
    use crate::infra::storage::entity::user::Entity as UserEntity;
    use sea_orm::Set;

    let now = OffsetDateTime::now_utc();
    let user = ActiveModel {
        id: Set(id),
        tenant_id: Set(tenant_id),
        email: Set(email.to_owned()),
        display_name: Set(display_name.to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let scope = AccessScope::tenants_only(vec![tenant_id]);
    let _ = secure_insert::<UserEntity>(user, &scope, db)
        .await
        .expect("Failed to seed user");
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
            parent_id: None,
            self_managed: false,
        })
    }

    async fn get_tenants(
        &self,
        ctx: &SecurityContext,
        ids: &[tenant_resolver_sdk::TenantId],
        _options: &GetTenantsOptions,
    ) -> Result<Vec<tenant_resolver_sdk::TenantInfo>, TenantResolverError> {
        // Return only tenants that match the context's tenant
        let tenant_id = ctx.tenant_id();
        Ok(ids
            .iter()
            .filter(|id| **id == tenant_id)
            .map(|id| tenant_resolver_sdk::TenantInfo {
                id: *id,
                name: format!("Tenant {id}"),
                status: TenantStatus::Active,
                tenant_type: None,
                parent_id: None,
                self_managed: false,
            })
            .collect())
    }

    async fn get_ancestors(
        &self,
        _ctx: &SecurityContext,
        id: tenant_resolver_sdk::TenantId,
        _options: &GetAncestorsOptions,
    ) -> Result<GetAncestorsResponse, TenantResolverError> {
        // Single-tenant mock: no ancestors
        Ok(GetAncestorsResponse {
            tenant: TenantRef {
                id,
                status: TenantStatus::Active,
                tenant_type: None,
                parent_id: None,
                self_managed: false,
            },
            ancestors: vec![],
        })
    }

    async fn get_descendants(
        &self,
        _ctx: &SecurityContext,
        id: tenant_resolver_sdk::TenantId,
        _options: &GetDescendantsOptions,
    ) -> Result<GetDescendantsResponse, TenantResolverError> {
        // Single-tenant mock: no descendants
        Ok(GetDescendantsResponse {
            tenant: TenantRef {
                id,
                status: TenantStatus::Active,
                tenant_type: None,
                parent_id: None,
                self_managed: false,
            },
            descendants: vec![],
        })
    }

    async fn is_ancestor(
        &self,
        _ctx: &SecurityContext,
        _ancestor_id: tenant_resolver_sdk::TenantId,
        _descendant_id: tenant_resolver_sdk::TenantId,
        _options: &IsAncestorOptions,
    ) -> Result<bool, TenantResolverError> {
        // Self is not an ancestor of self
        Ok(false)
    }
}

pub fn build_services(db: Db, config: ServiceConfig) -> Arc<ConcreteAppServices> {
    let limit_cfg = config.limit_cfg();

    let users_repo = OrmUsersRepository::new(limit_cfg);
    let cities_repo = OrmCitiesRepository::new(limit_cfg);
    let addresses_repo = OrmAddressesRepository::new(limit_cfg);

    let db: Arc<DBProvider<DbError>> = Arc::new(DBProvider::new(db));

    Arc::new(ConcreteAppServices::new(
        users_repo,
        cities_repo,
        addresses_repo,
        db,
        Arc::new(MockEventPublisher),
        Arc::new(MockAuditPort),
        Arc::new(MockTenantResolver),
        config,
    ))
}
