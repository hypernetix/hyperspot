//! Advisory locks implementation with namespacing and retry policies.
//!
//! Cross-database advisory locking with proper namespacing and configurable
//! retry/backoff. For PostgreSQL and MySQL we use native DB advisory locks and
//! **hold the same connection** inside the guard; for SQLite (or when native
//! locks aren't available) we fall back to file-based locks held by an open
//! file descriptor.
//!
//! Notes:
//! - Prefer calling `guard.release().await` for deterministic unlock;
//!   `Drop` provides best-effort cleanup only (may be skipped on runtime shutdown).
//! - File-based locks use `create_new(true)` semantics and keep the file open,
//!   then remove it on release. Consider using `fs2::FileExt::try_lock_exclusive()`
//!   if you want kernel-level advisory locks across processes.

#![cfg_attr(
    not(any(feature = "pg", feature = "mysql", feature = "sqlite")),
    allow(unused_imports, unused_variables, dead_code, unreachable_code)
)]

use std::path::PathBuf;
use std::time::{Duration, Instant};
use thiserror::Error;
use xxhash_rust::xxh3::xxh3_64;

#[cfg(feature = "mysql")]
use sqlx::{pool::PoolConnection, MySql};
#[cfg(feature = "pg")]
use sqlx::{pool::PoolConnection, Postgres};
use tokio::fs::File;

use crate::{DbEngine, DbPool};

// --------------------------- Config ------------------------------------------

/// Configuration for lock acquisition attempts.
#[derive(Debug, Clone)]
pub struct LockConfig {
    /// Maximum duration to wait for lock acquisition (`None` = unlimited).
    pub max_wait: Option<Duration>,
    /// Initial delay between retry attempts.
    pub initial_backoff: Duration,
    /// Maximum delay between retry attempts (cap for exponential backoff).
    pub max_backoff: Duration,
    /// Backoff multiplier for exponential backoff.
    pub backoff_multiplier: f64,
    /// Jitter percentage in [0.0, 1.0]; e.g. 0.2 means ±20% jitter.
    pub jitter_pct: f32,
    /// Maximum number of retry attempts (`None` = unlimited).
    pub max_attempts: Option<u32>,
}

impl Default for LockConfig {
    fn default() -> Self {
        Self {
            max_wait: Some(Duration::from_secs(30)),
            initial_backoff: Duration::from_millis(50),
            max_backoff: Duration::from_secs(5),
            backoff_multiplier: 1.5,
            jitter_pct: 0.2,
            max_attempts: None,
        }
    }
}

/* --------------------------- Guard ------------------------------------------- */

#[derive(Debug)]
enum GuardInner {
    #[cfg(feature = "pg")]
    Postgres {
        /// The SAME connection that acquired `pg_advisory_lock`.
        conn: PoolConnection<Postgres>,
        key_hash: i64,
    },
    #[cfg(feature = "mysql")]
    MySql {
        /// The SAME connection that acquired `GET_LOCK`.
        conn: PoolConnection<MySql>,
        lock_name: String,
    },
    /// File-based fallback (keeps descriptor open until release).
    File { path: PathBuf, file: File },
}

/// Database lock guard that can release lock explicitly via `release()`.
/// `Drop` provides best-effort cleanup if you forget to call `release()`.
#[derive(Debug)]
pub struct DbLockGuard {
    namespaced_key: String,
    inner: Option<GuardInner>, // Option to allow moving inner out in Drop
}

impl DbLockGuard {
    /// Lock key with module namespace ("module:key").
    pub fn key(&self) -> &str {
        &self.namespaced_key
    }

    /// Deterministically release the lock (preferred).
    pub async fn release(mut self) {
        if let Some(inner) = self.inner.take() {
            unlock_inner(inner).await;
        }
        // drop self
    }
}

impl Drop for DbLockGuard {
    fn drop(&mut self) {
        // Best-effort async unlock if runtime is alive and inner still present.
        if let Some(inner) = self.inner.take() {
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(async move { unlock_inner(inner).await });
            } else {
                // No runtime; we cannot perform async cleanup here.
                // The lock may remain held until process exit (DB connection)
                // or lock file may remain on disk. Prefer calling `release().await`.
            }
        }
    }
}

