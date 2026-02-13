//! High-level secure database wrapper for ergonomic, type-safe access.
//!
//! This module provides `SecureConn`, a wrapper around a private `SeaORM` connection
//! that enforces access control policies on all operations.
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
//!             .one(self.db)
//!             .await?;
//!         Ok(user.map(Into::into))
//!     }
//!
//!     pub async fn find_all(&self) -> Result<Vec<User>, ScopeError> {
//!         let users = self.db
//!             .find::<user::Entity>()?
//!             .all(self.db)
//!             .await?;
//!         Ok(users.into_iter().map(Into::into).collect())
//!     }
//!
//!     pub async fn update_status(&self, status: String) -> Result<u64, ScopeError> {
//!         let result = self.db
//!             .update_many::<user::Entity>()?
//!             .col_expr(user::Column::Status, Expr::value(status))
//!             .exec(self.db)
//!             .await?;
//!         Ok(result.rows_affected)
//!     }
//! }
//! ```

use std::{future::Future, pin::Pin};

use sea_orm::{
    AccessMode, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    IsolationLevel, QueryFilter, TransactionTrait, sea_query::Expr,
};
use uuid::Uuid;

use crate::secure::tx_error::{InfraError, TxError};

use modkit_security::AccessScope;

use crate::secure::tx_config::TxConfig;

use crate::secure::{ScopableEntity, ScopeError, Scoped, SecureEntityExt, SecureSelect};

use crate::secure::db_ops::{SecureDeleteExt, SecureDeleteMany, SecureUpdateExt, SecureUpdateMany};

/// Secure transaction wrapper (capability).
///
/// This type intentionally does not expose any raw transaction or executor API.
pub struct SecureTx<'a> {
    pub(crate) tx: &'a DatabaseTransaction,
}

impl<'a> SecureTx<'a> {
    #[must_use]
    pub(crate) fn new(tx: &'a DatabaseTransaction) -> Self {
        Self { tx }
    }
}

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
///             .one(self.db)
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
/// - Modules cannot access raw database connections
pub struct SecureConn {
    pub(crate) conn: DatabaseConnection,
}

impl SecureConn {
    /// Create a new secure database connection wrapper.
    /// Internal-only accessor to the raw database connection.
    ///
    /// # Security
    ///
    /// This MUST NOT be exposed publicly. Any public raw access to the underlying database
    /// handle would allow bypassing scoping and tenant isolation.
    #[must_use]
    pub(crate) fn conn_internal(&self) -> &DatabaseConnection {
        &self.conn
    }

    /// Return database engine identifier for tracing / logging.
    #[must_use]
    pub fn db_engine(&self) -> &'static str {
        use sea_orm::DbBackend;

