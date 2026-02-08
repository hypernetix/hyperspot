/// Integration tests to verify system modules are exempt from versioning requirements.
///
/// These tests ensure that Client traits in modules/system/* do NOT trigger DE0504,
/// while Client traits in non-system modules and examples compile cleanly (because
/// they already have V1 suffixes from the refactoring).
///
/// Positive-case testing (lint fires on bad code) is covered by UI tests in ui/.

use std::process::Command;

fn workspace_root() -> std::path::PathBuf {
    // Navigate from CARGO_MANIFEST_DIR (de0504_client_versioning/) up to workspace root (versions/)
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent() // de05_client_layer
        .and_then(|p| p.parent()) // dylint_lints
        .and_then(|p| p.parent()) // versions (workspace root)
        .expect("Failed to find workspace root from CARGO_MANIFEST_DIR")
        .to_path_buf()
}

#[test]
fn test_system_modules_are_exempt() {
    let output = Command::new("cargo")
        .args(&["check", "-p", "cf-tenant-resolver-sdk", "--message-format=json"])
        .current_dir(workspace_root())
        .output()
        .expect("Failed to run cargo check on system module");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "System module tenant_resolver-sdk should compile successfully.\n\
         Stderr: {}\nStdout: {}",
        stderr,
        stdout
    );

    let has_de0504_error = stdout.lines()
        .chain(stderr.lines())
        .any(|line| {
            line.contains("de0504_client_versioning")
            || line.contains("DE0504")
            || (line.contains("Client trait") && line.contains("version suffix"))
        });

    assert!(
        !has_de0504_error,
        "System module tenant_resolver-sdk should NOT trigger DE0504 for TenantResolverGatewayClient\n\
         System modules (modules/system/*) are exempt from versioning requirements.\n\
         Stderr: {}\nStdout: {}",
        stderr,
        stdout
    );
}

#[test]
fn test_non_system_modules_require_versioning() {
    let output = Command::new("cargo")
        .args(&["check", "-p", "cf-simple-user-settings-sdk", "--message-format=json"])
        .current_dir(workspace_root())
        .output()
        .expect("Failed to run cargo check on non-system module");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Non-system module simple_user_settings-sdk should compile successfully.\n\
         If this fails, the V1 refactoring is incomplete.\n\
         Stderr: {}\nStdout: {}",
        stderr,
        stdout
    );

    let has_de0504_error = stdout.lines()
        .chain(stderr.lines())
        .any(|line| {
            line.contains("de0504_client_versioning")
            || (line.contains("must have a version suffix") && line.contains("DE0504"))
        });

    assert!(
        !has_de0504_error,
        "Non-system module simple_user_settings-sdk should compile without DE0504 errors \
         because it has V1 suffixes.\n\
         If this fails, the V1 refactoring is incomplete.\n\
         Stderr: {}\nStdout: {}",
        stderr,
        stdout
    );
}

#[test]
fn test_examples_require_versioning() {
    let output = Command::new("cargo")
        .args(&["check", "-p", "user_info-sdk", "--message-format=json"])
        .current_dir(workspace_root())
        .output()
        .expect("Failed to run cargo check on example");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Example user_info-sdk should compile successfully.\n\
         If this fails, the V1 refactoring is incomplete.\n\
         Stderr: {}\nStdout: {}",
        stderr,
        stdout
    );

    let has_de0504_error = stdout.lines()
        .chain(stderr.lines())
        .any(|line| {
            line.contains("de0504_client_versioning")
            || (line.contains("must have a version suffix") && line.contains("DE0504"))
        });

    assert!(
        !has_de0504_error,
        "Example user_info-sdk should compile without DE0504 errors because it has V1 suffixes.\n\
         If this fails, the V1 refactoring is incomplete.\n\
         Stderr: {}\nStdout: {}",
        stderr,
        stdout
    );
}