async fn unlock_inner(inner: GuardInner) {
    match inner {
        #[cfg(feature = "pg")]
        GuardInner::Postgres { mut conn, key_hash } => {
            if let Err(e) = sqlx::query("SELECT pg_advisory_unlock($1)")
                .bind(key_hash)
                .execute(&mut conn)
                .await
            {
                tracing::warn!(error=%e, "failed to release PostgreSQL advisory lock");
            }
            // conn returns to the pool here
        }
        #[cfg(feature = "mysql")]
        GuardInner::MySql {
            mut conn,
            lock_name,
        } => {
            // RELEASE_LOCK returns 1 on success, 0 if you didn't own, NULL on error.
            if let Err(e) = sqlx::query_scalar::<_, Option<i64>>("SELECT RELEASE_LOCK(?)")
                .bind(&lock_name)
                .fetch_one(&mut conn)
                .await
            {
                tracing::warn!(error=%e, "failed to release MySQL advisory lock");
            }
        }
        GuardInner::File { path, file } => {
            // Close file first, then try to remove marker. Ignore errors.
            drop(file);
            let _ = tokio::fs::remove_file(&path).await;
        }
    }
}

// --------------------------- Lock Manager ------------------------------------

/// Internal lock manager handling different database backends.
pub struct LockManager {
    engine: DbEngine,
    // Needed for PG/MySQL; in sqlite-only builds this may be unused.
    #[cfg_attr(
        all(feature = "sqlite", not(any(feature = "pg", feature = "mysql"))),
        allow(dead_code)
    )]
    pool: DbPool,
    dsn: String,
}

impl LockManager {
    pub fn new(engine: DbEngine, pool: DbPool, dsn: String) -> Self {
        Self { engine, pool, dsn }
    }

    /// Acquire an advisory lock for `{module}:{key}` (blocking for PG/MySQL).
    ///
    /// Returns a guard that releases the lock when dropped (best-effort) or
    /// deterministically when `release().await` is called.
    pub async fn lock(&self, module: &str, key: &str) -> Result<DbLockGuard, DbLockError> {
        let namespaced_key = format!("{module}:{key}");
        match self.engine {
            #[cfg(feature = "pg")]
            DbEngine::Postgres => self.lock_pg(&namespaced_key).await,
            #[cfg(not(feature = "pg"))]
            DbEngine::Postgres => Err(DbLockError::InvalidState(
                "PostgreSQL feature not enabled".to_string(),
            )),
            #[cfg(feature = "mysql")]
            DbEngine::MySql => self.lock_mysql(&namespaced_key).await,
            #[cfg(not(feature = "mysql"))]
            DbEngine::MySql => Err(DbLockError::InvalidState(
                "MySQL feature not enabled".to_string(),
            )),
            DbEngine::Sqlite => self.lock_file(&namespaced_key).await,
        }
    }

    /// Try to acquire an advisory lock with retry/backoff policy.
    ///
    /// Returns:
    /// - `Ok(Some(guard))` if lock acquired
    /// - `Ok(None)` if timed out or attempts exceeded
    /// - `Err(e)` on unrecoverable error
    pub async fn try_lock(
        &self,
        module: &str,
        key: &str,
        config: LockConfig,
    ) -> Result<Option<DbLockGuard>, DbLockError> {
        let namespaced_key = format!("{module}:{key}");
        let start = Instant::now();
        let mut attempt = 0u32;
        let mut backoff = config.initial_backoff;

        loop {
            attempt += 1;

            if let Some(max_attempts) = config.max_attempts {
                if attempt > max_attempts {
                    return Ok(None);
                }
            }
            if let Some(max_wait) = config.max_wait {
                if start.elapsed() >= max_wait {
                    return Ok(None);
                }
            }

            match self.try_acquire_once(&namespaced_key).await? {
                Some(guard) => return Ok(Some(guard)),
                None => {
                    // Sleep with jitter, capped by remaining time if any.
                    let remaining = config
                        .max_wait
                        .map(|mw| mw.saturating_sub(start.elapsed()))
                        .unwrap_or(backoff);

                    if remaining.is_zero() {
                        return Ok(None);
                    }

                    let jitter_factor = {
                        let pct = config.jitter_pct.max(0.0).min(1.0) as f64;
                        let lo = 1.0 - pct;
                        let hi = 1.0 + pct;
                        // Deterministic jitter from key hash (no rand dep).
                        let h = xxh3_64(namespaced_key.as_bytes());
                        let frac = h as f64 / u64::MAX as f64; // 0..1
                        lo + frac * (hi - lo)
                    };

                    let sleep_for = std::cmp::min(backoff, remaining);
                    tokio::time::sleep(sleep_for.mul_f64(jitter_factor)).await;

                    // Exponential backoff
                    let next = backoff.mul_f64(config.backoff_multiplier);
                    backoff = std::cmp::min(next, config.max_backoff);
                }
            }
        }
    }

