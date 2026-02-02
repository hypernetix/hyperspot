//! Database configuration types.
//!
//! This module contains the canonical definitions of all database configuration
//! structures used throughout the system. These types are deserialized directly
//! from Figment configuration.
//!
//! # Configuration Precedence Rules
//!
//! The database configuration system follows a strict precedence hierarchy when
//! merging global server configurations with module-specific overrides:
//!
//! | Priority | Source | Description | Example |
//! |----------|--------|-------------|---------|
//! | 1 (Highest) | Module `params` map | Key-value parameters in module config | `params: {synchronous: "FULL"}` |
//! | 2 | Module DSN query params | Parameters in module-level DSN | `sqlite://file.db?synchronous=NORMAL` |
//! | 3 | Module fields | Individual connection fields | `host: "localhost", port: 5432` |
//! | 4 | Module DSN base | Core DSN without query params | `postgres://user:pass@host/db` |
//! | 5 | Server `params` map | Key-value parameters in server config | Global server `params` |
//! | 6 | Server DSN query params | Parameters in server-level DSN | Server DSN query string |
//! | 7 | Server fields | Individual connection fields in server | Server `host`, `port`, etc. |
//! | 8 (Lowest) | Server DSN base | Core server DSN without query params | Base server connection string |
//!
//! ## Merge Rules
//!
//! 1. **Field Precedence**: Module fields always override server fields
//! 2. **DSN Precedence**: Module DSN overrides server DSN completely
//! 3. **Params Merging**: `params` maps are merged, with module params taking precedence
//! 4. **Pool Configuration**: Module pool config overrides server pool config entirely
//! 5. **`SQLite` Paths**: `file`/`path` fields are module-only and never inherited from servers
//!
//! ## Conflict Detection
//!
//! The system validates configurations and returns [`DbError::ConfigConflict`] for:
//! - `SQLite` DSN with server fields (`host`/`port`)
//! - Non-SQLite DSN with `SQLite` fields (`file`/`path`)
//! - Both `file` and `path` specified for `SQLite`
//! - `SQLite` fields mixed with server connection fields
//!

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Global database configuration with server-based DBs.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GlobalDatabaseConfig {
    /// Server-based DBs (postgres/mysql/sqlite/etc.), keyed by server name.
    #[serde(default)]
    pub servers: HashMap<String, DbConnConfig>,
    /// Optional dev-only flag to auto-provision DB/schema when missing.
    #[serde(default)]
    pub auto_provision: Option<bool>,
}

/// Reusable DB connection config for both global servers and modules.
/// DSN must be a FULL, valid DSN if provided (dsn crate compliant).
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DbConnConfig {
    /// Explicit database engine for this connection.
    ///
    /// This is required for configurations without `dsn`, where the engine cannot be inferred
    /// reliably (e.g. distinguishing `MySQL` vs `PostgreSQL`, or selecting `SQLite` for file/path configs).
    ///
    /// If both `engine` and `dsn` are provided, they must not conflict (validated at runtime).
    #[serde(default)]
    pub engine: Option<DbEngineCfg>,

    // DSN-style (full, valid). Optional: can be absent and rely on fields.
    pub dsn: Option<String>,

    // Field-based style; any of these override DSN parts when present:
    pub host: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub password: Option<String>, // literal password or ${VAR} for env expansion
    pub dbname: Option<String>,   // MUST be present in final for server-based DBs
    #[serde(default)]
    pub params: Option<HashMap<String, String>>,

    // SQLite file-based helpers (module-level only; ignored for global):
    pub file: Option<String>,  // relative name under home_dir/module
    pub path: Option<PathBuf>, // absolute path

    // Connection pool overrides:
    #[serde(default)]
    pub pool: Option<PoolCfg>,

    // Module-level only: reference to a global server by name.
    // If absent, this module config must be fully self-sufficient (dsn or fields).
    pub server: Option<String>,
}

