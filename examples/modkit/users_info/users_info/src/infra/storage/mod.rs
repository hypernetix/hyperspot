//! Infrastructure storage layer - database persistence and OData mapping.
//!
//! ## Architecture
//!
//! This module contains ALL SeaORM-specific code and database operations:
//! - `entity/` - SeaORM entity definitions (users, cities, languages, addresses, relations)
//! - `mapper.rs` - Conversions between SeaORM models and SDK contract types
//! - `odata_mapper.rs` - OData filter → SeaORM column mappings
//! - `migrations/` - Database schema migrations
//!
//! ## Layering Rules
//!
//! The infrastructure layer:
//! - **Contains**: ALL SeaORM imports and database-specific code
//! - **Uses**: `user_info_sdk` contract types as the domain model
//! - **Uses**: `user_info_sdk::odata` filter schemas (does NOT define them)
//! - **Provides**: Mappers implementing `ODataFieldMapping` trait
//!
//! ## OData Integration
//!
//! The `odata_mapper` module maps SDK filter enums to database columns:
//! - `UserODataMapper` - Maps `UserFilterField` → `user::Column`
//! - `CityODataMapper` - Maps `CityFilterField` → `city::Column`
//! - `LanguageODataMapper` - Maps `LanguageFilterField` → `language::Column`
//!
//! These mappers are used by the domain service's `paginate_odata` calls.

pub mod entity;
pub mod mapper;
pub mod migrations;
pub mod odata_mapper;
