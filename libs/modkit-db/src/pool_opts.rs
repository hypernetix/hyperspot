//! Pool options application trait to deduplicate configuration logic.

use crate::ConnectOpts;

/// Trait for applying connection options to pool builders.
///
/// This trait eliminates code duplication across different database backends
/// by providing a common interface for applying connection pool configuration.
pub trait ApplyPoolOpts<T> {
    /// Apply connection options to the pool builder.
    fn apply(self, opts: &ConnectOpts) -> Self;
}

#[cfg(feature = "pg")]
impl ApplyPoolOpts<sea_orm::sqlx::postgres::PgPoolOptions>
    for sea_orm::sqlx::postgres::PgPoolOptions
{
    fn apply(mut self, opts: &ConnectOpts) -> Self {
        if let Some(n) = opts.max_conns {
            self = self.max_connections(n);
        }
        if let Some(n) = opts.min_conns {
            self = self.min_connections(n);
        }
        if let Some(t) = opts.acquire_timeout {
            self = self.acquire_timeout(t);
        }
        if let Some(t) = opts.idle_timeout {
            self = self.idle_timeout(t);
        }
        if let Some(t) = opts.max_lifetime {
            self = self.max_lifetime(t);
        }
        if opts.test_before_acquire {
            self = self.test_before_acquire(true);
        }
        self
    }
}

#[cfg(feature = "mysql")]
impl ApplyPoolOpts<sea_orm::sqlx::mysql::MySqlPoolOptions>
    for sea_orm::sqlx::mysql::MySqlPoolOptions
{
    fn apply(mut self, opts: &ConnectOpts) -> Self {
        if let Some(n) = opts.max_conns {
            self = self.max_connections(n);
        }
        if let Some(n) = opts.min_conns {
            self = self.min_connections(n);
        }
        if let Some(t) = opts.acquire_timeout {
            self = self.acquire_timeout(t);
        }
        if let Some(t) = opts.idle_timeout {
            self = self.idle_timeout(t);
        }
        if let Some(t) = opts.max_lifetime {
            self = self.max_lifetime(t);
        }
        if opts.test_before_acquire {
            self = self.test_before_acquire(true);
        }
        self
    }
}

#[cfg(feature = "sqlite")]
impl ApplyPoolOpts<sea_orm::sqlx::sqlite::SqlitePoolOptions>
    for sea_orm::sqlx::sqlite::SqlitePoolOptions
{
    fn apply(mut self, opts: &ConnectOpts) -> Self {
        if let Some(n) = opts.max_conns {
            self = self.max_connections(n);
        }
        if let Some(n) = opts.min_conns {
            self = self.min_connections(n);
        }
        if let Some(t) = opts.acquire_timeout {
            self = self.acquire_timeout(t);
        }
        if let Some(t) = opts.idle_timeout {
            self = self.idle_timeout(t);
        }
        if let Some(t) = opts.max_lifetime {
            self = self.max_lifetime(t);
        }
        if opts.test_before_acquire {
            self = self.test_before_acquire(true);
        }
        self
    }
}