/// Serializable engine selector for configuration.
///
/// Keep this separate from `modkit_db::DbEngine` (runtime type) to avoid coupling it to serde.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DbEngineCfg {
    Postgres,
    Mysql,
    Sqlite,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PoolCfg {
    pub max_conns: Option<u32>,
    pub min_conns: Option<u32>,
    #[serde(with = "modkit_utils::humantime_serde::option", default)]
    pub acquire_timeout: Option<Duration>,
    #[serde(with = "modkit_utils::humantime_serde::option", default)]
    pub idle_timeout: Option<Duration>,
    #[serde(with = "modkit_utils::humantime_serde::option", default)]
    pub max_lifetime: Option<Duration>,
    pub test_before_acquire: Option<bool>,
}

impl PoolCfg {
    /// Apply pool configuration to `PostgreSQL` pool options.
    #[cfg(feature = "pg")]
    #[must_use]
    pub fn apply_pg(
        &self,
        mut opts: sqlx::postgres::PgPoolOptions,
    ) -> sqlx::postgres::PgPoolOptions {
        if let Some(max_conns) = self.max_conns {
            opts = opts.max_connections(max_conns);
        }
        if let Some(min_conns) = self.min_conns {
            opts = opts.min_connections(min_conns);
        }
        if let Some(acquire_timeout) = self.acquire_timeout {
            opts = opts.acquire_timeout(acquire_timeout);
        }
        if let Some(idle_timeout) = self.idle_timeout {
            opts = opts.idle_timeout(Some(idle_timeout));
        }
        if let Some(max_lifetime) = self.max_lifetime {
            opts = opts.max_lifetime(Some(max_lifetime));
        }
        if let Some(test_before_acquire) = self.test_before_acquire {
            opts = opts.test_before_acquire(test_before_acquire);
        }
        opts
    }

    /// Apply pool configuration to `MySQL` pool options.
    #[cfg(feature = "mysql")]
    #[must_use]
    pub fn apply_mysql(
        &self,
        mut opts: sqlx::mysql::MySqlPoolOptions,
    ) -> sqlx::mysql::MySqlPoolOptions {
        if let Some(max_conns) = self.max_conns {
            opts = opts.max_connections(max_conns);
        }
        if let Some(min_conns) = self.min_conns {
            opts = opts.min_connections(min_conns);
        }
        if let Some(acquire_timeout) = self.acquire_timeout {
            opts = opts.acquire_timeout(acquire_timeout);
        }
        if let Some(idle_timeout) = self.idle_timeout {
            opts = opts.idle_timeout(Some(idle_timeout));
        }
        if let Some(max_lifetime) = self.max_lifetime {
            opts = opts.max_lifetime(Some(max_lifetime));
        }
        if let Some(test_before_acquire) = self.test_before_acquire {
            opts = opts.test_before_acquire(test_before_acquire);
        }
        opts
    }

    /// Apply pool configuration to `SQLite` pool options.
    #[cfg(feature = "sqlite")]
    #[must_use]
    pub fn apply_sqlite(
        &self,
        mut opts: sqlx::sqlite::SqlitePoolOptions,
    ) -> sqlx::sqlite::SqlitePoolOptions {
        if let Some(max_conns) = self.max_conns {
            opts = opts.max_connections(max_conns);
        }
        if let Some(min_conns) = self.min_conns {
            opts = opts.min_connections(min_conns);
        }
        if let Some(acquire_timeout) = self.acquire_timeout {
            opts = opts.acquire_timeout(acquire_timeout);
        }
        if let Some(idle_timeout) = self.idle_timeout {
            opts = opts.idle_timeout(Some(idle_timeout));
        }
        if let Some(max_lifetime) = self.max_lifetime {
            opts = opts.max_lifetime(Some(max_lifetime));
        }
        if let Some(test_before_acquire) = self.test_before_acquire {
            opts = opts.test_before_acquire(test_before_acquire);
        }
        opts
    }
}
