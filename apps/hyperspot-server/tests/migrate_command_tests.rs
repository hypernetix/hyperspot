#![allow(clippy::unwrap_used, clippy::expect_used)]

//! End-to-end tests for the `migrate` command
//!
//! These tests verify that the migrate CLI command works correctly
//! by invoking the hyperspot-server binary and checking its output.

use std::process::Command;

/// Helper to get the path to the hyperspot-server binary
fn hyperspot_binary() -> &'static str {
    env!("CARGO_BIN_EXE_hyperspot-server")
}

#[test]
fn test_migrate_command_help_text() {
    let output = Command::new(hyperspot_binary())
        .args(["migrate", "--help"])
        .output()
        .expect("failed to execute hyperspot-server");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Run database migrations and exit"),
        "Help text should describe migrate command"
    );
}

#[test]
fn test_migrate_command_runs_migration_phases() {
    let output = Command::new(hyperspot_binary())
        .arg("migrate")
        .output()
        .expect("failed to execute hyperspot-server");

    // Should complete successfully (with or without actual database)
    assert!(
        output.status.success(),
        "migrate command should exit successfully. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Only verify stable user-facing output
    assert!(
        combined.contains("[OK] Database migrations completed successfully"),
        "Should print success message to user"
    );
}
