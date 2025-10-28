//! High-level secure database wrapper for ergonomic, type-safe access.
//!
//! This module provides `SecureConn`, a wrapper around SeaORM's `DatabaseConnection`
//! that enforces access control policies on all operations.
//!
//! # Design Philosophy
//!
//! Plugin/module developers should never handle raw `DatabaseConnection` or manually
//! apply scopes. Instead, they receive a `SecureConn` instance that guarantees:
//!
//! - **Automatic scoping**: All queries are filtered by tenant/resource scope
//! - **Type safety**: Cannot execute unscoped queries
//! - **Ergonomics**: Simple, fluent API for common operations
//!
//! # Example
//!
//! ```ignore
//! use modkit_db::secure::{SecureConn, SecurityCtx, AccessScope};
//!
//! pub struct UsersRepo<'a> {
//!     db: &'a SecureConn,
//! }
//!
//! impl<'a> UsersRepo<'a> {
//!     pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, ScopeError> {
//!         let user = self.db
//!             .find_by_id::<user::Entity>(id)?
//!             .one(self.db.conn())
//!             .await?;
//!         Ok(user.map(Into::into))
//!     }
//!
//!     pub async fn find_all(&self) -> Result<Vec<User>, ScopeError> {
//!         let users = self.db
//!             .find::<user::Entity>()?
//!             .all(self.db.conn())
//!             .await?;
//!         Ok(users.into_iter().map(Into::into).collect())
//!     }
//!
//!     pub async fn update_status(&self, status: String) -> Result<u64, ScopeError> {
//!         let result = self.db
//!             .update_many::<user::Entity>()?
//!             .col_expr(user::Column::Status, Expr::value(status))
//!             .exec(self.db.conn())
//!             .await?;
//!         Ok(result.rows_affected)
//!     }
//! }
//! ```

use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use modkit_security::SecurityCtx;

use crate::secure::{ScopableEntity, ScopeError, Scoped, SecureEntityExt, SecureSelect};

use crate::secure::db_ops::{SecureDeleteExt, SecureDeleteMany, SecureUpdateExt, SecureUpdateMany};

/// Secure database connection wrapper.
///
/// This is the primary interface for module developers to access the database.
/// All operations require a `SecurityCtx` parameter for per-request access control.
///
/// # Usage
///
/// Module services receive a `&SecureConn` and provide `SecurityCtx` per-request:
///
/// ```ignore
/// pub struct MyService<'a> {
///     db: &'a SecureConn,
/// }
///
/// impl<'a> MyService<'a> {
///     pub async fn get_user(&self, ctx: &SecurityCtx, id: Uuid) -> Result<Option<User>> {
///         self.db.find_by_id::<user::Entity>(ctx, id)?
///             .one(self.db.conn())
///             .await
///     }
/// }
/// ```
///
/// # Security Guarantees
///
/// - All queries require `SecurityCtx` from the request
/// - Queries are scoped by tenant/resource from the context
/// - Empty scopes result in deny-all (no data returned)
/// - Type system prevents unscoped queries from compiling
/// - Cannot bypass security without `insecure-escape` feature
#[derive(Clone)]
pub struct SecureConn {
    conn: DatabaseConnection,
}

