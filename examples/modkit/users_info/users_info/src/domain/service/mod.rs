//! Domain service layer - business logic and rules.
//!
//! ## Architecture
//!
//! This module implements the domain service pattern with per-resource submodules:
//! - `users` - User CRUD and business rules (email/display name validation)
//! - `cities` - City CRUD operations
//! - `addresses` - Address management (1-to-1 with users)
//!
//! ## Layering Rules
//!
//! The domain layer:
//! - **MAY** import: `user_info_sdk` (contract types), `infra` (data access), `modkit` libs
//! - **MUST NOT** import: `api::*` (one-way dependency: API → Domain)
//! - **Uses**: SDK contract types (`User`, `NewUser`, etc.) as primary domain models
//! - **Uses**: `OData` filter schemas from `user_info_sdk::odata` (not defined here)
//!
//! ## `OData` Integration
//!
//! The service uses type-safe `OData` filtering via SDK filter enums:
//! - Filter schemas: `user_info_sdk::odata::{UserFilterField, CityFilterField, ...}`
//! - Pagination: `modkit_db::odata::paginate_odata` with filter type parameter
//! - Mapping: Infrastructure layer (`odata_mapper`) maps filters to `SeaORM` columns
//!
//! ## Security
//!
//! All operations use `DBRunner` for tenant isolation and RBAC:
//! - Queries filtered by security context automatically
//! - Operations checked against policy engine
//! - Audit events published for compliance
//!
//! ## Connection Management
//!
//! Services acquire database connections internally via `DBProvider`. Handlers
//! do NOT touch database objects - they simply call service methods with
//! business parameters only.
//!
//! This design:
//! - Keeps handlers clean and focused on HTTP concerns
//! - Maintains transaction safety via the task-local guard

use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::events::UserDomainEvent;
use crate::domain::ports::{AuditPort, EventPublisher};
use crate::domain::repos::{AddressesRepository, CitiesRepository, UsersRepository};
use modkit_db::DBProvider;
use modkit_db::odata::LimitCfg;
use modkit_security::{PolicyEngineRef, SecurityContext};
use tenant_resolver_sdk::{TenantFilter, TenantResolverGatewayClient, TenantStatus};
use uuid::Uuid;

mod addresses;
mod cities;
mod users;

pub(crate) use addresses::AddressesService;
pub(crate) use cities::CitiesService;
pub(crate) use users::UsersService;

pub(crate) type DbProvider = DBProvider<modkit_db::DbError>;

/// Resolve accessible tenants for the current security context.
/// Returns the context's tenant and all its active descendants.
pub(crate) async fn resolve_accessible_tenants(
    resolver: &dyn TenantResolverGatewayClient,
    ctx: &SecurityContext,
) -> Result<Vec<Uuid>, DomainError> {
    let tenant_id = ctx.tenant_id();
    if tenant_id == Uuid::nil() {
        // Anonymous context - no accessible tenants
        return Ok(vec![]);
    }

    // Filter for active descendants only
    let filter = TenantFilter {
        status: vec![TenantStatus::Active],
    };

    // Get tenant and all active descendants (max_depth=None means unlimited)
    let response = resolver
        .get_descendants(ctx, tenant_id, Some(&filter), None, None)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to get descendants");
            DomainError::InternalError
        })?;

    // Check if the starting tenant is active (filter doesn't apply to it)
    if response.tenant.status != TenantStatus::Active {
        return Ok(vec![]);
    }

    // Return tenant + all active descendants
    let mut result = Vec::with_capacity(1 + response.descendants.len());
    result.push(response.tenant.id);
    result.extend(response.descendants.iter().map(|t| t.id));
    Ok(result)
}

/// Configuration for the domain service
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub max_display_name_length: usize,
    pub default_page_size: u32,
    pub max_page_size: u32,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            max_display_name_length: 100,
            default_page_size: 50,
            max_page_size: 1000,
        }
    }
}

impl ServiceConfig {
    #[must_use]
    pub fn limit_cfg(&self) -> LimitCfg {
        LimitCfg {
            default: u64::from(self.default_page_size),
            max: u64::from(self.max_page_size),
        }
    }
}

// DI Container - aggregates all domain services
//
// # Database Access
//
// Services acquire database connections internally via `DBProvider`. Handlers
// do NOT touch database objects - they call service methods with business
// parameters only (e.g., `svc.users.get_user(&ctx, id)`).
//
// **Security**: A task-local guard prevents `Db::conn()` from being called
// inside transaction closures, eliminating the factory bypass vulnerability.
pub(crate) struct AppServices<UR, CR, AR>
where
    UR: UsersRepository + 'static,
    CR: CitiesRepository,
    AR: AddressesRepository,
{
    pub(crate) users: UsersService<UR, CR, AR>,
    pub(crate) cities: Arc<CitiesService<CR>>,
    pub(crate) addresses: Arc<AddressesService<AR, UR>>,
}

#[cfg(test)]
mod tests_security_scoping;

#[cfg(test)]
mod tests_entities;

#[cfg(test)]
mod tests_cursor_pagination;

impl<UR, CR, AR> AppServices<UR, CR, AR>
where
    UR: UsersRepository + 'static,
    CR: CitiesRepository,
    AR: AddressesRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        users_repo: UR,
        cities_repo: CR,
        addresses_repo: AR,
        db: Arc<DbProvider>,
        events: Arc<dyn EventPublisher<UserDomainEvent>>,
        audit: Arc<dyn AuditPort>,
        resolver: Arc<dyn TenantResolverGatewayClient>,
        config: ServiceConfig,
    ) -> Self {
        let policy_engine: PolicyEngineRef = Arc::new(modkit_security::NoopPolicyEngine);

        let users_repo = Arc::new(users_repo);
        let cities_repo = Arc::new(cities_repo);
        let addresses_repo = Arc::new(addresses_repo);

        let cities = Arc::new(CitiesService::new(
            Arc::clone(&db),
            Arc::clone(&cities_repo),
            policy_engine.clone(),
            resolver.clone(),
        ));
        let addresses = Arc::new(AddressesService::new(
            Arc::clone(&db),
            Arc::clone(&addresses_repo),
            Arc::clone(&users_repo),
            policy_engine.clone(),
            resolver.clone(),
        ));

        Self {
            users: UsersService::new(
                db,
                Arc::clone(&users_repo),
                events,
                audit,
                policy_engine.clone(),
                resolver,
                config,
                cities.clone(),
                addresses.clone(),
            ),
            cities,
            addresses,
        }
    }
}
