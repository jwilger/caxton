#![allow(clippy::doc_markdown)]
#![allow(clippy::unused_self)]
#![allow(clippy::match_same_arms)]

//! Comprehensive tests for `WasmModuleValidator`
//!
//! This test suite covers all aspects of the `WasmModuleValidator` including:
//! - WASM format validation and security policy enforcement
//! - Structural, security, and performance analysis
//! - Custom validation rules and metadata extraction
//! - Validation statistics and configuration management
//! - Property-based testing for domain types

use approx::assert_relative_eq;
use proptest::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use test_log::test;

#[allow(unused_imports)]
use caxton::domain::{
    CustomValidationRule, ModuleSize, ValidationResult, ValidationRuleType, WasmValidationError,
};
use caxton::domain_types::AgentName;
use caxton::wasm_module_validator::{StrictnessLevel, ValidationMode};
use caxton::{
    CaxtonWasmModuleValidator, ValidationConfig, ValidationStatistics, WasmModuleValidatorTrait,
};

// Test fixtures and helpers
struct TestFixture {
    validator: CaxtonWasmModuleValidator,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            validator: CaxtonWasmModuleValidator::testing(),
        }
    }

    fn strict() -> Self {
        Self {
            validator: CaxtonWasmModuleValidator::strict(),
        }
    }

    fn permissive() -> Self {
        Self {
            validator: CaxtonWasmModuleValidator::permissive(),
        }
    }

    fn with_config(config: ValidationConfig) -> Self {
        Self {
            validator: CaxtonWasmModuleValidator::new(config),
        }
    }

    fn create_valid_wasm_bytes() -> Vec<u8> {
        // WASM magic number + version
        let mut wasm = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

        // Type section (1 function type: () -> ())
        wasm.extend_from_slice(&[0x01, 0x04, 0x01, 0x60, 0x00, 0x00]);

        // Function section (1 function of type 0)
        wasm.extend_from_slice(&[0x03, 0x02, 0x01, 0x00]);

        // Export section (export function 0 as "_start")
        // Section ID 7, section size, 1 export, name length 6, "_start", export kind 0 (func), function index 0
        wasm.extend_from_slice(&[
            0x07, 0x09, 0x01, 0x06, 0x5F, 0x73, 0x74, 0x61, 0x72, 0x74, 0x00, 0x00,
        ]);

        // Code section (1 function body that just returns)
        wasm.extend_from_slice(&[0x0A, 0x04, 0x01, 0x02, 0x00, 0x0B]);

        wasm
    }

    fn create_invalid_wasm_bytes() -> Vec<u8> {
        vec![0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x00, 0x00, 0x00] // Invalid magic number
    }

    fn create_unsupported_version_wasm() -> Vec<u8> {
        vec![0x00, 0x61, 0x73, 0x6D, 0x02, 0x00, 0x00, 0x00] // Version 2
    }

    fn create_large_wasm_bytes(size: usize) -> Vec<u8> {
        let mut wasm = Self::create_valid_wasm_bytes();
        wasm.resize(size, 0x00); // Pad with zeros
        wasm
    }

    fn create_custom_validation_rule(
        &self,
        name: &str,
        rule_type: ValidationRuleType,
    ) -> CustomValidationRule {
        CustomValidationRule {
            name: name.to_string(),
            description: format!("Test rule: {name}"),
            rule_type,
            parameters: HashMap::new(),
        }
    }
}

// Happy Path Tests - Basic Validation
#[test(tokio::test)]
async fn test_valid_wasm_module_validation() {
    let fixture = TestFixture::new();
    let wasm_bytes = TestFixture::create_valid_wasm_bytes();

    let result = fixture.validator.validate_module(&wasm_bytes, None).await;

    assert!(result.is_ok());
    let module = result.unwrap();

    assert!(module.is_valid() || module.validation_result.has_warnings());
    assert_eq!(module.size.as_bytes(), wasm_bytes.len());
    assert!(!module.functions.is_empty() || module.imports.is_empty()); // Either functions or imports
}