    // ------------------------ PG / MySQL / File helpers ----------------------

    #[cfg(feature = "pg")]
    async fn lock_pg(&self, namespaced_key: &str) -> Result<DbLockGuard, DbLockError> {
        let DbPool::Postgres(ref pool) = self.pool else {
            return Err(DbLockError::InvalidState(
                "not a PostgreSQL pool".to_string(),
            ));
        };
        let mut conn = pool.acquire().await?; // sqlx::Error via #[from]
        let key_hash = xxh3_64(namespaced_key.as_bytes()) as i64;

        sqlx::query("SELECT pg_advisory_lock($1)")
            .bind(key_hash)
            .execute(&mut conn)
            .await?; // sqlx::Error via #[from]

        Ok(DbLockGuard {
            namespaced_key: namespaced_key.to_string(),
            inner: Some(GuardInner::Postgres { conn, key_hash }),
        })
    }

    #[cfg(feature = "pg")]
    async fn try_lock_pg(&self, namespaced_key: &str) -> Result<Option<DbLockGuard>, DbLockError> {
        let DbPool::Postgres(ref pool) = self.pool else {
            return Err(DbLockError::InvalidState(
                "not a PostgreSQL pool".to_string(),
            ));
        };
        let mut conn = pool.acquire().await?; // sqlx::Error via #[from]
        let key_hash = xxh3_64(namespaced_key.as_bytes()) as i64;

        let (ok,): (bool,) = sqlx::query_as("SELECT pg_try_advisory_lock($1)")
            .bind(key_hash)
            .fetch_one(&mut conn)
            .await?; // sqlx::Error via #[from]

        if ok {
            Ok(Some(DbLockGuard {
                namespaced_key: namespaced_key.to_string(),
                inner: Some(GuardInner::Postgres { conn, key_hash }),
            }))
        } else {
            drop(conn);
            Ok(None)
        }
    }

    #[cfg(feature = "mysql")]
    async fn lock_mysql(&self, namespaced_key: &str) -> Result<DbLockGuard, DbLockError> {
        let DbPool::MySql(ref pool) = self.pool else {
            return Err(DbLockError::InvalidState("not a MySQL pool".to_string()));
        };
        let mut conn = pool.acquire().await?; // sqlx::Error via #[from]

        // GET_LOCK(name, timeout_seconds)
        // Note: timeout=0 returns immediately (non-blocking). Use a large timeout to block.
        let (ok,): (i64,) = sqlx::query_as("SELECT GET_LOCK(?, 31536000)") // ~1 year
            .bind(namespaced_key)
            .fetch_one(&mut conn)
            .await?; // sqlx::Error via #[from]

        if ok != 1 {
            return Err(DbLockError::InvalidState(
                "failed to acquire MySQL lock".to_string(),
            ));
        }

        Ok(DbLockGuard {
            namespaced_key: namespaced_key.to_string(),
            inner: Some(GuardInner::MySql {
                conn,
                lock_name: namespaced_key.to_string(),
            }),
        })
    }

