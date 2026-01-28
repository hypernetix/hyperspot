//! Database connection options and configuration types.

use crate::config::{DbConnConfig, DbEngineCfg, GlobalDatabaseConfig, PoolCfg};
use crate::{DbError, DbHandle, Result};
use thiserror::Error;

// Pool configuration moved to config.rs

/// Database connection options using typed sqlx `ConnectOptions`.
#[derive(Debug, Clone)]
pub enum DbConnectOptions {
    #[cfg(feature = "sqlite")]
    Sqlite(sea_orm::sqlx::sqlite::SqliteConnectOptions),
    #[cfg(feature = "pg")]
    Postgres(sea_orm::sqlx::postgres::PgConnectOptions),
    #[cfg(feature = "mysql")]
    MySql(sea_orm::sqlx::mysql::MySqlConnectOptions),
}

/// Errors that can occur during connection option building.
#[derive(Debug, Error)]
pub enum ConnectionOptionsError {
    #[error("Invalid SQLite PRAGMA parameter '{key}': {message}")]
    InvalidSqlitePragma { key: String, message: String },

    #[error("Unknown SQLite PRAGMA parameter: {0}")]
    UnknownSqlitePragma(String),

    #[error("Invalid connection parameter: {0}")]
    InvalidParameter(String),

    #[error("Feature not enabled: {0}")]
    FeatureDisabled(&'static str),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),
}

impl std::fmt::Display for DbConnectOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "sqlite")]
            DbConnectOptions::Sqlite(opts) => {
                let filename = opts.get_filename().display().to_string();
                if filename.is_empty() {
                    write!(f, "sqlite://memory")
                } else {
                    write!(f, "sqlite://{filename}")
                }
            }
            #[cfg(feature = "pg")]
            DbConnectOptions::Postgres(opts) => {
                write!(
                    f,
                    "postgresql://<redacted>@{}:{}/{}",
                    opts.get_host(),
                    opts.get_port(),
                    opts.get_database().unwrap_or("")
                )
            }
            #[cfg(feature = "mysql")]
            DbConnectOptions::MySql(_opts) => {
                write!(f, "mysql://<redacted>@...")
            }
            #[cfg(not(any(feature = "sqlite", feature = "pg", feature = "mysql")))]
            _ => {
                unreachable!("No database features enabled")
            }
        }
    }
}

impl DbConnectOptions {
    /// Connect to the database using the configured options.
    ///
    /// # Errors
    /// Returns an error if the database connection fails.
    pub async fn connect(&self, pool: PoolCfg) -> Result<DbHandle> {
        match self {
            #[cfg(feature = "sqlite")]
            DbConnectOptions::Sqlite(opts) => {
                let pool_opts = pool.apply_sqlite(sea_orm::sqlx::sqlite::SqlitePoolOptions::new());

                let sqlx_pool = pool_opts.connect_with(opts.clone()).await?;

                let sea = sea_orm::SqlxSqliteConnector::from_sqlx_sqlite_pool(sqlx_pool.clone());

                let filename = opts.get_filename().display().to_string();
                let handle = DbHandle {
                    engine: crate::DbEngine::Sqlite,
                    pool: crate::DbPool::Sqlite(sqlx_pool),
                    dsn: format!("sqlite://{filename}"),
                    sea,
                };

                Ok(handle)
            }
            #[cfg(feature = "pg")]
            DbConnectOptions::Postgres(opts) => {
                let pool_opts = pool.apply_pg(sea_orm::sqlx::postgres::PgPoolOptions::new());

                let sqlx_pool = pool_opts.connect_with(opts.clone()).await?;

                let sea =
                    sea_orm::SqlxPostgresConnector::from_sqlx_postgres_pool(sqlx_pool.clone());

                let handle = DbHandle {
                    engine: crate::DbEngine::Postgres,
                    pool: crate::DbPool::Postgres(sqlx_pool),
                    dsn: format!(
                        "postgresql://<redacted>@{}:{}/{}",
                        opts.get_host(),
                        opts.get_port(),
                        opts.get_database().unwrap_or("")
                    ),
                    sea,
                };

                Ok(handle)
            }
            #[cfg(feature = "mysql")]
            DbConnectOptions::MySql(opts) => {
                let pool_opts = pool.apply_mysql(sea_orm::sqlx::mysql::MySqlPoolOptions::new());

                let sqlx_pool = pool_opts.connect_with(opts.clone()).await?;

                let sea = sea_orm::SqlxMySqlConnector::from_sqlx_mysql_pool(sqlx_pool.clone());

                let handle = DbHandle {
                    engine: crate::DbEngine::MySql,
                    pool: crate::DbPool::MySql(sqlx_pool),
                    dsn: "mysql://<redacted>@...".to_owned(),
                    sea,
                };

                Ok(handle)
            }
            #[cfg(not(any(feature = "sqlite", feature = "pg", feature = "mysql")))]
            _ => {
                unreachable!("No database features enabled")
            }
        }
    }
}

