use modkit_db::{DbConnConfig, DbEngine, PoolCfg, build_db_handle};
use std::collections::HashMap;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn test_build_db_handle_env_expansion() {
    temp_env::with_var("TEST_SQLITE_SYNC", Some("NORMAL"), || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let config = DbConnConfig {
                engine: Some(modkit_db::config::DbEngineCfg::Sqlite),
                dsn: Some("sqlite::memory:".to_owned()),
                params: Some({
                    let mut params = HashMap::new();
                    // Exercise env expansion in params
                    params.insert("synchronous".to_owned(), "${TEST_SQLITE_SYNC}".to_owned());
                    params
                }),
                ..Default::default()
            };

            let result = build_db_handle(config, None).await;
            assert!(result.is_ok(), "Expected Ok, got: {result:?}");
        });
    });
}

#[tokio::test]
async fn test_build_db_handle_sqlite_memory() {
    let config = DbConnConfig {
        engine: Some(modkit_db::config::DbEngineCfg::Sqlite),
        dsn: Some("sqlite::memory:".to_owned()),
        params: Some({
            let mut params = HashMap::new();
            params.insert("journal_mode".to_owned(), "WAL".to_owned());
            params
        }),
        ..Default::default()
    };

    let result = build_db_handle(config, None).await;
    assert!(result.is_ok());

    let handle = result.unwrap();
    assert_eq!(handle.engine(), DbEngine::Sqlite);
}

/// Verifies that a file-backed SQLite database handle can be constructed with specified PRAGMA parameters.
///
/// This test creates a temporary database file, sets `journal_mode` and `synchronous` parameters,
/// builds a database handle, and asserts the handle's engine is SQLite.
///
/// # Examples
///
/// ```
/// # use std::collections::HashMap;
/// # use tempfile::TempDir;
/// # use modkit_db::config::DbEngineCfg;
/// # use modkit_db::{DbConnConfig, DbEngine};
/// # use crate::build_db_handle;
/// let temp_dir = TempDir::new().unwrap();
/// let db_path = temp_dir.path().join("test.db");
///
/// let config = DbConnConfig {
///     engine: Some(DbEngineCfg::Sqlite),
///     path: Some(db_path),
///     params: Some({
///         let mut params = HashMap::new();
///         params.insert("journal_mode".to_owned(), "DELETE".to_owned());
///         params.insert("synchronous".to_owned(), "NORMAL".to_owned());
///         params
///     }),
///     ..Default::default()
/// };
///
/// let result = tokio_test::block_on(async { build_db_handle(config, None).await });
/// assert!(result.is_ok());
/// let handle = result.unwrap();
/// assert_eq!(handle.engine(), DbEngine::Sqlite);
/// ```
#[tokio::test]
async fn test_build_db_handle_sqlite_file() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let config = DbConnConfig {
        engine: Some(modkit_db::config::DbEngineCfg::Sqlite),
        path: Some(db_path),
        params: Some({
            let mut params = HashMap::new();
            params.insert("journal_mode".to_owned(), "DELETE".to_owned());
            params.insert("synchronous".to_owned(), "NORMAL".to_owned());
            params
        }),
        ..Default::default()
    };

    let result = build_db_handle(config, None).await;
    assert!(result.is_ok());

    let handle = result.unwrap();
    assert_eq!(handle.engine(), DbEngine::Sqlite);
}

#[test]
fn test_display_sqlite_memory() {
    use modkit_db::DbConnectOptions;

    let opts = sea_orm::sqlx::sqlite::SqliteConnectOptions::new().filename(":memory:");
    let db_opts = DbConnectOptions::Sqlite(opts);

    let display_str = format!("{db_opts}");
    assert_eq!(display_str, "sqlite://:memory:");
}

#[test]
fn test_display_sqlite_file() {
    use modkit_db::DbConnectOptions;

    let opts = sea_orm::sqlx::sqlite::SqliteConnectOptions::new().filename("/tmp/test.db");
    let db_opts = DbConnectOptions::Sqlite(opts);

    let display_str = format!("{db_opts}");
    assert_eq!(display_str, "sqlite:///tmp/test.db");
}

