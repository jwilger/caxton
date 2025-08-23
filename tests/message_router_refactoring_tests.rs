//! Architectural compliance tests for message router refactoring
//!
//! These tests verify that the message router follows architectural constraints
//! and will fail until proper refactoring is completed.

use std::fs;
use std::path::Path;

#[test]
fn test_should_enforce_file_size_limit_for_router_implementation() {
    // Test that verifies router.rs file size is under architectural limit
    let router_path = Path::new("src/message_router/router.rs");
    assert!(router_path.exists(), "Router file should exist");

    let content = fs::read_to_string(router_path).expect("Should be able to read router file");

    let line_count = content.lines().count();

    // Architectural constraint: router.rs should be under 500 lines after refactoring
    assert!(
        line_count < 500,
        "Router file has {line_count} lines but should be under 500 lines. Large files violate maintainability constraints."
    );
}

#[test]
fn test_should_have_zero_todo_comments_in_production_code() {
    // Test that verifies all TODO comments are resolved in router implementation
    let router_path = Path::new("src/message_router/router.rs");
    assert!(router_path.exists(), "Router file should exist");

    let content = fs::read_to_string(router_path).expect("Should be able to read router file");

    let todo_lines: Vec<_> = content
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("TODO"))
        .collect();

    assert!(
        todo_lines.is_empty(),
        "Found {} TODO comments in router.rs at lines: {:?}. All TODOs must be resolved for production readiness.",
        todo_lines.len(),
        todo_lines.iter().map(|(i, _)| i + 1).collect::<Vec<_>>()
    );
}

#[test]
fn test_should_have_extracted_embedded_trait_implementations() {
    // Test that verifies embedded trait implementations are extracted to separate modules
    let router_path = Path::new("src/message_router/router.rs");
    assert!(router_path.exists(), "Router file should exist");

    let content = fs::read_to_string(router_path).expect("Should be able to read router file");

    // Count impl blocks with async_trait - these should be extracted
    let async_trait_impl_count = content
        .lines()
        .filter(|line| line.trim().starts_with("#[async_trait]"))
        .count();

    // After refactoring, there should be no embedded trait implementations
    // All trait implementations should be in separate modules
    assert_eq!(
        async_trait_impl_count, 0,
        "Found {async_trait_impl_count} embedded async trait implementations. These should be extracted to separate modules for better organization."
    );
}

#[test]
fn test_should_have_proper_module_structure_for_implementations() {
    // Test that verifies separate implementation modules exist
    let expected_modules = [
        "src/message_router/implementations/delivery_engine.rs",
        "src/message_router/implementations/conversation_manager.rs",
        "src/message_router/implementations/agent_registry.rs",
        "src/message_router/implementations/failure_handler.rs",
        "src/message_router/implementations/metrics_collector.rs",
    ];

    for module_path in &expected_modules {
        let path = Path::new(module_path);
        assert!(
            path.exists(),
            "Module {module_path} should exist after refactoring. Embedded implementations should be extracted to separate modules."
        );
    }
}

#[test]
fn test_should_have_implementations_directory_with_mod_file() {
    // Test that verifies proper module structure with mod.rs
    let implementations_dir = Path::new("src/message_router/implementations");
    assert!(
        implementations_dir.exists() && implementations_dir.is_dir(),
        "Implementations directory should exist: {implementations_dir:?}"
    );

    let mod_file = implementations_dir.join("mod.rs");
    assert!(mod_file.exists(), "Module file should exist: {mod_file:?}");
}

#[test]
fn test_should_have_real_implementations_not_placeholders() {
    // Test that verifies placeholder implementations are properly handled
    let router_path = Path::new("src/message_router/router.rs");
    assert!(router_path.exists(), "Router file should exist");

    let content = fs::read_to_string(router_path).expect("Should be able to read router file");

    // Look for placeholder patterns that should be replaced
    let placeholder_patterns = [
        "// TODO:",
        "unimplemented!()",
        "todo!()",
        "0, // TODO:",
        "0.0, // TODO:",
        "HashMap::new(), // TODO:",
    ];

    let mut found_placeholders = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        for pattern in &placeholder_patterns {
            if line.contains(pattern) {
                found_placeholders.push((line_num + 1, pattern, line.trim()));
            }
        }
    }

    assert!(
        found_placeholders.is_empty(),
        "Found {} placeholder implementations that need proper implementation: {:?}",
        found_placeholders.len(),
        found_placeholders
    );
}

#[test]
fn test_should_have_separated_router_struct_from_implementations() {
    // Test that verifies router struct is cleanly separated from trait implementations
    let router_path = Path::new("src/message_router/router.rs");
    assert!(router_path.exists(), "Router file should exist");

    let content = fs::read_to_string(router_path).expect("Should be able to read router file");

    // After refactoring, the main router.rs should primarily contain:
    // 1. The MessageRouterImpl struct definition
    // 2. Basic impl block with new() and helper methods
    // 3. Module imports and re-exports

    // It should NOT contain large impl blocks for traits
    let impl_block_starts = content
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let trimmed = line.trim();
            trimmed.starts_with("impl ") && !trimmed.contains("MessageRouterImpl {")
        })
        .count();

    // After refactoring, there should be minimal impl blocks in router.rs
    // Most trait implementations should be in separate modules
    assert!(
        impl_block_starts <= 2,
        "Found {impl_block_starts} impl blocks in router.rs. Most trait implementations should be extracted to separate modules."
    );
}
