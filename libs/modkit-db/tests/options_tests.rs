#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for options module functionality.

use modkit_db::{build_db_handle, DbConnConfig, DbEngine, PoolCfg};
use std::collections::HashMap;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::test]
async fn test_build_db_handle_sqlite_memory() {
    let config = DbConnConfig {
        dsn: Some("sqlite::memory:".to_string()),
        params: Some({
            let mut params = HashMap::new();
            params.insert("journal_mode".to_string(), "WAL".to_string());
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
            params.insert("journal_mode".to_string(), "DELETE".to_string());
            params.insert("synchronous".to_string(), "NORMAL".to_string());
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
    std::env::set_var("TEST_DB_PASSWORD", "secret123");

    let config = DbConnConfig {
        dsn: Some("sqlite::memory:".to_string()),
        password: Some("${TEST_DB_PASSWORD}".to_string()),
        ..Default::default()
    };

    let result = build_db_handle(config, None).await;
    assert!(result.is_ok());

    // Clean up
    std::env::remove_var("TEST_DB_PASSWORD");
}

#[tokio::test]
async fn test_build_db_handle_invalid_env_var() {
    let config = DbConnConfig {
        dsn: Some("sqlite::memory:".to_string()),
        password: Some("${NONEXISTENT_VAR}".to_string()),
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
        dsn: Some("sqlite::memory:".to_string()),
        params: Some({
            let mut params = HashMap::new();
            params.insert("invalid_pragma".to_string(), "some_value".to_string());
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
        dsn: Some("sqlite::memory:".to_string()),
        params: Some({
            let mut params = HashMap::new();
            params.insert("journal_mode".to_string(), "INVALID_MODE".to_string());
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
        dsn: Some("sqlite::memory:".to_string()),
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
        server: Some("postgres".to_string()),
        host: Some("localhost".to_string()),
        port: Some(5432),
        user: Some("testuser".to_string()),
        password: Some("testpass".to_string()),
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
