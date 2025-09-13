//! Integration tests for the Caxton CLI
//!
//! These tests invoke the actual CLI binary to verify end-to-end behavior
//! from the user's perspective.

use std::process::Command;

/// Test that verifies the CLI binary can be invoked and returns success on --help
///
/// This is the first test in the TDD cycle for Story 001: Basic CLI Foundation.
/// It tests the most basic CLI behavior - that --help works and exits successfully.
#[test]
fn test_cli_help_returns_success() {
    // Execute: caxton-cli --help
    let output = Command::new("cargo")
        .args(["run", "--bin", "caxton-cli", "--", "--help"])
        .output()
        .expect("Failed to execute CLI command");

    // Single assertion: Command should exit successfully
    assert!(
        output.status.success(),
        "CLI --help command should exit successfully, but got exit code: {:?}",
        output.status.code()
    );
}

/// Test that verifies the CLI binary responds to --version flag with crate version
///
/// This is the second test in the TDD cycle for Story 001: Basic CLI Foundation.
/// It verifies that --version outputs the current crate version number.
#[test]
fn test_cli_version_contains_crate_version() {
    // Execute: caxton-cli --version
    let output = Command::new("cargo")
        .args(["run", "--bin", "caxton-cli", "--", "--version"])
        .output()
        .expect("Failed to execute CLI command");

    // Convert output to string for version checking
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Single assertion: Version output should contain the crate version
    assert!(
        stdout.contains("0.1.4"),
        "CLI --version should contain version '0.1.4', but got: {stdout}"
    );
}

/// Test that verifies the CLI accepts "server" as a subcommand
///
/// This test establishes the foundation for Story 002: Server Lifecycle Management.
/// It verifies that the CLI recognizes "server" as a valid subcommand and can show help.
#[test]
fn test_cli_server_subcommand_help_returns_success() {
    // Execute: caxton-cli server --help
    let output = Command::new("cargo")
        .args(["run", "--bin", "caxton-cli", "--", "server", "--help"])
        .output()
        .expect("Failed to execute CLI command");

    // Single assertion: Server subcommand help should exit successfully
    assert!(
        output.status.success(),
        "CLI server --help command should exit successfully, but got exit code: {:?}",
        output.status.code()
    );
}
