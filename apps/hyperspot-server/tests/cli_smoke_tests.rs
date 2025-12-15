#![allow(clippy::unwrap_used, clippy::expect_used, clippy::non_ascii_literal)]

//! CLI smoke tests for hyperspot-server binary
//!
//! These tests verify that the CLI commands work correctly, including
//! configuration validation, help output, and basic command functionality.

use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

/// Helper to run the hyperspot-server binary with given arguments
fn run_hyperspot_server(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_hyperspot-server"))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute hyperspot-server")
}

/// Helper to run the hyperspot-server binary with timeout
async fn run_hyperspot_server_with_timeout(
    args: &[&str],
    timeout_duration: Duration,
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let mut cmd = tokio::process::Command::new(env!("CARGO_BIN_EXE_hyperspot-server"));
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true); // Ensure process is killed if dropped

    let child = cmd.spawn()?;

    match timeout(timeout_duration, child.wait_with_output()).await {
        Ok(result) => result.map_err(Into::into),
        Err(_elapsed) => {
            // Timeout occurred - this is actually expected for server runs
            Err("elapsed".into())
        }
    }
}

#[test]
fn test_cli_help_command() {
    let output = run_hyperspot_server(&["--help"]);

    assert!(output.status.success(), "Help command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("hyperspot-server") || stdout.contains("HyperSpot"),
        "Should contain binary name"
    );
    assert!(
        stdout.contains("Usage:") || stdout.contains("USAGE:"),
        "Should contain usage information"
    );
    assert!(stdout.contains("run"), "Should contain 'run' subcommand");
    assert!(
        stdout.contains("check"),
        "Should contain 'check' subcommand"
    );
    assert!(stdout.contains("--config"), "Should mention config option");
}

#[test]
fn test_cli_version_command() {
    let output = run_hyperspot_server(&["--version"]);

    assert!(output.status.success(), "Version command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("hyperspot-server"),
        "Should contain binary name"
    );
    // Version might be 0.1.0 or similar
    assert!(
        stdout.chars().any(|c| c.is_ascii_digit()),
        "Should contain version numbers"
    );
}

#[test]
fn test_cli_invalid_command() {
    let output = run_hyperspot_server(&["invalid-command"]);

    assert!(!output.status.success(), "Invalid command should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error") || stderr.contains("invalid") || stderr.contains("unexpected"),
        "Should contain error message about invalid command"
    );
}

#[test]
fn test_cli_config_validation_missing_file() {
    let output = run_hyperspot_server(&["--config", "/nonexistent/config.yaml", "check"]);

    // The application should fail when an explicitly specified config file doesn't exist
    assert!(
        !output.status.success(),
        "Should fail when config file doesn't exist"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("does not exist")
            || stderr.contains("not found")
            || stderr.contains("config"),
        "Should indicate config file not found: {stderr}"
    );
}

#[test]
fn test_cli_config_validation_invalid_yaml() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("invalid.yaml");

    // Write invalid YAML
    std::fs::write(&config_path, "invalid: yaml: content: [unclosed")
        .expect("Failed to write file");

    let output = run_hyperspot_server(&["--config", config_path.to_str().unwrap(), "check"]);

    assert!(!output.status.success(), "Should fail with invalid YAML");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("yaml") || stderr.contains("parse") || stderr.contains("format"),
        "Should mention YAML parsing issue: {stderr}"
    );
}

#[test]
fn test_cli_config_validation_valid_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("valid.yaml");

    // Write valid configuration
    let config_content = r#"
database:
  servers:
    test_sqlite:
      dsn: "sqlite:///tmp/test.db"

logging:
  # global section
  default:
    console_level: info
    file: "logs/hyperspot.log"
    file_level: info
    max_age_days: 28
    max_backups: 3
    max_size_mb: 1000
"#;

    std::fs::write(&config_path, config_content).expect("Failed to write config file");

    let output = run_hyperspot_server(&["--config", config_path.to_str().unwrap(), "check"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("STDERR: {stderr}");
        eprintln!("STDOUT: {stdout}");
    }

    assert!(output.status.success(), "Should succeed with valid config");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain some indication of successful validation
    assert!(
        stdout.contains("valid")
            || stdout.contains("OK")
            || stdout.contains("success")
            || stdout.is_empty(),
        "Should indicate successful validation or be empty: {stdout}"
    );
}

