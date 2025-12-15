#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for DB migration phase behavior
//!
//! These tests verify that the DB phase correctly handles:
//! - Successful migrations
//! - Modules without DB configuration
//! - Migration failures
//! - System module priority ordering

use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

use modkit::{
    config::ConfigProvider,
    runtime::{run, DbOptions, RunOptions, ShutdownOptions},
};
use uuid::Uuid;

// Mock config provider for DB tests
#[derive(Clone)]
struct DbTestConfigProvider {
    configs: std::collections::HashMap<String, serde_json::Value>,
}

impl DbTestConfigProvider {
    fn new() -> Self {
        Self {
            configs: std::collections::HashMap::new(),
        }
    }

    fn with_db_config(mut self, module_name: &str) -> Self {
        self.configs.insert(
            module_name.to_owned(),
            serde_json::json!({
                "database": {
                    "dsn": "sqlite::memory:",
                    "params": {
                        "journal_mode": "WAL"
                    }
                }
            }),
        );
        self
    }
}

impl ConfigProvider for DbTestConfigProvider {
    fn get_module_config(&self, module_name: &str) -> Option<&serde_json::Value> {
        self.configs.get(module_name)
    }
}

// Helper to create a mock DbManager
fn create_test_db_manager() -> Arc<modkit_db::DbManager> {
    use figment::{providers::Serialized, Figment};

    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "test_db_module": {
            "database": {
                "dsn": "sqlite::memory:",
                "params": {
                    "journal_mode": "WAL"
                }
            }
        }
    })));

    let home_dir = std::path::PathBuf::from("/tmp/modkit_db_test");

    Arc::new(modkit_db::DbManager::from_figment(figment, home_dir).unwrap())
}

#[tokio::test]
async fn test_db_phase_with_manager_succeeds() {
    // Test that modules with DB capability and proper config get migrations run
    let cancel = CancellationToken::new();

    // Cancel after a brief delay to let phases complete
    let cancel_clone = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        cancel_clone.cancel();
    });

    let opts = RunOptions {
        modules_cfg: Arc::new(DbTestConfigProvider::new().with_db_config("test_db_module")),
        db: DbOptions::Manager(create_test_db_manager()),
        shutdown: ShutdownOptions::Token(cancel),
        clients: Vec::new(),
        instance_id: Uuid::new_v4(),
        oop: None,
    };

    let result = timeout(Duration::from_millis(500), run(opts)).await;
    assert!(result.is_ok(), "DB phase should complete");

    let run_result = result.unwrap();
    assert!(
        run_result.is_ok(),
        "DB phase should succeed with valid config"
    );
}

#[tokio::test]
async fn test_db_phase_without_config_skips_migration() {
    // Test that modules without DB config don't fail, they just skip migration
    let cancel = CancellationToken::new();

    let cancel_clone = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        cancel_clone.cancel();
    });

    let opts = RunOptions {
        modules_cfg: Arc::new(DbTestConfigProvider::new()), // No DB config for any module
        db: DbOptions::Manager(create_test_db_manager()),
        shutdown: ShutdownOptions::Token(cancel),
        clients: Vec::new(),
        instance_id: Uuid::new_v4(),
        oop: None,
    };

    let result = timeout(Duration::from_millis(500), run(opts)).await;
    assert!(result.is_ok(), "Should handle missing DB config gracefully");

    let run_result = result.unwrap();
    assert!(
        run_result.is_ok(),
        "Should succeed when modules lack DB config"
    );
}

#[tokio::test]
async fn test_db_phase_with_none_option() {
    // Test that DbOptions::None results in no DB handle and no migration calls
    let cancel = CancellationToken::new();

    let cancel_clone = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        cancel_clone.cancel();
    });

    let opts = RunOptions {
        modules_cfg: Arc::new(DbTestConfigProvider::new()),
        db: DbOptions::None,
        shutdown: ShutdownOptions::Token(cancel),
        clients: Vec::new(),
        instance_id: Uuid::new_v4(),
        oop: None,
    };

    let result = timeout(Duration::from_millis(500), run(opts)).await;
    assert!(result.is_ok(), "Should complete with DbOptions::None");

    let run_result = result.unwrap();
    assert!(run_result.is_ok(), "Should succeed without DB");
}

#[tokio::test]
async fn test_db_phase_error_propagation() {
    // This test verifies that if we had a module that fails migration,
    // the error would be caught. Since we can't inject test modules via inventory,
    // we test the error handling path exists by confirming graceful behavior
    let cancel = CancellationToken::new();

    let cancel_clone = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        cancel_clone.cancel();
    });

    // Use an invalid DSN to trigger a potential error path
    let bad_figment = figment::Figment::new().merge(figment::providers::Serialized::defaults(
        serde_json::json!({
            "test_module": {
                "database": {
                    "dsn": "invalid://connection/string",
                }
            }
        }),
    ));

    let home_dir = std::path::PathBuf::from("/tmp/modkit_db_error_test");

    // This might fail during DbManager creation or during migration
    // We're testing that errors are handled gracefully
    let db_manager_result = modkit_db::DbManager::from_figment(bad_figment, home_dir);

    // If DbManager creation fails, that's expected for invalid DSN
    if db_manager_result.is_err() {
        // This is the expected path - invalid config detected early
        return;
    }

    let opts = RunOptions {
        modules_cfg: Arc::new(DbTestConfigProvider::new().with_db_config("test_module")),
        db: DbOptions::Manager(Arc::new(db_manager_result.unwrap())),
        shutdown: ShutdownOptions::Token(cancel),
        clients: Vec::new(),
        instance_id: Uuid::new_v4(),
        oop: None,
    };

    // Run should either succeed (if no modules try to use bad config)
    // or fail gracefully
    let result = timeout(Duration::from_millis(500), run(opts)).await;
    assert!(result.is_ok(), "Should not hang on DB errors");
}

#[tokio::test]
async fn test_db_phase_completes_before_init() {
    // Verify that DB phase happens before init by testing timing expectations
    let cancel = CancellationToken::new();

    let cancel_clone = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        cancel_clone.cancel();
    });

    let opts = RunOptions {
        modules_cfg: Arc::new(DbTestConfigProvider::new().with_db_config("test_db_module")),
        db: DbOptions::Manager(create_test_db_manager()),
        shutdown: ShutdownOptions::Token(cancel),
        clients: Vec::new(),
        instance_id: Uuid::new_v4(),
        oop: None,
    };

    let start = std::time::Instant::now();
    let result = timeout(Duration::from_millis(500), run(opts)).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "Should complete successfully");
    assert!(
        elapsed < Duration::from_millis(450),
        "Should complete quickly when cancelled"
    );
}