        match self.conn.get_database_backend() {
            DbBackend::Postgres => "postgres",
            DbBackend::MySql => "mysql",
            DbBackend::Sqlite => "sqlite",
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
    ///     .all(db)
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
    ///     .one(db)
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
    ///     .exec(db)
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
    ///     .exec(db)
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

    /// Create a scoped insert builder with `on_conflict()` support.
    ///
    /// Unlike the simpler `insert()` method, this returns a builder that allows
    /// setting `on_conflict()` for upsert semantics while still enforcing
    /// tenant validation through the secure typestate pattern.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use sea_orm::sea_query::OnConflict;
    ///
    /// let scope = AccessScope::tenants_only(vec![tenant_id]);
    /// let am = settings::ActiveModel {
    ///     tenant_id: Set(tenant_id),
    ///     user_id: Set(user_id),
    ///     theme: Set(Some("dark".to_string())),
    ///     ..Default::default()
    /// };
    ///
    /// db.insert_one(&scope, am)?
    ///     .on_conflict(
    ///         OnConflict::columns([Column::TenantId, Column::UserId])
    ///             .update_columns([Column::Theme])
    ///             .to_owned()
    ///     )
    ///     .exec(db)
    ///     .await?;
    /// ```
    ///
    /// # Errors
    ///
    /// - `ScopeError::Invalid` if `tenant_id` is not set for tenant-scoped entities
    /// - `ScopeError::TenantNotInScope` if `tenant_id` is not in the provided scope
    #[allow(clippy::needless_pass_by_value)] // We clone for insert and borrow for validation
    pub fn insert_one<E>(
        &self,
        scope: &AccessScope,
        am: E::ActiveModel,
    ) -> Result<crate::secure::SecureInsertOne<E::ActiveModel, Scoped>, ScopeError>
    where
        E: ScopableEntity + EntityTrait,
        E::Column: ColumnTrait + Copy,
        E::ActiveModel: sea_orm::ActiveModelTrait<Entity = E> + Send,
    {
        use crate::secure::SecureInsertExt;
        E::insert(am.clone()).secure().scope_with_model(scope, &am)
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
        crate::secure::secure_insert::<E>(am, scope, self).await
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
    ///     .one(db)
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
        E::Model: sea_orm::IntoActiveModel<E::ActiveModel> + sea_orm::ModelTrait<Entity = E>,
    {
        crate::secure::secure_update_with_scope::<E>(am, scope, id, self).await
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
            .exec(self)
            .await?;

        Ok(result.rows_affected > 0)
    }

    // ========================================================================
    // Transaction support
    // ========================================================================

    /// Execute a closure inside a database transaction.
    ///
    /// This method starts a `SeaORM` transaction, provides the transaction handle
    /// to the closure as `&SecureTx`, and handles commit/rollback.
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
    /// # Security
    ///
    /// This method **consumes** `self` and returns it after the transaction completes.
    /// This prevents accidental use of the outer connection inside the transaction,
    /// making transaction bypass impossible by construction.
    ///
    /// Only `&SecureTx` is available inside the closure, ensuring all operations
    /// execute within the transaction scope.
    ///
    /// # Returns
    ///
    /// Returns a tuple `(Self, Result<()>)` where:
    /// - `Self` is the connection (always returned, even on error)
    /// - `Result<()>` indicates transaction success or failure
    ///
    /// # Errors
    ///
    /// The `Result` component is `Err(anyhow::Error)` if:
    /// - The transaction cannot be started
    /// - A database operation fails (transaction is rolled back)
    /// - The commit fails
    pub async fn transaction<F>(self, f: F) -> (Self, anyhow::Result<()>)
    where
        F: for<'a> FnOnce(
                &'a SecureTx<'a>,
            )
                -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>
            + Send,
    {
        let txn = match self.conn_internal().begin().await {
            Ok(t) => t,
            Err(e) => return (self, Err(e.into())),
        };
        let tx = SecureTx::new(&txn);

        let res = f(&tx).await;

        match res {
            Ok(()) => match txn.commit().await {
                Ok(()) => (self, Ok(())),
                Err(e) => (self, Err(e.into())),
            },
            Err(e) => {
                _ = txn.rollback().await;
                (self, Err(e))
            }
        }
    }

    /// Execute a transaction and return both the connection and a result value.
    ///
    /// This method consumes `self` and returns both the connection and the result
    /// from the transaction closure. Use this when you need to return data from
    /// within the transaction.
    ///
    /// # Security
    ///
    /// Like [`transaction`](Self::transaction), this method prevents transaction bypass
    /// by consuming `self`, making it impossible to access the outer connection
    /// inside the transaction closure.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let (conn, result) = conn.transaction_with(|tx| {
    ///     Box::pin(async move {
    ///         let user = repo.create(tx, &scope, new_user).await?;
    ///         Ok(user)
    ///     })
    /// }).await;
    /// let user = result?;
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a tuple `(Self, Result<T>)` where:
    /// - `Self` is the connection (always returned)
    /// - `Result<T>` contains the transaction result or error
    ///
    /// # Errors
    ///
    /// The `Result` component is `Err(anyhow::Error)` if:
    /// - The transaction cannot be started
    /// - A database operation fails (transaction is rolled back)
    /// - The commit fails
    pub async fn transaction_with<T, F>(self, f: F) -> (Self, anyhow::Result<T>)
    where
        T: Send + 'static,
        F: for<'a> FnOnce(
                &'a SecureTx<'a>,
            )
                -> Pin<Box<dyn Future<Output = anyhow::Result<T>> + Send + 'a>>
            + Send,
    {
        let txn = match self.conn_internal().begin().await {
            Ok(t) => t,
            Err(e) => return (self, Err(e.into())),
        };
        let tx = SecureTx::new(&txn);

        let res = f(&tx).await;

        match res {
            Ok(v) => match txn.commit().await {
                Ok(()) => (self, Ok(v)),
                Err(e) => (self, Err(e.into())),
            },
            Err(e) => {
                _ = txn.rollback().await;
                (self, Err(e))
            }
        }
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
    /// # Security
    ///
    /// This method consumes `self` and returns both the connection and result,
    /// preventing transaction bypass by making the outer connection unavailable
    /// inside the closure.
    ///
    /// # Returns
    ///
    /// Returns a tuple `(Self, Result<T>)` where:
    /// - `Self` is the connection (always returned)
    /// - `Result<T>` contains the transaction result or error
    ///
    /// # Errors
    ///
    /// The `Result` component is `Err(anyhow::Error)` if:
    /// - The transaction cannot be started with the specified configuration
    /// - A database operation fails (transaction is rolled back)
    /// - The commit fails
    pub async fn transaction_with_config<T, F>(
        self,
        cfg: TxConfig,
        f: F,
    ) -> (Self, anyhow::Result<T>)
    where
        T: Send + 'static,
        F: for<'a> FnOnce(
                &'a SecureTx<'a>,
            )
                -> Pin<Box<dyn Future<Output = anyhow::Result<T>> + Send + 'a>>
            + Send,
    {
        let isolation: Option<IsolationLevel> = cfg.isolation.map(Into::into);
        let access_mode: Option<AccessMode> = cfg.access_mode.map(Into::into);

        let txn = match self
            .conn_internal()
            .begin_with_config(isolation, access_mode)
            .await
        {
            Ok(t) => t,
            Err(e) => return (self, Err(e.into())),
        };
        let tx = SecureTx::new(&txn);

        let res = f(&tx).await;

        match res {
            Ok(v) => match txn.commit().await {
                Ok(()) => (self, Ok(v)),
                Err(e) => (self, Err(e.into())),
            },
            Err(e) => {
                _ = txn.rollback().await;
                (self, Err(e))
            }
        }
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
    /// # Security
    ///
    /// This method consumes `self` and returns both the connection and result,
    /// preventing transaction bypass by making the outer connection unavailable
    /// inside the closure.
    ///
    /// # Returns
    ///
    /// Returns a tuple `(Self, Result<T, TxError<E>>)` where:
    /// - `Self` is the connection (always returned)
    /// - `Result<T, TxError<E>>` contains the transaction result or error
    ///
    /// # Errors
    ///
    /// The `Result` component is `Err(TxError<E>)` if:
    /// - The callback returns a domain error (`TxError::Domain(E)`).
    /// - The transaction fails due to a database/infrastructure error (`TxError::Infra(InfraError)`).
    pub async fn in_transaction<T, E, F>(self, f: F) -> (Self, Result<T, TxError<E>>)
    where
        T: Send + 'static,
        E: std::fmt::Debug + std::fmt::Display + Send + 'static,
        F: for<'a> FnOnce(
                &'a SecureTx<'a>,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'a>>
            + Send,
    {
        let txn = match self.conn_internal().begin().await {
            Ok(t) => t,
            Err(e) => return (self, Err(TxError::Infra(InfraError::new(e.to_string())))),
        };
        let tx = SecureTx::new(&txn);

        let res = f(&tx).await;

        match res {
            Ok(v) => match txn.commit().await {
                Ok(()) => (self, Ok(v)),
                Err(e) => (self, Err(TxError::Infra(InfraError::new(e.to_string())))),
            },
            Err(e) => {
                _ = txn.rollback().await;
                (self, Err(TxError::Domain(e)))
            }
        }
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
    /// # Security
    ///
    /// This method consumes `self` and returns both the connection and result,
    /// preventing transaction bypass.
    ///
    /// # Returns
    ///
    /// Returns a tuple `(Self, Result<T, E>)` where:
    /// - `Self` is the connection (always returned)
    /// - `Result<T, E>` contains the transaction result or mapped error
    ///
    /// # Errors
    ///
    /// The `Result` component is `Err(E)` if:
    /// - The callback returns a domain error (`E`).
    /// - The transaction fails due to a database/infrastructure error, mapped via `map_infra`.
    pub async fn in_transaction_mapped<T, E, F, M>(self, map_infra: M, f: F) -> (Self, Result<T, E>)
    where
        T: Send + 'static,
        E: std::fmt::Debug + std::fmt::Display + Send + 'static,
        M: FnOnce(InfraError) -> E + Send,
        F: for<'a> FnOnce(
                &'a SecureTx<'a>,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'a>>
            + Send,
    {
        let (conn, result) = self.in_transaction(f).await;
        (conn, result.map_err(|tx_err| tx_err.into_domain(map_infra)))
    }
}
