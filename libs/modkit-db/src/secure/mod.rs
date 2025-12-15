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
//! use modkit_db::secure::AccessScope;
//! use uuid::Uuid;
//!
//! // Create an empty scope (deny-all)
//! let deny_scope = AccessScope::default();
//! assert!(deny_scope.is_empty());
//!
//! // Create a tenant-scoped access
//! let tenant_id = Uuid::new_v4();
//! let scope = AccessScope::tenant(tenant_id);
//! assert!(scope.has_tenants());
//! assert!(!scope.is_empty());
//!
//! // Create a resource-scoped access
//! let resource_id = Uuid::new_v4();
//! let resource_scope = AccessScope::resource(resource_id);
//! assert!(resource_scope.has_resources());
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
//! let scope = AccessScope::tenants_only(vec![tenant_id]);
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
mod db_ops;
pub mod docs;
#[allow(clippy::module_inception)]
mod entity_traits;
mod error;
pub mod migrate;
pub mod provider;
mod secure_conn;
mod select;
mod tests;

// Public API re-exports

// Core types
pub use entity_traits::ScopableEntity;
pub use error::ScopeError;

// Security types from modkit-security
pub use modkit_security::{AccessScope, SecurityCtx, Subject};

// High-level secure database wrapper
pub use secure_conn::SecureConn;

// Select operations
pub use select::{Scoped, SecureEntityExt, SecureSelect, Unscoped};

// Update operations
pub use db_ops::{
    secure_insert, validate_tenant_in_scope, SecureDeleteExt, SecureDeleteMany, SecureUpdateExt,
    SecureUpdateMany,
};

// Provider pattern for advanced tenant filtering
pub use provider::{SimpleTenantFilter, TenantFilterProvider};

// Re-export the derive macro when the feature is enabled
#[cfg(feature = "sea-orm")]
pub use modkit_db_macros::Scopable;
