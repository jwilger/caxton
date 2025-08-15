//! CI Pipeline configuration validation module
//!
//! This module provides type-safe CI pipeline configuration validation
//! following domain-driven design principles. It maintains backward compatibility
//! with existing string-based APIs while providing strong typing internally.

use nutype::nutype;

/// Operating system name for CI matrix testing
#[nutype(
    sanitize(trim),
    validate(len_char_min = 1, len_char_max = 50),
    derive(Clone, Debug, Eq, PartialEq, Display, TryFrom, Into)
)]
pub struct OperatingSystemName(String);

/// Rust toolchain identifier for CI testing
#[nutype(
    sanitize(trim),
    validate(len_char_min = 1, len_char_max = 20),
    derive(Clone, Debug, Eq, PartialEq, Display, TryFrom, Into)
)]
pub struct RustToolchainName(String);

/// Feature flag name for CI feature matrix testing
#[nutype(
    sanitize(trim),
    validate(len_char_min = 1, len_char_max = 100),
    derive(Clone, Debug, Eq, PartialEq, Display, TryFrom, Into)
)]
pub struct FeatureName(String);

/// Validation error for CI pipeline configuration
#[nutype(
    sanitize(trim),
    validate(len_char_min = 1, len_char_max = 500),
    derive(Clone, Debug, Eq, PartialEq, Display, TryFrom, Into)
)]
pub struct ValidationError(String);

/// Configuration for CI matrix testing
///
/// Uses String types for backward compatibility but validates through domain types
#[derive(Debug, Clone)]
pub struct MatrixConfiguration {
    /// List of operating systems for matrix testing
    pub operating_systems: Vec<String>,
    /// List of Rust toolchains to test
    pub rust_toolchains: Vec<String>,
    /// List of feature combinations to test
    pub features: Vec<String>,
    /// Whether security scanning is enabled
    pub security_scanning_enabled: bool,
    /// Whether performance benchmarks are enabled
    pub performance_benchmarks_enabled: bool,
    /// Whether documentation builds are enabled
    pub documentation_builds_enabled: bool,
}

impl MatrixConfiguration {
    /// Convert this configuration to validated domain types
    ///
    /// This method performs the conversion from string-based configuration
    /// to strongly-typed domain objects for validation
    fn to_validated(&self) -> Result<ValidatedMatrixConfiguration, Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Convert and validate operating systems
        let os_results: Vec<_> = self
            .operating_systems
            .iter()
            .map(|s| OperatingSystemName::try_new(s.clone()))
            .collect();

        let (valid_os, os_errors): (Vec<_>, Vec<_>) =
            os_results.into_iter().partition(Result::is_ok);

        let validated_os: Vec<_> = valid_os.into_iter().map(Result::unwrap).collect();

        if !os_errors.is_empty()
            && let Ok(error) =
                ValidationError::try_new("Invalid operating system names".to_string())
        {
            errors.push(error);
        }

        // Convert and validate toolchains
        let toolchain_results: Vec<_> = self
            .rust_toolchains
            .iter()
            .map(|s| RustToolchainName::try_new(s.clone()))
            .collect();

        let (valid_toolchains, toolchain_errors): (Vec<_>, Vec<_>) =
            toolchain_results.into_iter().partition(Result::is_ok);

        let validated_toolchains: Vec<_> =
            valid_toolchains.into_iter().map(Result::unwrap).collect();

        if !toolchain_errors.is_empty()
            && let Ok(error) = ValidationError::try_new("Invalid Rust toolchain names".to_string())
        {
            errors.push(error);
        }

        // Convert and validate features
        let feature_results: Vec<_> = self
            .features
            .iter()
            .map(|s| FeatureName::try_new(s.clone()))
            .collect();

        let (valid_features, feature_errors): (Vec<_>, Vec<_>) =
            feature_results.into_iter().partition(Result::is_ok);

        let validated_features: Vec<_> = valid_features.into_iter().map(Result::unwrap).collect();

        if !feature_errors.is_empty()
            && let Ok(error) = ValidationError::try_new("Invalid feature names".to_string())
        {
            errors.push(error);
        }

