//! Secure ORM layer for scoped database access.
//!
//! This module provides a type-safe wrapper around `SeaORM` that enforces
//! access control scoping at compile time using the typestate pattern.
//!
//! # Basic Example
//!
//! Creating and using access scopes:
//!
//! ```rust
//! use modkit_db::secure::{AccessScope, pep_properties};
//! use uuid::Uuid;
//!
//! // Create an empty scope (deny-all)
//! let deny_scope = AccessScope::default();
//! assert!(deny_scope.is_deny_all());
//!
//! // Create a tenant-scoped access
//! let tenant_id = Uuid::new_v4();
//! let scope = AccessScope::for_tenant(tenant_id);
//! assert!(scope.has_property(pep_properties::OWNER_TENANT_ID));
//! assert!(!scope.is_deny_all());
//!
//! // Create a resource-scoped access
//! let resource_id = Uuid::new_v4();
//! let resource_scope = AccessScope::for_resource(resource_id);
//! assert!(resource_scope.has_property(pep_properties::RESOURCE_ID));
//! ```
//!
//! # Quick Start with `SeaORM`
//!
//! ```rust,ignore
//! use modkit_db::secure::{AccessScope, SecureEntityExt, Scopable};
//! use sea_orm::entity::prelude::*;
//!
//! // 1. Derive Scopable for your entity (or implement manually)
//! #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
//! #[sea_orm(table_name = "users")]
//! #[secure(
//!     tenant_col = "tenant_id",
//!     resource_col = "id",
//!     no_owner,
//!     no_type
//! )]
//! pub struct Model {
//!     #[sea_orm(primary_key)]
//!     pub id: Uuid,
//!     pub tenant_id: Uuid,
//!     pub email: String,
//! }
//!
//! // 2. Create an access scope
//! let scope = AccessScope::for_tenants(vec![tenant_id]);
//!
//! // 3. Execute scoped queries
//! let users = Entity::find()
//!     .secure()
//!     .scope_with(&scope)?
//!     .all(conn)
//!     .await?;
//! ```
//!
//! # Manual Implementation
//!
//! If you prefer not to use the derive macro:
//!
//! ```rust,ignore
//! use modkit_db::secure::ScopableEntity;
//!
//! impl ScopableEntity for Entity {
//!     fn tenant_col() -> Option<Self::Column> {
//!         Some(Column::TenantId)
//!     }
//!     fn resource_col() -> Option<Self::Column> {
//!         Some(Column::Id)
//!     }
//!     fn owner_col() -> Option<Self::Column> {
//!         None
//!     }
//!     fn type_col() -> Option<Self::Column> {
//!         None
//!     }
//! }
//! ```
//!
//! # Features
//!
//! - **Typestate enforcement**: Prevents unscoped queries at compile time
//! - **Implicit policy**: Automatic deny-all for empty scopes
//! - **Multi-tenant support**: Enforces tenant isolation when applicable
//! - **Resource-level access**: Fine-grained control via explicit IDs
//! - **Zero runtime overhead**: All checks at compile/build time
//!
//! # Policy
//!
//! | Scope | Behavior |
//! |-------|----------|
//! | Empty | Deny all (`WHERE 1=0`) |
//! | Tenants only | Filter by tenant column |
//! | Resources only | Filter by ID column |
//! | Both | AND them together |
//!
//! See the [docs module](docs) for comprehensive examples and usage patterns.

// Module declarations
mod cond;
mod db;
mod db_ops;
pub mod docs;
#[allow(clippy::module_inception)]
mod entity_traits;
mod error;
pub mod provider;
mod runner;
mod secure_conn;
mod select;
mod tests;
mod tx_config;
mod tx_error;

// Public API re-exports

// Core types
pub use entity_traits::ScopableEntity;
pub use error::ScopeError;

// Security types from modkit-security
pub use modkit_security::{AccessScope, FilterOp, ScopeConstraint, ScopeFilter, pep_properties};

// Ergonomic secure connection API (no raw SeaORM types leaked)
pub use secure_conn::{SecureConn, SecureTx};

// Hidden runner capability used by repositories.
#[doc(hidden)]
pub use runner::DBRunner;

pub(crate) use runner::{DBRunnerInternal, SeaOrmRunner};

// Primary database types (new secure API)
pub use db::{Db, DbConn, DbTx};

// Transaction error types (no SeaORM types leaked)
pub use tx_error::{InfraError, TxError};

// Transaction configuration (no SeaORM types leaked)
pub use tx_config::{TxAccessMode, TxConfig, TxIsolationLevel};

// Select operations
pub use select::{
    Scoped, SecureEntityExt, SecureFindRelatedExt, SecureSelect, SecureSelectTwo,
    SecureSelectTwoMany, Unscoped,
};

// Update/Delete/Insert operations
pub use db_ops::{
    SecureDeleteExt, SecureDeleteMany, SecureInsertExt, SecureInsertOne, SecureOnConflict,
    SecureUpdateExt, SecureUpdateMany, secure_insert, secure_update_with_scope,
    validate_tenant_in_scope,
};

// Provider pattern for advanced tenant filtering
pub use provider::{SimpleTenantFilter, TenantFilterProvider};

pub use modkit_db_macros::Scopable;
