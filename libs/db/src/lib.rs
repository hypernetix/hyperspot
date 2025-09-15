#![cfg_attr(
    not(any(feature = "pg", feature = "mysql", feature = "sqlite")),
    allow(
        unused_imports,
        unused_variables,
        dead_code,
        unreachable_code,
        unused_lifetimes
    )
)]
//! Database abstraction crate providing a database-agnostic `DbHandle`.
//!
//! This crate provides a unified interface for working with different databases
//! (SQLite, PostgreSQL, MySQL) through SQLx, with optional SeaORM integration.
//!
//! # Features
//! - `pg`, `mysql`, `sqlite`: enable SQLx backends
//! - `sea-orm`: add SeaORM integration for type-safe operations
//!
//! # Example
//! ```rust,no_run
//! #[tokio::main]
//! async fn main() -> db::Result<()> {
//!     use db::{DbHandle, ConnectOpts};
//!
//!     let db = DbHandle::connect("postgres://user:pass@localhost/app", ConnectOpts::default()).await?;
//!
//!     // sqlx
//!     #[cfg(feature="pg")]
//!     {
//!         let pool = db.sqlx_postgres().unwrap();
//!         // Help type inference for doctests: specify database type explicitly
//!         sqlx::query::<sqlx::Postgres>("select 1").execute(pool).await?;
//!
//!         // Execute within a dedicated connection (doctest-friendly)
//!         let mut conn = pool.acquire().await?;
//!         sqlx::query::<sqlx::Postgres>("select 2").execute(&mut *conn).await?;
//!     }
//!
//!     #[cfg(feature="mysql")]
//!     {
//!         let pool = db.sqlx_mysql().unwrap();
//!         sqlx::query::<sqlx::MySql>("select 1").execute(pool).await?;
//!
//!         let mut conn = pool.acquire().await?;
//!         sqlx::query::<sqlx::MySql>("select 2").execute(&mut *conn).await?;
//!     }
//!
//!     #[cfg(feature="sqlite")]
//!     {
//!         let pool = db.sqlx_sqlite().unwrap();
//!         sqlx::query::<sqlx::Sqlite>("select 1").execute(pool).await?;
//!
//!         let mut conn = pool.acquire().await?;
//!         sqlx::query::<sqlx::Sqlite>("select 2").execute(&mut *conn).await?;
//!     }
//!
//!     // sea-orm (if enabled)
//!     #[cfg(feature="sea-orm")]
//!     {
//!         use sea_orm::{ConnectionTrait, Statement, DatabaseBackend};
//!         db.sea().execute(Statement::from_string(DatabaseBackend::Postgres, "SELECT 3")).await?;
//!     }
//!
//!     db.close().await;
//!     Ok(())
//! }
//! ```
use std::time::Duration;

// Used for parsing SQLite DSN query parameters

#[cfg(feature = "mysql")]
use sqlx::{mysql::MySqlPoolOptions, MySql, MySqlPool};
#[cfg(feature = "pg")]
use sqlx::{postgres::PgPoolOptions, PgPool, Postgres};
#[cfg(feature = "sqlite")]
use sqlx::{sqlite::SqlitePoolOptions, Sqlite, SqlitePool};

#[cfg(feature = "sea-orm")]
use sea_orm::DatabaseConnection;
#[cfg(all(feature = "sea-orm", feature = "mysql"))]
use sea_orm::SqlxMySqlConnector;
#[cfg(all(feature = "sea-orm", feature = "pg"))]
use sea_orm::SqlxPostgresConnector;
#[cfg(all(feature = "sea-orm", feature = "sqlite"))]
use sea_orm::SqlxSqliteConnector;

use thiserror::Error;

// Re-export key types for public API
pub use advisory_locks::{DbLockGuard, LockConfig};

// Advisory locks module
pub mod advisory_locks;

/// Library-local result type.
pub type Result<T> = std::result::Result<T, DbError>;

/// Typed error for the DB handle and helpers.
#[derive(Debug, Error)]
pub enum DbError {
    #[error("Unknown DSN: {0}")]
    UnknownDsn(String),