/// `SQLite` PRAGMA whitelist and validation.
#[cfg(feature = "sqlite")]
pub mod sqlite_pragma {
    use crate::DbError;
    use std::collections::HashMap;
    use std::hash::BuildHasher;

    /// Whitelisted `SQLite` PRAGMA parameters.
    const ALLOWED_PRAGMAS: &[&str] = &["wal", "synchronous", "busy_timeout", "journal_mode"];

    /// Validate and apply `SQLite` PRAGMA parameters to connection options.
    ///
    /// # Errors
    /// Returns `DbError::UnknownSqlitePragma` if an unsupported pragma is provided.
    /// Returns `DbError::InvalidSqlitePragmaValue` if a pragma value is invalid.
    pub fn apply_pragmas<S: BuildHasher>(
        mut opts: sea_orm::sqlx::sqlite::SqliteConnectOptions,
        params: &HashMap<String, String, S>,
    ) -> crate::Result<sea_orm::sqlx::sqlite::SqliteConnectOptions> {
        for (key, value) in params {
            let key_lower = key.to_lowercase();

            if !ALLOWED_PRAGMAS.contains(&key_lower.as_str()) {
                return Err(DbError::UnknownSqlitePragma(key.clone()));
            }

            match key_lower.as_str() {
                "wal" => {
                    let journal_mode = validate_wal_pragma(value)?;
                    opts = opts.pragma("journal_mode", journal_mode);
                }
                "journal_mode" => {
                    let mode = validate_journal_mode_pragma(value)?;
                    opts = opts.pragma("journal_mode", mode);
                }
                "synchronous" => {
                    let sync_mode = validate_synchronous_pragma(value)?;
                    opts = opts.pragma("synchronous", sync_mode);
                }
                "busy_timeout" => {
                    let timeout = validate_busy_timeout_pragma(value)?;
                    opts = opts.pragma("busy_timeout", timeout.to_string());
                }
                _ => unreachable!("Checked against whitelist above"),
            }
        }

        Ok(opts)
    }

    /// Validate WAL PRAGMA value.
    fn validate_wal_pragma(value: &str) -> crate::Result<&'static str> {
        match value.to_lowercase().as_str() {
            "true" | "1" => Ok("WAL"),
            "false" | "0" => Ok("DELETE"),
            _ => Err(DbError::InvalidSqlitePragma {
                key: "wal".to_owned(),
                message: format!("must be true/false/1/0, got '{value}'"),
            }),
        }
    }

    /// Validate synchronous PRAGMA value.
    fn validate_synchronous_pragma(value: &str) -> crate::Result<String> {
        match value.to_uppercase().as_str() {
            "OFF" | "NORMAL" | "FULL" | "EXTRA" => Ok(value.to_uppercase()),
            _ => Err(DbError::InvalidSqlitePragma {
                key: "synchronous".to_owned(),
                message: format!("must be OFF/NORMAL/FULL/EXTRA, got '{value}'"),
            }),
        }
    }

    /// Validate `busy_timeout` PRAGMA value.
    fn validate_busy_timeout_pragma(value: &str) -> crate::Result<i64> {
        let timeout = value
            .parse::<i64>()
            .map_err(|_| DbError::InvalidSqlitePragma {
                key: "busy_timeout".to_owned(),
                message: format!("must be a non-negative integer, got '{value}'"),
            })?;

        if timeout < 0 {
            return Err(DbError::InvalidSqlitePragma {
                key: "busy_timeout".to_owned(),
                message: format!("must be non-negative, got '{timeout}'"),
            });
        }

        Ok(timeout)
    }

    /// Validate `journal_mode` PRAGMA value.
    fn validate_journal_mode_pragma(value: &str) -> crate::Result<String> {
        match value.to_uppercase().as_str() {
            "DELETE" | "WAL" | "MEMORY" | "TRUNCATE" | "PERSIST" | "OFF" => {
                Ok(value.to_uppercase())
            }
            _ => Err(DbError::InvalidSqlitePragma {
                key: "journal_mode".to_owned(),
                message: format!("must be DELETE/WAL/MEMORY/TRUNCATE/PERSIST/OFF, got '{value}'"),
            }),
        }
    }
}