#[tokio::test]
async fn test_cli_run_command_with_mock_database() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("test.yaml");

    // Use a random port to avoid conflicts with other tests
    let test_port = 8900 + (std::process::id() % 100) as u16;

    // Write minimal test configuration - the --mock flag will override the database
    let config_content = format!(
        r#"
database:
  servers:
    test_postgres:
      dsn: "postgresql://localhost/nonexistent"

logging:
  default:
    console_level: error
    file: "{}"
    file_level: error
    max_age_days: 1
    max_backups: 1
    max_size_mb: 10

server:
  home_dir: "{}"

modules:
  api_ingress:
    config:
      bind_addr: "127.0.0.1:{}"
      enable_docs: false
      cors_enabled: false
      auth_disabled: true
"#,
        temp_dir
            .path()
            .join("test.log")
            .to_string_lossy()
            .replace('\\', "/"),
        temp_dir.path().to_string_lossy().replace('\\', "/"),
        test_port
    );

    std::fs::write(&config_path, config_content).expect("Failed to write config file");

    // Use --mock flag to bypass database connection issues
    let result = run_hyperspot_server_with_timeout(
        &["--config", config_path.to_str().unwrap(), "--mock", "run"],
        Duration::from_secs(3),
    )
    .await;

    // Server should start and timeout (which means it was running)
    match result {
        Err(err) => {
            // Timeout is expected - server was running
            if err.to_string().contains("elapsed") {
                println!(
                    "✓ Server started successfully with mock database (timed out as expected)"
                );
            } else {
                eprintln!("Server failed to start: {err}");
                panic!("Server should start successfully with mock database");
            }
        }
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                // If it completed successfully, that's also fine
                println!("✓ Server completed successfully with mock database");
            } else {
                eprintln!("Server failed to start:");
                eprintln!("STDOUT: {stdout}");
                eprintln!("STDERR: {stderr}");
                panic!("Server should start successfully with mock database");
            }
        }
    }
}

#[test]
fn test_cli_run_command_config_validation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("invalid.yaml");

    // Write configuration with invalid bind address
    let config_content = r#"
database:
  servers:
    test_sqlite:
      dsn: "sqlite:///tmp/test.db"

logging:
  level: "info"
"#;

    std::fs::write(&config_path, config_content).expect("Failed to write config file");

    let output = run_hyperspot_server(&["--config", config_path.to_str().unwrap(), "run"]);

    assert!(
        !output.status.success(),
        "Should fail with invalid bind address"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("address") || stderr.contains("parse") || stderr.contains("invalid"),
        "Should mention address parsing issue: {stderr}"
    );
}

#[test]
fn test_cli_mock_flag() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("mock.yaml");

    // Write configuration with PostgreSQL (which should be overridden by --mock)
    let config_content = r#"
database:
  servers:
    test_postgres:
      dsn: "postgresql://localhost/nonexistent"

logging:
  default:
    console_level: error
    file: "logs/hyperspot.log"
    file_level: error
"#;

    std::fs::write(&config_path, config_content).expect("Failed to write config file");

    // The check command with --mock should succeed even with invalid PostgreSQL config
    let output =
        run_hyperspot_server(&["--config", config_path.to_str().unwrap(), "--mock", "check"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("STDERR: {stderr}");
        eprintln!("STDOUT: {stdout}");
    }

    assert!(
        output.status.success(),
        "Should succeed with mock database even if PostgreSQL config is invalid"
    );
}

