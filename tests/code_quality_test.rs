//! Code quality enforcement tests
//!
//! This module contains tests that enforce code quality standards across the codebase.

use std::process::Command;

#[test]
fn test_no_clippy_allow_attributes() {
    // Search for clippy allow attributes in src/ and tests/
    let src_allows = find_clippy_allows("src/");
    let test_allows = find_clippy_allows("tests/");

    let mut all_allows = Vec::new();
    all_allows.extend(src_allows);
    all_allows.extend(test_allows);

    if !all_allows.is_empty() {
        let error_message = format!(
            "Found {} clippy allow attributes in the codebase. \
            This violates the zero-tolerance policy for allow attributes.\n\
            \n\
            Found allow attributes:\n{}\n\
            \n\
            Policy: Fix the underlying clippy warnings instead of suppressing them.\n\
            If you must add an allow attribute, get explicit team approval first.\n\
            \n\
            To fix this:\n\
            1. Remove the allow attribute\n\
            2. Fix the underlying clippy warning\n\
            3. If the warning cannot be fixed, create a GitHub issue for team review\n\
            \n\
            See CLAUDE.md section 'Code Quality Enforcement - CRITICAL' for details.",
            all_allows.len(),
            all_allows.join("\n")
        );

        panic!("{}", error_message);
    }
}

/// Find all clippy allow attributes in the specified directory
fn find_clippy_allows(dir_path: &str) -> Vec<String> {
    let mut allows = Vec::new();

    // Search for function/item-level allow attributes: #[allow(clippy::...)]
    if let Ok(output) = Command::new("grep")
        .args([
            "-rn",
            "--include=*.rs",
            "^[[:space:]]*#\\[allow(clippy::",
            dir_path,
        ])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if !line.trim().is_empty() {
                    allows.push(format!("  ITEM-LEVEL: {}", line.trim()));
                }
            }
        }
    }

    // Search for crate-level allow attributes: #![allow(clippy::...)]
    if let Ok(output) = Command::new("grep")
        .args([
            "-rn",
            "--include=*.rs",
            "^[[:space:]]*#!\\[allow(clippy::",
            dir_path,
        ])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if !line.trim().is_empty() {
                    allows.push(format!("  CRATE-LEVEL: {}", line.trim()));
                }
            }
        }
    }

    allows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_clippy_allows_function_works() {
        // This test ensures our allow detection function works correctly
        // It should find allows in the current test directory
        let test_allows = find_clippy_allows("tests/");

        // We expect to find some allows in tests/ based on our grep results
        // This validates our detection mechanism is working
        assert!(
            !test_allows.is_empty(),
            "Expected to find some clippy allows in tests/ directory for validation"
        );

        // Verify the format of detected allows
        for allow in &test_allows {
            assert!(
                allow.contains("ITEM-LEVEL:") || allow.contains("CRATE-LEVEL:"),
                "Allow format should include level prefix: {allow}"
            );
        }
    }
}