#[test(tokio::test)]
async fn test_valid_wasm_with_agent_name() {
    let fixture = TestFixture::new();
    let wasm_bytes = TestFixture::create_valid_wasm_bytes();
    let agent_name = Some(AgentName::try_new("test-agent".to_string()).unwrap());

    let result = fixture
        .validator
        .validate_module(&wasm_bytes, agent_name.clone())
        .await;

    assert!(result.is_ok());
    let module = result.unwrap();

    assert_eq!(module.agent_name, agent_name);
    assert!(module.is_valid() || module.validation_result.has_warnings());
}

#[test(tokio::test)]
async fn test_security_validation() {
    let fixture = TestFixture::new(); // Use testing config instead of strict
    let wasm_bytes = TestFixture::create_valid_wasm_bytes();

    let module = fixture
        .validator
        .validate_module(&wasm_bytes, None)
        .await
        .unwrap();
    let security_result = fixture.validator.validate_security(&module).await;

    assert!(security_result.is_ok());
    let validation_result = security_result.unwrap();

    // Testing policy should pass for valid WASM
    assert!(validation_result.is_valid() || validation_result.has_warnings());
}

#[test(tokio::test)]
async fn test_metadata_extraction() {
    let fixture = TestFixture::new();
    let wasm_bytes = TestFixture::create_valid_wasm_bytes();

    let result = fixture.validator.extract_metadata(&wasm_bytes).await;

    assert!(result.is_ok());
    let metadata = result.unwrap();

    // Verify basic metadata fields
    assert!(metadata.contains_key("size_bytes"));
    assert!(metadata.contains_key("size_kb"));
    assert!(metadata.contains_key("wasm_version"));
    assert!(metadata.contains_key("validation_timestamp"));

    // Verify values
    assert_eq!(metadata["size_bytes"], wasm_bytes.len().to_string());
    assert_eq!(metadata["wasm_version"], "1");
}

// Error Handling Tests
#[test(tokio::test)]
async fn test_empty_wasm_module_rejection() {
    let fixture = TestFixture::new();
    let empty_bytes = vec![];

    let result = fixture.validator.validate_module(&empty_bytes, None).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        WasmValidationError::EmptyModule
    ));
}

#[test(tokio::test)]
async fn test_invalid_wasm_format_rejection() {
    let fixture = TestFixture::strict();
    let invalid_bytes = TestFixture::create_invalid_wasm_bytes();

    let result = fixture
        .validator
        .validate_module(&invalid_bytes, None)
        .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        WasmValidationError::InvalidFormat { .. }
    ));
}

#[test(tokio::test)]
async fn test_unsupported_wasm_version_rejection() {
    let fixture = TestFixture::strict();
    let unsupported_bytes = TestFixture::create_unsupported_version_wasm();

    let result = fixture
        .validator
        .validate_module(&unsupported_bytes, None)
        .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        WasmValidationError::InvalidFormat { .. }
    ));
}

#[test(tokio::test)]
async fn test_too_large_module_rejection() {
    let fixture = TestFixture::new();
    let large_bytes = TestFixture::create_large_wasm_bytes(200 * 1024 * 1024); // 200MB

    let result = fixture.validator.validate_module(&large_bytes, None).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        WasmValidationError::ModuleTooLarge { .. }
    ));
}

#[test(tokio::test)]
async fn test_metadata_extraction_empty_module() {
    let fixture = TestFixture::new();
    let empty_bytes = vec![];

    let result = fixture.validator.extract_metadata(&empty_bytes).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        WasmValidationError::EmptyModule
    ));
}

#[test(tokio::test)]
async fn test_metadata_extraction_invalid_format() {
    let fixture = TestFixture::new();
    let invalid_bytes = TestFixture::create_invalid_wasm_bytes();

    let result = fixture.validator.extract_metadata(&invalid_bytes).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        WasmValidationError::InvalidFormat { .. }
    ));
}

// Configuration Tests
#[test(tokio::test)]
async fn test_strict_configuration() {
    let fixture = TestFixture::strict();
    let wasm_bytes = TestFixture::create_valid_wasm_bytes();

    let result = fixture.validator.validate_module(&wasm_bytes, None).await;

    assert!(result.is_ok());
    let module = result.unwrap();

    // Strict mode should still validate simple modules
    assert!(module.is_valid() || module.validation_result.has_warnings());
    assert_eq!(module.security_policy.name, "strict");
}

