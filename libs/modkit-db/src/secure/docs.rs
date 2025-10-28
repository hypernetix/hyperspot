//! # Secure ORM Layer Documentation
//!
//! The secure ORM layer provides type-safe, scoped access to database entities using SeaORM.
//! It enforces an implicit security policy that prevents unscoped queries from executing.
//!
//! ## Core Concepts
//!
//! ### 1. AccessScope
//!
//! The [`AccessScope`](crate::secure::AccessScope) struct defines the security boundary:
//!
//! ```rust
//! use modkit_db::secure::AccessScope;
//! use uuid::Uuid;
//!
//! let tenant_id = Uuid::new_v4();
//! let resource_id = Uuid::new_v4();
//!
//! // Scope to specific tenants
//! let scope = AccessScope::tenants_only(vec![tenant_id]);
//!
//! // Scope to specific resources
//! let scope = AccessScope::resources_only(vec![resource_id]);
//!
//! // Scope to both (AND relationship)
//! let scope = AccessScope::both(vec![tenant_id], vec![resource_id]);
//!
//! // Empty scope (will deny all)
//! let scope = AccessScope::default();
//! ```
//!
//! ### 2. ScopableEntity
//!
//! Entities must implement [`ScopableEntity`](crate::secure::ScopableEntity) to declare
//! which columns are used for scoping:
//!
//! ```rust,ignore
//! use modkit_db::secure::ScopableEntity;
//!
//! impl ScopableEntity for user::Entity {
//!     fn tenant_col() -> Option<Self::Column> {
//!         Some(user::Column::TenantId)  // Multi-tenant entity
//!     }
//!     fn id_col() -> Self::Column {
//!         user::Column::Id
//!     }
//! }
//!
//! // Global entity (no tenant scoping)
//! impl ScopableEntity for system_config::Entity {
//!     fn tenant_col() -> Option<Self::Column> {
//!         None  // Global entity
//!     }
//!     fn id_col() -> Self::Column {
//!         system_config::Column::Id
//!     }
//! }
//! ```
//!
//! ### 3. Typestate-Based Queries
//!
//! The [`SecureSelect`](crate::secure::SecureSelect) wrapper uses typestates to prevent
//! executing unscoped queries at compile time:
//!
//! ```rust,ignore
//! use modkit_db::secure::{AccessScope, SecureEntityExt};
//!
//! // This works ✓
//! let users = user::Entity::find()
//!     .secure()              // Returns SecureSelect<E, Unscoped>
//!     .scope_with(&scope)?   // Returns SecureSelect<E, Scoped>
//!     .all(conn)             // Now can execute
//!     .await?;
//!
//! // This won't compile ✗
//! let users = user::Entity::find()
//!     .secure()
//!     .all(conn);  // ERROR: method not found in `SecureSelect<E, Unscoped>`
//! ```
//!
//! ## Implicit Security Policy
//!
//! The layer enforces these rules automatically:
//!
//! | Scope Condition | SQL Result |
//! |----------------|------------|
//! | Empty (no tenant, no resource) | `WHERE 1=0` (deny all) |
//! | Tenants only | `WHERE tenant_id IN (...)` |
//! | Tenants only + entity has no tenant_col | `WHERE 1=0` (deny all) |
//! | Resources only | `WHERE id IN (...)` |
//! | Both tenants and resources | `WHERE tenant_id IN (...) AND id IN (...)` |
//!
//! ## Usage Examples
//!
//! ### Example 1: List users for a tenant
//!
//! ```rust,ignore
//! use modkit_db::secure::{AccessScope, SecureEntityExt};
//!
//! pub async fn list_tenant_users(
//!     conn: &DatabaseConnection,
//!     tenant_id: Uuid,
//! ) -> Result<Vec<user::Model>, anyhow::Error> {
//!     let scope = AccessScope::tenants_only(vec![tenant_id]);
//!     
//!     let users = user::Entity::find()
//!         .secure()
//!         .scope_with(&scope)?
//!         .all(conn)
//!         .await?;
//!     
//!     Ok(users)
//! }
//! ```
//!
//! ### Example 2: Get specific user by ID (with tenant check)
//!
//! ```rust,ignore
//! use modkit_db::secure::{AccessScope, SecureEntityExt};
//!
//! pub async fn get_user(
//!     conn: &DatabaseConnection,
//!     tenant_id: Uuid,
//!     user_id: Uuid,
//! ) -> Result<Option<user::Model>, anyhow::Error> {
//!     // This ensures the user belongs to the tenant (implicit AND)
//!     let scope = AccessScope::both(vec![tenant_id], vec![user_id]);
//!     
//!     let user = user::Entity::find()
//!         .secure()
//!         .scope_with(&scope)?
//!         .one(conn)
//!         .await?;
//!     
//!     Ok(user)
//! }
//! ```
//!
//! ### Example 3: List specific resources regardless of tenant
//!
//! ```rust,ignore
//! // Useful for admin operations or cross-tenant reports
//! pub async fn get_users_by_ids(
//!     conn: &DatabaseConnection,
//!     user_ids: Vec<Uuid>,
//! ) -> Result<Vec<user::Model>, anyhow::Error> {
//!     let scope = AccessScope::resources_only(user_ids);
//!     
//!     let users = user::Entity::find()
//!         .secure()
//!         .scope_with(&scope)?
//!         .all(conn)
//!         .await?;
//!     
//!     Ok(users)
//! }
//! ```
//!
//! ### Example 4: Additional filtering after scoping
//!
//! ```rust,ignore
//! use sea_orm::{ColumnTrait, QueryFilter};
//!
//! pub async fn list_active_users(
//!     conn: &DatabaseConnection,
//!     tenant_id: Uuid,
//! ) -> Result<Vec<user::Model>, anyhow::Error> {
//!     let scope = AccessScope::tenants_only(vec![tenant_id]);
//!     
//!     let users = user::Entity::find()
//!         .secure()
//!         .scope_with(&scope)?
//!         .filter(user::Column::IsActive.eq(true))  // Additional filter
//!         .order_by(user::Column::Email, Order::Asc)
//!         .limit(100)
//!         .all(conn)
//!         .await?;
//!     
//!     Ok(users)
//! }
//! ```
//!
//! ### Example 5: Working with global entities
//!
//! ```rust,ignore
//! // Global entities (no tenant column) work with resource IDs only
//! pub async fn get_system_config(
//!     conn: &DatabaseConnection,
//!     config_id: Uuid,
//! ) -> Result<Option<system_config::Model>, anyhow::Error> {
//!     let scope = AccessScope::resources_only(vec![config_id]);
//!     
//!     let config = system_config::Entity::find()
//!         .secure()
//!         .scope_with(&scope)?
//!         .one(conn)
//!         .await?;
//!     
//!     Ok(config)
//! }
//! ```
//!
//! ### Example 6: Escape hatch for advanced queries
//!
//! ```rust,ignore
//! use sea_orm::JoinType;
//!
//! pub async fn complex_query(
//!     conn: &DatabaseConnection,
//!     tenant_id: Uuid,
//! ) -> Result<Vec<user::Model>, anyhow::Error> {
//!     let scope = AccessScope::tenants_only(vec![tenant_id]);
//!     
//!     let scoped = user::Entity::find()
//!         .secure()
//!         .scope_with(&scope)?;
//!     
//!     // Use into_inner() to access full SeaORM API
//!     let users = scoped.into_inner()
//!         .join(JoinType::InnerJoin, user::Relation::Profile.def())
//!         .all(conn)
//!         .await?;
//!     
//!     Ok(users)
//! }
//! ```
//!
//! ## Integration with Repository Pattern
//!
//! A typical repository would look like:
//!
//! ```rust,ignore
//! use modkit_db::secure::{AccessScope, SecureEntityExt, ScopeError};
//! use sea_orm::DatabaseConnection;
//! use uuid::Uuid;
//!
//! pub struct UserRepository {
//!     conn: DatabaseConnection,
//! }
//!
//! impl UserRepository {
//!     pub async fn list_for_scope(
//!         &self,
//!         scope: &AccessScope,
//!     ) -> Result<Vec<user::Model>, ScopeError> {
//!         user::Entity::find()
//!             .secure()
//!             .scope_with(scope)?
//!             .all(&self.conn)
//!             .await
//!     }
//!     
//!     pub async fn find_by_id(
//!         &self,
//!         tenant_id: Uuid,
//!         user_id: Uuid,
//!     ) -> Result<Option<user::Model>, ScopeError> {
//!         let scope = AccessScope::both(vec![tenant_id], vec![user_id]);
//!         
//!         user::Entity::find()
//!             .secure()
//!             .scope_with(&scope)?
//!             .one(&self.conn)
//!             .await
//!     }
//! }
//! ```
//!
//! ## Security Guarantees
//!
//! 1. **No unscoped execution**: Queries cannot be executed without calling `.scope_with()`
//! 2. **Explicit deny-all**: Empty scopes are denied rather than returning all data
//! 3. **Tenant isolation**: When tenant_ids are provided, they're always enforced
//! 4. **Type safety**: Typestates prevent misuse at compile time
//! 5. **No runtime overhead**: All checks happen at compile time or query build time
//!
//! ## Phase 2: Planned Enhancements
//!
//! Future versions will include:
//!
//! - `#[derive(Scopable)]` macro to auto-implement `ScopableEntity`
//! - Support for scoped UPDATE and DELETE operations
//! - Row-level security helpers for PostgreSQL
//! - Audit logging integration
//! - Policy composition (e.g., role-based filters)
//!
//! ## Error Handling
//!
//! The layer uses [`ScopeError`](crate::secure::ScopeError) for all errors:
//!
//! ```rust,ignore
//! match user::Entity::find().secure().scope_with(&scope) {
//!     Ok(scoped) => {
//!         // Execute query
//!     }
//!     Err(ScopeError::Db(msg)) => {
//!         // Handle database error
//!     }
//! }
//! ```

#[cfg(doc)]
use crate::secure::{AccessScope, ScopableEntity, SecureSelect};
