//! Code quality enforcement tests
//!
//! This module contains tests that validate code quality standards
//! and project policies are maintained across the codebase.

// Test that verifies the codebase doesn't contain improper clippy allow attributes
use std::fs;
use std::path::Path;

#[test]
fn test_no_clippy_allow_attributes() {
    let violations = find_clippy_allow_attributes();

    if !violations.is_empty() {
        let violation_list: Vec<String> = violations
            .iter()
            .map(|(file, line, content)| format!("{file}:{line} -> {content}"))
            .collect();

        panic!(
            "Found {} clippy allow attribute violations:\n{}",
            violations.len(),
            violation_list.join("\n")
        );
    }
}

fn find_clippy_allow_attributes() -> Vec<(String, usize, String)> {
    let mut violations = Vec::new();

    if let Ok(entries) = fs::read_dir("src") {
        for entry in entries.flatten() {
            if entry.path().extension().is_some_and(|ext| ext == "rs") {
                search_file_for_violations(&entry.path(), &mut violations);
            }
        }
    }

    // Also check subdirectories in src/
    search_directory_recursive(Path::new("src"), &mut violations);

    violations
}

fn search_directory_recursive(dir: &Path, violations: &mut Vec<(String, usize, String)>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                search_directory_recursive(&path, violations);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                search_file_for_violations(&path, violations);
            }
        }
    }
}

fn search_file_for_violations(file_path: &Path, violations: &mut Vec<(String, usize, String)>) {
    if let Ok(content) = fs::read_to_string(file_path) {
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Check for problematic allow attributes
            if is_problematic_allow_attribute(trimmed) {
                violations.push((
                    file_path.display().to_string(),
                    line_num + 1,
                    trimmed.to_string(),
                ));
            }
        }
    }
}

fn is_problematic_allow_attribute(line: &str) -> bool {
    // Look for clippy allow attributes that violate the zero-tolerance policy
    if line.starts_with("#[allow(clippy::") || line.starts_with("#![allow(clippy::") {
        return true;
    }

    // Also check for allow attributes in comments (shouldn't exist but worth checking)
    if line.contains("allow(clippy::") && (line.contains("#[") || line.contains("#![")) {
        return true;
    }

    false
}