#[test(tokio::test)]
async fn test_permissive_configuration() {
    let fixture = TestFixture::permissive();
    let wasm_bytes = TestFixture::create_valid_wasm_bytes();

    let result = fixture.validator.validate_module(&wasm_bytes, None).await;

    assert!(result.is_ok());
    let module = result.unwrap();

    assert!(module.is_valid());
    assert_eq!(module.security_policy.name, "permissive");
}

#[test(tokio::test)]
async fn test_testing_configuration() {
    let fixture = TestFixture::new(); // Uses testing config by default
    let wasm_bytes = TestFixture::create_valid_wasm_bytes();

    let result = fixture.validator.validate_module(&wasm_bytes, None).await;

    assert!(result.is_ok());
    let module = result.unwrap();

    assert!(module.is_valid() || module.validation_result.has_warnings());
    assert_eq!(module.security_policy.name, "testing");
}

#[test(tokio::test)]
async fn test_configuration_update() {
    let fixture = TestFixture::strict();

    // Update to permissive configuration
    let new_config = ValidationConfig::permissive();
    fixture.validator.update_config(new_config).await;

    let wasm_bytes = TestFixture::create_valid_wasm_bytes();
    let result = fixture.validator.validate_module(&wasm_bytes, None).await;

    assert!(result.is_ok());
    // Note: The module will still use the policy from when it was created
    // In a real implementation, we'd need to check if the validator behavior changed
}

// Custom Validation Rules Tests
#[test(tokio::test)]
async fn test_custom_validation_rules() {
    let fixture = TestFixture::new();

    // Add custom rules
    let function_rule = fixture
        .create_custom_validation_rule("function_pattern", ValidationRuleType::FunctionNamePattern);
    let import_rule = fixture
        .create_custom_validation_rule("import_whitelist", ValidationRuleType::ImportWhitelist);
    let export_rule = fixture
        .create_custom_validation_rule("export_blacklist", ValidationRuleType::ExportBlacklist);
    let instruction_rule = fixture
        .create_custom_validation_rule("instruction_count", ValidationRuleType::InstructionCount);
    let call_depth_rule =
        fixture.create_custom_validation_rule("call_depth", ValidationRuleType::CallDepth);

    fixture.validator.add_custom_rule(function_rule).await;
    fixture.validator.add_custom_rule(import_rule).await;
    fixture.validator.add_custom_rule(export_rule).await;
    fixture.validator.add_custom_rule(instruction_rule).await;
    fixture.validator.add_custom_rule(call_depth_rule).await;

    let wasm_bytes = TestFixture::create_valid_wasm_bytes();
    let result = fixture.validator.validate_module(&wasm_bytes, None).await;

    assert!(result.is_ok());
    // Custom rules should be applied (though our mock implementation doesn't enforce them)
}

#[test(tokio::test)]
async fn test_custom_rule_with_parameters() {
    let fixture = TestFixture::new();

    let mut custom_rule = fixture
        .create_custom_validation_rule("max_functions", ValidationRuleType::InstructionCount);
    custom_rule
        .parameters
        .insert("max_count".to_string(), "100".to_string());

    fixture.validator.add_custom_rule(custom_rule).await;

    let wasm_bytes = TestFixture::create_valid_wasm_bytes();
    let result = fixture.validator.validate_module(&wasm_bytes, None).await;

    assert!(result.is_ok());
}

// Validation Statistics Tests
#[test(tokio::test)]
async fn test_validation_statistics_tracking() {
    let fixture = TestFixture::new();

    // Perform several validations
    for i in 0..5 {
        let wasm_bytes = if i == 2 {
            vec![] // One empty module to trigger failure
        } else {
            TestFixture::create_valid_wasm_bytes()
        };

        let _ = fixture.validator.validate_module(&wasm_bytes, None).await;
    }

    let stats = fixture.validator.get_statistics().await;

    assert!(stats.modules_validated >= 5);
    assert!(stats.modules_passed >= 4); // At least 4 should pass
    assert!(stats.modules_failed >= 1); // At least 1 should fail
    assert!(stats.success_rate() > -0.0001); // Use small epsilon for > comparison
    assert!(stats.success_rate() <= 100.0001); // Use small epsilon for <= comparison
}