/// Build a database handle from configuration.
/// This is the unified entry point for building database handles from configuration.
///
/// # Errors
/// Returns an error if the database connection fails or configuration is invalid.
pub async fn build_db_handle(
    mut cfg: DbConnConfig,
    _global: Option<&GlobalDatabaseConfig>,
) -> Result<DbHandle> {
    // Expand environment variables in DSN and password
    if let Some(dsn) = &cfg.dsn {
        cfg.dsn = Some(expand_env_vars(dsn)?);
    }
    if let Some(password) = &cfg.password {
        cfg.password = Some(resolve_password(password)?);
    }

    // Expand environment variables in params
    if let Some(ref mut params) = cfg.params {
        for (_, value) in params.iter_mut() {
            if value.contains("${") {
                *value = expand_env_vars(value)?;
            }
        }
    }

    // Validate configuration for conflicts
    validate_config_consistency(&cfg)?;

    // Determine database engine and build connection options.
    let engine = determine_engine(&cfg)?;
    let connect_options = match engine {
        DbEngineCfg::Sqlite => build_sqlite_options(&cfg)?,
        DbEngineCfg::Postgres | DbEngineCfg::Mysql => build_server_options(&cfg, engine)?,
    };

    // Build pool configuration
    let pool_cfg = cfg.pool.unwrap_or_default();

    // Log connection attempt (without credentials)
    let log_dsn = redact_credentials_in_dsn(cfg.dsn.as_deref());
    tracing::debug!(dsn = log_dsn, engine = ?engine, "Building database connection");

    // Connect to database
    let handle = connect_options.connect(pool_cfg).await?;

    Ok(handle)
}

fn determine_engine(cfg: &DbConnConfig) -> Result<DbEngineCfg> {
    // If both engine and DSN are provided, validate they don't conflict.
    // (We do the same check in validate_config_consistency, but keep this here to ensure
    // determine_engine() never returns a misleading value.)
    if let Some(engine) = cfg.engine {
        if let Some(dsn) = cfg.dsn.as_deref() {
            let inferred = engine_from_dsn(dsn)?;
            if inferred != engine {
                return Err(DbError::ConfigConflict(format!(
                    "engine='{engine:?}' conflicts with DSN scheme inferred as '{inferred:?}'"
                )));
            }
        }
        return Ok(engine);
    }

    // If DSN is not provided, engine is required.
    //
    // Rationale:
    // - Without DSN we cannot reliably distinguish Postgres vs MySQL.
    // - For SQLite we also want explicit intent (file/path alone is not a transport selector).
    if cfg.dsn.is_none() {
        return Err(DbError::InvalidParameter(
            "Missing 'engine': required when 'dsn' is not provided".to_owned(),
        ));
    }

    // Infer from DSN scheme when present.
    let Some(dsn) = cfg.dsn.as_deref() else {
        // SAFETY: guarded above by `cfg.dsn.is_none()`.
        return Err(DbError::InvalidParameter(
            "Missing 'dsn': required to infer database engine".to_owned(),
        ));
    };
    engine_from_dsn(dsn)
}

fn engine_from_dsn(dsn: &str) -> Result<DbEngineCfg> {
    let s = dsn.trim_start();
    if s.starts_with("postgres://") || s.starts_with("postgresql://") {
        Ok(DbEngineCfg::Postgres)
    } else if s.starts_with("mysql://") {
        Ok(DbEngineCfg::Mysql)
    } else if s.starts_with("sqlite:") || s.starts_with("sqlite://") {
        Ok(DbEngineCfg::Sqlite)
    } else {
        Err(DbError::UnknownDsn(dsn.to_owned()))
    }
}

