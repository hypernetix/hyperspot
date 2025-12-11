#![allow(clippy::unwrap_used, clippy::expect_used, clippy::use_debug)]

//! Tests for concurrency and caching behavior of DbManager.

use figment::{providers::Serialized, Figment};
use modkit_db::manager::DbManager;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

/// Test race condition: two concurrent get() calls for the same module.
/// Only one handle should be built, both callers should get the same Arc.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_concurrent_get_same_module() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "file": format!("concurrent_same_{}.db", std::process::id())
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager =
        Arc::new(DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap());

    // Launch two concurrent get() calls for the same module
    let manager1 = manager.clone();
    let manager2 = manager.clone();

    let (result1, result2) = tokio::join!(manager1.get("test_module"), manager2.get("test_module"));

    // Both should succeed
    let handle1 = result1.unwrap().expect("First call should return a handle");
    let handle2 = result2
        .unwrap()
        .expect("Second call should return a handle");

    // Both should be the same Arc (same pointer)
    assert!(
        Arc::ptr_eq(&handle1, &handle2),
        "Both calls should return the same Arc<DbHandle>"
    );
}

/// Test concurrent get() calls for different modules.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_concurrent_get_different_modules() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "module_a": {
                "database": {
                    "file": format!("module_a_{}.db", std::process::id())
                }
            },
            "module_b": {
                "database": {
                    "file": format!("module_b_{}.db", std::process::id())
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager =
        Arc::new(DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap());

    // Launch concurrent get() calls for different modules
    let manager1 = manager.clone();
    let manager2 = manager.clone();

    let (result1, result2) = tokio::join!(manager1.get("module_a"), manager2.get("module_b"));

    // Both should succeed
    let handle1 = result1.unwrap().expect("First call should return a handle");
    let handle2 = result2
        .unwrap()
        .expect("Second call should return a handle");

    // Should be different Arc instances
    assert!(
        !Arc::ptr_eq(&handle1, &handle2),
        "Different modules should have different handles"
    );

    // Verify correct database files were used
    assert!(handle1.dsn().contains("module_a_"));
    assert!(handle2.dsn().contains("module_b_"));
}

/// Test caching behavior: second call for same module should return cached handle.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_caching_behavior() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "file": format!("caching_test_{}.db", std::process::id())
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    // First call
    let handle1 = manager
        .get("test_module")
        .await
        .unwrap()
        .expect("First call should succeed");

    // Second call - should return cached handle
    let handle2 = manager
        .get("test_module")
        .await
        .unwrap()
        .expect("Second call should succeed");

    // Should be the same Arc
    assert!(
        Arc::ptr_eq(&handle1, &handle2),
        "Second call should return cached handle"
    );
}

