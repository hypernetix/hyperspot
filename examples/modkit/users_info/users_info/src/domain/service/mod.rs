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
//! All operations use `SecureConn` for tenant isolation and RBAC:
//! - Queries filtered by security context automatically
//! - Operations checked against policy engine
//! - Audit events published for compliance

use std::sync::Arc;

use crate::domain::events::UserDomainEvent;
use crate::domain::ports::{AuditPort, EventPublisher};
use crate::domain::repos::{AddressesRepository, CitiesRepository, UsersRepository};
use modkit_db::odata::LimitCfg;
use modkit_db::secure::SecureConn;
use modkit_security::PolicyEngineRef;

mod addresses;
mod cities;
mod users;

pub(crate) use addresses::AddressesService;
pub(crate) use cities::CitiesService;
pub(crate) use users::UsersService;

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
#[derive(Clone)]
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
    pub fn new(
        users_repo: UR,
        cities_repo: CR,
        addresses_repo: AR,
        db: SecureConn,
        events: Arc<dyn EventPublisher<UserDomainEvent>>,
        audit: Arc<dyn AuditPort>,
        config: ServiceConfig,
    ) -> Self {
        let policy_engine: PolicyEngineRef = Arc::new(modkit_security::DummyPolicyEngine);

        let users_repo = Arc::new(users_repo);
        let cities_repo = Arc::new(cities_repo);
        let addresses_repo = Arc::new(addresses_repo);

        let cities = Arc::new(CitiesService::new(
            Arc::clone(&cities_repo),
            db.clone(),
            policy_engine.clone(),
        ));
        let addresses = Arc::new(AddressesService::new(
            Arc::clone(&addresses_repo),
            Arc::clone(&users_repo),
            db.clone(),
            policy_engine.clone(),
        ));

        Self {
            users: UsersService::new(
                Arc::clone(&users_repo),
                db,
                events,
                audit,
                policy_engine.clone(),
                config,
                cities.clone(),
                addresses.clone(),
            ),
            cities,
            addresses,
        }
    }
}
