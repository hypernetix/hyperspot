//! Infrastructure storage layer - database persistence and `OData` mapping.
//!
//! ## Architecture
//!
//! This module contains ALL `SeaORM`-specific code and database operations:
//! - `entity/` - `SeaORM` entity definitions (users, cities, addresses)
//! - `mapper.rs` - Conversions between `SeaORM` models and SDK contract types
//! - `odata_mapper.rs` - `OData` filter → `SeaORM` column mappings
//! - `migrations/` - Database schema migrations
//!
//! ## Layering Rules
//!
//! The infrastructure layer:
//! - **Contains**: ALL `SeaORM` imports and database-specific code
//! - **Uses**: `users_info_sdk` contract types as the domain model
//! - **Uses**: `users_info_sdk::odata` filter schemas (does NOT define them)
//! - **Provides**: Mappers implementing `ODataFieldMapping` trait
//!
//! ## `OData` Integration
//!
//! The `odata_mapper` module maps SDK filter enums to database columns:
//! - `UserODataMapper` - Maps `UserFilterField` → `user::Column`
//! - `CityODataMapper` - Maps `CityFilterField` → `city::Column`
//!
//! These mappers are used by the domain service's `paginate_odata` calls.

pub mod entity;
pub mod mapper;
pub mod migrations;
pub mod odata_mapper;

mod addresses_sea_repo;
mod cities_sea_repo;
mod db;
mod users_sea_repo;

pub use addresses_sea_repo::OrmAddressesRepository;
pub use cities_sea_repo::OrmCitiesRepository;
pub use users_sea_repo::OrmUsersRepository;