/// Test behavior on unknown module.
#[tokio::test]
async fn test_unknown_module_behavior() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "known_module": {
                "database": {
                    "file": format!("known_{}.db", std::process::id())
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    // Request unknown module
    let result = manager.get("unknown_module").await;

    match result {
        Ok(None) => {
            // This is the expected behavior: no config = None return
        }
        Ok(Some(_)) => {
            panic!("Expected None for unknown module, got Some(handle)");
        }
        Err(err) => {
            panic!("Expected Ok(None) for unknown module, got error: {:?}", err);
        }
    }
}

/// Test concurrent access with mixed success/failure scenarios.
#[tokio::test]
async fn test_concurrent_mixed_scenarios() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "valid_module": {
                "database": {
                    "file": format!("valid_{}.db", std::process::id())
                }
            },
            "invalid_module": {
                "database": {
                    "dsn": format!("sqlite:file:mixed_invalid_{}.db", std::process::id()),
                    "host": "localhost"  // Conflict: SQLite DSN with host field
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager =
        Arc::new(DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap());

    // Launch concurrent calls for valid and invalid modules
    let manager1 = manager.clone();
    let manager2 = manager.clone();
    let manager3 = manager.clone();

    let (result1, result2, result3) = tokio::join!(
        manager1.get("valid_module"),
        manager2.get("invalid_module"),
        manager3.get("nonexistent_module")
    );

    // Valid module should succeed
    assert!(result1.is_ok() && result1.as_ref().unwrap().is_some());

    // Invalid module should fail with config conflict
    assert!(result2.is_err());

    // Nonexistent module should return None
    assert!(result3.is_ok() && result3.as_ref().unwrap().is_none());
}

/// Test performance: many concurrent requests for the same module.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_concurrent_performance() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "file": format!("perf_test_{}.db", std::process::id())
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager =
        Arc::new(DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap());

    // Launch many concurrent requests
    let mut tasks = Vec::new();
    for _ in 0..50 {
        let manager_clone = manager.clone();
        let task = tokio::spawn(async move { manager_clone.get("test_module").await });
        tasks.push(task);
    }

    // Wait for all tasks with timeout
    let results = timeout(Duration::from_secs(10), async {
        let mut results = Vec::new();
        for task in tasks {
            results.push(task.await.unwrap());
        }
        results
    })
    .await
    .expect("All tasks should complete within timeout");

    // All should succeed and return the same handle
    let first_handle = results[0].as_ref().unwrap().as_ref().unwrap();

    for result in &results {
        let handle = result.as_ref().unwrap().as_ref().unwrap();
        assert!(
            Arc::ptr_eq(first_handle, handle),
            "All concurrent calls should return the same cached handle"
        );
    }

    println!("Successfully handled {} concurrent requests", results.len());
}

/// Test cache behavior across different manager instances.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_cache_isolation_across_managers() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "test_module": {
                "database": {
                    "file": format!("isolation_test_{}.db", std::process::id())
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();

    // Create two separate manager instances
    let manager1 = DbManager::from_figment(figment.clone(), temp_dir.path().to_path_buf()).unwrap();
    let manager2 = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    // Get handles from both managers
    let handle1 = manager1.get("test_module").await.unwrap().unwrap();
    let handle2 = manager2.get("test_module").await.unwrap().unwrap();

    // Should be different Arc instances (different caches)
    assert!(
        !Arc::ptr_eq(&handle1, &handle2),
        "Different managers should have separate caches"
    );

    // But should point to the same database
    assert_eq!(handle1.dsn(), handle2.dsn());
}

/// Test that errors are not cached.
#[tokio::test]
async fn test_errors_not_cached() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "bad_module": {
                "database": {
                    "dsn": format!("sqlite:file:error_test_{}.db", std::process::id()),
                    "host": "localhost"  // Conflict
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager = DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap();

    // First call should fail
    let result1 = manager.get("bad_module").await;
    assert!(result1.is_err());

    // Second call should also fail (errors should not be cached)
    let result2 = manager.get("bad_module").await;
    assert!(result2.is_err());

    // Both should be the same type of error
    match (result1, result2) {
        (Err(err1), Err(err2)) => {
            assert_eq!(std::mem::discriminant(&err1), std::mem::discriminant(&err2));
        }
        _ => panic!("Both calls should fail"),
    }
}

/// Test concurrent initialization with slow database connections.
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_concurrent_slow_initialization() {
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
        "modules": {
            "slow_module": {
                "database": {
                    "file": format!("slow_test_{}.db", std::process::id()),
                    "pool": {
                        "max_conns": 1,           // Force serialization
                        "acquire_timeout": "5s"   // Longer timeout
                    }
                }
            }
        }
    })));

    let temp_dir = TempDir::new().unwrap();
    let manager =
        Arc::new(DbManager::from_figment(figment, temp_dir.path().to_path_buf()).unwrap());

    // Launch multiple concurrent requests
    let manager1 = manager.clone();
    let manager2 = manager.clone();
    let manager3 = manager.clone();

    let start = std::time::Instant::now();

    let (result1, result2, result3) = tokio::join!(
        manager1.get("slow_module"),
        manager2.get("slow_module"),
        manager3.get("slow_module")
    );

    let elapsed = start.elapsed();

    // All should succeed
    let handle1 = result1.unwrap().unwrap();
    let handle2 = result2.unwrap().unwrap();
    let handle3 = result3.unwrap().unwrap();

    // All should be the same handle
    assert!(Arc::ptr_eq(&handle1, &handle2));
    assert!(Arc::ptr_eq(&handle2, &handle3));

    // Should complete in reasonable time (not 3x slower due to concurrency)
    assert!(
        elapsed < Duration::from_secs(10),
        "Concurrent initialization took too long: {:?}",
        elapsed
    );

    println!("Concurrent slow initialization completed in {:?}", elapsed);
}
