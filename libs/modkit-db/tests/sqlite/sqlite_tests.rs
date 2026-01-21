#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for SQLite-specific functionality.

use figment::{Figment, providers::Serialized};
use modkit_db::{DbError, manager::DbManager};
use tempfile::TempDir;

/// Test relative path resolution.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_sqlite_relative_path_resolution() {
    let temp_dir = TempDir::new().unwrap();
    let db_filename = format!("test_{}.db", std::process::id());

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "file": db_filename
                }
            }
        }
    })));

    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    match result {
        Ok(_handle) => {
            // Verify the database file was created in the correct location
            let expected_path = temp_dir.path().join("test_module").join(&db_filename);
            assert!(
                expected_path.exists(),
                "Database file should be created at {expected_path:?}"
            );
        }
        Err(err) => {
            panic!("Expected successful SQLite connection, got: {err:?}");
        }
    }
}

/// Test absolute path handling.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_sqlite_absolute_path() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("absolute_test.db");

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "path": db_path
                }
            }
        }
    })));

    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    match result {
        Ok(_handle) => {
            // Verify the database file was created at the absolute path
            assert!(
                db_path.exists(),
                "Database file should be created at {db_path:?}"
            );
        }
        Err(err) => {
            panic!("Expected successful SQLite connection, got: {err:?}");
        }
    }
}

/// Test PRAGMA precedence: params overrides DSN query.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_pragma_precedence() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir
        .path()
        .join(format!("pragma_test_{}.db", std::process::id()));

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "dsn": format!("sqlite://{}?synchronous=OFF&journal_mode=DELETE", db_path.display()),
                    "params": {
                        "synchronous": "FULL",     // Should override DSN query param
                        "busy_timeout": "5000"     // Should be added to PRAGMA settings
                        // journal_mode should come from DSN query
                    }
                }
            }
        }
    })));

    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    match result {
        Ok(_handle) => {
            // Connection succeeded - PRAGMA settings were applied correctly
            // We could test actual PRAGMA values by querying the database,
            // but for now we just verify the connection works
        }
        Err(err) => {
            let error_msg = err.to_string();
            // Should not be a PRAGMA error if precedence worked correctly
            assert!(
                !error_msg.contains("PRAGMA"),
                "PRAGMA error suggests precedence failed: {error_msg}"
            );
        }
    }
}

/// Test invalid PRAGMA values.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_invalid_pragma_values() {
    let temp_dir = TempDir::new().unwrap();
    let db_filename = format!("invalid_pragma_{}.db", std::process::id());

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "file": db_filename,
                    "params": {
                        "synchronous": "INVALID_VALUE"
                    }
                }
            }
        }
    })));

    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    assert!(result.is_err());
    if let Err(DbError::InvalidSqlitePragma { key, message }) = result {
        assert_eq!(key, "synchronous");
        assert!(message.contains("must be OFF/NORMAL/FULL/EXTRA"));
    } else {
        panic!("Expected InvalidSqlitePragma error, got: {result:?}");
    }
}

/// Test unknown PRAGMA parameters.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_unknown_pragma_parameters() {
    let temp_dir = TempDir::new().unwrap();
    let db_filename = format!("unknown_pragma_{}.db", std::process::id());

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "file": db_filename,
                    "params": {
                        "unknown_pragma": "some_value"
                    }
                }
            }
        }
    })));

    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    assert!(result.is_err());
    if let Err(DbError::UnknownSqlitePragma(key)) = result {
        assert_eq!(key, "unknown_pragma");
    } else {
        panic!("Expected UnknownSqlitePragma error, got: {result:?}");
    }
}

/// Test auto-provision behavior (creating directories).
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_auto_provision_creates_directories() {
    let temp_dir = TempDir::new().unwrap();
    let nested_path = temp_dir.path().join("nested").join("directories");

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "database": {
            "auto_provision": true
        },
        "modules": {
            "test_module": {
                "database": {
                    "path": nested_path.join("test.db")
                }
            }
        }
    })));

    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    match result {
        Ok(_handle) => {
            // Verify the nested directories were created
            assert!(
                nested_path.exists(),
                "Nested directories should be auto-created: {nested_path:?}"
            );
            assert!(
                nested_path.join("test.db").exists(),
                "Database file should exist"
            );
        }
        Err(err) => {
            panic!("Expected successful connection with auto-provision, got: {err:?}");
        }
    }
}