    #[error("Feature not enabled: {0}")]
    FeatureDisabled(&'static str),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[cfg(feature = "sea-orm")]
    #[error(transparent)]
    Sea(#[from] sea_orm::DbErr),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("SQLite pragma error: {0}")]
    SqlitePragma(String),

    // make advisory_locks errors flow into DbError via `?`
    #[error(transparent)]
    Lock(#[from] advisory_locks::DbLockError),
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

    /// SQLite-specific: busy timeout used via PRAGMA busy_timeout.
    pub sqlite_busy_timeout: Option<Duration>,
    /// For SQLite file DSNs, create parent directories if missing.
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

            sqlite_busy_timeout: Some(Duration::from_millis(5_000)),
            create_sqlite_dirs: true,
        }
    }
}

/// One concrete sqlx pool.
#[derive(Clone)]
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
    Postgres(sqlx::Transaction<'a, Postgres>),
    #[cfg(feature = "mysql")]
    MySql(sqlx::Transaction<'a, MySql>),
    #[cfg(feature = "sqlite")]
    Sqlite(sqlx::Transaction<'a, Sqlite>),
    // When no concrete DB feature is enabled, keep a variant to tie `'a` so
    // the type still compiles and can be referenced in signatures.
    #[cfg(not(any(feature = "pg", feature = "mysql", feature = "sqlite")))]
    _Phantom(std::marker::PhantomData<&'a ()>),
}

impl<'a> DbTransaction<'a> {
    /// Commit the transaction.
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
pub struct DbHandle {
    engine: DbEngine,
    pool: DbPool,
    dsn: String,
    #[cfg(feature = "sea-orm")]
    sea: DatabaseConnection,
}

impl DbHandle {
    /// Detect engine by DSN.
    ///
    /// Note: we only check scheme prefixes and don't mutate the tail (credentials etc.).
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
            Err(DbError::UnknownDsn(dsn.to_string()))
        }
    }

    /// Connect and build handle.
    pub async fn connect(dsn: &str, opts: ConnectOpts) -> Result<Self> {
        let engine = Self::detect(dsn)?;
        match engine {
            #[cfg(feature = "pg")]
            DbEngine::Postgres => {
                let mut o = PgPoolOptions::new();
                if let Some(n) = opts.max_conns {
                    o = o.max_connections(n);
                }
                if let Some(n) = opts.min_conns {
                    o = o.min_connections(n);
                }
                if let Some(t) = opts.acquire_timeout {
                    o = o.acquire_timeout(t);
                }
                if let Some(t) = opts.idle_timeout {
                    o = o.idle_timeout(t);
                }
                if let Some(t) = opts.max_lifetime {
                    o = o.max_lifetime(t);
                }
                if opts.test_before_acquire {
                    o = o.test_before_acquire(true);
                }
                let pool = o.connect(dsn).await?;
                #[cfg(feature = "sea-orm")]
                let sea = SqlxPostgresConnector::from_sqlx_postgres_pool(pool.clone());
                Ok(Self {
                    engine,
                    pool: DbPool::Postgres(pool),
                    dsn: dsn.to_string(),
                    #[cfg(feature = "sea-orm")]
                    sea,
                })
            }
            #[cfg(feature = "mysql")]
            DbEngine::MySql => {
                let mut o = MySqlPoolOptions::new();
                if let Some(n) = opts.max_conns {
                    o = o.max_connections(n);
                }
                if let Some(n) = opts.min_conns {
                    o = o.min_connections(n);
                }
                if let Some(t) = opts.acquire_timeout {
                    o = o.acquire_timeout(t);
                }
                if let Some(t) = opts.idle_timeout {
                    o = o.idle_timeout(t);
                }
                if let Some(t) = opts.max_lifetime {
                    o = o.max_lifetime(t);
                }
                if opts.test_before_acquire {
                    o = o.test_before_acquire(true);
                }
                let pool = o.connect(dsn).await?;
                #[cfg(feature = "sea-orm")]
                let sea = SqlxMySqlConnector::from_sqlx_mysql_pool(pool.clone());
                Ok(Self {
                    engine,
                    pool: DbPool::MySql(pool),
                    dsn: dsn.to_string(),
                    #[cfg(feature = "sea-orm")]
                    sea,
                })
            }
            #[cfg(feature = "sqlite")]
            DbEngine::Sqlite => {
                let dsn = prepare_sqlite_path(dsn, opts.create_sqlite_dirs)?;

                // Parse pragma settings from DSN query parameters BEFORE removing them
                let dsn_pragmas = parse_sqlite_pragmas_from_dsn(&dsn);

                // Remove SQLite-specific parameters from DSN for SQLx connection
                let clean_dsn = remove_sqlite_pragmas_from_dsn(&dsn);

                let mut o = SqlitePoolOptions::new();

                if let Some(n) = opts.max_conns {
                    o = o.max_connections(n);
                }
                if let Some(n) = opts.min_conns {
                    o = o.min_connections(n);
                }
                if let Some(t) = opts.acquire_timeout {
                    o = o.acquire_timeout(t);
                }
                if let Some(t) = opts.idle_timeout {
                    o = o.idle_timeout(t);
                }
                if let Some(t) = opts.max_lifetime {
                    o = o.max_lifetime(t);
                }
                if opts.test_before_acquire {
                    o = o.test_before_acquire(true);
                }

                o = o.after_connect(move |conn, _meta| {
                    let pragmas = dsn_pragmas.clone();
                    let fallback_busy_timeout = opts.sqlite_busy_timeout;
                    Box::pin(async move {
                        // Apply journal_mode (prefers explicit setting, then wal setting, then default)
                        if let Some(journal_mode) = pragmas.get("journal_mode") {
                            let stmt = format!("PRAGMA journal_mode = {}", journal_mode);
                            sqlx::query(&stmt).execute(&mut *conn).await?;
                        } else if let Some(wal_mode) = pragmas.get("wal") {
                            // wal parameter is already validated and converted to journal_mode value
                            let stmt = format!("PRAGMA journal_mode = {}", wal_mode);
                            sqlx::query(&stmt).execute(&mut *conn).await?;
                        } else {
                            // Default to WAL mode
                            sqlx::query("PRAGMA journal_mode = WAL")
                                .execute(&mut *conn)
                                .await?;
                        }

                        // Apply synchronous mode if specified in params (default: NORMAL)
                        if let Some(sync_mode) = pragmas.get("synchronous") {
                            // synchronous value is already validated
                            let stmt = format!("PRAGMA synchronous = {}", sync_mode);
                            sqlx::query(&stmt).execute(&mut *conn).await?;
                        } else {
                            // Default to NORMAL
                            sqlx::query("PRAGMA synchronous = NORMAL")
                                .execute(&mut *conn)
                                .await?;
                        }

                        // Apply busy timeout from params or fallback to opts
                        if let Some(timeout_str) = pragmas.get("busy_timeout") {
                            // busy_timeout value is already validated as a positive integer
                            let stmt = format!("PRAGMA busy_timeout = {}", timeout_str);
                            sqlx::query(&stmt).execute(&mut *conn).await?;
                        } else if let Some(timeout) = fallback_busy_timeout {
                            let ms = std::cmp::min(timeout.as_millis(), i64::MAX as u128) as i64;
                            let stmt = format!("PRAGMA busy_timeout = {ms}");
                            sqlx::query(&stmt).execute(&mut *conn).await?;
                        } else {
                            // Default to 5 seconds
                            sqlx::query("PRAGMA busy_timeout = 5000")
                                .execute(&mut *conn)
                                .await?;
                        }

                        Ok(())
                    })
                });

                let pool = o.connect(&clean_dsn).await?;
                // No extra call to apply_sqlite_pragmas here anymore.
                #[cfg(feature = "sea-orm")]
                let sea = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool.clone());

                Ok(Self {
                    engine,
                    pool: DbPool::Sqlite(pool),
                    dsn: clean_dsn,
                    #[cfg(feature = "sea-orm")]
                    sea,
                })
            }
            #[cfg(not(feature = "pg"))]
            DbEngine::Postgres => Err(DbError::FeatureDisabled("PostgreSQL feature not enabled")),
            #[cfg(not(feature = "mysql"))]
            DbEngine::MySql => Err(DbError::FeatureDisabled("MySQL feature not enabled")),
            #[cfg(not(any(feature = "pg", feature = "mysql", feature = "sqlite")))]
            _ => Err(DbError::FeatureDisabled("no DB features enabled")),
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
    pub fn engine(&self) -> DbEngine {
        self.engine
    }

    // --- sqlx accessors ---
    #[cfg(feature = "pg")]
    pub fn sqlx_postgres(&self) -> Option<&PgPool> {
        match self.pool {
            DbPool::Postgres(ref p) => Some(p),
            #[cfg(any(feature = "mysql", feature = "sqlite"))]
            _ => None,
        }
    }
    #[cfg(feature = "mysql")]
    pub fn sqlx_mysql(&self) -> Option<&MySqlPool> {
        match self.pool {
            DbPool::MySql(ref p) => Some(p),
            #[cfg(any(feature = "pg", feature = "sqlite"))]
            _ => None,
        }
    }
    #[cfg(feature = "sqlite")]
    pub fn sqlx_sqlite(&self) -> Option<&SqlitePool> {
        match self.pool {
            DbPool::Sqlite(ref p) => Some(p),
            #[cfg(any(feature = "pg", feature = "mysql"))]
            _ => None,
        }
    }

    // --- SeaORM accessor ---
    #[cfg(feature = "sea-orm")]
    /// Get SeaORM connection (clone; cheap handle).
    pub fn sea(&self) -> DatabaseConnection {
        self.sea.clone()
    }

    #[cfg(feature = "sea-orm")]
    /// Backward-compatible alias; not async (no await inside).
    pub fn seaorm(&self) -> &DatabaseConnection {
        &self.sea
    }

    // --- Transaction helpers (engine-specific) ---
    #[cfg(feature = "pg")]
    pub async fn with_pg_tx<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut sqlx::Transaction<'_, Postgres>) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
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

    #[cfg(feature = "mysql")]
    pub async fn with_mysql_tx<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut sqlx::Transaction<'_, MySql>) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
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

    #[cfg(feature = "sqlite")]
    pub async fn with_sqlite_tx<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut sqlx::Transaction<'_, Sqlite>) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
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
    pub async fn lock(&self, module: &str, key: &str) -> Result<DbLockGuard> {
        let lock_manager =
            advisory_locks::LockManager::new(self.engine, self.pool.clone(), self.dsn.clone());
        let guard = lock_manager.lock(module, key).await?;
        Ok(guard)
    }

    /// Try to acquire an advisory lock with configurable retry/backoff policy.
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
    pub async fn begin<'a>(&'a self) -> Result<DbTransaction<'a>> {
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

// ===================== helpers =====================

#[cfg(feature = "sqlite")]
fn prepare_sqlite_path(dsn: &str, create_dirs: bool) -> Result<String> {
    // Only try to create directories for plain file paths; ignore :memory: cases.
    if !create_dirs || dsn.contains(":memory:") {
        return Ok(dsn.to_string());
    }

    // This is a pragmatic parser: it handles "sqlite:/path" and "sqlite://path".
    // For URI forms like "sqlite:file:memdb?..." there is no filesystem dir to create.
    let raw = if let Some(rest) = dsn.strip_prefix("sqlite://") {
        rest
    } else if let Some(rest) = dsn.strip_prefix("sqlite:") {
        rest
    } else {
        dsn
    };

    // If DSN looks like a plain path (no "file:" scheme or query), create parent dir.
    if !raw.starts_with("file:") && !raw.contains('?') {
        if let Some(parent) = std::path::Path::new(raw).parent() {
            if !parent.as_os_str().is_empty() {
                // One-time blocking call during startup; acceptable for setup paths.
                std::fs::create_dir_all(parent)?;
            }
        }
    }

    Ok(dsn.to_string())
}

/// Remove SQLite-specific PRAGMA parameters from DSN for SQLx connection.
/// SQLx doesn't understand these parameters, so we need to clean them before connecting.
fn remove_sqlite_pragmas_from_dsn(dsn: &str) -> String {
    // List of SQLite-specific parameters that need to be removed
    const SQLITE_PRAGMA_PARAMS: &[&str] = &["wal", "synchronous", "busy_timeout", "journal_mode"];

    if let Ok(mut url) = url::Url::parse(dsn) {
        // Get current query parameters
        let query_pairs: Vec<(String, String)> = url
            .query_pairs()
            .filter(|(key, _)| {
                // Keep parameters that are NOT in our SQLite pragma list
                !SQLITE_PRAGMA_PARAMS.contains(&key.to_lowercase().as_str())
            })
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();

        // Clear all query parameters
        url.set_query(None);

        // Re-add only the non-SQLite parameters
        if !query_pairs.is_empty() {
            let mut query_parts = Vec::new();
            for (key, value) in query_pairs {
                query_parts.push(format!("{}={}", key, value));
            }
            url.set_query(Some(&query_parts.join("&")));
        }

        url.to_string()
    } else {
        // If URL parsing fails, return the original DSN
        // This handles cases like plain file paths or other non-URL formats
        dsn.to_string()
    }
}

/// Parse and validate SQLite PRAGMA settings from DSN query parameters.
/// Only accepts parameters from a strict whitelist and validates their values.
/// Invalid parameters are logged as warnings but don't cause failures.
fn parse_sqlite_pragmas_from_dsn(dsn: &str) -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;

    let mut pragmas = HashMap::new();

    // Parse the DSN as a URL to extract query parameters
    if let Ok(url) = url::Url::parse(dsn) {
        for (key, value) in url.query_pairs() {
            let key_lower = key.to_lowercase();
            let value_str = value.to_string();

            // Strict whitelist of allowed PRAGMA parameters
            match key_lower.as_str() {
                "wal" => {
                    if let Some(validated) = validate_wal_pragma(&value_str) {
                        pragmas.insert("wal".to_string(), validated);
                    } else {
                        tracing::warn!(
                            "Invalid 'wal' PRAGMA value '{}' in DSN '{}', ignoring",
                            value_str,
                            dsn
                        );
                    }
                }
                "synchronous" => {
                    if let Some(validated) = validate_synchronous_pragma(&value_str) {
                        pragmas.insert("synchronous".to_string(), validated);
                    } else {
                        tracing::warn!(
                            "Invalid 'synchronous' PRAGMA value '{}' in DSN '{}', ignoring",
                            value_str,
                            dsn
                        );
                    }
                }
                "busy_timeout" => {
                    if let Some(validated) = validate_busy_timeout_pragma(&value_str) {
                        pragmas.insert("busy_timeout".to_string(), validated.to_string());
                    } else {
                        tracing::warn!(
                            "Invalid 'busy_timeout' PRAGMA value '{}' in DSN '{}', ignoring",
                            value_str,
                            dsn
                        );
                    }
                }
                "journal_mode" => {
                    if let Some(validated) = validate_journal_mode_pragma(&value_str) {
                        pragmas.insert("journal_mode".to_string(), validated);
                    } else {
                        tracing::warn!(
                            "Invalid 'journal_mode' PRAGMA value '{}' in DSN '{}', ignoring",
                            value_str,
                            dsn
                        );
                    }
                }
                _ => {
                    // Unknown parameters are silently ignored (not logged as warnings to avoid spam)
                    tracing::debug!(
                        "Unknown SQLite parameter '{}={}' in DSN '{}', ignoring",
                        key,
                        value_str,
                        dsn
                    );
                }
            }
        }
    }

    pragmas
}

/// Validate WAL PRAGMA value.
/// Accepts: "true", "false", "1", "0" (case-insensitive)
/// Returns: "WAL" or "DELETE"
fn validate_wal_pragma(value: &str) -> Option<String> {
    match value.to_lowercase().as_str() {
        "true" | "1" => Some("WAL".to_string()),
        "false" | "0" => Some("DELETE".to_string()),
        _ => None,
    }
}

/// Validate synchronous PRAGMA value.
/// Accepts: "OFF", "NORMAL", "FULL", "EXTRA" (case-insensitive)
fn validate_synchronous_pragma(value: &str) -> Option<String> {
    match value.to_uppercase().as_str() {
        "OFF" | "NORMAL" | "FULL" | "EXTRA" => Some(value.to_uppercase()),
        _ => None,
    }
}

/// Validate busy_timeout PRAGMA value.
/// Must be a positive integer in milliseconds.
fn validate_busy_timeout_pragma(value: &str) -> Option<i64> {
    value.parse::<i64>().ok().filter(|&timeout| timeout >= 0)
}

/// Validate journal_mode PRAGMA value.
/// Accepts: "DELETE", "WAL", "MEMORY", "TRUNCATE", "PERSIST", "OFF" (case-insensitive)
fn validate_journal_mode_pragma(value: &str) -> Option<String> {
    match value.to_uppercase().as_str() {
        "DELETE" | "WAL" | "MEMORY" | "TRUNCATE" | "PERSIST" | "OFF" => Some(value.to_uppercase()),
        _ => None,
    }
}

// ===================== tests =====================

#[cfg(test)]
mod tests {
    use super::*;
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
        sqlx::query("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)")
            .execute(pool)
            .await?;
        sqlx::query("INSERT INTO test (name) VALUES (?)")
            .bind("test_value")
            .execute(pool)
            .await?;

        let row: (i64, String) = sqlx::query_as("SELECT id, name FROM test WHERE id = 1")
            .fetch_one(pool)
            .await?;

        assert_eq!(row.0, 1);
        assert_eq!(row.1, "test_value");

        Ok(())
    }

    #[tokio::test]
    async fn test_backend_detection() {
        assert_eq!(
            DbHandle::detect("sqlite://test.db").unwrap(),
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
            .ok()
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let test_id = format!("test_basic_{now}");

        let guard1 = db.lock("test_module", &format!("{}_key1", test_id)).await?;
        let _guard2 = db.lock("test_module", &format!("{}_key2", test_id)).await?;
        let _guard3 = db
            .lock("different_module", &format!("{}_key1", test_id))
            .await?;

        // Deterministic unlock to avoid races with async Drop cleanup
        guard1.release().await;
        let _guard4 = db.lock("test_module", &format!("{}_key1", test_id)).await?;
        Ok(())
    }

    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn test_advisory_lock_different_keys() -> Result<()> {
        let dsn = "sqlite:file:memdb_diff_keys?mode=memory&cache=shared";
        let db = DbHandle::connect(dsn, ConnectOpts::default()).await?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let test_id = format!("test_diff_{now}");

        let _guard1 = db.lock("test_module", &format!("{}_key1", test_id)).await?;
        let _guard2 = db.lock("test_module", &format!("{}_key2", test_id)).await?;
        let _guard3 = db
            .lock("other_module", &format!("{}_key1", test_id))
            .await?;
        Ok(())
    }

    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn test_try_lock_with_config() -> Result<()> {
        let dsn = "sqlite:file:memdb2?mode=memory&cache=shared";
        let db = DbHandle::connect(dsn, ConnectOpts::default()).await?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let test_id = format!("test_config_{now}");

        let _guard1 = db.lock("test_module", &format!("{}_key", test_id)).await?;

        let config = LockConfig {
            max_wait: Some(Duration::from_millis(200)),
            initial_backoff: Duration::from_millis(50),
            max_attempts: Some(3),
            ..Default::default()
        };

        let result = db
            .try_lock("test_module", &format!("{}_different_key", test_id), config)
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

    #[cfg(all(feature = "sea-orm", feature = "sqlite"))]
    #[tokio::test]
    async fn test_seaorm_connection() -> Result<()> {
        let dsn = "sqlite::memory:";
        let db = DbHandle::connect(dsn, ConnectOpts::default()).await?;
        let _conn = db.sea();
        Ok(())
    }

    #[test]
    fn test_pragma_validation_valid_values() {
        // Test valid WAL pragma values
        assert_eq!(
            parse_sqlite_pragmas_from_dsn("sqlite://test.db?wal=true").get("wal"),
            Some(&"WAL".to_string())
        );
        assert_eq!(
            parse_sqlite_pragmas_from_dsn("sqlite://test.db?wal=false").get("wal"),
            Some(&"DELETE".to_string())
        );
        assert_eq!(
            parse_sqlite_pragmas_from_dsn("sqlite://test.db?wal=1").get("wal"),
            Some(&"WAL".to_string())
        );
        assert_eq!(
            parse_sqlite_pragmas_from_dsn("sqlite://test.db?wal=0").get("wal"),
            Some(&"DELETE".to_string())
        );

        // Test valid synchronous pragma values
        assert_eq!(
            parse_sqlite_pragmas_from_dsn("sqlite://test.db?synchronous=NORMAL").get("synchronous"),
            Some(&"NORMAL".to_string())
        );
        assert_eq!(
            parse_sqlite_pragmas_from_dsn("sqlite://test.db?synchronous=full").get("synchronous"),
            Some(&"FULL".to_string())
        );

        // Test valid busy_timeout pragma values
        assert_eq!(
            parse_sqlite_pragmas_from_dsn("sqlite://test.db?busy_timeout=5000").get("busy_timeout"),
            Some(&"5000".to_string())
        );
        assert_eq!(
            parse_sqlite_pragmas_from_dsn("sqlite://test.db?busy_timeout=0").get("busy_timeout"),
            Some(&"0".to_string())
        );

        // Test valid journal_mode pragma values
        assert_eq!(
            parse_sqlite_pragmas_from_dsn("sqlite://test.db?journal_mode=wal").get("journal_mode"),
            Some(&"WAL".to_string())
        );
        assert_eq!(
            parse_sqlite_pragmas_from_dsn("sqlite://test.db?journal_mode=delete")
                .get("journal_mode"),
            Some(&"DELETE".to_string())
        );
    }

    #[test]
    fn test_pragma_validation_invalid_values() {
        // Test invalid WAL pragma values - should be ignored
        assert!(!parse_sqlite_pragmas_from_dsn("sqlite://test.db?wal=invalid").contains_key("wal"));
        assert!(!parse_sqlite_pragmas_from_dsn("sqlite://test.db?wal=2").contains_key("wal"));

        // Test invalid synchronous pragma values - should be ignored
        assert!(
            !parse_sqlite_pragmas_from_dsn("sqlite://test.db?synchronous=INVALID")
                .contains_key("synchronous")
        );
        assert!(
            !parse_sqlite_pragmas_from_dsn("sqlite://test.db?synchronous=yes")
                .contains_key("synchronous")
        );

        // Test invalid busy_timeout pragma values - should be ignored
        assert!(
            !parse_sqlite_pragmas_from_dsn("sqlite://test.db?busy_timeout=-1")
                .contains_key("busy_timeout")
        );
        assert!(
            !parse_sqlite_pragmas_from_dsn("sqlite://test.db?busy_timeout=abc")
                .contains_key("busy_timeout")
        );
        assert!(
            !parse_sqlite_pragmas_from_dsn("sqlite://test.db?busy_timeout=1.5")
                .contains_key("busy_timeout")
        );

        // Test invalid journal_mode pragma values - should be ignored
        assert!(
            !parse_sqlite_pragmas_from_dsn("sqlite://test.db?journal_mode=invalid")
                .contains_key("journal_mode")
        );
        assert!(
            !parse_sqlite_pragmas_from_dsn("sqlite://test.db?journal_mode=true")
                .contains_key("journal_mode")
        );
    }

    #[test]
    fn test_pragma_unknown_parameters_ignored() {
        let pragmas =
            parse_sqlite_pragmas_from_dsn("sqlite://test.db?unknown_param=value&foo=bar&wal=true");

        // Valid parameter should be included
        assert_eq!(pragmas.get("wal"), Some(&"WAL".to_string()));

        // Unknown parameters should be ignored
        assert!(!pragmas.contains_key("unknown_param"));
        assert!(!pragmas.contains_key("foo"));
    }

    #[test]
    fn test_pragma_case_insensitive_matching() {
        // Test case-insensitive parameter names
        assert_eq!(
            parse_sqlite_pragmas_from_dsn("sqlite://test.db?WAL=true").get("wal"),
            Some(&"WAL".to_string())
        );
        assert_eq!(
            parse_sqlite_pragmas_from_dsn("sqlite://test.db?Synchronous=Normal").get("synchronous"),
            Some(&"NORMAL".to_string())
        );
        assert_eq!(
            parse_sqlite_pragmas_from_dsn("sqlite://test.db?JOURNAL_MODE=wal").get("journal_mode"),
            Some(&"WAL".to_string())
        );
    }

    #[test]
    fn test_pragma_multiple_parameters() {
        let pragmas = parse_sqlite_pragmas_from_dsn(
            "sqlite://test.db?wal=true&synchronous=FULL&busy_timeout=10000&journal_mode=DELETE&unknown=ignored"
        );

        assert_eq!(pragmas.get("wal"), Some(&"WAL".to_string()));
        assert_eq!(pragmas.get("synchronous"), Some(&"FULL".to_string()));
        assert_eq!(pragmas.get("busy_timeout"), Some(&"10000".to_string()));
        assert_eq!(pragmas.get("journal_mode"), Some(&"DELETE".to_string()));
        assert!(!pragmas.contains_key("unknown"));
    }

    #[test]
    fn test_remove_sqlite_pragmas_from_dsn() {
        // Test removing SQLite-specific parameters while keeping others
        let original_dsn = "sqlite://test.db?wal=true&synchronous=NORMAL&busy_timeout=5000&journal_mode=WAL&mode=rwc&cache=shared";
        let clean_dsn = remove_sqlite_pragmas_from_dsn(original_dsn);

        // Should keep SQLx-compatible parameters but remove SQLite-specific ones
        assert!(clean_dsn.contains("mode=rwc"));
        assert!(clean_dsn.contains("cache=shared"));
        assert!(!clean_dsn.contains("wal=true"));
        assert!(!clean_dsn.contains("synchronous=NORMAL"));
        assert!(!clean_dsn.contains("busy_timeout=5000"));
        assert!(!clean_dsn.contains("journal_mode=WAL"));

        // Test DSN with only SQLite parameters (should have no query string)
        let sqlite_only_dsn = "sqlite://test.db?wal=true&synchronous=NORMAL&busy_timeout=5000";
        let clean_sqlite_dsn = remove_sqlite_pragmas_from_dsn(sqlite_only_dsn);
        assert_eq!(clean_sqlite_dsn, "sqlite://test.db");

        // Test DSN with no parameters
        let no_params_dsn = "sqlite://test.db";
        let clean_no_params = remove_sqlite_pragmas_from_dsn(no_params_dsn);
        assert_eq!(clean_no_params, "sqlite://test.db");

        // Test case insensitive parameter removal
        let case_dsn = "sqlite://test.db?WAL=true&Synchronous=NORMAL&BUSY_TIMEOUT=5000&mode=rwc";
        let clean_case_dsn = remove_sqlite_pragmas_from_dsn(case_dsn);
        assert!(clean_case_dsn.contains("mode=rwc"));
        assert!(!clean_case_dsn.contains("WAL=true"));
        assert!(!clean_case_dsn.contains("Synchronous=NORMAL"));
        assert!(!clean_case_dsn.contains("BUSY_TIMEOUT=5000"));
    }
}
