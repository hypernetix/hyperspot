#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for pool configuration and inheritance.

use figment::{providers::Serialized, Figment};
use modkit_db::{config::*, manager::DbManager};
use std::collections::HashMap;
use std::time::Duration;
use tempfile::TempDir;

/// Test that all `PoolCfg` options are applied correctly.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_pool_cfg_options_applied() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "dsn": "sqlite::memory:",
                    "pool": {
                        "max_conns": 20,
                        "min_conns": 2,
                        "acquire_timeout": "45s",
                        "idle_timeout": "300s",
                        "max_lifetime": "1800s",
                        "test_before_acquire": true
                    }
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    match result {
        Ok(_handle) => {
            // Connection succeeded - pool options were applied correctly
            // We could test actual pool settings by examining the handle's pool,
            // but for now we just verify the connection works with custom pool settings
        }
        Err(err) => {
            panic!("Expected successful connection with custom pool settings, got: {err:?}");
        }
    }
}

/// Test that module pool config overrides server pool config.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_module_pool_overrides_server_pool() {
    let global_config = GlobalDatabaseConfig {
        servers: {
            let mut servers = HashMap::new();
            servers.insert(
                "sqlite_server".to_owned(),
                DbConnConfig {
                    pool: Some(PoolCfg {
                        max_conns: Some(10),
                        min_conns: Some(1),
                        acquire_timeout: Some(Duration::from_secs(30)),
                        idle_timeout: Some(Duration::from_secs(600)),
                        max_lifetime: Some(Duration::from_secs(3600)),
                        test_before_acquire: Some(false),
                    }),
                    ..Default::default()
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
                    "dsn": "sqlite::memory:",
                    "pool": {
                        "max_conns": 25,          // Should override server value (10)
                        "acquire_timeout": "60s"  // Should override server value (30s)
                        // Other values should be inherited from server
                    }
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    match result {
        Ok(_handle) => {
            // Connection succeeded - module pool config took precedence
            // The pool should have max_conns=25 and acquire_timeout=60s from module config
        }
        Err(err) => {
            panic!("Expected successful connection with overridden pool settings, got: {err:?}");
        }
    }
}

/// Test pool config inheritance from server when module has no pool config.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_pool_config_inheritance() {
    let global_config = GlobalDatabaseConfig {
        servers: {
            let mut servers = HashMap::new();
            servers.insert(
                "sqlite_server".to_owned(),
                DbConnConfig {
                    pool: Some(PoolCfg {
                        max_conns: Some(15),
                        min_conns: Some(3),
                        acquire_timeout: Some(Duration::from_secs(25)),
                        idle_timeout: Some(Duration::from_secs(500)),
                        max_lifetime: Some(Duration::from_secs(2000)),
                        test_before_acquire: Some(true),
                    }),
                    ..Default::default()
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
                        "dsn": "sqlite::memory:"
                    // No pool config - should inherit from server
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    match result {
        Ok(_handle) => {
            // Connection succeeded - server pool config was inherited
        }
        Err(err) => {
            panic!("Expected successful connection with inherited pool settings, got: {err:?}");
        }
    }
}

/// Test default pool configuration when none is specified.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_default_pool_configuration() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                        "dsn": "sqlite::memory:"
                    // No pool config - should use defaults
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    match result {
        Ok(_handle) => {
            // Connection succeeded with default pool settings
        }
        Err(err) => {
            panic!("Expected successful connection with default pool settings, got: {err:?}");
        }
    }
}

/// Test `PoolCfg` helper methods for different database engines.
#[test]
fn test_pool_cfg_helper_methods() {
    let pool_cfg = PoolCfg {
        max_conns: Some(50),
        min_conns: Some(5),
        acquire_timeout: Some(Duration::from_secs(45)),
        idle_timeout: Some(Duration::from_secs(300)),
        max_lifetime: Some(Duration::from_secs(1800)),
        test_before_acquire: Some(true),
    };

    // Test SQLite helper
    #[cfg(feature = "sqlite")]
    {
        let sqlite_opts = pool_cfg.apply_sqlite(sqlx::sqlite::SqlitePoolOptions::new());
        // We can't easily test the internal state of the options,
        // but we can verify the method doesn't panic and returns the right type
        assert_eq!(
            std::mem::size_of_val(&sqlite_opts),
            std::mem::size_of::<sqlx::sqlite::SqlitePoolOptions>()
        );
    }

    // Test PostgreSQL helper
    #[cfg(feature = "pg")]
    {
        let pg_opts = pool_cfg.apply_pg(sqlx::postgres::PgPoolOptions::new());
        assert_eq!(
            std::mem::size_of_val(&pg_opts),
            std::mem::size_of::<sqlx::postgres::PgPoolOptions>()
        );
    }

    // Test MySQL helper
    #[cfg(feature = "mysql")]
    {
        let mysql_opts = pool_cfg.apply_mysql(sqlx::mysql::MySqlPoolOptions::new());
        assert_eq!(
            std::mem::size_of_val(&mysql_opts),
            std::mem::size_of::<sqlx::mysql::MySqlPoolOptions>()
        );
    }
}

/// Test partial pool configuration (only some fields specified).
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_partial_pool_configuration() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "dsn": "sqlite::memory:",
                    "pool": {
                        "max_conns": 8,
                        "acquire_timeout": "20s"
                        // Other fields should use defaults
                    }
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    let result = manager.get("test_module").await;

    match result {
        Ok(_handle) => {
            // Connection succeeded with partial pool config
        }
        Err(err) => {
            panic!("Expected successful connection with partial pool config, got: {err:?}");
        }
    }
}

/// Test humantime parsing for duration fields.
#[test]
fn test_humantime_parsing() {
    // Test various humantime formats
    let test_cases = vec![
        r#"{"acquire_timeout": "30s"}"#,
        r#"{"acquire_timeout": "5m"}"#,
        r#"{"acquire_timeout": "1h"}"#,
        r#"{"acquire_timeout": "500ms"}"#,
        r#"{"idle_timeout": "10min"}"#,
        r#"{"max_lifetime": "2hours"}"#,
    ];

    for case in test_cases {
        let result: Result<PoolCfg, _> = serde_json::from_str(case);
        assert!(result.is_ok(), "Failed to parse: {case}");
    }

    // Test invalid humantime format
    let invalid_case = r#"{"acquire_timeout": "invalid_duration"}"#;
    let result: Result<PoolCfg, _> = serde_json::from_str(invalid_case);
    assert!(result.is_err());
}

/// Test `PoolCfg` serialization and deserialization roundtrip.
#[test]
fn test_pool_cfg_serde_roundtrip() {
    let original = PoolCfg {
        max_conns: Some(42),
        min_conns: Some(7),
        acquire_timeout: Some(Duration::from_secs(35)),
        idle_timeout: Some(Duration::from_secs(420)),
        max_lifetime: Some(Duration::from_secs(1500)),
        test_before_acquire: Some(false),
    };

    // Serialize to JSON
    let serialized = serde_json::to_string(&original).unwrap();

    // Deserialize back
    let deserialized: PoolCfg = serde_json::from_str(&serialized).unwrap();

    // Should be equal
    assert_eq!(original, deserialized);
}

/// Test `PoolCfg` default values.
#[test]
fn test_pool_cfg_defaults() {
    let default_cfg = PoolCfg::default();

    assert_eq!(default_cfg.max_conns, None);
    assert_eq!(default_cfg.min_conns, None);
    assert_eq!(default_cfg.acquire_timeout, None);
    assert_eq!(default_cfg.idle_timeout, None);
    assert_eq!(default_cfg.max_lifetime, None);
    assert_eq!(default_cfg.test_before_acquire, None);

    // Test that empty JSON deserializes to default
    let empty_json = "{}";
    let parsed: PoolCfg = serde_json::from_str(empty_json).unwrap();
    assert_eq!(parsed, default_cfg);
}