        if errors.is_empty() {
            Ok(ValidatedMatrixConfiguration {
                operating_systems: validated_os,
                rust_toolchains: validated_toolchains,
                features: validated_features,
                security_scanning_enabled: self.security_scanning_enabled,
                performance_benchmarks_enabled: self.performance_benchmarks_enabled,
                documentation_builds_enabled: self.documentation_builds_enabled,
            })
        } else {
            Err(errors)
        }
    }
}

/// Validated matrix configuration with domain types
///
/// This represents a configuration that has been validated and uses strong types
#[derive(Debug, Clone)]
pub struct ValidatedMatrixConfiguration {
    /// List of operating systems for matrix testing
    pub operating_systems: Vec<OperatingSystemName>,
    /// List of Rust toolchains to test
    pub rust_toolchains: Vec<RustToolchainName>,
    /// List of feature combinations to test
    pub features: Vec<FeatureName>,
    /// Whether security scanning is enabled
    pub security_scanning_enabled: bool,
    /// Whether performance benchmarks are enabled
    pub performance_benchmarks_enabled: bool,
    /// Whether documentation builds are enabled
    pub documentation_builds_enabled: bool,
}

/// Result of pipeline validation
#[derive(Debug, Clone)]
pub enum PipelineValidationResult {
    /// Configuration is valid
    Valid,
    /// Configuration is invalid with validation errors
    Invalid(Vec<String>),
}

/// Validator for CI pipeline configuration (Imperative Shell)
#[derive(Debug, Clone)]
pub struct CiPipelineValidator;

impl CiPipelineValidator {
    /// Create a new validator instance
    pub fn new() -> Self {
        Self
    }

    /// Validate matrix configuration using functional core validation
    pub fn validate_matrix_configuration(
        &self,
        config: &MatrixConfiguration,
    ) -> PipelineValidationResult {
        // Delegate to functional core for validation logic
        validate_matrix_configuration_pure(config)
    }
}

impl Default for CiPipelineValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Pure function for matrix configuration validation (Functional Core)
///
/// This function contains the core validation logic without any I/O or side effects.
/// It's easily testable and reason about.
fn validate_matrix_configuration_pure(config: &MatrixConfiguration) -> PipelineValidationResult {
    // First, try to validate the configuration by converting to domain types
    match config.to_validated() {
        Ok(validated_config) => {
            // Run business logic validation on the validated configuration
            let mut error_messages = Vec::new();

            // Validate operating systems business rules
            if let Err(os_errors) =
                validate_operating_systems_business_rules(&validated_config.operating_systems)
            {
                for error in os_errors {
                    error_messages.push(error.into_inner());
                }
            }

            // Validate toolchains business rules
            if let Err(toolchain_errors) =
                validate_rust_toolchains_business_rules(&validated_config.rust_toolchains)
            {
                for error in toolchain_errors {
                    error_messages.push(error.into_inner());
                }
            }

            // Validate features business rules
            if let Err(feature_errors) =
                validate_features_business_rules(&validated_config.features)
            {
                for error in feature_errors {
                    error_messages.push(error.into_inner());
                }
            }

            if error_messages.is_empty() {
                PipelineValidationResult::Valid
            } else {
                PipelineValidationResult::Invalid(error_messages)
            }
        }
        Err(validation_errors) => {
            let error_messages: Vec<String> = validation_errors
                .into_iter()
                .map(ValidationError::into_inner)
                .collect();
            PipelineValidationResult::Invalid(error_messages)
        }
    }
}

/// Validate operating system configuration business rules (Pure Function)
fn validate_operating_systems_business_rules(
    operating_systems: &[OperatingSystemName],
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    if operating_systems.is_empty()
        && let Ok(error) =
            ValidationError::try_new("At least one operating system must be specified".to_string())
    {
        errors.push(error);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate Rust toolchain configuration business rules (Pure Function)
fn validate_rust_toolchains_business_rules(
    toolchains: &[RustToolchainName],
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    if toolchains.is_empty()
        && let Ok(error) =
            ValidationError::try_new("At least one Rust toolchain must be specified".to_string())
    {
        errors.push(error);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate feature configuration business rules (Pure Function)
fn validate_features_business_rules(features: &[FeatureName]) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    if features.is_empty()
        && let Ok(error) =
            ValidationError::try_new("At least one feature must be specified".to_string())
    {
        errors.push(error);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
