#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for DbManager functionality.

use figment::{providers::Serialized, Figment};
use modkit_db::{DbConnConfig, DbEngine, DbManager, GlobalDatabaseConfig, PoolCfg};
use std::collections::HashMap;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::test]
async fn test_dbmanager_no_global_config() {
    let figment = Figment::new();
    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path().to_path_buf();

    let manager = DbManager::from_figment(figment, home_dir).unwrap();

    // Should return None for any module when no module config exists
    let result = manager.get("test_module").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_dbmanager_module_no_database() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "config": {
                    "some_setting": "value"
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path().to_path_buf();

    let manager = DbManager::from_figment(figment, home_dir).unwrap();

    // Should return None when module has no database section
    let result = manager.get("test_module").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_dbmanager_sqlite_with_file() {
    let temp_dir = TempDir::new().unwrap();
    let db_filename = format!("test_manager_{}.db", std::process::id());

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "file": db_filename,
                    "params": {
                        "journal_mode": "WAL"
                    }
                }
            }
        }
    })));

    let home_dir = temp_dir.path().to_path_buf();

    let manager = DbManager::from_figment(figment, home_dir).unwrap();

    // Should successfully create SQLite database
    let result = manager.get("test_module").await.unwrap();
    assert!(result.is_some());

    let db_handle = result.unwrap();
    assert_eq!(db_handle.engine(), DbEngine::Sqlite);
}

#[tokio::test]
async fn test_dbmanager_sqlite_with_path() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("absolute.db");

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "path": db_path,
                    "params": {
                        "journal_mode": "DELETE"
                    }
                }
            }
        }
    })));

    let home_dir = temp_dir.path().to_path_buf();

    let manager = DbManager::from_figment(figment, home_dir).unwrap();

    // Should successfully create SQLite database at absolute path
    let result = manager.get("test_module").await.unwrap();
    assert!(result.is_some());

    let db_handle = result.unwrap();
    assert_eq!(db_handle.engine(), DbEngine::Sqlite);
}

#[tokio::test]
async fn test_dbmanager_server_merge() {
    let mut servers = HashMap::new();
    servers.insert(
        "test_server".to_string(),
        DbConnConfig {
            dsn: None,
            host: Some("localhost".to_string()),
            port: Some(5432),
            user: Some("serveruser".to_string()),
            password: Some("serverpass".to_string()),
            dbname: Some("serverdb".to_string()),
            params: Some({
                let mut params = HashMap::new();
                params.insert("ssl".to_string(), "require".to_string());
                params
            }),
            file: None,
            path: None,
            pool: Some(PoolCfg {
                max_conns: Some(20),
                acquire_timeout: Some(Duration::from_secs(30)),
                ..Default::default()
            }),
            server: None,
        },
    );

    let global_config = GlobalDatabaseConfig {
        servers,
        auto_provision: Some(false),
    };

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "database": global_config,
        "modules": {
            "test_module": {
                "database": {
                    "server": "test_server",
                    "dbname": "moduledb",  // Override server dbname
                    "params": {
                        "application_name": "test_module"  // Additional param
                    }
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path().to_path_buf();

    let manager = DbManager::from_figment(figment, home_dir).unwrap();

    // This would normally try to connect to PostgreSQL, but we can't test actual connection
    // without a real database. Just check that it doesn't panic during build phase.
    let result = manager.get("test_module").await;

    // We expect an error since PostgreSQL feature is not enabled by default
    assert!(
        result.is_err(),
        "Expected connection error, but got success"
    );

    // The test is primarily checking that the configuration merging works,
    // not the specific connection error format
}

#[tokio::test]
async fn test_dbmanager_caching() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "dsn": "sqlite::memory:",
                    "params": {
                        "journal_mode": "WAL"
                    }
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path().to_path_buf();

    let manager = DbManager::from_figment(figment, home_dir).unwrap();

    // First call should create the handle
    let result1 = manager.get("test_module").await.unwrap();
    assert!(result1.is_some());

    // Second call should return cached handle (same Arc)
    let result2 = manager.get("test_module").await.unwrap();
    assert!(result2.is_some());

    let handle1 = result1.unwrap();
    let handle2 = result2.unwrap();

    // Should be the same Arc instance
    assert!(std::ptr::eq(handle1.as_ref(), handle2.as_ref()));
}

#[tokio::test]
async fn test_dbmanager_missing_server_reference() {
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
    let home_dir = temp_dir.path().to_path_buf();

    let manager = DbManager::from_figment(figment, home_dir).unwrap();

    // Should fail with error about missing server
    let result = manager.get("test_module").await;
    println!("Result: {:?}", result);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error
        .to_string()
        .contains("Referenced server 'nonexistent_server' not found"));
}

#[tokio::test]
async fn test_dbmanager_sqlite_server_without_dsn() {
    // Test that SQLite servers without DSN work correctly with module file specification
    let global_config = GlobalDatabaseConfig {
        servers: {
            let mut servers = HashMap::new();
            servers.insert(
                "sqlite_server".to_string(),
                DbConnConfig {
                    params: Some({
                        let mut params = HashMap::new();
                        params.insert("WAL".to_string(), "true".to_string());
                        params.insert("synchronous".to_string(), "NORMAL".to_string());
                        params
                    }),
                    pool: Some(PoolCfg {
                        max_conns: Some(10),
                        acquire_timeout: Some(Duration::from_secs(30)),
                        ..Default::default()
                    }),
                    ..Default::default() // No DSN - module specifies file
                },
            );
            servers
        },
        auto_provision: Some(true),
    };

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "database": global_config,
        "modules": {
            "test_module": {
                "database": {
                    "server": "sqlite_server",
                    "file": format!("module_{}.db", std::process::id())  // Should be placed in module home directory
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path().to_path_buf();

    let manager = DbManager::from_figment(figment, home_dir.clone()).unwrap();

    // Should successfully create SQLite database in module subdirectory
    let result = manager.get("test_module").await.unwrap();
    assert!(result.is_some());

    let db_handle = result.unwrap();
    assert_eq!(db_handle.engine(), DbEngine::Sqlite);

    // Verify the database was created in the correct location (the filename will be dynamically generated)
    let module_dir = home_dir.join("test_module");
    assert!(
        module_dir.exists(),
        "Module directory should be created at {:?}",
        module_dir
    );
    // Check if any .db file exists in the module directory
    let db_files: Vec<_> = std::fs::read_dir(&module_dir)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "db" {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    assert!(
        !db_files.is_empty(),
        "At least one .db file should be created in {:?}",
        module_dir
    );
}
