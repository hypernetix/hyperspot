#![allow(clippy::use_debug)]
#![allow(clippy::non_ascii_literal)]

//! Demo of the enhanced advisory locks with namespacing and try_lock functionality.

use modkit_db::{ConnectOpts, DbHandle, LockConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set up a temporary SQLite database for demonstration
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("demo.db");
    let dsn = format!("sqlite://{}", db_path.display());

    let db = DbHandle::connect(&dsn, ConnectOpts::default()).await?;
    println!("Connected to database: {dsn}");

    // Demo 1: Basic namespaced locking
    println!("\n=== Demo 1: Basic Namespaced Locking ===");
    {
        let _guard1 = db.lock("payments", "process_batch").await?;
        println!("✓ Acquired lock: payments:process_batch");

        let _guard2 = db.lock("inventory", "process_batch").await?;
        println!("✓ Acquired lock: inventory:process_batch (different namespace)");

        // Both locks can coexist because they're in different namespaces
        println!("✓ Both locks coexist due to namespace separation");
    }
    println!("✓ Both locks automatically released");

    // Demo 2: try_lock with timeout
    println!("\n=== Demo 2: try_lock with Retry Policy ===");

    // First, acquire a lock in one thread/context
    let _guard = db.lock("user_mgmt", "bulk_update").await?;
    println!("✓ Acquired lock: user_mgmt:bulk_update");

    // Now try to acquire the same lock with a timeout policy
    let config = LockConfig {
        max_wait: Some(Duration::from_millis(500)),
        initial_backoff: Duration::from_millis(50),
        max_backoff: Duration::from_millis(200),
        backoff_multiplier: 1.5,
        jitter_pct: 0.2,
        max_attempts: Some(5),
    };

    let start = std::time::Instant::now();
    match db.try_lock("user_mgmt", "bulk_update", config).await? {
        Some(_guard) => {
            println!("✓ Unexpectedly acquired lock (this shouldn't happen)");
        }
        None => {
            let elapsed = start.elapsed();
            println!("✓ Lock acquisition timed out after {elapsed:?} (expected)");
            println!("  This demonstrates the retry/backoff policy in action");
        }
    }

    // Demo 3: Successful try_lock
    println!("\n=== Demo 3: Successful try_lock ===");

    let config = LockConfig::default();
    match db.try_lock("analytics", "daily_report", config).await? {
        Some(_guard) => {
            println!("✓ Successfully acquired lock: analytics:daily_report");
            // Do some work...
            tokio::time::sleep(Duration::from_millis(100)).await;
            println!("✓ Work completed, lock will be released automatically");
        }
        None => {
            println!("✗ Failed to acquire lock (unexpected)");
        }
    }

    // Demo 4: Show lock file behavior for SQLite
    println!("\n=== Demo 4: Lock File Information ===");
    {
        let _guard = db.lock("sysinfo", "host_update").await?;
        println!("✓ Acquired lock: sysinfo:host_update");

        // For SQLite, the lock is stored as a file in the cache directory
        if let Some(cache_dir) = dirs::cache_dir() {
            let lock_dir = cache_dir.join("hyperspot").join("locks");
            println!("  Lock files are stored in: {}", lock_dir.display());
            println!("  Each lock creates a file with PID and timestamp for debugging");
        }
    }
    println!("✓ Lock released, file cleaned up");

    println!("\n=== Demo Complete ===");
    println!("Key features demonstrated:");
    println!("• Module namespacing prevents conflicts between different modules");
    println!("• try_lock provides configurable retry/backoff policies");
    println!("• File-based locks for SQLite with automatic cleanup");
    println!("• All locks are automatically released on guard drop");

    Ok(())
}