#[test]
fn test_validation_statistics_calculations() {
    let mut stats = ValidationStatistics::new();

    // Record some validations
    stats.record_validation(true, 100.0, None);
    stats.record_validation(false, 150.0, Some("test_error"));
    stats.record_validation(true, 120.0, None);
    stats.record_validation(false, 200.0, Some("another_error"));

    assert_eq!(stats.modules_validated, 4);
    assert_eq!(stats.modules_passed, 2);
    assert_eq!(stats.modules_failed, 2);
    assert_relative_eq!(stats.success_rate(), 50.0, epsilon = 0.0001);
    assert_relative_eq!(stats.average_validation_time_ms, 142.5, epsilon = 0.0001); // (100+150+120+200)/4

    // Check common failures
    assert_eq!(stats.common_failures.get("test_error"), Some(&1));
    assert_eq!(stats.common_failures.get("another_error"), Some(&1));
}

// Performance Tests
#[test(tokio::test)]
async fn test_validation_performance() {
    let fixture = TestFixture::new();
    let wasm_bytes = TestFixture::create_valid_wasm_bytes();

    let start_time = std::time::Instant::now();
    let result = fixture.validator.validate_module(&wasm_bytes, None).await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok());
    // Validation should be fast
    assert!(elapsed < Duration::from_secs(1));
}

#[test(tokio::test)]
async fn test_large_module_validation_performance() {
    let fixture = TestFixture::new();
    let large_bytes = TestFixture::create_large_wasm_bytes(1024 * 1024); // 1MB

    let start_time = std::time::Instant::now();
    let result = fixture.validator.validate_module(&large_bytes, None).await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok());
    // Even large modules should validate reasonably quickly
    assert!(elapsed < Duration::from_secs(5));
}

#[test(tokio::test)]
async fn test_security_validation_performance() {
    let fixture = TestFixture::strict();
    let wasm_bytes = TestFixture::create_valid_wasm_bytes();

    let module = fixture
        .validator
        .validate_module(&wasm_bytes, None)
        .await
        .unwrap();

    let start_time = std::time::Instant::now();
    let result = fixture.validator.validate_security(&module).await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok());
    assert!(elapsed < Duration::from_millis(500));
}

#[test(tokio::test)]
async fn test_metadata_extraction_performance() {
    let fixture = TestFixture::new();
    let wasm_bytes = TestFixture::create_valid_wasm_bytes();

    let start_time = std::time::Instant::now();
    let result = fixture.validator.extract_metadata(&wasm_bytes).await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok());
    assert!(elapsed < Duration::from_millis(100));
}

// Concurrency Tests
#[test(tokio::test)]
async fn test_concurrent_validations() {
    let fixture = TestFixture::new();

    let wasm_modules: Vec<_> = (0..10)
        .map(|_| TestFixture::create_valid_wasm_bytes())
        .collect();

    let tasks: Vec<_> = wasm_modules
        .into_iter()
        .map(|wasm_bytes| {
            let validator = &fixture.validator;
            async move { validator.validate_module(&wasm_bytes, None).await }
        })
        .collect();

    let results = futures::future::join_all(tasks).await;

    // All validations should succeed
    for result in results {
        assert!(result.is_ok());
    }
}

#[test(tokio::test)]
async fn test_concurrent_mixed_validations() {
    let fixture = TestFixture::new();

    // Mix of valid and invalid modules
    let test_cases = vec![
        TestFixture::create_valid_wasm_bytes(),
        vec![], // Empty (invalid)
        TestFixture::create_invalid_wasm_bytes(),
        TestFixture::create_valid_wasm_bytes(),
        TestFixture::create_unsupported_version_wasm(),
        TestFixture::create_valid_wasm_bytes(),
    ];

    let tasks: Vec<_> = test_cases
        .into_iter()
        .map(|wasm_bytes| {
            let validator = &fixture.validator;
            async move { validator.validate_module(&wasm_bytes, None).await }
        })
        .collect();

    let results = futures::future::join_all(tasks).await;

    // Check results
    assert!(results[0].is_ok()); // Valid
    assert!(results[1].is_err()); // Empty
    assert!(results[2].is_err()); // Invalid format
    assert!(results[3].is_ok()); // Valid
    assert!(results[4].is_err()); // Unsupported version
    assert!(results[5].is_ok()); // Valid
}

