use figment::Figment;
use figment::providers::Serialized;
use modkit_db::{DbConnConfig, DbEngine, DbManager, GlobalDatabaseConfig, PoolCfg};
use std::collections::HashMap;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::test]
async fn test_dbmanager_sqlite_with_file() {
    let temp_dir = TempDir::new().unwrap();
    let db_filename = format!("test_manager_{}.db", std::process::id());

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "engine": "sqlite",
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
                    "engine": "sqlite",
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

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_dbmanager_caching() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "engine": "sqlite",
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
async fn test_dbmanager_sqlite_server_without_dsn() {
    // Test that SQLite servers without DSN work correctly with module file specification
    let global_config = GlobalDatabaseConfig {
        servers: {
            let mut servers = HashMap::new();
            servers.insert(
                "sqlite_server".to_owned(),
                DbConnConfig {
                    engine: Some(modkit_db::config::DbEngineCfg::Sqlite),
                    params: Some({
                        let mut params = HashMap::new();
                        params.insert("WAL".to_owned(), "true".to_owned());
                        params.insert("synchronous".to_owned(), "NORMAL".to_owned());
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
                    "engine": "sqlite",
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
        "Module directory should be created at {module_dir:?}"
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
        "At least one .db file should be created in {module_dir:?}"
    );
}