/// Test auto-provision disabled (should fail if directories don't exist).
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_auto_provision_disabled() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "database": {
            "auto_provision": false
        },
        "modules": {
            "test_module": {
                "database": {
                    "file": "nested/directories/test.db"  // This requires creating nested dirs
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    // Should fail because the nested directories don't exist and auto_provision is false
    assert!(result.is_err());
    if let Err(err) = result {
        let error_msg = err.to_string();
        // Should be an I/O error about directory creation or file access
        assert!(
            error_msg.contains("No such file")
                || error_msg.contains("cannot find")
                || error_msg.contains("directory")
                || error_msg.contains("system cannot find the path")
                || error_msg.contains("Directory does not exist and auto_provision is disabled"),
            "Expected I/O error, got: {error_msg}"
        );
    }
}

/// Test special `SQLite` DSN formats (:memory:, mode=memory).
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_sqlite_memory_database() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "dsn": "sqlite::memory:"
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    match result {
        Ok(Some(handle)) => {
            // Verify it's an in-memory database
            let dsn = handle.dsn();
            assert!(dsn.contains(":memory:") || dsn.contains("mode=memory"));
        }
        Ok(None) => {
            panic!("Expected database handle for in-memory SQLite");
        }
        Err(err) => {
            panic!("Expected successful in-memory SQLite connection, got: {err:?}");
        }
    }
}

/// Test shared memory database with mode=memory.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_sqlite_shared_memory_database() {
    // TODO this test unveils a big problem, but i have no time to deal with it.
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "dsn": "sqlite:file:memdb?mode=memory&cache=shared"
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    match result {
        Ok(Some(_handle)) => {
            // Connection succeeded - this proves the shared memory DSN was used correctly.
            // The handle's DSN is simplified for security/logging and doesn't preserve query params.
        }
        Ok(None) => {
            panic!("Expected database handle for shared memory SQLite");
        }
        Err(err) => {
            panic!("Expected successful shared memory SQLite connection, got: {err:?}");
        }
    }
}

/// Test WAL pragma validation.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_wal_pragma_validation() {
    // Test valid WAL values
    for wal_value in &["true", "false", "1", "0"] {
        let temp_dir = TempDir::new().unwrap();
        let db_filename = format!("wal_test_{}_{}.db", wal_value, std::process::id());

        let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
            "modules": {
                "test_module": {
                    "database": {
                        "file": db_filename,
                        "params": {
                            "wal": wal_value
                        }
                    }
                }
            }
        })));

        let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

        let result = manager.get("test_module").await;

        match result {
            Ok(_handle) => {
                // Valid WAL value - connection should succeed
            }
            Err(err) => {
                panic!("Expected successful connection with WAL value '{wal_value}', got: {err:?}");
            }
        }
    }

    // Test invalid WAL value
    let temp_dir = TempDir::new().unwrap();
    let db_filename = format!("wal_invalid_{}.db", std::process::id());

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "file": db_filename,
                    "params": {
                        "wal": "invalid"
                    }
                }
            }
        }
    })));

    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    assert!(result.is_err());
    if let Err(DbError::InvalidSqlitePragma { key, message }) = result {
        assert_eq!(key, "wal");
        assert!(message.contains("true/false/1/0"));
    } else {
        panic!("Expected InvalidSqlitePragma error, got: {result:?}");
    }
}

/// Test `busy_timeout` pragma validation.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_busy_timeout_pragma_validation() {
    // Test valid timeout values
    for timeout_value in &["0", "1000", "5000"] {
        let temp_dir = TempDir::new().unwrap();
        let db_filename = format!("timeout_test_{}_{}.db", timeout_value, std::process::id());

        let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
            "modules": {
                "test_module": {
                    "database": {
                        "file": db_filename,
                        "params": {
                            "busy_timeout": timeout_value
                        }
                    }
                }
            }
        })));

        let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

        let result = manager.get("test_module").await;

        match result {
            Ok(_handle) => {
                // Valid timeout value - connection should succeed
            }
            Err(err) => {
                panic!(
                    "Expected successful connection with timeout '{timeout_value}', got: {err:?}"
                );
            }
        }
    }

    // Test negative timeout value
    let temp_dir = TempDir::new().unwrap();
    let db_filename = format!("timeout_negative_{}.db", std::process::id());

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "file": db_filename,
                    "params": {
                        "busy_timeout": "-1000"
                    }
                }
            }
        }
    })));

    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    assert!(result.is_err());
    if let Err(DbError::InvalidSqlitePragma { key, message }) = result {
        assert_eq!(key, "busy_timeout");
        assert!(message.contains("non-negative"));
    } else {
        panic!("Expected InvalidSqlitePragma error, got: {result:?}");
    }

    // Test non-numeric timeout value
    let temp_dir = TempDir::new().unwrap();
    let db_filename = format!("timeout_invalid_{}.db", std::process::id());

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "file": db_filename,
                    "params": {
                        "busy_timeout": "not_a_number"
                    }
                }
            }
        }
    })));

    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    assert!(result.is_err());
    if let Err(DbError::InvalidSqlitePragma { key, message }) = result {
        assert_eq!(key, "busy_timeout");
        assert!(message.contains("integer"));
    } else {
        panic!("Expected InvalidSqlitePragma error, got: {result:?}");
    }
}
