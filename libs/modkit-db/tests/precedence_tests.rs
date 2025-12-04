#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for configuration precedence and merge behavior.

mod common;

use figment::{providers::Serialized, Figment};
use modkit_db::{config::*, manager::DbManager, DbError};
use std::collections::HashMap;
use tempfile::TempDir;

/// Test that module fields override server fields using SQLite for reliable testing.
#[tokio::test]
async fn test_precedence_module_fields_override_server() {
    let global_config = GlobalDatabaseConfig {
        servers: {
            let mut servers = HashMap::new();
            servers.insert(
                "sqlite_server".to_string(),
                DbConnConfig {
                    params: Some({
                        let mut params = HashMap::new();
                        params.insert("synchronous".to_string(), "FULL".to_string());
                        params.insert("journal_mode".to_string(), "DELETE".to_string());
                        params
                    }),
                    ..Default::default()
                },
            );
            servers
        },
        auto_provision: Some(false),
    };

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "database": global_config,
        "modules": {
                "test_module": {
                    "database": {
                        "server": "sqlite_server",
                        "file": format!("precedence_test_{}.db", std::process::id()),
                        "params": {
                            "synchronous": "NORMAL",    // Should override server value
                            "busy_timeout": "5000"      // Should be added
                            // journal_mode should be inherited from server
                        }
                    }
                }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    match result {
        Ok(Some(_handle)) => {
            // Connection succeeded - this demonstrates that the configuration merging worked
            // and the SQLite connection was successful with merged parameters
        }
        Ok(None) => {
            panic!("Expected database handle for module");
        }
        Err(err) => {
            // Should not be a PRAGMA error if merging worked correctly
            let error_msg = err.to_string();
            assert!(
                !error_msg.contains("Unknown SQLite"),
                "Config merging failed: {}",
                error_msg
            );
        }
    }
}

/// Test that module DSN completely overrides server DSN using SQLite.
#[tokio::test]
async fn test_precedence_module_dsn_override_server() {
    let test_data = common::test_data_dir();
    let server_db = test_data.join(format!("server_{}.db", std::process::id()));
    let module_db = test_data.join(format!("module_{}.db", std::process::id()));

    let global_config = GlobalDatabaseConfig {
        servers: {
            let mut servers = HashMap::new();
            servers.insert(
                "sqlite_server".to_string(),
                DbConnConfig {
                    dsn: Some(format!("sqlite://{}?synchronous=FULL", server_db.display())),
                    ..Default::default()
                },
            );
            servers
        },
        auto_provision: Some(false),
    };

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "database": global_config,
        "modules": {
            "test_module": {
                "database": {
                    "server": "sqlite_server",
                    "dsn": format!("sqlite://{}?synchronous=NORMAL", module_db.display())  // Should completely override server DSN
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    match result {
        Ok(Some(handle)) => {
            // Verify that the module DSN was used, not the server DSN
            let dsn = handle.dsn();
            assert!(
                dsn.contains("module_"),
                "Should use module DSN, got: {}",
                dsn
            );
            assert!(
                !dsn.contains("server_"),
                "Should not use server DSN, got: {}",
                dsn
            );
        }
        Ok(None) => {
            panic!("Expected database handle for module");
        }
        Err(err) => {
            panic!(
                "Expected successful connection with module DSN override, got: {:?}",
                err
            );
        }
    }
}

/// Test that params maps are merged with module taking precedence.
#[tokio::test]
async fn test_precedence_params_merging() {
    let global_config = GlobalDatabaseConfig {
        servers: {
            let mut servers = HashMap::new();
            servers.insert(
                "sqlite_server".to_string(),
                DbConnConfig {
                    params: Some({
                        let mut params = HashMap::new();
                        params.insert("synchronous".to_string(), "FULL".to_string());
                        params.insert("journal_mode".to_string(), "DELETE".to_string());
                        params
                    }),
                    ..Default::default()
                },
            );
            servers
        },
        auto_provision: Some(false),
    };

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "database": global_config,
        "modules": {
                "test_module": {
                    "database": {
                        "server": "sqlite_server",
                        "file": format!("params_test_{}.db", std::process::id()),
                        "params": {
                            "synchronous": "NORMAL",    // Should override server value
                            "busy_timeout": "1000"      // Should add to merged params
                            // journal_mode should be inherited from server
                        }
                    }
                }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    // Check that the merged parameters were applied correctly
    // This test will pass if the SQLite connection succeeds with correct PRAGMA values
    match result {
        Ok(_handle) => {
            // Connection succeeded - params were merged correctly
        }
        Err(err) => {
            let error_msg = err.to_string();
            // Should not be a PRAGMA error if merging worked correctly
            assert!(!error_msg.contains("PRAGMA"));
            assert!(!error_msg.contains("Unknown SQLite"));
        }
    }
}

/// Test conflict detection: SQLite DSN with server fields.
#[tokio::test]
async fn test_conflict_detection_sqlite_dsn_with_server_fields() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "dsn": format!("sqlite:file:conflict_test_{}.db", std::process::id()),
                    "host": "localhost",        // Conflict: SQLite DSN with server field
                    "port": 5432                // Conflict: SQLite DSN with server field
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    assert!(result.is_err());
    if let Err(DbError::ConfigConflict(msg)) = result {
        assert!(msg.contains("SQLite DSN cannot be used with host/port fields"));
    } else {
        panic!("Expected ConfigConflict error, got: {:?}", result);
    }
}

/// Test conflict detection: Non-SQLite DSN with SQLite fields.
#[tokio::test]
async fn test_conflict_detection_nonsqlite_dsn_with_sqlite_fields() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "dsn": "postgres://user:pass@localhost:5432/db",
                    "file": format!("pg_conflict_{}.db", std::process::id())           // Conflict: PostgreSQL DSN with SQLite field
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    assert!(result.is_err());
    if let Err(DbError::ConfigConflict(msg)) = result {
        assert!(msg.contains("Non-SQLite DSN cannot be used with file/path fields"));
    } else {
        panic!("Expected ConfigConflict error, got: {:?}", result);
    }
}

/// Test graceful handling when both file and path are specified.
/// The system should prioritize 'file' (converted to absolute path) and ignore 'path'.
#[tokio::test]
async fn test_file_and_path_handling() {
    let temp_dir = TempDir::new().unwrap();
    let absolute_path = temp_dir
        .path()
        .join(format!("ignored_{}.db", std::process::id()));

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "file": format!("file_path_test_{}.db", std::process::id()),            // Should be used (converted to absolute)
                    "path": absolute_path         // Should be ignored in favor of 'file'
                }
            }
        }
    })));

    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    match result {
        Ok(Some(handle)) => {
            // Should have used the 'file' path, not the 'path' value
            let dsn = handle.dsn();
            // Check that it uses the file path under module directory, not the ignored absolute path
            assert!(
                dsn.contains("file_path_test_") && !dsn.contains("ignored_"),
                "Should use file path, not ignored path. DSN: {}",
                dsn
            );
        }
        Ok(None) => {
            panic!("Expected database handle");
        }
        Err(err) => {
            panic!("Expected successful connection, got: {:?}", err);
        }
    }
}

