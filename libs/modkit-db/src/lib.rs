#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! `ModKit` Database abstraction crate.
//!
//! This crate provides a unified interface for working with different databases
//! (`SQLite`, `PostgreSQL`, `MySQL`) through `SQLx`, with optional `SeaORM` integration.
//! It emphasizes typed connection options over DSN string manipulation and
//! implements strict security controls (e.g., `SQLite` PRAGMA whitelist).
//!
//! # Features
//! - `pg`, `mysql`, `sqlite`: enable `SQLx` backends
//! - `sea-orm`: add `SeaORM` integration for type-safe operations
//!
//! # New Architecture
//! The crate now supports:
//! - Typed `DbConnectOptions` using sqlx `ConnectOptions` (no DSN string building)
//! - Per-module database factories with configuration merging
//! - `SQLite` PRAGMA whitelist for security
//! - Environment variable expansion in passwords and DSNs
//!
//! # Example (`DbManager` API)
//! ```rust,no_run
//! use modkit_db::{DbManager, GlobalDatabaseConfig, DbConnConfig};
//! use figment::{Figment, providers::Serialized};
//! use std::path::PathBuf;
//! use std::sync::Arc;
//!
//! // Create configuration using Figment
//! let figment = Figment::new()
//!     .merge(Serialized::defaults(serde_json::json!({
//!         "db": {
//!             "servers": {
//!                 "main": {
//!                     "host": "localhost",
//!                     "port": 5432,
//!                     "user": "app",
//!                     "password": "${DB_PASSWORD}",
//!                     "dbname": "app_db"
//!                 }
//!             }
//!         },
//!         "test_module": {
//!             "database": {
//!                 "server": "main",
//!                 "dbname": "module_db"
//!             }
//!         }
//!     })));
//!
//! // Create DbManager
//! let home_dir = PathBuf::from("/app/data");
//! let db_manager = Arc::new(DbManager::from_figment(figment, home_dir).unwrap());
//!
//! // Use in runtime with DbOptions::Manager(db_manager)
//! // Modules can then use: ctx.db_required_async().await?
//! ```

#![cfg_attr(
    not(any(feature = "pg", feature = "mysql", feature = "sqlite")),
    allow(
        unused_imports,
        unused_variables,
        dead_code,
        unreachable_code,
        unused_lifetimes,
        clippy::unused_async,
    )
)]

// Re-export key types for public API
pub use advisory_locks::{DbLockGuard, LockConfig};

pub use sea_orm::ConnectionTrait as DbConnTrait;

// Core modules
pub mod advisory_locks;
pub mod config;
pub mod manager;
pub mod odata;
pub mod options;
pub mod secure;

// Internal modules
mod pool_opts;
#[cfg(feature = "sqlite")]
mod sqlite;

// Re-export important types from new modules
pub use config::{DbConnConfig, GlobalDatabaseConfig, PoolCfg};
pub use manager::DbManager;
pub use options::{
    ConnectionOptionsError, DbConnectOptions, build_db_handle, redact_credentials_in_dsn,
};

use std::time::Duration;

// Internal imports
#[cfg(any(feature = "pg", feature = "mysql", feature = "sqlite"))]
use pool_opts::ApplyPoolOpts;
#[cfg(feature = "sqlite")]
use sqlite::{Pragmas, extract_sqlite_pragmas, is_memory_dsn, prepare_sqlite_path};

// Used for parsing SQLite DSN query parameters

#[cfg(feature = "mysql")]
use sea_orm::sqlx::{MySql, MySqlPool, mysql::MySqlPoolOptions};
#[cfg(feature = "pg")]
use sea_orm::sqlx::{PgPool, Postgres, postgres::PgPoolOptions};
#[cfg(feature = "sqlite")]
use sea_orm::sqlx::{Sqlite, SqlitePool, sqlite::SqlitePoolOptions};

use sea_orm::DatabaseConnection;
#[cfg(feature = "mysql")]
use sea_orm::SqlxMySqlConnector;
#[cfg(feature = "pg")]
use sea_orm::SqlxPostgresConnector;
#[cfg(feature = "sqlite")]
use sea_orm::SqlxSqliteConnector;

