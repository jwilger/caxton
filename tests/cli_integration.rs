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