/// Build `SQLite` connection options from configuration.
#[cfg(feature = "sqlite")]
fn build_sqlite_options(cfg: &DbConnConfig) -> Result<DbConnectOptions> {
    let db_path = if let Some(dsn) = &cfg.dsn {
        parse_sqlite_path_from_dsn(dsn)?
    } else if let Some(path) = &cfg.path {
        path.clone()
    } else if let Some(_file) = &cfg.file {
        // This should not happen as manager.rs should have resolved file to path
        return Err(DbError::InvalidParameter(
            "File path should have been resolved to absolute path".to_owned(),
        ));
    } else {
        return Err(DbError::InvalidParameter(
            "SQLite connection requires either DSN, path, or file".to_owned(),
        ));
    };

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut opts = sea_orm::sqlx::sqlite::SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);

    // Apply PRAGMA parameters with whitelist validation
    if let Some(params) = &cfg.params {
        opts = sqlite_pragma::apply_pragmas(opts, params)?;
    }

    Ok(DbConnectOptions::Sqlite(opts))
}

#[cfg(not(feature = "sqlite"))]
fn build_sqlite_options(_: &DbConnConfig) -> Result<DbConnectOptions> {
    Err(DbError::FeatureDisabled("SQLite feature not enabled"))
}

/// Build server-based connection options from configuration.
fn build_server_options(cfg: &DbConnConfig, engine: DbEngineCfg) -> Result<DbConnectOptions> {
    // When neither `pg` nor `mysql` features are enabled, the match arms that would use `cfg`
    // are compiled out, but the function still needs to compile cleanly under `-D warnings`.
    #[cfg(not(any(feature = "pg", feature = "mysql")))]
    let _ = cfg;

    match engine {
        DbEngineCfg::Postgres => {
            #[cfg(feature = "pg")]
            {
                let mut opts = if let Some(dsn) = &cfg.dsn {
                    dsn.parse::<sea_orm::sqlx::postgres::PgConnectOptions>()
                        .map_err(|e| DbError::InvalidParameter(e.to_string()))?
                } else {
                    sea_orm::sqlx::postgres::PgConnectOptions::new()
                };

                // Override with individual fields
                if let Some(host) = &cfg.host {
                    opts = opts.host(host);
                }
                if let Some(port) = cfg.port {
                    opts = opts.port(port);
                }
                if let Some(user) = &cfg.user {
                    opts = opts.username(user);
                }
                if let Some(password) = &cfg.password {
                    opts = opts.password(password);
                }
                if let Some(dbname) = &cfg.dbname {
                    opts = opts.database(dbname);
                } else if cfg.dsn.is_none() {
                    return Err(DbError::InvalidParameter(
                        "dbname is required for PostgreSQL connections".to_owned(),
                    ));
                }

                // Apply additional parameters
                if let Some(params) = &cfg.params {
                    for (key, value) in params {
                        opts = opts.options([(key.as_str(), value.as_str())]);
                    }
                }

                Ok(DbConnectOptions::Postgres(opts))
            }
            #[cfg(not(feature = "pg"))]
            {
                Err(DbError::FeatureDisabled("PostgreSQL feature not enabled"))
            }
        }
        DbEngineCfg::Mysql => {
            #[cfg(feature = "mysql")]
            {
                let mut opts = if let Some(dsn) = &cfg.dsn {
                    dsn.parse::<sea_orm::sqlx::mysql::MySqlConnectOptions>()
                        .map_err(|e| DbError::InvalidParameter(e.to_string()))?
                } else {
                    sea_orm::sqlx::mysql::MySqlConnectOptions::new()
                };

                // Override with individual fields
                if let Some(host) = &cfg.host {
                    opts = opts.host(host);
                }
                if let Some(port) = cfg.port {
                    opts = opts.port(port);
                }
                if let Some(user) = &cfg.user {
                    opts = opts.username(user);
                }
                if let Some(password) = &cfg.password {
                    opts = opts.password(password);
                }
                if let Some(dbname) = &cfg.dbname {
                    opts = opts.database(dbname);
                } else if cfg.dsn.is_none() {
                    return Err(DbError::InvalidParameter(
                        "dbname is required for MySQL connections".to_owned(),
                    ));
                }

                Ok(DbConnectOptions::MySql(opts))
            }
            #[cfg(not(feature = "mysql"))]
            {
                Err(DbError::FeatureDisabled("MySQL feature not enabled"))
            }
        }
        DbEngineCfg::Sqlite => Err(DbError::InvalidParameter(
            "build_server_options called with sqlite engine".to_owned(),
        )),
    }
}

