//! CLI Integration Tests
//!
//! Tests for the Caxton CLI binary functionality using outside-in approach

use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

#[test]
fn test_cli_version_flag_returns_success() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "caxton", "--", "--version"])
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
        .args(["run", "--bin", "caxton", "--", "--help"])
        .output()
        .expect("Failed to execute CLI command");

    assert!(
        output.status.success(),
        "CLI --help command should exit successfully"
    );
}

#[test]
fn test_cli_recognizes_serve_subcommand() {
    let mut child = Command::new("cargo")
        .args(["run", "--bin", "caxton", "--", "serve"])
        .spawn()
        .expect("Failed to execute CLI command");

    // Give the server a moment to start
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Verify the process is still running (server started successfully)
    match child.try_wait() {
        Ok(Some(_)) => {
            panic!("CLI serve command should start a long-running server, not exit immediately")
        }
        Ok(None) => {
            // Process is still running - this is what we expect
            child.kill().expect("Failed to kill serve process");
            let _ = child.wait();
        }
        Err(e) => panic!("Error checking process status: {e}"),
    }
}

#[test]
fn test_cli_invalid_subcommand_produces_helpful_error_message() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "caxton", "--", "invalid-subcommand"])
        .output()
        .expect("Failed to execute CLI command");

    let stderr_text = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr_text.contains("error: unrecognized subcommand 'invalid-subcommand'"),
        "Error message should clearly identify the unrecognized subcommand. Actual stderr: {stderr_text}"
    );
}

#[tokio::test]
async fn test_serve_command_starts_http_server_on_port_8080() {
    // Start the serve command in background
    let mut child = Command::new("cargo")
        .args(["run", "--bin", "caxton", "--", "serve"])
        .spawn()
        .expect("Failed to start serve command");

    // Give server time to start
    sleep(Duration::from_millis(100)).await;

    // Attempt to connect to HTTP server on port 8080
    let client = reqwest::Client::new();
    let response = client.get("http://localhost:8080/").send().await;

    // Clean up: terminate the serve process
    let _ = child.kill();
    let _ = child.wait();

    assert!(
        response.is_ok(),
        "HTTP server should be reachable on port 8080 after running 'caxton serve'"
    );
}