#[test]
fn test_cli_verbose_flag() {
    let output = run_hyperspot_server(&["--verbose", "--help"]);

    assert!(output.status.success(), "Verbose help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should still show help output
    assert!(
        stdout.contains("Usage:") || stdout.contains("USAGE:"),
        "Should still contain usage information"
    );
}

#[test]
fn test_cli_config_flag_short_form() {
    // Test short form of config flag with missing file
    let output = run_hyperspot_server(&["-c", "/nonexistent/config.yaml", "check"]);

    // The application should fail when an explicitly specified config file doesn't exist
    assert!(
        !output.status.success(),
        "Should fail when config file doesn't exist using short flag"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("does not exist")
            || stderr.contains("not found")
            || stderr.contains("config"),
        "Should indicate config file not found using short flag: {stderr}"
    );
}

#[test]
fn test_cli_check_with_database_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("db_config.yaml");

    // Write configuration with database servers
    let config_content = format!(
        r#"
database:
  servers:
    test_sqlite:
      dsn: "sqlite://{}/test.db"
      params:
        journal_mode: "WAL"

logging:
  default:
    console_level: error
    file: "{}"
    file_level: error
"#,
        temp_dir.path().to_string_lossy().replace('\\', "/"),
        temp_dir
            .path()
            .join("test.log")
            .to_string_lossy()
            .replace('\\', "/")
    );

    std::fs::write(&config_path, config_content).expect("Failed to write config file");

    let output = run_hyperspot_server(&["--config", config_path.to_str().unwrap(), "check"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("STDERR: {stderr}");
        eprintln!("STDOUT: {stdout}");
    }

    assert!(
        output.status.success(),
        "Should succeed with valid database config"
    );
}

#[test]
fn test_cli_check_without_database_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("no_db_config.yaml");

    // Write configuration without database section
    let config_content = format!(
        r#"
logging:
  default:
    console_level: error
    file: "{}"
    file_level: error

server:
  home_dir: "{}"
"#,
        temp_dir
            .path()
            .join("test.log")
            .to_string_lossy()
            .replace('\\', "/"),
        temp_dir.path().to_string_lossy().replace('\\', "/")
    );

    std::fs::write(&config_path, config_content).expect("Failed to write config file");

    let output = run_hyperspot_server(&["--config", config_path.to_str().unwrap(), "check"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("STDERR: {stderr}");
        eprintln!("STDOUT: {stdout}");
    }

    // Should succeed even without database config (modules can run without DB)
    assert!(
        output.status.success(),
        "Should succeed without database config"
    );
}

#[test]
fn test_cli_subcommand_help() {
    // Test help for run subcommand
    let output = run_hyperspot_server(&["run", "--help"]);

    assert!(
        output.status.success(),
        "Run subcommand help should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("run") || stdout.contains("server"),
        "Should contain information about run command"
    );

    // Test help for check subcommand
    let output = run_hyperspot_server(&["check", "--help"]);

    assert!(
        output.status.success(),
        "Check subcommand help should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("check") || stdout.contains("configuration"),
        "Should contain information about check command"
    );
}

#[test]
fn test_cli_config_precedence() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("precedence.yaml");

    // Write minimal valid configuration
    let config_content = r#"
database:
  servers:
    test_sqlite:
      dsn: "sqlite:///tmp/test.db"

logging:
  default:
    console_level: debug
    file: "logs/hyperspot.log"
    file_level: debug
"#;

    std::fs::write(&config_path, config_content).expect("Failed to write config file");

    let output = run_hyperspot_server(&["--config", config_path.to_str().unwrap(), "check"]);

    assert!(
        output.status.success(),
        "Should succeed with valid minimal config"
    );
}

#[tokio::test]
async fn test_cli_no_arguments() {
    // When no subcommand is provided, the app may default to 'run' and keep running.
    // Use a short timeout and accept timeout as success (server started).
    match run_hyperspot_server_with_timeout(&[], Duration::from_secs(2)).await {
        Err(e) if e.to_string().contains("elapsed") => {
            // Timed out: treated as success because server is running.
        }
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stdout.contains("Usage:")
                    || stdout.contains("USAGE:")
                    || stderr.contains("required")
                    || stderr.contains("subcommand")
                    || stderr.contains("Error")
                    || stdout.contains("help")
                    || stdout.contains("HyperSpot Server starting"),
                "Should show usage, help, or run with potential error"
            );
        }
        Err(other) => panic!("Unexpected failure: {other}"),
    }
}