/// Parse `SQLite` path from DSN.
#[cfg(feature = "sqlite")]
fn parse_sqlite_path_from_dsn(dsn: &str) -> Result<std::path::PathBuf> {
    if dsn.starts_with("sqlite:") {
        let path_part = dsn
            .strip_prefix("sqlite:")
            .ok_or_else(|| DbError::InvalidParameter("Invalid SQLite DSN".to_owned()))?;
        let path_part = if path_part.starts_with("//") {
            path_part
                .strip_prefix("//")
                .ok_or_else(|| DbError::InvalidParameter("Invalid SQLite DSN".to_owned()))?
        } else {
            path_part
        };

        // Remove query parameters
        let path_part = if let Some(pos) = path_part.find('?') {
            &path_part[..pos]
        } else {
            path_part
        };

        Ok(std::path::PathBuf::from(path_part))
    } else {
        Err(DbError::InvalidParameter(format!(
            "Invalid SQLite DSN: {dsn}"
        )))
    }
}

/// Expand environment variables in a string.
fn expand_env_vars(input: &str) -> Result<String> {
    let re = regex::Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}")
        .map_err(|e| DbError::InvalidParameter(e.to_string()))?;
    let mut result = input.to_owned();

    for caps in re.captures_iter(input) {
        let full_match = &caps[0];
        let var_name = &caps[1];
        let value = std::env::var(var_name)?;
        result = result.replace(full_match, &value);
    }

    Ok(result)
}

/// Resolve password from environment variable if it starts with ${VAR}.
fn resolve_password(password: &str) -> Result<String> {
    if password.starts_with("${") && password.ends_with('}') {
        let var_name = &password[2..password.len() - 1];
        Ok(std::env::var(var_name)?)
    } else {
        Ok(password.to_owned())
    }
}

/// Validate configuration for consistency and detect conflicts.
fn validate_config_consistency(cfg: &DbConnConfig) -> Result<()> {
    // Validate engine against DSN if both are present
    if let (Some(engine), Some(dsn)) = (cfg.engine, cfg.dsn.as_deref()) {
        let inferred = engine_from_dsn(dsn)?;
        if inferred != engine {
            return Err(DbError::ConfigConflict(format!(
                "engine='{engine:?}' conflicts with DSN scheme inferred as '{inferred:?}'"
            )));
        }
    }

    // Check for SQLite vs server engine conflicts
    if let Some(dsn) = &cfg.dsn {
        let is_sqlite_dsn = dsn.starts_with("sqlite");
        let has_sqlite_fields = cfg.file.is_some() || cfg.path.is_some();
        let has_server_fields = cfg.host.is_some() || cfg.port.is_some();

        if is_sqlite_dsn && has_server_fields {
            return Err(DbError::ConfigConflict(
                "SQLite DSN cannot be used with host/port fields".to_owned(),
            ));
        }

        if !is_sqlite_dsn && has_sqlite_fields {
            return Err(DbError::ConfigConflict(
                "Non-SQLite DSN cannot be used with file/path fields".to_owned(),
            ));
        }

        // Check for server vs non-server DSN conflicts
        if !is_sqlite_dsn
            && cfg.server.is_some()
            && (cfg.host.is_some()
                || cfg.port.is_some()
                || cfg.user.is_some()
                || cfg.password.is_some()
                || cfg.dbname.is_some())
        {
            // This is actually allowed - server provides base config, DSN can override
            // Fields here override DSN parts intentionally.
        }
    }

    // Check for SQLite-specific conflicts
    if cfg.file.is_some() && cfg.path.is_some() {
        return Err(DbError::ConfigConflict(
            "Cannot specify both 'file' and 'path' for SQLite - use one or the other".to_owned(),
        ));
    }

    if (cfg.file.is_some() || cfg.path.is_some()) && (cfg.host.is_some() || cfg.port.is_some()) {
        return Err(DbError::ConfigConflict(
            "SQLite file/path fields cannot be used with host/port fields".to_owned(),
        ));
    }

    // If engine explicitly says SQLite, reject server connection fields early (even without DSN)
    if cfg.engine == Some(DbEngineCfg::Sqlite)
        && (cfg.host.is_some()
            || cfg.port.is_some()
            || cfg.user.is_some()
            || cfg.password.is_some()
            || cfg.dbname.is_some())
    {
        return Err(DbError::ConfigConflict(
            "engine=sqlite cannot be used with host/port/user/password/dbname fields".to_owned(),
        ));
    }

    // If engine explicitly says server-based, reject sqlite file/path early (even without DSN)
    if matches!(cfg.engine, Some(DbEngineCfg::Postgres | DbEngineCfg::Mysql))
        && (cfg.file.is_some() || cfg.path.is_some())
    {
        return Err(DbError::ConfigConflict(
            "engine=postgres/mysql cannot be used with file/path fields".to_owned(),
        ));
    }

    Ok(())
}