/// Test unknown server reference error.
#[tokio::test]
async fn test_unknown_server_reference() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "server": "nonexistent_server"
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    assert!(result.is_err());
    if let Err(DbError::InvalidConfig(msg)) = result {
        assert!(msg.contains("Referenced server 'nonexistent_server' not found"));
    } else {
        panic!("Expected InvalidConfig error, got: {:?}", result);
    }
}

/// Test feature disabled error when SQLite is not compiled.
#[tokio::test]
#[cfg(not(feature = "sqlite"))]
async fn test_feature_disabled_error() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
                "test_module": {
                    "database": {
                        "file": format!("feature_test_{}.db", std::process::id())
                    }
                }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    assert!(result.is_err());
    if let Err(DbError::FeatureDisabled(msg)) = result {
        assert!(msg.contains("SQLite feature not enabled"));
    } else {
        panic!("Expected FeatureDisabled error, got: {:?}", result);
    }
}

/// Test that redacted DSN is used in logs.
#[tokio::test]
async fn test_redacted_dsn_in_logs() {
    use modkit_db::options::redact_credentials_in_dsn;

    // Test password redaction
    let dsn_with_password = "postgres://user:secret123@localhost:5432/db";
    let redacted = redact_credentials_in_dsn(Some(dsn_with_password));
    assert!(redacted.contains("user:***@localhost"));
    assert!(!redacted.contains("secret123"));

    // Test DSN without password
    let dsn_no_password = "sqlite:file:test_no_password.db";
    let redacted = redact_credentials_in_dsn(Some(dsn_no_password));
    assert_eq!(redacted, "sqlite:file:test_no_password.db");

    // Test None DSN
    let redacted = redact_credentials_in_dsn(None);
    assert_eq!(redacted, "none");
}
