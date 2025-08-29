//! Code quality enforcement tests that verify project standards are maintained.
//!
//! This module contains tests that enforce code quality standards across the
//! entire codebase, ensuring that quality rules defined in CLAUDE.md are
//! automatically enforced during the build process.

use std::fs;
use std::path::Path;

/// Test that verifies no clippy allow attributes are present in the codebase.
///
/// This test enforces the zero-tolerance policy for clippy allow attributes
/// established in CLAUDE.md. It searches all Rust source files in the src/
/// directory for clippy allow attributes and fails if any are found.
///
/// The project uses forbid-level clippy configuration in Cargo.toml which
/// cannot be overridden by allow attributes, but this test provides an
/// additional safety net and clear failure messaging when violations occur.
#[test]
fn test_no_clippy_allow_attributes() {
    let src_path = Path::new("src");
    assert!(
        src_path.exists(),
        "src directory not found - test must run from project root"
    );

    let mut violations = Vec::new();
    search_directory_for_allow_attributes(src_path, &mut violations);

    if !violations.is_empty() {
        let violation_report = violations
            .iter()
            .map(|(file, line_num, content)| format!("  {}:{}: {}", file, line_num, content.trim()))
            .collect::<Vec<_>>()
            .join("\n");

        panic!(
            "Found {} clippy allow attribute violation(s):\n{}\n\nClippy allow attributes are not permitted per CLAUDE.md zero-tolerance policy.\nFix the underlying clippy warnings instead of suppressing them.",
            violations.len(),
            violation_report
        );
    }
}

/// Recursively search a directory for clippy allow attributes.
fn search_directory_for_allow_attributes(
    dir: &Path,
    violations: &mut Vec<(String, usize, String)>,
) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                search_directory_for_allow_attributes(&path, violations);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                search_file_for_allow_attributes(&path, violations);
            }
        }
    }
}

/// Search a single Rust file for clippy allow attributes.
fn search_file_for_allow_attributes(
    file_path: &Path,
    violations: &mut Vec<(String, usize, String)>,
) {
    if let Ok(content) = fs::read_to_string(file_path) {
        for (line_num, line) in content.lines().enumerate() {
            if is_clippy_allow_violation(line) {
                violations.push((
                    file_path.display().to_string(),
                    line_num + 1, // Convert to 1-based line numbers
                    line.to_string(),
                ));
            }
        }
    }
}

/// Check if a line contains a clippy allow attribute.
fn is_clippy_allow_violation(line: &str) -> bool {
    let trimmed = line.trim();
    // Check for both attribute styles: #[allow(clippy::...)] and #![allow(clippy::...)]
    trimmed.contains("#[allow(clippy::") || trimmed.contains("#![allow(clippy::")
}
