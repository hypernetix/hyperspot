use std::sync::Arc;

use modkit_macros::domain_model;
use tracing::{debug, info, instrument};

use crate::domain::error::DomainError;
use crate::domain::repos::CitiesRepository;
use crate::domain::service::DbProvider;
use modkit_odata::{ODataQuery, Page};
use modkit_security::{PolicyEngineRef, SecurityContext};
use tenant_resolver_sdk::TenantResolverGatewayClient;
use time::OffsetDateTime;
use user_info_sdk::{City, CityPatch, NewCity};
use uuid::Uuid;

/// Cities service.
///
/// # Design
///
/// Services acquire database connections internally via `DBProvider`. Handlers
/// call service methods with business parameters only - no DB objects.
///
/// This design:
/// - Keeps handlers clean and focused on HTTP concerns
/// - Centralizes DB error mapping in the domain layer
/// - Maintains transaction safety via the task-local guard
#[domain_model]
pub struct CitiesService<R: CitiesRepository> {
    db: Arc<DbProvider>,
    policy_engine: PolicyEngineRef,
    repo: Arc<R>,
    resolver: Arc<dyn TenantResolverGatewayClient>,
}

impl<R: CitiesRepository> CitiesService<R> {
    pub fn new(
        db: Arc<DbProvider>,
        repo: Arc<R>,
        policy_engine: PolicyEngineRef,
        resolver: Arc<dyn TenantResolverGatewayClient>,
    ) -> Self {
        Self {
            db,
            policy_engine,
            repo,
            resolver,
        }
    }
}

// Business logic methods
impl<R: CitiesRepository> CitiesService<R> {
    #[instrument(skip(self, ctx), fields(city_id = %id))]
    pub async fn get_city(&self, ctx: &SecurityContext, id: Uuid) -> Result<City, DomainError> {
        debug!("Getting city by id");

        let conn = self.db.conn().map_err(DomainError::from)?;

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let found = self.repo.get(&conn, &scope, id).await?;

        found.ok_or_else(|| DomainError::not_found("City", id))
    }

    #[instrument(skip(self, ctx, query))]
    pub async fn list_cities_page(
        &self,
        ctx: &SecurityContext,
        query: &ODataQuery,
    ) -> Result<Page<City>, DomainError> {
        debug!("Listing cities with cursor pagination");

        let conn = self.db.conn().map_err(DomainError::from)?;

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let page = self.repo.list_page(&conn, &scope, query).await?;

        debug!("Successfully listed {} cities in page", page.items.len());
        Ok(page)
    }

    #[instrument(skip(self, ctx), fields(name = %new_city.name, country = %new_city.country))]
    pub async fn create_city(
        &self,
        ctx: &SecurityContext,
        new_city: NewCity,
    ) -> Result<City, DomainError> {
        info!("Creating new city");

        let conn = self.db.conn().map_err(DomainError::from)?;

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let now = OffsetDateTime::now_utc();
        let id = new_city.id.unwrap_or_else(Uuid::now_v7);

        let city = City {
            id,
            tenant_id: new_city.tenant_id,
            name: new_city.name,
            country: new_city.country,
            created_at: now,
            updated_at: now,
        };

        let _ = self.repo.create(&conn, &scope, city.clone()).await?;

        info!("Successfully created city with id={}", city.id);
        Ok(city)
    }

    #[instrument(skip(self, ctx), fields(city_id = %id))]
    pub async fn update_city(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        patch: CityPatch,
    ) -> Result<City, DomainError> {
        info!("Updating city");

        let conn = self.db.conn().map_err(DomainError::from)?;

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let found = self.repo.get(&conn, &scope, id).await?;

        let mut current: City = found.ok_or_else(|| DomainError::not_found("City", id))?;

        if let Some(name) = patch.name {
            current.name = name;
        }
        if let Some(country) = patch.country {
            current.country = country;
        }
        current.updated_at = OffsetDateTime::now_utc();

        let _ = self.repo.update(&conn, &scope, current.clone()).await?;

        info!("Successfully updated city");
        Ok(current)
    }

    #[instrument(skip(self, ctx), fields(city_id = %id))]
    pub async fn delete_city(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), DomainError> {
        info!("Deleting city");

        let conn = self.db.conn().map_err(DomainError::from)?;

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let deleted = self.repo.delete(&conn, &scope, id).await?;

        if !deleted {
            return Err(DomainError::not_found("City", id));
        }

        info!("Successfully deleted city");
        Ok(())
    }
}