use thiserror::Error;

/// Library-local result type.
pub type Result<T> = std::result::Result<T, DbError>;

/// Typed error for the DB handle and helpers.
#[derive(Debug, Error)]
pub enum DbError {
    #[error("Unknown DSN: {0}")]
    UnknownDsn(String),

    #[error("Feature not enabled: {0}")]
    FeatureDisabled(&'static str),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Configuration conflict: {0}")]
    ConfigConflict(String),

    #[error("Invalid SQLite PRAGMA parameter '{key}': {message}")]
    InvalidSqlitePragma { key: String, message: String },

    #[error("Unknown SQLite PRAGMA parameter: {0}")]
    UnknownSqlitePragma(String),

    #[error("Invalid connection parameter: {0}")]
    InvalidParameter(String),

    #[error("SQLite pragma error: {0}")]
    SqlitePragma(String),

    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),

    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[cfg(any(feature = "pg", feature = "mysql", feature = "sqlite"))]
    #[error(transparent)]
    Sqlx(#[from] sea_orm::sqlx::Error),

    #[error(transparent)]
    Sea(#[from] sea_orm::DbErr),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    // make advisory_locks errors flow into DbError via `?`
    #[error(transparent)]
    Lock(#[from] advisory_locks::DbLockError),

    // Convert from the old ConnectionOptionsError
    #[error(transparent)]
    ConnectionOptions(#[from] ConnectionOptionsError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Supported engines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DbEngine {
    Postgres,
    MySql,
    Sqlite,
}

/// Connection options.
/// Extended to cover common sqlx pool knobs; each driver applies the subset it supports.
#[derive(Clone, Debug)]
pub struct ConnectOpts {
    /// Maximum number of connections in the pool.
    pub max_conns: Option<u32>,
    /// Minimum number of connections in the pool.
    pub min_conns: Option<u32>,
    /// Timeout to acquire a connection from the pool.
    pub acquire_timeout: Option<Duration>,
    /// Idle timeout before a connection is closed.
    pub idle_timeout: Option<Duration>,
    /// Maximum lifetime for a connection.
    pub max_lifetime: Option<Duration>,
    /// Test connection health before acquire.
    pub test_before_acquire: bool,
    /// For `SQLite` file DSNs, create parent directories if missing.
    pub create_sqlite_dirs: bool,
}
impl Default for ConnectOpts {
    fn default() -> Self {
        Self {
            max_conns: Some(10),
            min_conns: None,
            acquire_timeout: Some(Duration::from_secs(30)),
            idle_timeout: None,
            max_lifetime: None,
            test_before_acquire: false,

            create_sqlite_dirs: true,
        }
    }
}

/// One concrete sqlx pool.
#[derive(Clone, Debug)]
pub enum DbPool {
    #[cfg(feature = "pg")]
    Postgres(PgPool),
    #[cfg(feature = "mysql")]
    MySql(MySqlPool),
    #[cfg(feature = "sqlite")]
    Sqlite(SqlitePool),
}

/// Database transaction wrapper (lifetime-bound to the pool).
pub enum DbTransaction<'a> {
    #[cfg(feature = "pg")]
    Postgres(sea_orm::sqlx::Transaction<'a, Postgres>),
    #[cfg(feature = "mysql")]
    MySql(sea_orm::sqlx::Transaction<'a, MySql>),
    #[cfg(feature = "sqlite")]
    Sqlite(sea_orm::sqlx::Transaction<'a, Sqlite>),
    // When no concrete DB feature is enabled, keep a variant to tie `'a` so
    // the type still compiles and can be referenced in signatures.
    #[cfg(not(any(feature = "pg", feature = "mysql", feature = "sqlite")))]
    _Phantom(std::marker::PhantomData<&'a ()>),
}

impl DbTransaction<'_> {
    /// Commit the transaction.
    ///
    /// # Errors
    /// Returns an error if the commit operation fails.
    pub async fn commit(self) -> Result<()> {
        match self {
            #[cfg(feature = "pg")]
            DbTransaction::Postgres(tx) => tx.commit().await.map_err(Into::into),
            #[cfg(feature = "mysql")]
            DbTransaction::MySql(tx) => tx.commit().await.map_err(Into::into),
            #[cfg(feature = "sqlite")]
            DbTransaction::Sqlite(tx) => tx.commit().await.map_err(Into::into),
            #[cfg(not(any(feature = "pg", feature = "mysql", feature = "sqlite")))]
            DbTransaction::_Phantom(_) => Ok(()),
        }
    }

    /// Roll back the transaction.
    ///
    /// # Errors
    /// Returns an error if the rollback operation fails.
    pub async fn rollback(self) -> Result<()> {
        match self {
            #[cfg(feature = "pg")]
            DbTransaction::Postgres(tx) => tx.rollback().await.map_err(Into::into),
            #[cfg(feature = "mysql")]
            DbTransaction::MySql(tx) => tx.rollback().await.map_err(Into::into),
            #[cfg(feature = "sqlite")]
            DbTransaction::Sqlite(tx) => tx.rollback().await.map_err(Into::into),
            #[cfg(not(any(feature = "pg", feature = "mysql", feature = "sqlite")))]
            DbTransaction::_Phantom(_) => Ok(()),
        }
    }
}

/// Main handle.
#[derive(Debug, Clone)]
pub struct DbHandle {
    engine: DbEngine,
    pool: DbPool,
    dsn: String,
    sea: DatabaseConnection,
}

#[cfg(feature = "sqlite")]
const DEFAULT_SQLITE_BUSY_TIMEOUT: i32 = 5000;

impl DbHandle {
    /// Detect engine by DSN.
    ///
    /// Note: we only check scheme prefixes and don't mutate the tail (credentials etc.).
    ///
    /// # Errors
    /// Returns `DbError::UnknownDsn` if the DSN scheme is not recognized.
    pub fn detect(dsn: &str) -> Result<DbEngine> {
        // Trim only leading spaces/newlines to be forgiving with env files.
        let s = dsn.trim_start();

        // Explicit, case-sensitive checks for common schemes.
        // Add more variants as needed (e.g., postgres+unix://).
        if s.starts_with("postgres://") || s.starts_with("postgresql://") {
            Ok(DbEngine::Postgres)
        } else if s.starts_with("mysql://") {
            Ok(DbEngine::MySql)
        } else if s.starts_with("sqlite:") || s.starts_with("sqlite://") {
            Ok(DbEngine::Sqlite)
        } else {
            Err(DbError::UnknownDsn(dsn.to_owned()))
        }
    }

    /// Connect and build handle.
    ///
    /// # Errors
    /// Returns an error if the connection fails or the DSN is invalid.
    pub async fn connect(dsn: &str, opts: ConnectOpts) -> Result<Self> {
        let engine = Self::detect(dsn)?;
        match engine {
            #[cfg(feature = "pg")]
            DbEngine::Postgres => {
                let o = PgPoolOptions::new().apply(&opts);
                let pool = o.connect(dsn).await?;
                let sea = SqlxPostgresConnector::from_sqlx_postgres_pool(pool.clone());
                Ok(Self {
                    engine,
                    pool: DbPool::Postgres(pool),
                    dsn: dsn.to_owned(),
                    sea,
                })
            }
            #[cfg(not(feature = "pg"))]
            DbEngine::Postgres => Err(DbError::FeatureDisabled("PostgreSQL feature not enabled")),
            #[cfg(feature = "mysql")]
            DbEngine::MySql => {
                let o = MySqlPoolOptions::new().apply(&opts);
                let pool = o.connect(dsn).await?;
                let sea = SqlxMySqlConnector::from_sqlx_mysql_pool(pool.clone());
                Ok(Self {
                    engine,
                    pool: DbPool::MySql(pool),
                    dsn: dsn.to_owned(),
                    sea,
                })
            }
            #[cfg(not(feature = "mysql"))]
            DbEngine::MySql => Err(DbError::FeatureDisabled("MySQL feature not enabled")),
            #[cfg(feature = "sqlite")]
            DbEngine::Sqlite => {
                let dsn = prepare_sqlite_path(dsn, opts.create_sqlite_dirs)?;

                // Extract SQLite PRAGMA parameters from DSN
                let (clean_dsn, pairs) = extract_sqlite_pragmas(&dsn);
                let pragmas = Pragmas::from_pairs(&pairs);

                // Build pool options with shared trait
                let mut o = SqlitePoolOptions::new().apply(&opts);

                // Apply SQLite pragmas with special handling for in-memory databases
                let is_memory = is_memory_dsn(&clean_dsn);
                o = o.after_connect(move |conn, _meta| {
                    let pragmas = pragmas.clone();
                    Box::pin(async move {
                        // Apply journal_mode
                        let journal_mode = if let Some(mode) = &pragmas.journal_mode {
                            mode.as_sql()
                        } else if let Some(wal_toggle) = pragmas.wal_toggle {
                            if wal_toggle { "WAL" } else { "DELETE" }
                        } else if is_memory {
                            // Default: DELETE for memory, WAL for file
                            "DELETE"
                        } else {
                            "WAL"
                        };

                        let stmt = format!("PRAGMA journal_mode = {journal_mode}");
                        sea_orm::sqlx::query(&stmt).execute(&mut *conn).await?;

                        // Apply synchronous mode
                        let sync_mode = pragmas
                            .synchronous
                            .as_ref()
                            .map_or("NORMAL", |s| s.as_sql());
                        let stmt = format!("PRAGMA synchronous = {sync_mode}");
                        sea_orm::sqlx::query(&stmt).execute(&mut *conn).await?;

                        // Apply busy timeout (skip for in-memory databases)
                        if !is_memory {
                            let timeout = pragmas
                                .busy_timeout_ms
                                .unwrap_or(DEFAULT_SQLITE_BUSY_TIMEOUT.into());
                            sea_orm::sqlx::query("PRAGMA busy_timeout = ?")
                                .bind(timeout)
                                .execute(&mut *conn)
                                .await?;
                        }

                        Ok(())
                    })
                });

                let pool = o.connect(&clean_dsn).await?;
                let sea = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool.clone());

                Ok(Self {
                    engine,
                    pool: DbPool::Sqlite(pool),
                    dsn: clean_dsn,
                    sea,
                })
            }
            #[cfg(not(feature = "sqlite"))]
            DbEngine::Sqlite => Err(DbError::FeatureDisabled("SQLite feature not enabled")),
        }
    }

    /// Graceful pool close. (Dropping the pool also closes it; this just makes it explicit.)
    pub async fn close(self) {
        match self.pool {
            #[cfg(feature = "pg")]
            DbPool::Postgres(p) => p.close().await,
            #[cfg(feature = "mysql")]
            DbPool::MySql(p) => p.close().await,
            #[cfg(feature = "sqlite")]
            DbPool::Sqlite(p) => p.close().await,
        }
    }

    /// Get the backend.
    #[must_use]
    pub fn engine(&self) -> DbEngine {
        self.engine
    }

    /// Get the DSN used for this connection.
    #[must_use]
    pub fn dsn(&self) -> &str {
        &self.dsn
    }

    // --- sqlx accessors ---
    #[cfg(feature = "pg")]
    #[must_use]
    pub fn sqlx_postgres(&self) -> Option<&PgPool> {
        match self.pool {
            DbPool::Postgres(ref p) => Some(p),
            #[cfg(any(feature = "mysql", feature = "sqlite"))]
            _ => None,
        }
    }
    #[cfg(feature = "mysql")]
    #[must_use]
    pub fn sqlx_mysql(&self) -> Option<&MySqlPool> {
        match self.pool {
            DbPool::MySql(ref p) => Some(p),
            #[cfg(any(feature = "pg", feature = "sqlite"))]
            _ => None,
        }
    }
    #[cfg(feature = "sqlite")]
    #[must_use]
    pub fn sqlx_sqlite(&self) -> Option<&SqlitePool> {
        match self.pool {
            DbPool::Sqlite(ref p) => Some(p),
            #[cfg(any(feature = "pg", feature = "mysql"))]
            _ => None,
        }
    }

    // --- SeaORM accessor ---

    /// Create a secure database connection for scoped operations.
    ///
    /// Returns a `SecureConn` wrapper that requires `SecurityCtx` for each operation.
    /// This is the **recommended** way to access the database from application code.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use modkit_db::secure::{SecurityCtx, AccessScope};
    ///
    /// let secure_conn = db_handle.sea_secure();
    ///
    /// // Security context from API request
    /// let ctx = SecurityCtx::for_tenants(vec![tenant_id], user_id);
    ///
    /// // All queries require context and are automatically scoped
    /// let users = secure_conn.find::<user::Entity>(&ctx)?
    ///     .all(secure_conn.conn())
    ///     .await?;
    /// ```
    #[must_use]
    pub fn sea_secure(&self) -> crate::secure::SecureConn {
        crate::secure::SecureConn::new(self.sea.clone())
    }

    /// **INSECURE**: Get raw `SeaORM` connection (bypasses all security).
    ///
    /// This method is **only available** when compiled with `--features insecure-escape`.
    /// It provides direct access to the database connection, bypassing all tenant
    /// isolation and access control.
    ///
    /// # Security Warning
    ///
    /// This completely bypasses the secure ORM layer. Use only for:
    /// - Administrative maintenance tools
    /// - Database migrations
    /// - Emergency data recovery
    /// - Internal infrastructure code
    ///
    /// **Never use in application/business logic code.**
    ///
    /// # Example
    ///
    /// ```ignore
    /// #[cfg(feature = "insecure-escape")]
    /// async fn admin_operation(db: &DbHandle) {
    ///     let raw_conn = db.sea();  // No security!
    ///     // Direct database access...
    /// }
    /// ```
    #[cfg(feature = "insecure-escape")]
    pub fn sea(&self) -> DatabaseConnection {
        tracing::warn!(
            target: "security",
            "DbHandle::sea() called - bypassing secure ORM layer"
        );
        self.sea.clone()
    }

    // --- Transaction helpers (engine-specific) ---

    /// Execute a closure within a `PostgreSQL` transaction.
    ///
    /// # Errors
    /// Returns an error if the transaction fails or the closure returns an error.
    #[cfg(feature = "pg")]
    pub async fn with_pg_tx<F, T>(&self, f: F) -> Result<T>
    where
        F: for<'a> FnOnce(
            &'a mut sea_orm::sqlx::Transaction<'_, Postgres>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T>> + Send + 'a>,
        >,
    {
        let pool = self
            .sqlx_postgres()
            .ok_or(DbError::FeatureDisabled("not a postgres pool"))?;
        let mut tx = pool.begin().await?;
        let res = f(&mut tx).await;
        match res {
            Ok(v) => {
                tx.commit().await?;
                Ok(v)
            }
            Err(e) => {
                // Best-effort rollback; keep the original error.
                let _ = tx.rollback().await;
                Err(e)
            }
        }
    }

    /// Execute a closure within a `MySQL` transaction.
    ///
    /// # Errors
    /// Returns an error if the transaction fails or the closure returns an error.
    #[cfg(feature = "mysql")]
    pub async fn with_mysql_tx<F, T>(&self, f: F) -> Result<T>
    where
        F: for<'a> FnOnce(
            &'a mut sea_orm::sqlx::Transaction<'_, MySql>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T>> + Send + 'a>,
        >,
    {
        let pool = self
            .sqlx_mysql()
            .ok_or(DbError::FeatureDisabled("not a mysql pool"))?;
        let mut tx = pool.begin().await?;
        let res = f(&mut tx).await;
        match res {
            Ok(v) => {
                tx.commit().await?;
                Ok(v)
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Err(e)
            }
        }
    }

    /// Execute a closure within a `SQLite` transaction.
    ///
    /// # Errors
    /// Returns an error if the transaction fails or the closure returns an error.
    #[cfg(feature = "sqlite")]
    pub async fn with_sqlite_tx<F, T>(&self, f: F) -> Result<T>
    where
        F: for<'a> FnOnce(
            &'a mut sea_orm::sqlx::Transaction<'_, Sqlite>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T>> + Send + 'a>,
        >,
    {
        let pool = self
            .sqlx_sqlite()
            .ok_or(DbError::FeatureDisabled("not a sqlite pool"))?;
        let mut tx = pool.begin().await?;
        let res = f(&mut tx).await;
        match res {
            Ok(v) => {
                tx.commit().await?;
                Ok(v)
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Err(e)
            }
        }
    }

    // --- Advisory locks ---

    /// Acquire an advisory lock with the given key and module namespace.
    ///
    /// # Errors
    /// Returns an error if the lock cannot be acquired.
    pub async fn lock(&self, module: &str, key: &str) -> Result<DbLockGuard> {
        let lock_manager =
            advisory_locks::LockManager::new(self.engine, self.pool.clone(), self.dsn.clone());
        let guard = lock_manager.lock(module, key).await?;
        Ok(guard)
    }

    /// Try to acquire an advisory lock with configurable retry/backoff policy.
    ///
    /// # Errors
    /// Returns an error if an unrecoverable lock error occurs.
    pub async fn try_lock(
        &self,
        module: &str,
        key: &str,
        config: LockConfig,
    ) -> Result<Option<DbLockGuard>> {
        let lock_manager =
            advisory_locks::LockManager::new(self.engine, self.pool.clone(), self.dsn.clone());
        let res = lock_manager.try_lock(module, key, config).await?;
        Ok(res)
    }

    // --- Generic transaction begin (returns proper enum with lifetime) ---

    /// Begin a transaction (returns appropriate transaction type based on backend).
    ///
    /// # Errors
    /// Returns an error if the transaction cannot be started.
    pub async fn begin(&self) -> Result<DbTransaction<'_>> {
        match &self.pool {
            #[cfg(feature = "pg")]
            DbPool::Postgres(pool) => {
                let tx = pool.begin().await?;
                Ok(DbTransaction::Postgres(tx))
            }
            #[cfg(feature = "mysql")]
            DbPool::MySql(pool) => {
                let tx = pool.begin().await?;
                Ok(DbTransaction::MySql(tx))
            }
            #[cfg(feature = "sqlite")]
            DbPool::Sqlite(pool) => {
                let tx = pool.begin().await?;
                Ok(DbTransaction::Sqlite(tx))
            }
            #[cfg(not(any(feature = "pg", feature = "mysql", feature = "sqlite")))]
            _ => Err(DbError::FeatureDisabled("no database backends enabled")),
        }
    }
}

// ===================== tests =====================

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    #[cfg(feature = "sqlite")]
    use tokio::time::Duration;

    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn test_sqlite_connection() -> Result<()> {
        let dsn = "sqlite::memory:";
        let opts = ConnectOpts::default();
        let db = DbHandle::connect(dsn, opts).await?;
        assert_eq!(db.engine(), DbEngine::Sqlite);
        Ok(())
    }

    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn test_sqlite_connection_with_pragma_parameters() -> Result<()> {
        // Test that SQLite connections work with PRAGMA parameters in DSN
        let dsn = "sqlite::memory:?wal=true&synchronous=NORMAL&busy_timeout=5000&journal_mode=WAL";
        let opts = ConnectOpts::default();
        let db = DbHandle::connect(dsn, opts).await?;
        assert_eq!(db.engine(), DbEngine::Sqlite);

        // Verify that the stored DSN has been cleaned (SQLite parameters removed)
        // Note: For memory databases, the DSN should still be sqlite::memory: after cleaning
        assert!(db.dsn == "sqlite::memory:" || db.dsn.starts_with("sqlite::memory:"));

        // Test that we can execute queries (confirming the connection works)
        let pool = db.sqlx_sqlite().unwrap();
        sea_orm::sqlx::query("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)")
            .execute(pool)
            .await?;
        sea_orm::sqlx::query("INSERT INTO test (name) VALUES (?)")
            .bind("test_value")
            .execute(pool)
            .await?;

        let row: (i64, String) = sea_orm::sqlx::query_as("SELECT id, name FROM test WHERE id = 1")
            .fetch_one(pool)
            .await?;

        assert_eq!(row.0, 1);
        assert_eq!(row.1, "test_value");

        Ok(())
    }

    #[tokio::test]
    async fn test_backend_detection() {
        assert_eq!(
            DbHandle::detect("sqlite::memory:").unwrap(),
            DbEngine::Sqlite
        );
        assert_eq!(
            DbHandle::detect("postgres://localhost/test").unwrap(),
            DbEngine::Postgres
        );
        assert_eq!(
            DbHandle::detect("mysql://localhost/test").unwrap(),
            DbEngine::MySql
        );
        assert!(DbHandle::detect("unknown://test").is_err());
    }

    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn test_advisory_lock_sqlite() -> Result<()> {
        let dsn = "sqlite:file:memdb1?mode=memory&cache=shared";
        let db = DbHandle::connect(dsn, ConnectOpts::default()).await?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let test_id = format!("test_basic_{now}");

        let guard1 = db.lock("test_module", &format!("{test_id}_key1")).await?;
        let _guard2 = db.lock("test_module", &format!("{test_id}_key2")).await?;
        let _guard3 = db
            .lock("different_module", &format!("{test_id}_key1"))
            .await?;

        // Deterministic unlock to avoid races with async Drop cleanup
        guard1.release().await;
        let _guard4 = db.lock("test_module", &format!("{test_id}_key1")).await?;
        Ok(())
    }

    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn test_advisory_lock_different_keys() -> Result<()> {
        let dsn = "sqlite:file:memdb_diff_keys?mode=memory&cache=shared";
        let db = DbHandle::connect(dsn, ConnectOpts::default()).await?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let test_id = format!("test_diff_{now}");

        let _guard1 = db.lock("test_module", &format!("{test_id}_key1")).await?;
        let _guard2 = db.lock("test_module", &format!("{test_id}_key2")).await?;
        let _guard3 = db.lock("other_module", &format!("{test_id}_key1")).await?;
        Ok(())
    }

    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn test_try_lock_with_config() -> Result<()> {
        let dsn = "sqlite:file:memdb2?mode=memory&cache=shared";
        let db = DbHandle::connect(dsn, ConnectOpts::default()).await?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let test_id = format!("test_config_{now}");

        let _guard1 = db.lock("test_module", &format!("{test_id}_key")).await?;

        let config = LockConfig {
            max_wait: Some(Duration::from_millis(200)),
            initial_backoff: Duration::from_millis(50),
            max_attempts: Some(3),
            ..Default::default()
        };

        let result = db
            .try_lock("test_module", &format!("{test_id}_different_key"), config)
            .await?;
        assert!(
            result.is_some(),
            "expected lock acquisition for different key"
        );
        Ok(())
    }

    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn test_transaction() -> Result<()> {
        let dsn = "sqlite::memory:";
        let db = DbHandle::connect(dsn, ConnectOpts::default()).await?;
        let tx = db.begin().await?;
        tx.commit().await?;
        Ok(())
    }

    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn test_secure_conn() -> Result<()> {
        let dsn = "sqlite::memory:";
        let db = DbHandle::connect(dsn, ConnectOpts::default()).await?;

        let _secure_conn = db.sea_secure();
        Ok(())
    }

    #[cfg(all(feature = "sqlite", feature = "insecure-escape"))]
    #[tokio::test]
    async fn test_insecure_sea_access() -> Result<()> {
        let dsn = "sqlite::memory:";
        let db = DbHandle::connect(dsn, ConnectOpts::default()).await?;

        // Only available with insecure-escape feature
        let _raw = db.sea();
        Ok(())
    }
}
