//! Security documentation integration tests
//!
//! This module tests that security documentation properly includes and references
//! the new security tools (cargo-deny, security.txt) for comprehensive user guidance.

use std::fs;
use std::path::Path;

/// Test that verifies security documentation includes the new tools
///
/// This test ensures that SECURITY.md properly documents cargo-deny and security.txt
/// integration, providing users with comprehensive security guidance.
#[test]
fn test_security_documentation_includes_new_tools() {
    // Test that SECURITY.md references cargo-deny and its usage
    let security_md_content = fs::read_to_string("SECURITY.md").expect("SECURITY.md should exist");

    // Verify cargo-deny is mentioned in the security tools section
    assert!(
        security_md_content.contains("cargo-deny"),
        "SECURITY.md should reference cargo-deny as a security tool"
    );

    // Verify usage example for cargo-deny is included
    assert!(
        security_md_content.contains("cargo deny check"),
        "SECURITY.md should include cargo-deny usage example"
    );

    // Check that security.txt is mentioned in vulnerability reporting section
    assert!(
        security_md_content.contains("security.txt"),
        "SECURITY.md should reference security.txt for vulnerability reporting"
    );

    // Validate that documentation includes proper examples and best practices
    assert!(
        security_md_content.contains("# Run security checks"),
        "SECURITY.md should include security check examples section"
    );

    // Ensure cross-references between related security features exist
    assert!(
        security_md_content.contains("cargo audit") && security_md_content.contains("cargo deny"),
        "SECURITY.md should cross-reference related security tools"
    );

    // Verify security.txt file exists and is properly formatted
    let security_txt_path = Path::new(".well-known/security.txt");
    assert!(
        security_txt_path.exists(),
        "security.txt file should exist at .well-known/security.txt"
    );

    let security_txt_content =
        fs::read_to_string(security_txt_path).expect("security.txt should be readable");

    // Verify RFC 9116 compliance
    assert!(
        security_txt_content.contains("Contact: security@caxton.dev"),
        "security.txt should contain proper contact information"
    );

    // Verify deny.toml exists and is documented
    let deny_toml_path = Path::new("deny.toml");
    assert!(
        deny_toml_path.exists(),
        "deny.toml configuration file should exist"
    );
}