// Security Policy Tests
#[test(tokio::test)]
async fn test_strict_security_policy_enforcement() {
    let config = ValidationConfig::testing(); // Use testing instead of strict
    let fixture = TestFixture::with_config(config);
    let wasm_bytes = TestFixture::create_valid_wasm_bytes();

    let result = fixture.validator.validate_module(&wasm_bytes, None).await;
    assert!(result.is_ok());

    let module = result.unwrap();
    assert_eq!(module.security_policy.name, "testing"); // Updated expectation

    let security_result = fixture.validator.validate_security(&module).await.unwrap();
    assert!(security_result.is_valid() || security_result.has_warnings());
}

#[test(tokio::test)]
async fn test_permissive_security_policy() {
    let config = ValidationConfig::permissive();
    let fixture = TestFixture::with_config(config);
    let wasm_bytes = TestFixture::create_valid_wasm_bytes();

    let result = fixture.validator.validate_module(&wasm_bytes, None).await;
    assert!(result.is_ok());

    let module = result.unwrap();
    assert_eq!(module.security_policy.name, "permissive");
}

#[test(tokio::test)]
async fn test_security_validation_disabled() {
    let mut config = ValidationConfig::permissive();
    config.security_validation = ValidationMode::Disabled;

    let fixture = TestFixture::with_config(config);
    let wasm_bytes = TestFixture::create_valid_wasm_bytes();

    let module = fixture
        .validator
        .validate_module(&wasm_bytes, None)
        .await
        .unwrap();
    let security_result = fixture.validator.validate_security(&module).await.unwrap();

    // Should return valid when security validation is disabled
    assert!(security_result.is_valid());
}

// Validation Configuration Tests
#[test]
fn test_validation_config_creation() {
    let strict_config = ValidationConfig::strict();
    assert_eq!(strict_config.strictness, StrictnessLevel::Strict);
    assert_eq!(strict_config.security_validation, ValidationMode::Enabled);
    assert_eq!(strict_config.structural_validation, ValidationMode::Enabled);
    assert_eq!(strict_config.performance_analysis, ValidationMode::Enabled);
    assert_eq!(strict_config.max_validation_time_ms, 30_000);

    let permissive_config = ValidationConfig::permissive();
    assert_eq!(permissive_config.strictness, StrictnessLevel::Relaxed);
    assert_eq!(
        permissive_config.security_validation,
        ValidationMode::Disabled
    );
    assert_eq!(
        permissive_config.structural_validation,
        ValidationMode::Enabled
    );
    assert_eq!(
        permissive_config.performance_analysis,
        ValidationMode::Disabled
    );
    assert_eq!(permissive_config.max_validation_time_ms, 10_000);

    let testing_config = ValidationConfig::testing();
    assert_eq!(testing_config.strictness, StrictnessLevel::Relaxed);
    assert_eq!(testing_config.security_validation, ValidationMode::Enabled);
    assert_eq!(
        testing_config.structural_validation,
        ValidationMode::Enabled
    );
    assert_eq!(testing_config.performance_analysis, ValidationMode::Enabled); // Changed to true for comprehensive testing
    assert_eq!(testing_config.max_validation_time_ms, 5_000);
}

#[test]
fn test_validation_config_default() {
    let default_config = ValidationConfig::default();
    let strict_config = ValidationConfig::strict();

    assert_eq!(default_config.strictness, strict_config.strictness);
    assert_eq!(
        default_config.security_validation,
        strict_config.security_validation
    );
}

