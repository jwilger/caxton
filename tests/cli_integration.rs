//! CLI Integration Tests
//!
//! Tests for the Caxton CLI binary functionality using outside-in approach

use std::process::Command;

#[test]
fn test_cli_version_flag_returns_success() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "caxton-cli", "--", "--version"])
        .output()
        .expect("Failed to execute CLI command");

    assert!(
        output.status.success(),
        "CLI --version command should exit successfully"
    );
}

#[test]
fn test_cli_help_flag_returns_success() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "caxton-cli", "--", "--help"])
        .output()
        .expect("Failed to execute CLI command");

    assert!(
        output.status.success(),
        "CLI --help command should exit successfully"
    );
}

#[test]
fn test_cli_recognizes_serve_subcommand() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "caxton-cli", "--", "serve"])
        .output()
        .expect("Failed to execute CLI command");

    assert!(
        output.status.success(),
        "CLI should recognize 'serve' as a valid subcommand"
    );
}

#[test]
fn test_cli_invalid_subcommand_produces_helpful_error_message() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "caxton-cli", "--", "invalid-subcommand"])
        .output()
        .expect("Failed to execute CLI command");

    let stderr_text = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr_text.contains("error: unrecognized subcommand 'invalid-subcommand'"),
        "Error message should clearly identify the unrecognized subcommand. Actual stderr: {stderr_text}"
    );
}
