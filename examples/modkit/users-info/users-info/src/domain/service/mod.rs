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
//! - **MAY** import: `users_info_sdk` (contract types), `infra` (data access), `modkit` libs
//! - **MUST NOT** import: `api::*` (one-way dependency: API → Domain)
//! - **Uses**: SDK contract types (`User`, `NewUser`, etc.) as primary domain models
//! - **Uses**: `OData` filter schemas from `users_info_sdk::odata` (not defined here)
//!
//! ## `OData` Integration
//!
//! The service uses type-safe `OData` filtering via SDK filter enums:
//! - Filter schemas: `users_info_sdk::odata::{UserFilterField, CityFilterField, ...}`
//! - Pagination: `modkit_db::odata::paginate_odata` with filter type parameter
//! - Mapping: Infrastructure layer (`odata_mapper`) maps filters to `SeaORM` columns
//!
//! ## Security
//!
//! All operations use the `AuthZ` Resolver PEP (Policy Enforcement Point) pattern
//! via [`PolicyEnforcer`](authz_resolver_sdk::PolicyEnforcer):
//! 1. Construct a `PolicyEnforcer` (once, during init — serves all resource types)
//! 2. Call `enforcer.access_scope(&ctx, &resource, action, resource_id)`
//! 3. The enforcer builds the request, evaluates via PDP, and compiles to `AccessScope`
//! 4. Pass scope to repository methods for tenant-isolated queries
//!
//! ### Subtree authorization (no closure table)
//!
//! Enforcers are created with empty `capabilities` (no `tenant_hierarchy`).
//! This means the PDP must expand the subtree into explicit tenant IDs in
//! the constraints it returns.
//!
//! For **point operations** (GET/UPDATE/DELETE by ID), a prefetch pattern
//! would be more efficient: PEP fetches the resource first, sends its
//! `owner_tenant_id` as a resource property, and PDP returns a narrow `eq`
//! constraint instead of an expanded subtree. This also improves TOCTOU
//! protection for mutations.
//!
//! Reference: `docs/arch/authorization/AUTHZ_USAGE_SCENARIOS.md`
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

use modkit_macros::domain_model;

use crate::domain::events::UserDomainEvent;
use crate::domain::ports::{AuditPort, EventPublisher};
use crate::domain::repos::{AddressesRepository, CitiesRepository, UsersRepository};
use authz_resolver_sdk::AuthZResolverClient;
use authz_resolver_sdk::PolicyEnforcer;
use authz_resolver_sdk::pep::ResourceType;
use modkit_db::DBProvider;
use modkit_db::odata::LimitCfg;

mod addresses;
mod cities;
mod users;

/// Authorization resource types and their PEP-supported properties.
///
/// Each resource declares which properties the PEP can compile from PDP
/// constraints into SQL WHERE clauses. The PDP uses `supported_properties`
/// to decide which predicates it can return.
///
/// # Authorization model per resource
///
/// ## `USER`
/// - **Tenant isolation**: `owner_tenant_id` — every query is scoped to the
///   subject's tenant (or tenant subtree, depending on PDP policy).
/// - **Resource-level access**: `id` — PDP may restrict access to specific
///   user IDs (e.g., "user can only read their own profile").
/// - No owner dimension — users don't "belong to" another user.
///
/// ## `CITY`
/// - **Tenant isolation**: `owner_tenant_id` — cities are tenant-scoped.
/// - **Resource-level access**: `id` — PDP may restrict to specific city IDs.
/// - No owner or custom properties — cities are typically managed by admins;
///   no per-user ownership rules.
///
/// ## `ADDRESS`
/// - **Tenant isolation**: `owner_tenant_id` — addresses are tenant-scoped.
/// - **Resource-level access**: `id` — PDP may restrict to specific address IDs.
/// - **Owner-based access**: `owner_id` (maps to `user_id` column) — PDP can
///   enforce "users may only create/update/delete their own addresses" by
///   returning `eq(owner_id, <subject_id>)` predicates.
/// - **City-based access**: `city_id` — PDP can enforce per-user city
///   restrictions, e.g., "user A may only have addresses in city 1,
///   user B — only in city 2". PDP returns `eq(city_id, <allowed_city>)`
///   or `in(city_id, [city1, city2])` predicates.
pub(crate) mod resources {
    use super::ResourceType;
    use modkit_security::pep_properties;

    /// Domain-specific PEP properties for users-info.
    pub mod properties {
        /// City identifier property for address authorization.
        pub const CITY_ID: &str = "city_id";
    }

    pub const USER: ResourceType = ResourceType {
        name: "users_info.user",
        supported_properties: &[pep_properties::OWNER_TENANT_ID, pep_properties::RESOURCE_ID],
    };

    pub const CITY: ResourceType = ResourceType {
        name: "users_info.city",
        supported_properties: &[pep_properties::OWNER_TENANT_ID, pep_properties::RESOURCE_ID],
    };

    pub const ADDRESS: ResourceType = ResourceType {
        name: "users_info.address",
        supported_properties: &[
            pep_properties::OWNER_TENANT_ID,
            pep_properties::RESOURCE_ID,
            pep_properties::OWNER_ID,
            properties::CITY_ID,
        ],
    };
}

pub(crate) mod actions {
    pub const GET: &str = "get";
    pub const LIST: &str = "list";
    pub const CREATE: &str = "create";
    pub const UPDATE: &str = "update";
    pub const DELETE: &str = "delete";
}

pub(crate) use addresses::AddressesService;
pub(crate) use cities::CitiesService;
pub(crate) use users::UsersService;

pub(crate) type DbProvider = DBProvider<modkit_db::DbError>;

/// Configuration for the domain service
#[domain_model]
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
#[domain_model]
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
        authz: Arc<dyn AuthZResolverClient>,
        config: ServiceConfig,
    ) -> Self {
        let users_repo = Arc::new(users_repo);
        let cities_repo = Arc::new(cities_repo);
        let addresses_repo = Arc::new(addresses_repo);

        let enforcer = PolicyEnforcer::new(authz);

        let cities = Arc::new(CitiesService::new(
            Arc::clone(&db),
            Arc::clone(&cities_repo),
            enforcer.clone(),
        ));
        let addresses = Arc::new(AddressesService::new(
            Arc::clone(&db),
            Arc::clone(&addresses_repo),
            Arc::clone(&users_repo),
            enforcer.clone(),
        ));

        Self {
            users: UsersService::new(
                db,
                Arc::clone(&users_repo),
                events,
                audit,
                enforcer,
                config,
                cities.clone(),
                addresses.clone(),
            ),
            cities,
            addresses,
        }
    }
}