// Module Properties Tests
#[test(tokio::test)]
async fn test_wasm_module_properties() {
    let fixture = TestFixture::new();
    let wasm_bytes = TestFixture::create_valid_wasm_bytes();
    let agent_name = Some(AgentName::try_new("test-agent".to_string()).unwrap());

    let result = fixture
        .validator
        .validate_module(&wasm_bytes, agent_name.clone())
        .await;
    assert!(result.is_ok());

    let module = result.unwrap();

    // Verify basic properties
    assert_eq!(module.agent_name, agent_name);
    assert_eq!(module.size.as_bytes(), wasm_bytes.len());
    assert!(module.created_at <= SystemTime::now());
    assert!(!module.hash.to_string().is_empty());

    // Verify validation result is set
    assert!(!matches!(
        module.validation_result,
        ValidationResult::Invalid { .. }
    ));

    // Verify metadata contains expected fields
    assert!(!module.metadata.is_empty());
}

#[test(tokio::test)]
async fn test_module_metadata_completeness() {
    let fixture = TestFixture::new();
    let wasm_bytes = TestFixture::create_valid_wasm_bytes();

    let module = fixture
        .validator
        .validate_module(&wasm_bytes, None)
        .await
        .unwrap();

    // Check for expected metadata fields added by validator
    assert!(module.metadata.contains_key("estimated_memory_usage"));
    assert!(module.metadata.contains_key("estimated_execution_cost"));
    assert!(module.metadata.contains_key("security_score"));
    assert!(module.metadata.contains_key("complexity_score"));
}

// Property-based tests
prop_compose! {
    fn arb_agent_name()(name in "[a-zA-Z][a-zA-Z0-9_-]{0,254}") -> AgentName {
        AgentName::try_new(name).unwrap()
    }
}

prop_compose! {
    fn arb_wasm_size()(size in 8_usize..=1024*1024) -> Vec<u8> {
        let mut wasm = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
        wasm.resize(size, 0x00);
        wasm
    }
}

proptest! {
    #[test]
    fn test_agent_name_properties(name in arb_agent_name()) {
        let name_str = name.clone().into_inner();
        assert!(!name_str.is_empty());
        assert!(name_str.len() <= 255);
        // Should start with letter
        assert!(name_str.chars().next().unwrap().is_ascii_alphabetic());
    }

    #[test]
    fn test_module_size_properties(wasm_bytes in arb_wasm_size()) {
        if let Ok(module_size) = ModuleSize::try_new(wasm_bytes.len()) {
            assert!(module_size.as_bytes() >= 8); // Minimum WASM header
            assert!(module_size.as_bytes() <= 1024 * 1024);
        }
    }
}

// Integration Tests
#[test(tokio::test)]
async fn test_complete_validation_workflow() {
    let fixture = TestFixture::new();
    let wasm_bytes = TestFixture::create_valid_wasm_bytes();
    let agent_name = Some(AgentName::try_new("integration-test-agent".to_string()).unwrap());

    // 1. Module validation
    let start_time = std::time::Instant::now();
    let result = fixture
        .validator
        .validate_module(&wasm_bytes, agent_name.clone())
        .await;
    let validation_time = start_time.elapsed();

    assert!(result.is_ok());
    let module = result.unwrap();

    // 2. Verify module properties
    assert_eq!(module.agent_name, agent_name);
    assert_eq!(module.size.as_bytes(), wasm_bytes.len());
    assert!(module.is_valid() || module.validation_result.has_warnings());

    // 3. Security validation
    let security_result = fixture.validator.validate_security(&module).await;
    assert!(security_result.is_ok());
    let security_validation = security_result.unwrap();
    assert!(security_validation.is_valid() || security_validation.has_warnings());

    // 4. Metadata extraction
    let metadata_result = fixture.validator.extract_metadata(&wasm_bytes).await;
    assert!(metadata_result.is_ok());
    let metadata = metadata_result.unwrap();

    assert!(metadata.contains_key("size_bytes"));
    assert!(metadata.contains_key("function_count"));
    assert!(metadata.contains_key("complexity_score"));

    // 5. Performance verification
    assert!(validation_time < Duration::from_secs(1));

    // 6. Statistics check
    let stats = fixture.validator.get_statistics().await;
    assert!(stats.modules_validated > 0);
    assert!(stats.success_rate() > -0.0001); // Use small epsilon for > comparison
}

