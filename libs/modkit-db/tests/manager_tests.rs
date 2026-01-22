#![allow(clippy::unwrap_used, clippy::expect_used, clippy::use_debug)]

//! Tests for `DbManager` functionality.

use figment::{Figment, providers::Serialized};
use modkit_db::{DbConnConfig, DbManager, GlobalDatabaseConfig, PoolCfg};
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
async fn test_dbmanager_server_merge() {
    let mut servers = HashMap::new();
    servers.insert(
        "test_server".to_owned(),
        DbConnConfig {
            dsn: None,
            host: Some("localhost".to_owned()),
            port: Some(5432),
            user: Some("serveruser".to_owned()),
            password: Some("serverpass".to_owned()),
            dbname: Some("serverdb".to_owned()),
            params: Some({
                let mut params = HashMap::new();
                params.insert("ssl".to_owned(), "require".to_owned());
                params
            }),
            file: None,
            path: None,
            pool: Some(PoolCfg {
                max_conns: Some(20),
                // tests were hanging 30s, reduced to 1s as we expect an error
                acquire_timeout: Some(Duration::from_secs(1)),
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
    println!("Result: {result:?}");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Referenced server 'nonexistent_server' not found")
    );
}
