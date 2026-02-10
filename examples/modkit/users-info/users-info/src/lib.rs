//! Users Info Module
//!
//! This module provides user management functionality with REST API,
//! database storage, and inter-module communication via `ClientHub`.
//!
//! ## Architecture
//!
//! This module follows clean architecture with strict layering:
//!
//! ### Contract Layer (`users-info-sdk`)
//! - **Location:** `examples/modkit/users-info/users-info-sdk/`
//! - **Purpose:** Public API contract for inter-module communication
//! - **Contains:**
//!   - `UsersInfoClient` trait
//!   - Model types: `User`, `Address`, `City`
//!   - Request/patch types: `NewUser`, `UserPatch`, etc.
//!   - Error type: `UsersInfoError`
//!   - `OData` filter schemas (behind `odata` feature): `UserFilterField`, `CityFilterField`, etc.
//! - **Dependencies:** Only modkit core libs (no server code)
//!
//! ### API Layer (`users_info::api`)
//! - **Location:** `src/api/`
//! - **Purpose:** HTTP/REST interface
//! - **Contains:**
//!   - `routes/` - Per-resource route definitions (users, cities, etc.)
//!   - `handlers/` - Request handlers per resource
//!   - `dto.rs` - REST-specific DTOs and serialization
//!   - `error.rs` - HTTP error mapping (domain errors → RFC9457 Problem)
//! - **Dependencies:** Domain service, SDK types
//! - **Rule:** May import `domain::service::Service` and `domain::error::DomainError` for orchestration
//!
//! ### Domain Layer (`users_info::domain`)
//! - **Location:** `src/domain/`
//! - **Purpose:** Business logic and domain rules
//! - **Contains:**
//!   - `service/` - Business operations per resource (users, cities, etc.)
//!   - `error.rs` - Domain error types
//!   - `events.rs` - Domain events
//!   - `ports.rs` - Interfaces for external dependencies
//! - **Dependencies:** SDK contract types, modkit libs, infra for data access
//! - **Rule:** MUST NOT import `api::*` (one-way dependency only)
//!
//! ### Infrastructure Layer (`users_info::infra`)
//! - **Location:** `src/infra/storage/`
//! - **Purpose:** Database persistence and `OData` mapping
//! - **Contains:**
//!   - `entity/` - `SeaORM` entity definitions
//!   - `mapper.rs` - Entity ↔ SDK model conversions
//!   - `odata_mapper.rs` - `OData` filter → `SeaORM` column mappings
//!   - `migrations/` - Database schema migrations
//! - **Dependencies:** SDK types (for models), `SeaORM`
//! - **Rule:** ALL `SeaORM` specifics contained here; `OData` schemas from SDK only
//!
//! ## Public API
//!
//! The public API is defined in the `users-info-sdk` crate and re-exported here:
//! - `UsersInfoClientV1` - trait for inter-module communication
//! - User, Address and City models and their request/patch types
//! - `UsersInfoError` - error types
//!
//! Other modules should use `hub.get::<dyn UsersInfoClientV1>()?` to obtain the client.
//!
//! ## `OData` Support
//!
//! `OData` filter schemas live in `users-info-sdk::odata` (behind `odata` feature):
//! - Type-safe filter enums for each resource
//! - Used by both REST API (`OpenAPI`) and domain pagination
//! - Mapped to database columns in `infra::storage::odata_mapper`
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
// === PUBLIC API (from SDK) ===
pub use users_info_sdk::{
    Address, AddressPatch, City, CityPatch, NewAddress, NewCity, NewUser, UpdateAddressRequest,
    UpdateCityRequest, UpdateUserRequest, User, UserPatch, UsersInfoClientV1, UsersInfoError,
};

// === ERROR CATALOG ===
// Generated error catalog from gts/errors.json
pub mod errors;

// === MODULE DEFINITION ===
// ModKit needs access to the module struct for instantiation
pub mod module;
pub use module::UsersInfo;

// === INTERNAL MODULES ===
// WARNING: These modules are internal implementation details!
// They are exposed only for comprehensive testing and should NOT be used by external consumers.
// Only use the SDK types for stable public APIs.
#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
#[doc(hidden)]
pub mod infra;

#[cfg(test)]
mod test_support;
