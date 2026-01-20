#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for options module functionality.

use modkit_db::{build_db_handle, DbConnConfig, DbEngine, PoolCfg};
use std::collections::HashMap;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::test]
async fn test_build_db_handle_sqlite_memory() {
    let config = DbConnConfig {
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

#[tokio::test]
async fn test_build_db_handle_sqlite_file() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let config = DbConnConfig {
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

#[tokio::test]
async fn test_build_db_handle_env_expansion() {
    // Set a test environment variable
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { std::env::set_var("TEST_DB_PASSWORD", "secret123") };

    let config = DbConnConfig {
        dsn: Some("sqlite::memory:".to_owned()),
        password: Some("${TEST_DB_PASSWORD}".to_owned()),
        ..Default::default()
    };

    let result = build_db_handle(config, None).await;
    assert!(result.is_ok());

    // Clean up
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { std::env::remove_var("TEST_DB_PASSWORD") };
}

#[tokio::test]
async fn test_build_db_handle_invalid_env_var() {
    let config = DbConnConfig {
        dsn: Some("sqlite::memory:".to_owned()),
        password: Some("${NONEXISTENT_VAR}".to_owned()),
        ..Default::default()
    };

    let result = build_db_handle(config, None).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(error.to_string().contains("environment variable not found"));
}

#[tokio::test]
async fn test_build_db_handle_invalid_sqlite_pragma() {
    let config = DbConnConfig {
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

#[tokio::test]
async fn test_build_db_handle_invalid_journal_mode() {
    let config = DbConnConfig {
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
    assert!(error
        .to_string()
        .contains("must be DELETE/WAL/MEMORY/TRUNCATE/PERSIST/OFF"));
}

#[tokio::test]
async fn test_build_db_handle_pool_config() {
    let config = DbConnConfig {
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

#[cfg(feature = "pg")]
#[tokio::test]
async fn test_build_db_handle_postgres_missing_dbname() {
    let config = DbConnConfig {
        server: Some("postgres".to_owned()),
        host: Some("localhost".to_owned()),
        port: Some(5432),
        user: Some("testuser".to_owned()),
        password: Some("testpass".to_owned()),
        // Missing dbname
        ..Default::default()
    };

    let result = build_db_handle(config, None).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    println!("Actual error: {error}");
    assert!(error
        .to_string()
        .contains("dbname is required for PostgreSQL connections"));
}

#[tokio::test]
async fn test_credential_redaction() {
    // This test ensures that sensitive information is not logged
    // We can't easily test the actual logging output, but we can test the function
    use modkit_db::options::redact_credentials_in_dsn;

    let dsn_with_password = Some("postgresql://user:secret@localhost/db");
    let redacted = redact_credentials_in_dsn(dsn_with_password);
    assert!(!redacted.contains("secret"));
    assert!(redacted.contains("***"));

    let dsn_without_password = Some("sqlite::memory:");
    let not_redacted = redact_credentials_in_dsn(dsn_without_password);
    assert_eq!(not_redacted, "sqlite::memory:");

    let no_dsn = redact_credentials_in_dsn(None);
    assert_eq!(no_dsn, "none");
}

#[cfg(feature = "sqlite")]
#[test]
fn test_display_sqlite_memory() {
    use modkit_db::DbConnectOptions;

    let opts = sqlx::sqlite::SqliteConnectOptions::new().filename(":memory:");
    let db_opts = DbConnectOptions::Sqlite(opts);

    let display_str = format!("{db_opts}");
    assert_eq!(display_str, "sqlite://:memory:");
}

#[cfg(feature = "sqlite")]
#[test]
fn test_display_sqlite_file() {
    use modkit_db::DbConnectOptions;

    let opts = sqlx::sqlite::SqliteConnectOptions::new().filename("/tmp/test.db");
    let db_opts = DbConnectOptions::Sqlite(opts);

    let display_str = format!("{db_opts}");
    assert_eq!(display_str, "sqlite:///tmp/test.db");
}

#[cfg(feature = "sqlite")]
#[test]
fn test_display_sqlite_relative_path() {
    use modkit_db::DbConnectOptions;

    let opts = sqlx::sqlite::SqliteConnectOptions::new().filename("./data/test.db");
    let db_opts = DbConnectOptions::Sqlite(opts);

    let display_str = format!("{db_opts}");
    assert_eq!(display_str, "sqlite://./data/test.db");
}

#[cfg(feature = "pg")]
#[test]
fn test_display_postgres() {
    use modkit_db::DbConnectOptions;

    let opts = sqlx::postgres::PgConnectOptions::new()
        .host("localhost")
        .port(5432)
        .database("testdb")
        .username("user")
        .password("secret");

    let db_opts = DbConnectOptions::Postgres(opts);

    let display_str = format!("{db_opts}");
    assert_eq!(display_str, "postgresql://<redacted>@localhost:5432/testdb");
    assert!(!display_str.contains("secret"));
    assert!(!display_str.contains("user"));
}

#[cfg(feature = "pg")]
#[test]
fn test_display_postgres_custom_port() {
    use modkit_db::DbConnectOptions;

    let opts = sqlx::postgres::PgConnectOptions::new()
        .host("db.example.com")
        .port(15432)
        .database("myapp");

    let db_opts = DbConnectOptions::Postgres(opts);

    let display_str = format!("{db_opts}");
    assert_eq!(
        display_str,
        "postgresql://<redacted>@db.example.com:15432/myapp"
    );
}

#[cfg(feature = "pg")]
#[test]
fn test_display_postgres_no_database() {
    use modkit_db::DbConnectOptions;

    let opts = sqlx::postgres::PgConnectOptions::new()
        .host("localhost")
        .port(5432);

    let db_opts = DbConnectOptions::Postgres(opts);

    let display_str = format!("{db_opts}");
    assert_eq!(display_str, "postgresql://<redacted>@localhost:5432/");
}

#[cfg(feature = "mysql")]
#[test]
fn test_display_mysql() {
    use modkit_db::DbConnectOptions;

    let opts = sqlx::mysql::MySqlConnectOptions::new()
        .host("localhost")
        .port(3306)
        .database("testdb")
        .username("user")
        .password("secret");

    let db_opts = DbConnectOptions::MySql(opts);

    let display_str = format!("{db_opts}");
    assert_eq!(display_str, "mysql://<redacted>@...");
    assert!(!display_str.contains("secret"));
    assert!(!display_str.contains("user"));
    assert!(!display_str.contains("localhost"));
    assert!(!display_str.contains("testdb"));
}