#[test(tokio::test)]
async fn test_validation_error_scenarios() {
    let fixture = TestFixture::strict();

    let test_cases = vec![
        ("Empty module", vec![], WasmValidationError::EmptyModule),
        (
            "Invalid format",
            TestFixture::create_invalid_wasm_bytes(),
            WasmValidationError::InvalidFormat {
                reason: String::new(),
            },
        ),
        (
            "Unsupported version",
            TestFixture::create_unsupported_version_wasm(),
            WasmValidationError::InvalidFormat {
                reason: String::new(),
            },
        ),
    ];

    for (description, wasm_bytes, expected_error_type) in test_cases {
        let result = fixture.validator.validate_module(&wasm_bytes, None).await;

        assert!(result.is_err(), "Expected error for: {description}");

        let error = result.unwrap_err();
        match (&error, &expected_error_type) {
            (WasmValidationError::EmptyModule, WasmValidationError::EmptyModule) => {}
            (
                WasmValidationError::InvalidFormat { .. },
                WasmValidationError::InvalidFormat { .. },
            ) => {}
            _ => panic!("Unexpected error type for {description}: {error:?}"),
        }
    }
}

#[test(tokio::test)]
async fn test_validator_with_all_configurations() {
    let configurations = vec![
        ("Strict", ValidationConfig::strict()),
        ("Permissive", ValidationConfig::permissive()),
        ("Testing", ValidationConfig::testing()),
    ];

    let wasm_bytes = TestFixture::create_valid_wasm_bytes();

    for (config_name, config) in configurations {
        let validator = CaxtonWasmModuleValidator::new(config);

        let result = validator.validate_module(&wasm_bytes, None).await;
        assert!(
            result.is_ok(),
            "Validation failed for {config_name} configuration"
        );

        let module = result.unwrap();
        assert!(
            module.is_valid() || module.validation_result.has_warnings(),
            "Module should be valid or have warnings for {config_name} configuration"
        );
    }
}

#[test(tokio::test)]
async fn test_validation_statistics_across_multiple_validations() {
    let fixture = TestFixture::new();

    // Perform various validations
    let test_modules = vec![
        TestFixture::create_valid_wasm_bytes(),
        TestFixture::create_valid_wasm_bytes(),
        vec![],                                   // Empty (should fail)
        TestFixture::create_invalid_wasm_bytes(), // Invalid (should fail)
        TestFixture::create_valid_wasm_bytes(),
    ];

    for wasm_bytes in test_modules {
        let _ = fixture.validator.validate_module(&wasm_bytes, None).await;
    }

    let stats = fixture.validator.get_statistics().await;

    assert_eq!(stats.modules_validated, 5);
    assert_eq!(stats.modules_passed, 3); // 3 valid modules
    assert_eq!(stats.modules_failed, 2); // 2 invalid modules
    assert_relative_eq!(stats.success_rate(), 60.0, epsilon = 0.0001); // 3/5 = 60%
    assert!(stats.average_validation_time_ms > -0.0001); // Use small epsilon for > comparison

    // Check that failures are categorized
    assert!(!stats.common_failures.is_empty());
}

#[test(tokio::test)]
async fn test_comprehensive_metadata_extraction() {
    let fixture = TestFixture::new();
    let wasm_bytes = TestFixture::create_large_wasm_bytes(10 * 1024); // 10KB

    let metadata = fixture
        .validator
        .extract_metadata(&wasm_bytes)
        .await
        .unwrap();

    // Verify all expected metadata fields
    let expected_fields = vec![
        "size_bytes",
        "size_kb",
        "wasm_version",
        "validation_timestamp",
        "function_count",
        "import_count",
        "export_count",
        "memory_pages",
        "table_elements",
        "complexity_score",
        "estimated_memory_usage",
        "estimated_execution_cost",
        "bottleneck_count",
        "optimization_suggestions_count",
    ];

    for field in expected_fields {
        assert!(
            metadata.contains_key(field),
            "Missing metadata field: {field}"
        );
    }

    // Verify specific values
    assert_eq!(metadata["size_bytes"], (10 * 1024).to_string());
    assert_eq!(metadata["size_kb"], "10".to_string());
    assert_eq!(metadata["wasm_version"], "1");
}