    #[cfg(feature = "mysql")]
    async fn try_lock_mysql(
        &self,
        namespaced_key: &str,
    ) -> Result<Option<DbLockGuard>, DbLockError> {
        let DbPool::MySql(ref pool) = self.pool else {
            return Err(DbLockError::InvalidState("not a MySQL pool".to_string()));
        };
        let mut conn = pool.acquire().await?; // sqlx::Error via #[from]

        // Try immediate acquisition; timeout 0 means immediate return.
        let (ok,): (i64,) = sqlx::query_as("SELECT GET_LOCK(?, 0)")
            .bind(namespaced_key)
            .fetch_one(&mut conn)
            .await?; // sqlx::Error via #[from]

        if ok == 1 {
            Ok(Some(DbLockGuard {
                namespaced_key: namespaced_key.to_string(),
                inner: Some(GuardInner::MySql {
                    conn,
                    lock_name: namespaced_key.to_string(),
                }),
            }))
        } else {
            drop(conn);
            Ok(None)
        }
    }

    async fn lock_file(&self, namespaced_key: &str) -> Result<DbLockGuard, DbLockError> {
        let path = self.get_lock_file_path(namespaced_key)?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // create_new semantics via tokio
        let file_res = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await;
        let file = match file_res {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(DbLockError::AlreadyHeld {
                    lock_name: namespaced_key.to_string(),
                });
            }
            Err(e) => return Err(e.into()),
        };

        // Write debug info (best-effort only)
        {
            use tokio::io::AsyncWriteExt;
            let mut f = file.try_clone().await?;
            let _ = f
                .write_all(
                    format!(
                        "PID: {}\nKey: {}\nTimestamp: {}\n",
                        std::process::id(),
                        namespaced_key,
                        chrono::Utc::now().to_rfc3339()
                    )
                    .as_bytes(),
                )
                .await;
        }