/// Redact credentials from DSN for logging.
#[must_use]
pub fn redact_credentials_in_dsn(dsn: Option<&str>) -> String {
    match dsn {
        Some(dsn) if dsn.contains('@') => {
            if let Ok(mut parsed) = url::Url::parse(dsn) {
                if parsed.password().is_some() {
                    let _ = parsed.set_password(Some("***"));
                }
                parsed.to_string()
            } else {
                "***".to_owned()
            }
        }
        Some(dsn) => dsn.to_owned(),
        None => "none".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determine_engine_requires_engine_when_dsn_missing() {
        let cfg = DbConnConfig {
            dsn: None,
            engine: None,
            ..Default::default()
        };

        let err = determine_engine(&cfg).unwrap_err();
        assert!(matches!(err, DbError::InvalidParameter(_)));
        assert!(err.to_string().contains("Missing 'engine'"));
    }

    #[test]
    fn determine_engine_infers_from_dsn_when_engine_missing() {
        let cfg = DbConnConfig {
            engine: None,
            dsn: Some("sqlite::memory:".to_owned()),
            ..Default::default()
        };

        let engine = determine_engine(&cfg).unwrap();
        assert_eq!(engine, DbEngineCfg::Sqlite);
    }

    #[test]
    fn engine_and_dsn_match_ok() {
        let cases = [
            (DbEngineCfg::Postgres, "postgres://user:pass@localhost/db"),
            (DbEngineCfg::Postgres, "postgresql://user:pass@localhost/db"),
            (DbEngineCfg::Mysql, "mysql://user:pass@localhost/db"),
            (DbEngineCfg::Sqlite, "sqlite::memory:"),
            (DbEngineCfg::Sqlite, "sqlite:///tmp/test.db"),
        ];

        for (engine, dsn) in cases {
            let cfg = DbConnConfig {
                engine: Some(engine),
                dsn: Some(dsn.to_owned()),
                ..Default::default()
            };
            validate_config_consistency(&cfg).unwrap();
            assert_eq!(determine_engine(&cfg).unwrap(), engine);
        }
    }

    #[test]
    fn engine_and_dsn_mismatch_is_error() {
        let cases = [
            (DbEngineCfg::Postgres, "mysql://user:pass@localhost/db"),
            (DbEngineCfg::Mysql, "postgres://user:pass@localhost/db"),
            (DbEngineCfg::Sqlite, "postgresql://user:pass@localhost/db"),
        ];

        for (engine, dsn) in cases {
            let cfg = DbConnConfig {
                engine: Some(engine),
                dsn: Some(dsn.to_owned()),
                ..Default::default()
            };

            let err = validate_config_consistency(&cfg).unwrap_err();
            assert!(matches!(err, DbError::ConfigConflict(_)));
        }
    }

    #[test]
    fn unknown_dsn_is_error() {
        let cfg = DbConnConfig {
            engine: None,
            dsn: Some("unknown://localhost/db".to_owned()),
            ..Default::default()
        };

        // Consistency validation doesn't validate unknown schemes unless `engine` is set,
        // but engine determination must fail.
        validate_config_consistency(&cfg).unwrap();
        let err = determine_engine(&cfg).unwrap_err();
        assert!(matches!(err, DbError::UnknownDsn(_)));
    }
}
