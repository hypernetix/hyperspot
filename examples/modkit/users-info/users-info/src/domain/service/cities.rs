use std::sync::Arc;

use modkit_macros::domain_model;
use tracing::{debug, info, instrument};

use crate::domain::error::DomainError;
use crate::domain::repos::CitiesRepository;
use crate::domain::service::DbProvider;
use authz_resolver_sdk::PolicyEnforcer;
use authz_resolver_sdk::pep::AccessRequest;

use super::{actions, resources};
use modkit_odata::{ODataQuery, Page};
use modkit_security::{AccessScope, SecurityContext, pep_properties};
use time::OffsetDateTime;
use users_info_sdk::{City, CityPatch, NewCity};
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
    repo: Arc<R>,
    policy_enforcer: PolicyEnforcer,
}

impl<R: CitiesRepository> CitiesService<R> {
    pub fn new(db: Arc<DbProvider>, repo: Arc<R>, policy_enforcer: PolicyEnforcer) -> Self {
        Self {
            db,
            repo,
            policy_enforcer,
        }
    }
}

// Business logic methods
impl<R: CitiesRepository> CitiesService<R> {
    #[instrument(skip(self, ctx), fields(city_id = %id))]
    pub async fn get_city(&self, ctx: &SecurityContext, id: Uuid) -> Result<City, DomainError> {
        debug!("Getting city by id");

        let conn = self.db.conn().map_err(DomainError::from)?;

        // Prefetch: load city to extract owner_tenant_id for PDP.
        // PDP returns a narrow `eq` constraint instead of expanding the subtree.
        let prefetch_scope = AccessScope::allow_all();
        let city = self
            .repo
            .get(&conn, &prefetch_scope, id)
            .await?
            .ok_or_else(|| DomainError::not_found("City", id))?;

        let scope = self
            .policy_enforcer
            .access_scope_with(
                ctx,
                &resources::CITY,
                actions::GET,
                Some(id),
                &AccessRequest::new()
                    .resource_property(pep_properties::OWNER_TENANT_ID, city.tenant_id)
                    .require_constraints(false),
            )
            .await?;

        // Unconstrained → PDP said "yes" without row-level filters; return prefetch.
        // Constrained  → scoped re-read validates against PDP constraints.
        if scope.is_unconstrained() {
            Ok(city)
        } else {
            self.repo
                .get(&conn, &scope, id)
                .await?
                .ok_or_else(|| DomainError::not_found("City", id))
        }
    }

    #[instrument(skip(self, ctx, query))]
    pub async fn list_cities_page(
        &self,
        ctx: &SecurityContext,
        query: &ODataQuery,
    ) -> Result<Page<City>, DomainError> {
        debug!("Listing cities with cursor pagination");

        let conn = self.db.conn().map_err(DomainError::from)?;

        let scope = self
            .policy_enforcer
            .access_scope(ctx, &resources::CITY, actions::LIST, None)
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

        let tenant_id = new_city.tenant_id;

        let scope = self
            .policy_enforcer
            .access_scope_with(
                ctx,
                &resources::CITY,
                actions::CREATE,
                None,
                &AccessRequest::new().resource_property(pep_properties::OWNER_TENANT_ID, tenant_id),
            )
            .await?;

        let now = OffsetDateTime::now_utc();
        let id = new_city.id.unwrap_or_else(Uuid::now_v7);

        let city = City {
            id,
            tenant_id,
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

        // Prefetch: load city to extract owner_tenant_id for PDP.
        // Narrow scope + WHERE constraint provides TOCTOU protection.
        let prefetch_scope = AccessScope::allow_all();
        let mut current = self
            .repo
            .get(&conn, &prefetch_scope, id)
            .await?
            .ok_or_else(|| DomainError::not_found("City", id))?;

        let scope = self
            .policy_enforcer
            .access_scope_with(
                ctx,
                &resources::CITY,
                actions::UPDATE,
                Some(id),
                &AccessRequest::new()
                    .resource_property(pep_properties::OWNER_TENANT_ID, current.tenant_id),
            )
            .await?;

        if let Some(name) = patch.name {
            current.name = name;
        }
        if let Some(country) = patch.country {
            current.country = country;
        }
        current.updated_at = OffsetDateTime::now_utc();

        // repo.update applies scope constraints via WHERE clause (TOCTOU-safe).
        let _ = self.repo.update(&conn, &scope, current.clone()).await?;

        info!("Successfully updated city");
        Ok(current)
    }

    #[instrument(skip(self, ctx), fields(city_id = %id))]
    pub async fn delete_city(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), DomainError> {
        info!("Deleting city");

        let conn = self.db.conn().map_err(DomainError::from)?;

        // Prefetch: load city to extract owner_tenant_id for PDP.
        // Narrow scope + WHERE constraint provides TOCTOU protection.
        let prefetch_scope = AccessScope::allow_all();
        let prefetched = self
            .repo
            .get(&conn, &prefetch_scope, id)
            .await?
            .ok_or_else(|| DomainError::not_found("City", id))?;

        let scope = self
            .policy_enforcer
            .access_scope_with(
                ctx,
                &resources::CITY,
                actions::DELETE,
                Some(id),
                &AccessRequest::new()
                    .resource_property(pep_properties::OWNER_TENANT_ID, prefetched.tenant_id),
            )
            .await?;

        let deleted = self.repo.delete(&conn, &scope, id).await?;

        if !deleted {
            return Err(DomainError::not_found("City", id));
        }

        info!("Successfully deleted city");
        Ok(())
    }
}