        Ok(DbLockGuard {
            namespaced_key: namespaced_key.to_string(),
            inner: Some(GuardInner::File { path, file }),
        })
    }

    async fn try_lock_file(
        &self,
        namespaced_key: &str,
    ) -> Result<Option<DbLockGuard>, DbLockError> {
        let path = self.get_lock_file_path(namespaced_key)?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(file) => {
                // Write debug info (best-effort only)
                {
                    use tokio::io::AsyncWriteExt;
                    let mut f = file.try_clone().await?;
                    let _ = f
                        .write_all(
                            format!(
                                "PID: {}\nKey: {}\nTimestamp: {}\n",
                                std::process::id(),
                                namespaced_key,
                                chrono::Utc::now().to_rfc3339()
                            )
                            .as_bytes(),
                        )
                        .await;
                }

                Ok(Some(DbLockGuard {
                    namespaced_key: namespaced_key.to_string(),
                    inner: Some(GuardInner::File { path, file }),
                }))
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn try_acquire_once(
        &self,
        namespaced_key: &str,
    ) -> Result<Option<DbLockGuard>, DbLockError> {
        match self.engine {
            #[cfg(feature = "pg")]
            DbEngine::Postgres => self.try_lock_pg(namespaced_key).await,
            #[cfg(not(feature = "pg"))]
            DbEngine::Postgres => Err(DbLockError::InvalidState(
                "PostgreSQL feature not enabled".to_string(),
            )),
            #[cfg(feature = "mysql")]
            DbEngine::MySql => self.try_lock_mysql(namespaced_key).await,
            #[cfg(not(feature = "mysql"))]
            DbEngine::MySql => Err(DbLockError::InvalidState(
                "MySQL feature not enabled".to_string(),
            )),
            DbEngine::Sqlite => self.try_lock_file(namespaced_key).await,
        }
    }

    /// Generate lock file path for SQLite (or when using file-based locks).
    fn get_lock_file_path(&self, namespaced_key: &str) -> Result<PathBuf, DbLockError> {
        // For ephemeral DSNs (like `memdb`) or tests, use temp dir to avoid global pollution.
        let base_dir = if self.dsn.contains("memdb") || cfg!(test) {
            std::env::temp_dir().join("hyperspot_test_locks")
        } else {
            // Prefer OS cache dir; fallback to temp dir if None (e.g. in minimal containers).
            let cache = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
            cache.join("hyperspot").join("locks")
        };

        let dsn_hash = format!("{:x}", xxh3_64(self.dsn.as_bytes()));
        let key_hash = format!("{:x}", xxh3_64(namespaced_key.as_bytes()));
        Ok(base_dir.join(dsn_hash).join(format!("{key_hash}.lock")))
    }
}

// --------------------------- Errors ------------------------------------------

#[derive(Error, Debug)]
pub enum DbLockError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// sqlx errors are propagated verbatim when database features are enabled.
    #[cfg(any(feature = "pg", feature = "mysql"))]
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Lock already held: {lock_name}")]
    AlreadyHeld { lock_name: String },

    #[error("Lock not found: {lock_name}")]
    NotFound { lock_name: String },

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

// --------------------------- Tests -------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::sync::Arc;

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_namespaced_locks() -> Result<()> {
        let dsn = "sqlite:file:memdb3?mode=memory&cache=shared";
        let pool = sqlx::SqlitePool::connect(dsn).await?;
        let lock_manager = LockManager::new(
            crate::DbEngine::Sqlite,
            crate::DbPool::Sqlite(pool),
            dsn.to_string(),
        );

        // Unique key suffix (avoid conflicts in parallel)
        let test_id = format!(
            "test_ns_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        let guard1 = lock_manager
            .lock("module1", &format!("{}_key", test_id))
            .await?;
        let guard2 = lock_manager
            .lock("module2", &format!("{}_key", test_id))
            .await?;

        assert!(!guard1.key().is_empty());
        assert!(!guard2.key().is_empty());

        guard1.release().await;
        guard2.release().await;
        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_try_lock_with_timeout() -> Result<()> {
        let dsn = "sqlite:file:memdb4?mode=memory&cache=shared";
        let pool = sqlx::SqlitePool::connect(dsn).await?;
        let lock_manager = Arc::new(LockManager::new(
            DbEngine::Sqlite,
            DbPool::Sqlite(pool),
            dsn.to_string(),
        ));

        let test_id = format!(
            "test_timeout_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        let _guard1 = lock_manager
            .lock("test_module", &format!("{}_key", test_id))
            .await?;

        // Different key should succeed quickly even with retries/timeouts
        let config = LockConfig {
            max_wait: Some(Duration::from_millis(200)),
            initial_backoff: Duration::from_millis(50),
            max_attempts: Some(3),
            ..Default::default()
        };

        let result = lock_manager
            .try_lock("test_module", &format!("{}_different_key", test_id), config)
            .await?;
        assert!(result.is_some(), "expected successful lock acquisition");
        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_try_lock_success() -> Result<()> {
        let dsn = "sqlite:file:memdb5?mode=memory&cache=shared";
        let pool = sqlx::SqlitePool::connect(dsn).await?;
        let lock_manager =
            LockManager::new(DbEngine::Sqlite, DbPool::Sqlite(pool), dsn.to_string());

        let test_id = format!(
            "test_success_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        let result = lock_manager
            .try_lock(
                "test_module",
                &format!("{}_key", test_id),
                LockConfig::default(),
            )
            .await?;
        assert!(result.is_some(), "expected lock acquisition");
        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_double_lock_same_key_errors() -> Result<()> {
        let dsn = "sqlite:file:memdb6?mode=memory&cache=shared";
        let pool = sqlx::SqlitePool::connect(dsn).await?;
        let lock_manager =
            LockManager::new(DbEngine::Sqlite, DbPool::Sqlite(pool), dsn.to_string());

        let test_id = format!(
            "test_double_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        let guard = lock_manager.lock("test_module", &test_id).await?;
        let err = lock_manager
            .lock("test_module", &test_id)
            .await
            .unwrap_err();
        match err {
            DbLockError::AlreadyHeld { lock_name } => {
                assert!(lock_name.contains(&test_id));
            }
            other => panic!("unexpected error: {other:?}"),
        }

        guard.release().await;
        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_try_lock_conflict_returns_none() -> Result<()> {
        let dsn = "sqlite:file:memdb7?mode=memory&cache=shared";
        let pool = sqlx::SqlitePool::connect(dsn).await?;
        let lock_manager =
            LockManager::new(DbEngine::Sqlite, DbPool::Sqlite(pool), dsn.to_string());

        let key = format!(
            "test_conflict_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        let _guard = lock_manager.lock("module", &key).await?;
        let config = LockConfig {
            max_wait: Some(Duration::from_millis(100)),
            max_attempts: Some(2),
            ..Default::default()
        };
        let res = lock_manager.try_lock("module", &key, config).await?;
        assert!(res.is_none());
        Ok(())
    }
}