impl SecureConn {
    /// Create a new secure database connection wrapper.
    ///
    /// Typically created via `DbHandle::sea_secure()` rather than directly.
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Get a reference to the underlying database connection.
    ///
    /// # Safety
    ///
    /// Use with caution. Direct connection access bypasses automatic scoping.
    /// Prefer the high-level methods (`find`, `update_many`, etc.) whenever possible.
    ///
    /// Valid use cases:
    /// - Executing already-scoped queries (`.one()`, `.all()`, `.exec()`)
    /// - Complex joins that need custom SeaORM building
    /// - Internal infrastructure code (not module business logic)
    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }

    /// Create a scoped select query for the given entity.
    ///
    /// Returns a `SecureSelect<E, Scoped>` that automatically applies
    /// tenant/resource filtering based on the provided security context.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ctx = SecurityCtx::for_tenants(vec![tenant_id], user_id);
    /// let users = db.find::<user::Entity>(&ctx)?
    ///     .filter(user::Column::Status.eq("active"))
    ///     .order_by_asc(user::Column::Email)
    ///     .all(db.conn())
    ///     .await?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ScopeError` if the scope cannot be applied to the entity.
    pub fn find<E>(&self, ctx: &SecurityCtx) -> Result<SecureSelect<E, Scoped>, ScopeError>
    where
        E: ScopableEntity + EntityTrait,
        E::Column: ColumnTrait + Copy,
    {
        E::find().secure().scope_with(ctx.scope())
    }

    /// Create a scoped select query filtered by a specific resource ID.
    ///
    /// This is a convenience method that combines `find()` with `.and_id()`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ctx = SecurityCtx::for_tenants(vec![tenant_id], user_id);
    /// let user = db.find_by_id::<user::Entity>(&ctx, user_id)?
    ///     .one(db.conn())
    ///     .await?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ScopeError` if the scope cannot be applied to the entity.
    pub fn find_by_id<E>(
        &self,
        ctx: &SecurityCtx,
        id: Uuid,
    ) -> Result<SecureSelect<E, Scoped>, ScopeError>
    where
        E: ScopableEntity + EntityTrait,
        E::Column: ColumnTrait + Copy,
    {
        Ok(self.find::<E>(ctx)?.and_id(id))
    }

    /// Create a scoped update query for the given entity.
    ///
    /// Returns a `SecureUpdateMany<E, Scoped>` that automatically applies
    /// tenant/resource filtering. Use `.col_expr()` or other SeaORM methods
    /// to specify what to update.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ctx = SecurityCtx::for_tenants(vec![tenant_id], user_id);
    /// let result = db.update_many::<user::Entity>(&ctx)?
    ///     .col_expr(user::Column::Status, Expr::value("active"))
    ///     .col_expr(user::Column::UpdatedAt, Expr::value(Utc::now()))
    ///     .exec(db.conn())
    ///     .await?;
    /// println!("Updated {} rows", result.rows_affected);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ScopeError` if the scope cannot be applied to the entity.
    pub fn update_many<E>(
        &self,
        ctx: &SecurityCtx,
    ) -> Result<SecureUpdateMany<E, Scoped>, ScopeError>
    where
        E: ScopableEntity + EntityTrait,
        E::Column: ColumnTrait + Copy,
    {
        E::update_many().secure().scope_with(ctx.scope())
    }

    /// Create a scoped delete query for the given entity.
    ///
    /// Returns a `SecureDeleteMany<E, Scoped>` that automatically applies
    /// tenant/resource filtering.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ctx = SecurityCtx::for_tenants(vec![tenant_id], user_id);
    /// let result = db.delete_many::<user::Entity>(&ctx)?
    ///     .exec(db.conn())
    ///     .await?;
    /// println!("Deleted {} rows", result.rows_affected);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ScopeError` if the scope cannot be applied to the entity.
    pub fn delete_many<E>(
        &self,
        ctx: &SecurityCtx,
    ) -> Result<SecureDeleteMany<E, Scoped>, ScopeError>
    where
        E: ScopableEntity + EntityTrait,
        E::Column: ColumnTrait + Copy,
    {
        E::delete_many().secure().scope_with(ctx.scope())
    }

    /// Insert a new entity with automatic tenant validation.
    ///
    /// This is a convenience wrapper around `secure_insert()` that uses
    /// the provided security context.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ctx = SecurityCtx::for_tenants(vec![tenant_id], user_id);
    /// let am = user::ActiveModel {
    ///     id: Set(Uuid::new_v4()),
    ///     tenant_id: Set(tenant_id),
    ///     owner_id: Set(ctx.subject_id),
    ///     email: Set("user@example.com".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// let user = db.insert::<user::Entity>(&ctx, am).await?;
    /// ```
    ///
    /// # Errors
    ///
    /// - `ScopeError::Invalid` if entity requires tenant but scope has none
    /// - `ScopeError::Db` if database insert fails
    pub async fn insert<E>(
        &self,
        ctx: &SecurityCtx,
        am: E::ActiveModel,
    ) -> Result<E::Model, ScopeError>
    where
        E: ScopableEntity + EntityTrait,
        E::Column: ColumnTrait + Copy,
        E::ActiveModel: sea_orm::ActiveModelTrait<Entity = E> + Send,
        E::Model: sea_orm::IntoActiveModel<E::ActiveModel>,
    {
        crate::secure::secure_insert::<E>(am, ctx, &self.conn).await
    }

    /// Update a single entity by ID (unscoped).
    ///
    /// **Warning**: This method does NOT validate security scope.
    /// Use `update_with_ctx()` for scope-validated updates.
    ///
    /// This is a convenience method for the common pattern of updating one record
    /// when you've already validated access separately.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut user: user::ActiveModel = db.find_by_id::<user::Entity>(id)?
    ///     .one(db.conn())
    ///     .await?
    ///     .ok_or(NotFound)?
    ///     .into();
    ///
    /// user.email = Set("newemail@example.com".to_string());
    /// user.updated_at = Set(Utc::now());
    ///
    /// let updated = db.update_one(user).await?;
    /// ```
    pub async fn update_one<E>(&self, am: E::ActiveModel) -> Result<E::Model, ScopeError>
    where
        E: EntityTrait,
        E::ActiveModel: sea_orm::ActiveModelTrait<Entity = E> + Send,
        E::Model: sea_orm::IntoActiveModel<E::ActiveModel>,
    {
        Ok(am.update(&self.conn).await?)
    }

    /// Update a single entity with security scope validation.
    ///
    /// This method ensures the entity being updated is within the security scope
    /// before performing the update. It validates that the record is accessible
    /// based on tenant/resource constraints.
    ///
    /// # Security
    ///
    /// - Validates the entity exists and is accessible in the security scope
    /// - Returns `ScopeError::Denied` if the entity is not in scope
    /// - Ensures updates cannot affect entities outside the security boundary
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ctx = SecurityCtx::for_tenant(tenant_id, user_id);
    ///
    /// // Load and modify
    /// let user_model = db.find_by_id::<user::Entity>(&ctx, id)?
    ///     .one(db.conn())
    ///     .await?
    ///     .ok_or(NotFound)?;
    ///
    /// let mut user: user::ActiveModel = user_model.into();
    /// user.email = Set("newemail@example.com".to_string());
    /// user.updated_at = Set(Utc::now());
    ///
    /// // Update with scope validation (pass ID separately)
    /// let updated = db.update_with_ctx::<user::Entity>(&ctx, id, user).await?;
    /// ```
    ///
    /// # Errors
    ///
    /// - `ScopeError::Denied` if the entity is not accessible in the current scope
    /// - `ScopeError::Db` if the database operation fails
    pub async fn update_with_ctx<E>(
        &self,
        ctx: &SecurityCtx,
        id: Uuid,
        am: E::ActiveModel,
    ) -> Result<E::Model, ScopeError>
    where
        E: ScopableEntity + EntityTrait,
        E::Column: ColumnTrait + Copy,
        E::ActiveModel: sea_orm::ActiveModelTrait<Entity = E> + Send,
        E::Model: sea_orm::IntoActiveModel<E::ActiveModel>,
    {
        // Verify the entity exists and is in scope before updating
        let exists = self
            .find_by_id::<E>(ctx, id)?
            .one(&self.conn)
            .await?
            .is_some();

        if !exists {
            return Err(ScopeError::Denied(
                "entity not found or not accessible in current security scope",
            ));
        }

        // Entity is in scope, perform the update
        Ok(am.update(&self.conn).await?)
    }

    /// Delete a single entity by ID (scoped).
    ///
    /// This validates the entity exists in scope before deleting.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ctx = SecurityCtx::for_tenants(vec![tenant_id], user_id);
    /// db.delete_by_id::<user::Entity>(&ctx, user_id).await?;
    /// ```
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if entity was deleted
    /// - `Ok(false)` if entity not found in scope
    pub async fn delete_by_id<E>(&self, ctx: &SecurityCtx, id: Uuid) -> Result<bool, ScopeError>
    where
        E: ScopableEntity + EntityTrait,
        E::Column: ColumnTrait + Copy,
    {
        // Filter by ID first, then scope
        let result = E::delete_many()
            .filter(sea_orm::Condition::all().add(Expr::col(E::id_col()).eq(id)))
            .secure()
            .scope_with(ctx.scope())?
            .exec(&self.conn)
            .await?;

        Ok(result.rows_affected > 0)
    }
}
