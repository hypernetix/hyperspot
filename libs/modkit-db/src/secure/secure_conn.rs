//! High-level secure database wrapper for ergonomic, type-safe access.
//!
//! This module provides `SecureConn`, a wrapper around `SeaORM`'s `DatabaseConnection`
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

use std::{future::Future, pin::Pin};

use sea_orm::{
    sea_query::Expr, AccessMode, ActiveModelTrait, ColumnTrait, ConnectionTrait,
    DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait, IsolationLevel, QueryFilter,
    TransactionTrait,
};
use uuid::Uuid;

use crate::secure::tx_error::{InfraError, TxError};

use modkit_security::AccessScope;

use crate::secure::tx_config::TxConfig;

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
///     pub async fn get_user(&self, scope: &AccessScope, id: Uuid) -> Result<Option<User>> {
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
    #[must_use]
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
    /// - Complex joins that need custom `SeaORM` building
    /// - Internal infrastructure code (not module business logic)
    #[must_use]
    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }

    /// Return database engine identifier for tracing / logging.
    #[must_use]
    pub fn db_engine(&self) -> &'static str {
        use sea_orm::DatabaseBackend;

        match self.conn.get_database_backend() {
            DatabaseBackend::Postgres => "postgres",
            DatabaseBackend::MySql => "mysql",
            DatabaseBackend::Sqlite => "sqlite",
        }
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
    #[allow(clippy::unused_self)] // Keep fluent &SecureConn API even when method only delegates
    pub fn find<E>(&self, scope: &AccessScope) -> SecureSelect<E, Scoped>
    where
        E: ScopableEntity + EntityTrait,
        E::Column: ColumnTrait + Copy,
    {
        E::find().secure().scope_with(scope)
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
    /// Returns `ScopeError` if the entity doesn't have a resource column or scoping fails.
    pub fn find_by_id<E>(
        &self,
        scope: &AccessScope,
        id: Uuid,
    ) -> Result<SecureSelect<E, Scoped>, ScopeError>
    where
        E: ScopableEntity + EntityTrait,
        E::Column: ColumnTrait + Copy,
    {
        self.find::<E>(scope).and_id(id)
    }

    /// Create a scoped update query for the given entity.
    ///
    /// Returns a `SecureUpdateMany<E, Scoped>` that automatically applies
    /// tenant/resource filtering. Use `.col_expr()` or other `SeaORM` methods
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
    #[allow(clippy::unused_self)] // Delegates but matches the rest of the connection API
    #[must_use]
    pub fn update_many<E>(&self, scope: &AccessScope) -> SecureUpdateMany<E, Scoped>
    where
        E: ScopableEntity + EntityTrait,
        E::Column: ColumnTrait + Copy,
    {
        E::update_many().secure().scope_with(scope)
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
    #[allow(clippy::unused_self)] // Retain method-style ergonomics for callers of SecureConn
    #[must_use]
    pub fn delete_many<E>(&self, scope: &AccessScope) -> SecureDeleteMany<E, Scoped>
    where
        E: ScopableEntity + EntityTrait,
        E::Column: ColumnTrait + Copy,
    {
        E::delete_many().secure().scope_with(scope)
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
        scope: &AccessScope,
        am: E::ActiveModel,
    ) -> Result<E::Model, ScopeError>
    where
        E: ScopableEntity + EntityTrait,
        E::Column: ColumnTrait + Copy,
        E::ActiveModel: sea_orm::ActiveModelTrait<Entity = E> + Send,
        E::Model: sea_orm::IntoActiveModel<E::ActiveModel>,
    {
        crate::secure::secure_insert::<E>(am, scope, &self.conn).await
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
    ///
    /// # Errors
    /// Returns `ScopeError::Db` if the database update fails.
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
        scope: &AccessScope,
        id: Uuid,
        am: E::ActiveModel,
    ) -> Result<E::Model, ScopeError>
    where
        E: ScopableEntity + EntityTrait,
        E::Column: ColumnTrait + Copy,
        E::ActiveModel: sea_orm::ActiveModelTrait<Entity = E> + Send,
        E::Model: sea_orm::IntoActiveModel<E::ActiveModel>,
    {
        let exists = self
            .find_by_id::<E>(scope, id)?
            .one(&self.conn)
            .await?
            .is_some();

        if !exists {
            return Err(ScopeError::Denied(
                "entity not found or not accessible in current security scope",
            ));
        }

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
    ///
    /// # Errors
    ///
    /// Returns `ScopeError::Invalid` if the entity does not have a `resource_col` defined.
    pub async fn delete_by_id<E>(&self, scope: &AccessScope, id: Uuid) -> Result<bool, ScopeError>
    where
        E: ScopableEntity + EntityTrait,
        E::Column: ColumnTrait + Copy,
    {
        let resource_col = E::resource_col().ok_or_else(|| {
            ScopeError::Invalid("Entity must have a resource_col to use delete_by_id()")
        })?;

        let result = E::delete_many()
            .filter(sea_orm::Condition::all().add(Expr::col(resource_col).eq(id)))
            .secure()
            .scope_with(scope)
            .exec(&self.conn)
            .await?;

        Ok(result.rows_affected > 0)
    }

    // ========================================================================
    // Transaction support
    // ========================================================================

    /// Execute a closure inside a database transaction.
    ///
    /// This method starts a `SeaORM` transaction, provides the transaction handle
    /// to the closure as `&dyn ConnectionTrait`, and handles commit/rollback.
    ///
    /// # Return Type
    ///
    /// Returns `anyhow::Result<Result<T, E>>` where:
    /// - Outer `Err`: Database/infrastructure error (transaction rolls back)
    /// - Inner `Ok(T)`: Success (transaction commits)
    /// - Inner `Err(E)`: Domain/validation error (transaction still commits)
    ///
    /// This design ensures domain validation errors don't cause rollback.
    ///
    /// # Architecture Note
    ///
    /// Transaction boundaries should be managed by **application/domain services**,
    /// not by REST handlers. REST handlers should call service methods that
    /// internally decide when to open transactions.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use modkit_db::secure::SecureConn;
    ///
    /// // In a domain service:
    /// pub async fn create_user(
    ///     db: &SecureConn,
    ///     repo: &UsersRepo,
    ///     user: User,
    /// ) -> Result<User, DomainError> {
    ///     let result = db.transaction(|conn| async move {
    ///         // Check email uniqueness
    ///         if repo.email_exists(conn, &user.email).await? {
    ///             return Ok(Err(DomainError::EmailExists));
    ///         }
    ///         // Create user
    ///         let created = repo.create(conn, user).await?;
    ///         Ok(Ok(created))
    ///     }).await?;
    ///     result
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err(anyhow::Error)` if:
    /// - The transaction cannot be started
    /// - A database operation fails (transaction is rolled back)
    /// - The commit fails
    pub async fn transaction<T, F>(&self, f: F) -> anyhow::Result<T>
    where
        T: Send + 'static,
        F: for<'c> FnOnce(
                &'c DatabaseTransaction,
            )
                -> Pin<Box<dyn Future<Output = anyhow::Result<T>> + Send + 'c>>
            + Send,
    {
        self.conn
            .transaction::<_, T, DbErr>(|txn| {
                let fut = f(txn);
                Box::pin(async move {
                    fut.await
                        .map_err(|e| DbErr::Custom(format!("transaction callback failed: {e:#}")))
                })
            })
            .await
            .map_err(|e| anyhow::anyhow!("transaction failed: {e}"))
    }

    /// Execute a closure inside a database transaction with custom configuration.
    ///
    /// This method is similar to [`transaction`](Self::transaction), but allows
    /// specifying the isolation level and access mode.
    ///
    /// # Configuration
    ///
    /// Use [`TxConfig`] to specify transaction settings without importing `SeaORM` types:
    ///
    /// ```ignore
    /// use modkit_db::secure::{TxConfig, TxIsolationLevel, TxAccessMode};
    ///
    /// let cfg = TxConfig {
    ///     isolation: Some(TxIsolationLevel::Serializable),
    ///     access_mode: Some(TxAccessMode::ReadWrite),
    /// };
    /// ```
    ///
    /// # Example
    ///
    /// ```ignore
    /// use modkit_db::secure::{SecureConn, TxConfig, TxIsolationLevel};
    ///
    /// // In a domain service requiring serializable isolation:
    /// pub async fn reconcile_accounts(
    ///     db: &SecureConn,
    ///     repo: &AccountsRepo,
    /// ) -> anyhow::Result<Result<ReconciliationResult, DomainError>> {
    ///     let cfg = TxConfig::serializable();
    ///
    ///     db.transaction_with_config(cfg, |conn| async move {
    ///         let accounts = repo.find_all_pending(conn).await?;
    ///         for account in accounts {
    ///             repo.reconcile(conn, &account).await?;
    ///         }
    ///         Ok(Ok(ReconciliationResult { processed: accounts.len() }))
    ///     }).await
    /// }
    /// ```
    ///
    /// # Backend Notes
    ///
    /// - **`PostgreSQL`**: Full support for all isolation levels and access modes.
    /// - **MySQL/InnoDB**: Full support for all isolation levels and access modes.
    /// - **`SQLite`**: Only supports `Serializable` isolation. Other levels are
    ///   mapped to `Serializable`. Read-only mode is a hint only.
    ///
    /// # Errors
    ///
    /// Returns `Err(anyhow::Error)` if:
    /// - The transaction cannot be started with the specified configuration
    /// - A database operation fails (transaction is rolled back)
    /// - The commit fails
    pub async fn transaction_with_config<T, F>(&self, cfg: TxConfig, f: F) -> anyhow::Result<T>
    where
        T: Send + 'static,
        F: for<'c> FnOnce(
                &'c DatabaseTransaction,
            )
                -> Pin<Box<dyn Future<Output = anyhow::Result<T>> + Send + 'c>>
            + Send,
    {
        let isolation: Option<IsolationLevel> = cfg.isolation.map(Into::into);
        let access_mode: Option<AccessMode> = cfg.access_mode.map(Into::into);

        self.conn
            .transaction_with_config::<_, T, DbErr>(
                |txn| {
                    let fut = f(txn);
                    Box::pin(async move {
                        fut.await.map_err(|e| {
                            DbErr::Custom(format!("transaction callback failed: {e:#}"))
                        })
                    })
                },
                isolation,
                access_mode,
            )
            .await
            .map_err(|e| anyhow::anyhow!("transaction_with_config failed: {e}"))
    }

    /// Execute a closure inside a typed domain transaction.
    ///
    /// This method returns [`TxError<E>`] which distinguishes domain errors from
    /// infrastructure errors, allowing callers to handle them appropriately.
    ///
    /// # Error Handling
    ///
    /// - Domain errors returned from the closure are wrapped in `TxError::Domain(e)`
    /// - Database infrastructure errors are wrapped in `TxError::Infra(InfraError)`
    ///
    /// Use [`TxError::into_domain`] to convert the result into your domain error type.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use modkit_db::secure::SecureConn;
    ///
    /// async fn create_user(db: &SecureConn, repo: &UsersRepo, user: User) -> Result<User, DomainError> {
    ///     db.in_transaction(move |tx| Box::pin(async move {
    ///         if repo.exists(tx, user.id).await? {
    ///             return Err(DomainError::already_exists(user.id));
    ///         }
    ///         repo.create(tx, user).await
    ///     }))
    ///     .await
    ///     .map_err(|e| e.into_domain(DomainError::database_infra))
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err(TxError<E>)` if:
    /// - The callback returns a domain error (`TxError::Domain(E)`).
    /// - The transaction fails due to a database/infrastructure error (`TxError::Infra(InfraError)`).
    pub async fn in_transaction<T, E, F>(&self, f: F) -> Result<T, TxError<E>>
    where
        T: Send + 'static,
        E: std::fmt::Debug + std::fmt::Display + Send + 'static,
        F: for<'c> FnOnce(
                &'c DatabaseTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
    {
        self.conn
            .transaction::<_, T, TxError<E>>(|txn| {
                let fut = f(txn);
                Box::pin(async move { fut.await.map_err(TxError::Domain) })
            })
            .await
            .map_err(|e| match e {
                sea_orm::TransactionError::Transaction(tx_err) => tx_err,
                sea_orm::TransactionError::Connection(db_err) => {
                    TxError::Infra(InfraError::new(db_err.to_string()))
                }
            })
    }

    /// Execute a typed domain transaction with automatic infrastructure error mapping.
    ///
    /// This is a convenience wrapper around [`in_transaction`](Self::in_transaction) that
    /// automatically converts [`TxError`] into the domain error type using the provided
    /// mapping function for infrastructure errors.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use modkit_db::secure::SecureConn;
    ///
    /// async fn create_user(db: &SecureConn, repo: &UsersRepo, user: User) -> Result<User, DomainError> {
    ///     db.in_transaction_mapped(DomainError::database_infra, move |tx| Box::pin(async move {
    ///         if repo.exists(tx, user.id).await? {
    ///             return Err(DomainError::already_exists(user.id));
    ///         }
    ///         repo.create(tx, user).await
    ///     })).await
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err(E)` if:
    /// - The callback returns a domain error (`E`).
    /// - The transaction fails due to a database/infrastructure error, mapped via `map_infra`.
    pub async fn in_transaction_mapped<T, E, F, M>(&self, map_infra: M, f: F) -> Result<T, E>
    where
        T: Send + 'static,
        E: std::fmt::Debug + std::fmt::Display + Send + 'static,
        M: FnOnce(InfraError) -> E + Send,
        F: for<'c> FnOnce(
                &'c DatabaseTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
    {
        self.in_transaction(f)
            .await
            .map_err(|tx_err| tx_err.into_domain(map_infra))
    }
}