#[test]
fn test_display_sqlite_relative_path() {
    use modkit_db::DbConnectOptions;

    let opts = sea_orm::sqlx::sqlite::SqliteConnectOptions::new().filename("./data/test.db");
    let db_opts = DbConnectOptions::Sqlite(opts);

    let display_str = format!("{db_opts}");
    assert_eq!(display_str, "sqlite://./data/test.db");
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_build_db_handle_invalid_env_var() {
    let config = DbConnConfig {
        engine: Some(modkit_db::config::DbEngineCfg::Sqlite),
        dsn: Some("sqlite::memory:".to_owned()),
        password: Some("${NONEXISTENT_VAR}".to_owned()),
        ..Default::default()
    };

    let result = build_db_handle(config, None).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(error.to_string().contains("environment variable not found"));
}

/// Verifies that building a SQLite database handle fails when provided with an invalid PRAGMA parameter.
///
/// Asserts that `build_db_handle` returns an error and that the error message contains the invalid parameter name.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "sqlite")]
/// # async fn _example() {
/// use std::collections::HashMap;
/// use modkit_db::config::DbEngineCfg;
///
/// let config = DbConnConfig {
///     engine: Some(DbEngineCfg::Sqlite),
///     dsn: Some("sqlite::memory:".to_owned()),
///     params: Some({
///         let mut params = HashMap::new();
///         params.insert("invalid_pragma".to_owned(), "some_value".to_owned());
///         params
///     }),
///     ..Default::default()
/// };
///
/// let result = build_db_handle(config, None).await;
/// assert!(result.is_err());
/// let error = result.unwrap_err();
/// assert!(error.to_string().contains("invalid_pragma"));
/// # }
/// ```
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_build_db_handle_invalid_sqlite_pragma() {
    let config = DbConnConfig {
        engine: Some(modkit_db::config::DbEngineCfg::Sqlite),
        dsn: Some("sqlite::memory:".to_owned()),
        params: Some({
            let mut params = HashMap::new();
            params.insert("invalid_pragma".to_owned(), "some_value".to_owned());
            params
        }),
        ..Default::default()
    };

    let result = build_db_handle(config, None).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(error.to_string().contains("invalid_pragma"));
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_build_db_handle_invalid_journal_mode() {
    let config = DbConnConfig {
        engine: Some(modkit_db::config::DbEngineCfg::Sqlite),
        dsn: Some("sqlite::memory:".to_owned()),
        params: Some({
            let mut params = HashMap::new();
            params.insert("journal_mode".to_owned(), "INVALID_MODE".to_owned());
            params
        }),
        ..Default::default()
    };

    let result = build_db_handle(config, None).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(error.to_string().contains("journal_mode"));
    assert!(
        error
            .to_string()
            .contains("must be DELETE/WAL/MEMORY/TRUNCATE/PERSIST/OFF")
    );
}

/// Verifies that a SQLite database handle can be built with a custom connection pool configuration.
///
/// Builds a `DbConnConfig` specifying the SQLite engine, an in-memory DSN, and a `PoolCfg` with
/// `max_conns` and `acquire_timeout`, then ensures `build_db_handle` succeeds and the resulting
/// handle reports the `Sqlite` engine.
///
/// # Examples
///
/// ```
/// # async fn run_example() -> Result<(), Box<dyn std::error::Error>> {
/// use std::time::Duration;
/// use modkit_db::config::{DbConnConfig, PoolCfg};
/// use modkit_db::DbEngine;
///
/// let config = DbConnConfig {
///     engine: Some(modkit_db::config::DbEngineCfg::Sqlite),
///     dsn: Some("sqlite::memory:".to_owned()),
///     pool: Some(PoolCfg {
///         max_conns: Some(5),
///         acquire_timeout: Some(Duration::from_secs(10)),
///         ..Default::default()
///     }),
///     ..Default::default()
/// };
///
/// let handle = crate::build_db_handle(config, None).await?;
/// assert_eq!(handle.engine(), DbEngine::Sqlite);
/// # Ok(()) }
/// ```
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_build_db_handle_pool_config() {
    let config = DbConnConfig {
        engine: Some(modkit_db::config::DbEngineCfg::Sqlite),
        dsn: Some("sqlite::memory:".to_owned()),
        pool: Some(PoolCfg {
            max_conns: Some(5),
            acquire_timeout: Some(Duration::from_secs(10)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = build_db_handle(config, None).await;
    assert!(result.is_ok());

    let handle = result.unwrap();
    assert_eq!(handle.engine(), DbEngine::Sqlite);
}