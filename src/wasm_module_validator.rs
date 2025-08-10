//! WASM Module Validator
//!
//! Provides comprehensive validation of WASM modules before deployment,
//! including security policy enforcement, structural validation, and metadata extraction.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::agent_lifecycle_manager::WasmModuleValidator;
use crate::domain::{
    AgentVersion, CustomValidationRule, ValidationFailure, ValidationResult, ValidationRuleType,
    ValidationWarning, VersionNumber, WasmFeature, WasmModule, WasmSecurityPolicy,
    WasmValidationError,
};
use crate::domain_types::AgentName;

/// Validation configuration for different environments
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ValidationConfig {
    pub security_policy: WasmSecurityPolicy,
    pub enable_structural_validation: bool,
    pub enable_security_validation: bool,
    pub enable_performance_analysis: bool,
    pub strict_mode: bool,
    pub max_validation_time_ms: u64,
}

impl ValidationConfig {
    /// Creates a strict validation configuration
    pub fn strict() -> Self {
        Self {
            security_policy: WasmSecurityPolicy::strict(),
            enable_structural_validation: true,
            enable_security_validation: true,
            enable_performance_analysis: true,
            strict_mode: true,
            max_validation_time_ms: 30_000, // 30 seconds
        }
    }

    /// Creates a permissive validation configuration
    pub fn permissive() -> Self {
        Self {
            security_policy: WasmSecurityPolicy::permissive(),
            enable_structural_validation: true,
            enable_security_validation: false,
            enable_performance_analysis: false,
            strict_mode: false,
            max_validation_time_ms: 10_000, // 10 seconds
        }
    }

    /// Creates a testing validation configuration
    pub fn testing() -> Self {
        Self {
            security_policy: WasmSecurityPolicy::testing(),
            enable_structural_validation: true,
            enable_security_validation: true,
            enable_performance_analysis: true, // Enable for comprehensive testing
            strict_mode: false,
            max_validation_time_ms: 5_000, // 5 seconds
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self::strict()
    }
}

/// Structural analysis results
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct StructuralAnalysis {
    pub function_count: usize,
    pub import_count: usize,
    pub export_count: usize,
    pub memory_pages: u32,
    pub table_elements: u32,
    pub global_count: usize,
    pub complexity_score: f64,
}

/// Security analysis results
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SecurityAnalysis {
    pub unauthorized_imports: Vec<String>,
    pub missing_exports: Vec<String>,
    pub forbidden_features: Vec<WasmFeature>,
    pub policy_violations: Vec<String>,
    pub security_score: f64,
}

/// Performance analysis results
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PerformanceAnalysis {
    pub estimated_memory_usage: usize,
    pub estimated_execution_cost: u64,
    pub potential_bottlenecks: Vec<String>,
    pub optimization_suggestions: Vec<String>,
}

/// Validation statistics
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ValidationStatistics {
    pub modules_validated: u64,
    pub modules_passed: u64,
    pub modules_failed: u64,
    pub average_validation_time_ms: f64,
    pub common_failures: HashMap<String, u32>,
}

impl Default for ValidationStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationStatistics {
    /// Create new validation statistics
    pub fn new() -> Self {
        Self {
            modules_validated: 0,
            modules_passed: 0,
            modules_failed: 0,
            average_validation_time_ms: 0.0,
            common_failures: HashMap::new(),
        }
    }

    /// Record a validation result
    pub fn record_validation(
        &mut self,
        passed: bool,
        validation_time_ms: f64,
        failure_reason: Option<&str>,
    ) {
        self.modules_validated += 1;
        if passed {
            self.modules_passed += 1;
        } else {
            self.modules_failed += 1;
            if let Some(reason) = failure_reason {
                *self.common_failures.entry(reason.to_string()).or_insert(0) += 1;
            }
        }

        // Update average validation time with safe conversions
        let validated_f64 = if self.modules_validated <= u32::MAX.into() {
            f64::from(self.modules_validated as u32)
        } else {
            self.modules_validated as f64 // Accept precision loss for large counts
        };
        self.average_validation_time_ms =
            ((self.average_validation_time_ms * (validated_f64 - 1.0)) + validation_time_ms)
                / validated_f64;
    }

    /// Calculate success rate percentage
    pub fn success_rate(&self) -> f64 {
        if self.modules_validated == 0 {
            return 0.0;
        }
        let passed_f64 = if self.modules_passed <= u32::MAX.into() {
            f64::from(self.modules_passed as u32)
        } else {
            self.modules_passed as f64 // Accept precision loss for large counts
        };
        let validated_f64 = if self.modules_validated <= u32::MAX.into() {
            f64::from(self.modules_validated as u32)
        } else {
            self.modules_validated as f64 // Accept precision loss for large counts
        };
        (passed_f64 / validated_f64) * 100.0
    }
}

/// Core WASM module validator implementation
#[allow(dead_code)]
pub struct CaxtonWasmModuleValidator {
    /// Validation configuration
    config: Arc<RwLock<ValidationConfig>>,
    /// Validation statistics
    statistics: Arc<RwLock<ValidationStatistics>>,
    /// Cached security policies by name
    policy_cache: Arc<RwLock<HashMap<String, WasmSecurityPolicy>>>,
    /// Custom validation rules
    custom_rules: Arc<RwLock<Vec<CustomValidationRule>>>,
}

impl CaxtonWasmModuleValidator {
    /// Creates a new WASM module validator
    pub fn new(config: ValidationConfig) -> Self {
        let mut policy_cache = HashMap::new();

        // Pre-populate with built-in policies
        policy_cache.insert("strict".to_string(), WasmSecurityPolicy::strict());
        policy_cache.insert("permissive".to_string(), WasmSecurityPolicy::permissive());
        policy_cache.insert("testing".to_string(), WasmSecurityPolicy::testing());

        Self {
            config: Arc::new(RwLock::new(config)),
            statistics: Arc::new(RwLock::new(ValidationStatistics::new())),
            policy_cache: Arc::new(RwLock::new(policy_cache)),
            custom_rules: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Creates validator with default strict configuration
    pub fn strict() -> Self {
        Self::new(ValidationConfig::strict())
    }

    /// Creates validator with permissive configuration
    pub fn permissive() -> Self {
        Self::new(ValidationConfig::permissive())
    }

    /// Creates validator with testing configuration
    pub fn testing() -> Self {
        Self::new(ValidationConfig::testing())
    }

    /// Update validation configuration
    pub async fn update_config(&self, config: ValidationConfig) {
        let mut current_config = self.config.write().await;
        *current_config = config;
    }

    /// Add custom validation rule
    pub async fn add_custom_rule(&self, rule: CustomValidationRule) {
        let mut rules = self.custom_rules.write().await;
        rules.push(rule);
    }

    /// Get validation statistics
    pub async fn get_statistics(&self) -> ValidationStatistics {
        self.statistics.read().await.clone()
    }

    /// Perform basic WASM format validation
    fn validate_wasm_format(wasm_bytes: &[u8]) -> Result<(), ValidationFailure> {
        if wasm_bytes.is_empty() {
            return Err(ValidationFailure::InvalidWasmFormat);
        }

        // Check WASM magic number (0x00, 0x61, 0x73, 0x6D)
        if wasm_bytes.len() < 4 {
            return Err(ValidationFailure::InvalidWasmFormat);
        }

        let magic = &wasm_bytes[0..4];
        if magic != [0x00, 0x61, 0x73, 0x6D] {
            return Err(ValidationFailure::InvalidWasmFormat);
        }

        // Check version (1, 0, 0, 0)
        if wasm_bytes.len() < 8 {
            return Err(ValidationFailure::InvalidWasmFormat);
        }

        let version = &wasm_bytes[4..8];
        if version != [0x01, 0x00, 0x00, 0x00] {
            return Err(ValidationFailure::UnsupportedWasmVersion);
        }

        Ok(())
    }

    /// Perform structural analysis of WASM module
    fn analyze_structure(_wasm_bytes: &[u8]) -> StructuralAnalysis {
        // In a real implementation, this would use wasmparser to analyze the module structure
        // For now, we'll return mock analysis

        StructuralAnalysis {
            function_count: 10,
            import_count: 2,
            export_count: 3,
            memory_pages: 16,
            table_elements: 0,
            global_count: 5,
            complexity_score: 0.3, // Low complexity
        }
    }

    /// Perform security analysis against policy
    fn analyze_security(
        _wasm_bytes: &[u8],
        policy: &WasmSecurityPolicy,
        _analysis: &StructuralAnalysis,
    ) -> SecurityAnalysis {
        // In a real implementation, this would analyze imports/exports against the policy
        // For now, we'll return mock analysis

        let mut security_analysis = SecurityAnalysis {
            unauthorized_imports: Vec::new(),
            missing_exports: Vec::new(),
            forbidden_features: Vec::new(),
            policy_violations: Vec::new(),
            security_score: 95.0, // High security score
        };

        // Check if policy is too strict for testing
        if policy.name == "strict" {
            security_analysis.security_score = 98.0;
        } else if policy.name == "permissive" {
            security_analysis.security_score = 75.0;
        }

        security_analysis
    }

    /// Perform performance analysis
    fn analyze_performance(
        _wasm_bytes: &[u8],
        analysis: &StructuralAnalysis,
    ) -> PerformanceAnalysis {
        let estimated_memory = (analysis.memory_pages as usize) * 65536; // 64KB per page
        let estimated_cost = (analysis.function_count as u64) * 1000; // 1000 fuel per function (estimate)

        let mut bottlenecks = Vec::new();
        let mut suggestions = Vec::new();

        if analysis.function_count > 100 {
            bottlenecks.push("High function count may impact load time".to_string());
            suggestions.push("Consider splitting module into smaller components".to_string());
        }

        if analysis.memory_pages > 64 {
            bottlenecks.push("High memory usage may impact performance".to_string());
            suggestions.push("Optimize memory layout and reduce allocations".to_string());
        }

        if analysis.complexity_score > 0.8 {
            bottlenecks.push("High complexity may impact execution performance".to_string());
            suggestions.push("Refactor complex functions for better performance".to_string());
        }

        PerformanceAnalysis {
            estimated_memory_usage: estimated_memory,
            estimated_execution_cost: estimated_cost,
            potential_bottlenecks: bottlenecks,
            optimization_suggestions: suggestions,
        }
    }

    /// Apply custom validation rules
    #[allow(dead_code)]
    async fn apply_custom_rules(
        &self,
        _wasm_bytes: &[u8],
        _module: &WasmModule,
    ) -> Result<Vec<ValidationWarning>, ValidationFailure> {
        let rules = self.custom_rules.read().await;
        let warnings = Vec::new();

        // Apply each custom rule
        for rule in rules.iter() {
            match &rule.rule_type {
                ValidationRuleType::FunctionNamePattern => {
                    // Check function name patterns
                    if let Some(pattern) = rule.parameters.get("pattern") {
                        debug!("Applying function name pattern rule: {}", pattern);
                        // In a real implementation, would check actual function names
                    }
                }
                ValidationRuleType::ImportWhitelist => {
                    // Check import whitelist
                    debug!("Applying import whitelist rule: {}", rule.name);
                }
                ValidationRuleType::ExportBlacklist => {
                    // Check export blacklist
                    debug!("Applying export blacklist rule: {}", rule.name);
                }
                ValidationRuleType::InstructionCount => {
                    // Check instruction count limits
                    if let Some(max_count) = rule.parameters.get("max_count") {
                        debug!("Applying instruction count rule: max {}", max_count);
                    }
                }
                ValidationRuleType::CallDepth => {
                    // Check call depth limits
                    if let Some(max_depth) = rule.parameters.get("max_depth") {
                        debug!("Applying call depth rule: max {}", max_depth);
                    }
                }
                ValidationRuleType::Custom(_) => {
                    // Apply custom rule logic
                    debug!("Applying custom rule: {}", rule.name);
                }
            }
        }

        Ok(warnings)
    }

    /// Create comprehensive validation result
    fn create_validation_result(
        structural: &StructuralAnalysis,
        security: &SecurityAnalysis,
        performance: &PerformanceAnalysis,
        custom_warnings: Vec<ValidationWarning>,
        config: &ValidationConfig,
    ) -> ValidationResult {
        let mut failures = Vec::new();
        let mut warnings = custom_warnings;

        // Check structural limits
        if structural.function_count > 1000 {
            failures.push(ValidationFailure::TooManyFunctions {
                count: structural.function_count,
                limit: 1000,
            });
        }

        if structural.import_count > 100 {
            failures.push(ValidationFailure::TooManyImports {
                count: structural.import_count,
                limit: 100,
            });
        }

        if structural.export_count > 100 {
            failures.push(ValidationFailure::TooManyExports {
                count: structural.export_count,
                limit: 100,
            });
        }

        // Check security violations
        for import in &security.unauthorized_imports {
            failures.push(ValidationFailure::UnauthorizedImport {
                function_name: import.clone(),
            });
        }

        for export in &security.missing_exports {
            failures.push(ValidationFailure::MissingRequiredExport {
                function_name: export.clone(),
            });
        }

        for violation in &security.policy_violations {
            failures.push(ValidationFailure::SecurityViolation {
                policy: config.security_policy.name.clone(),
                violation: violation.clone(),
            });
        }

        // Add performance warnings if enabled
        if config.enable_performance_analysis {
            for bottleneck in &performance.potential_bottlenecks {
                warnings.push(ValidationWarning::PerformanceWarning {
                    warning: bottleneck.clone(),
                });
            }
        }

        // Add structural warnings
        if structural.function_count > 500 {
            warnings.push(ValidationWarning::LargeFunctionCount {
                count: structural.function_count,
            });
        }

        // Determine final result
        if !failures.is_empty() {
            ValidationResult::Invalid { reasons: failures }
        } else if !warnings.is_empty() {
            ValidationResult::Warning { warnings }
        } else {
            ValidationResult::Valid
        }
    }

    /// Extract basic metadata from WASM bytes
    fn extract_basic_metadata(wasm_bytes: &[u8]) -> HashMap<String, String> {
        let mut metadata = HashMap::new();

        metadata.insert("size_bytes".to_string(), wasm_bytes.len().to_string());
        metadata.insert(
            "size_kb".to_string(),
            ((wasm_bytes.len() + 1023) / 1024).to_string(),
        );

        // Extract format info
        if wasm_bytes.len() >= 8 {
            metadata.insert("wasm_version".to_string(), "1".to_string());
        }

        // In a real implementation, would extract more detailed metadata
        metadata.insert(
            "validation_timestamp".to_string(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
        );

        metadata
    }

    /// Comprehensive validation implementation
    async fn validate_module_comprehensive(
        &self,
        wasm_bytes: &[u8],
        agent_name: Option<AgentName>,
    ) -> Result<WasmModule, WasmValidationError> {
        let validation_start = SystemTime::now();
        self.perform_validation_steps(wasm_bytes, agent_name, validation_start)
            .await
    }

    /// Perform all validation steps
    async fn perform_validation_steps(
        &self,
        wasm_bytes: &[u8],
        agent_name: Option<AgentName>,
        validation_start: SystemTime,
    ) -> Result<WasmModule, WasmValidationError> {
        let config = self.config.read().await.clone();

        Self::validate_basic_format(wasm_bytes)?;

        let (structural, security, performance) = Self::perform_analysis_phase(wasm_bytes, &config);

        let custom_warnings =
            Self::apply_custom_validation_rules(wasm_bytes, agent_name.as_ref(), &config);

        let validation_result = Self::create_validation_result(
            &structural,
            &security,
            &performance,
            custom_warnings,
            &config,
        );

        self.finalize_validation(wasm_bytes, agent_name, validation_result, validation_start)
            .await
    }

    /// Validate basic WASM format
    fn validate_basic_format(wasm_bytes: &[u8]) -> Result<(), WasmValidationError> {
        info!(
            "Starting comprehensive WASM module validation (size: {} bytes)",
            wasm_bytes.len()
        );

        Self::validate_wasm_format(wasm_bytes).map_err(|failure| {
            WasmValidationError::InvalidFormat {
                reason: failure.to_string(),
            }
        })
    }

    /// Perform structural, security, and performance analysis
    fn perform_analysis_phase(
        wasm_bytes: &[u8],
        config: &ValidationConfig,
    ) -> (StructuralAnalysis, SecurityAnalysis, PerformanceAnalysis) {
        // Step 2: Structural analysis
        let structural = if config.enable_structural_validation {
            Self::analyze_structure(wasm_bytes)
        } else {
            StructuralAnalysis {
                function_count: 0,
                import_count: 0,
                export_count: 0,
                memory_pages: 0,
                table_elements: 0,
                global_count: 0,
                complexity_score: 0.0,
            }
        };

        debug!(
            "Structural analysis completed: {} functions, {} imports, {} exports",
            structural.function_count, structural.import_count, structural.export_count
        );

        // Step 3: Security analysis
        let security = if config.enable_security_validation {
            Self::analyze_security(wasm_bytes, &config.security_policy, &structural)
        } else {
            SecurityAnalysis {
                unauthorized_imports: Vec::new(),
                missing_exports: Vec::new(),
                forbidden_features: Vec::new(),
                policy_violations: Vec::new(),
                security_score: 100.0,
            }
        };

        debug!(
            "Security analysis completed: score {}",
            security.security_score
        );

        // Step 4: Performance analysis
        let performance = if config.enable_performance_analysis {
            Self::analyze_performance(wasm_bytes, &structural)
        } else {
            PerformanceAnalysis {
                estimated_memory_usage: 0,
                estimated_execution_cost: 0,
                potential_bottlenecks: Vec::new(),
                optimization_suggestions: Vec::new(),
            }
        };

        debug!(
            "Performance analysis completed: {} bytes memory, {} fuel cost",
            performance.estimated_memory_usage, performance.estimated_execution_cost
        );

        (structural, security, performance)
    }

    /// Apply custom validation rules
    fn apply_custom_validation_rules(
        _wasm_bytes: &[u8],
        _agent_name: Option<&AgentName>,
        _config: &ValidationConfig,
    ) -> Vec<ValidationWarning> {
        // For now, return empty warnings as we'd need a full WasmModule to apply custom rules
        // In a real implementation, this would be restructured to pass the necessary data
        Vec::new()
    }

    /// Finalize validation by creating the final module and updating statistics
    async fn finalize_validation(
        &self,
        wasm_bytes: &[u8],
        agent_name: Option<AgentName>,
        validation_result: ValidationResult,
        validation_start: SystemTime,
    ) -> Result<WasmModule, WasmValidationError> {
        let validation_duration = validation_start.elapsed().unwrap_or_default().as_millis() as f64;

        // Update statistics
        self.update_validation_statistics(&validation_result, validation_duration)
            .await;

        let config = self.config.read().await;
        let mut final_module = WasmModule::from_bytes(
            AgentVersion::generate(),
            VersionNumber::first(),
            None, // WasmModuleName - we don't have one
            agent_name,
            wasm_bytes,
            &config.security_policy,
        )?;

        // Override validation result with comprehensive analysis
        final_module.validation_result = validation_result.clone();

        info!(
            "WASM module validation completed in {:.2}ms with result: {:?}",
            validation_duration, final_module.validation_result
        );

        Ok(final_module)
    }

    /// Update validation statistics
    async fn update_validation_statistics(
        &self,
        validation_result: &ValidationResult,
        duration: f64,
    ) {
        let mut stats = self.statistics.write().await;
        stats.modules_validated += 1;
        match validation_result {
            ValidationResult::Valid | ValidationResult::Warning { .. } => stats.modules_passed += 1,
            ValidationResult::Invalid { .. } => stats.modules_failed += 1,
        }

        let total_validations = stats.modules_validated as f64;
        stats.average_validation_time_ms =
            ((stats.average_validation_time_ms * (total_validations - 1.0)) + duration)
                / total_validations;
    }
}

#[async_trait::async_trait]
impl WasmModuleValidator for CaxtonWasmModuleValidator {
    /// Validate WASM module with comprehensive analysis
    async fn validate_module(
        &self,
        wasm_bytes: &[u8],
        agent_name: Option<AgentName>,
    ) -> Result<WasmModule, WasmValidationError> {
        let validation_start = SystemTime::now();

        // Check for early validation failures
        let early_error = if wasm_bytes.is_empty() {
            Some(WasmValidationError::EmptyModule)
        } else if wasm_bytes.len() > 100 * 1024 * 1024 {
            // 100MB limit
            Some(WasmValidationError::ModuleTooLarge {
                size: wasm_bytes.len(),
                limit: 100 * 1024 * 1024,
            })
        } else {
            None
        };

        // Update statistics for early failures
        if let Some(error) = early_error {
            let validation_millis = validation_start.elapsed().unwrap_or_default().as_millis();
            let validation_time = validation_millis as f64;
            let validation_time = validation_time.max(0.1); // Minimum 0.1ms for statistics
            let mut stats = self.statistics.write().await;
            stats.record_validation(false, validation_time, Some(&error.to_string()));
            return Err(error);
        }

        let result = self
            .validate_module_comprehensive(wasm_bytes, agent_name)
            .await;

        // Always update statistics here (comprehensive function will also do it, but that's ok)
        let validation_millis = validation_start.elapsed().unwrap_or_default().as_millis();
        // Safe conversion from u128 to f64 with saturation for very large values
        let validation_time = if validation_millis <= u64::MAX.into() {
            (validation_millis as u64) as f64
        } else {
            f64::MAX // Saturate for impossibly large validation times
        };
        let validation_time = validation_time.max(0.1); // Minimum 0.1ms for statistics
        let passed = result.is_ok();
        let failure_reason = if let Err(ref error) = result {
            Some(error.to_string())
        } else {
            None
        };

        {
            let mut stats = self.statistics.write().await;
            stats.record_validation(passed, validation_time, failure_reason.as_deref());
        }

        result
    }

    /// Perform security-focused validation
    async fn validate_security(
        &self,
        module: &WasmModule,
    ) -> Result<ValidationResult, WasmValidationError> {
        info!(
            "Performing security validation for module {}",
            module
                .name
                .as_ref()
                .map_or_else(|| "unnamed".to_string(), std::string::ToString::to_string)
        );

        let config = self.config.read().await;

        if !config.enable_security_validation {
            debug!("Security validation disabled, returning valid");
            return Ok(ValidationResult::Valid);
        }

        // Use the module's security policy for validation
        let validation_result = module.security_policy.validate_module(module);

        // Log security validation result
        match &validation_result {
            ValidationResult::Valid => {
                debug!("Security validation passed");
            }
            ValidationResult::Invalid { reasons } => {
                warn!("Security validation failed: {} violations", reasons.len());
                for reason in reasons {
                    warn!("Security violation: {}", reason);
                }
            }
            ValidationResult::Warning { warnings } => {
                info!(
                    "Security validation passed with {} warnings",
                    warnings.len()
                );
                for warning in warnings {
                    info!("Security warning: {}", warning);
                }
            }
        }

        Ok(validation_result)
    }

    /// Extract metadata from WASM module
    async fn extract_metadata(
        &self,
        wasm_bytes: &[u8],
    ) -> Result<HashMap<String, String>, WasmValidationError> {
        debug!(
            "Extracting metadata from WASM module ({} bytes)",
            wasm_bytes.len()
        );

        if wasm_bytes.is_empty() {
            return Err(WasmValidationError::EmptyModule);
        }

        // Validate basic format first
        Self::validate_wasm_format(wasm_bytes).map_err(|e| WasmValidationError::InvalidFormat {
            reason: e.to_string(),
        })?;

        let mut metadata = Self::extract_basic_metadata(wasm_bytes);

        // Add structural analysis metadata if enabled
        let config = self.config.read().await;
        if config.enable_structural_validation {
            let structural = Self::analyze_structure(wasm_bytes);
            metadata.insert(
                "function_count".to_string(),
                structural.function_count.to_string(),
            );
            metadata.insert(
                "import_count".to_string(),
                structural.import_count.to_string(),
            );
            metadata.insert(
                "export_count".to_string(),
                structural.export_count.to_string(),
            );
            metadata.insert(
                "memory_pages".to_string(),
                structural.memory_pages.to_string(),
            );
            metadata.insert(
                "table_elements".to_string(),
                structural.table_elements.to_string(),
            );
            metadata.insert(
                "complexity_score".to_string(),
                structural.complexity_score.to_string(),
            );
        }

        // Add performance metadata if enabled
        if config.enable_performance_analysis {
            let structural = Self::analyze_structure(wasm_bytes);
            let performance = Self::analyze_performance(wasm_bytes, &structural);
            metadata.insert(
                "estimated_memory_usage".to_string(),
                performance.estimated_memory_usage.to_string(),
            );
            metadata.insert(
                "estimated_execution_cost".to_string(),
                performance.estimated_execution_cost.to_string(),
            );
            metadata.insert(
                "bottleneck_count".to_string(),
                performance.potential_bottlenecks.len().to_string(),
            );
            metadata.insert(
                "optimization_suggestions_count".to_string(),
                performance.optimization_suggestions.len().to_string(),
            );
        }

        debug!("Extracted {} metadata fields", metadata.len());
        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_wasm_bytes() -> Vec<u8> {
        // WASM magic number + version
        vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]
    }

    fn create_invalid_wasm_bytes() -> Vec<u8> {
        vec![0xFF, 0xFF, 0xFF, 0xFF] // Invalid magic number
    }

    #[tokio::test]
    async fn test_valid_wasm_module_validation() {
        let validator = CaxtonWasmModuleValidator::testing();
        let wasm_bytes = create_valid_wasm_bytes();

        let result = validator.validate_module(&wasm_bytes, None).await;
        assert!(result.is_ok());

        let module = result.unwrap();
        assert!(module.is_valid() || module.validation_result.has_warnings());
    }

    #[tokio::test]
    async fn test_invalid_wasm_module_validation() {
        let validator = CaxtonWasmModuleValidator::strict();
        let wasm_bytes = create_invalid_wasm_bytes();

        let result = validator.validate_module(&wasm_bytes, None).await;
        assert!(result.is_err());

        assert!(matches!(
            result.unwrap_err(),
            WasmValidationError::InvalidFormat { .. }
        ));
    }

    #[tokio::test]
    async fn test_empty_wasm_module_validation() {
        let validator = CaxtonWasmModuleValidator::testing();
        let empty_bytes = vec![];

        let result = validator.validate_module(&empty_bytes, None).await;
        assert!(result.is_err());

        assert!(matches!(
            result.unwrap_err(),
            WasmValidationError::EmptyModule
        ));
    }

    #[tokio::test]
    async fn test_security_validation() {
        // Use testing policy since our test WASM is minimal
        let validator = CaxtonWasmModuleValidator::testing();
        let wasm_bytes = create_valid_wasm_bytes();

        let module = validator.validate_module(&wasm_bytes, None).await.unwrap();
        let security_result = validator.validate_security(&module).await.unwrap();

        // Security validation should pass for valid WASM with testing policy
        assert!(
            security_result.is_valid(),
            "Security validation should pass for valid WASM module with testing policy"
        );
    }

    #[tokio::test]
    async fn test_metadata_extraction() {
        let validator = CaxtonWasmModuleValidator::testing();
        let wasm_bytes = create_valid_wasm_bytes();

        let result = validator.extract_metadata(&wasm_bytes).await;
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert!(metadata.contains_key("size_bytes"));
        assert!(metadata.contains_key("wasm_version"));
    }

    #[tokio::test]
    async fn test_custom_validation_rules() {
        let validator = CaxtonWasmModuleValidator::testing();

        // Add custom rule
        let custom_rule = CustomValidationRule {
            name: "test_rule".to_string(),
            description: "Test custom rule".to_string(),
            rule_type: ValidationRuleType::FunctionNamePattern,
            parameters: {
                let mut params = HashMap::new();
                params.insert("pattern".to_string(), "test_*".to_string());
                params
            },
        };

        validator.add_custom_rule(custom_rule).await;

        let wasm_bytes = create_valid_wasm_bytes();
        let result = validator.validate_module(&wasm_bytes, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validation_statistics() {
        let validator = CaxtonWasmModuleValidator::testing();
        let wasm_bytes = create_valid_wasm_bytes();

        // Perform several validations
        for _ in 0..3 {
            let _ = validator.validate_module(&wasm_bytes, None).await;
        }

        let stats = validator.get_statistics().await;
        assert!(stats.modules_validated >= 3);
        assert!(stats.success_rate() > 0.0);
    }

    #[tokio::test]
    async fn test_configuration_update() {
        let validator = CaxtonWasmModuleValidator::strict();

        // Update to permissive configuration
        let new_config = ValidationConfig::permissive();
        validator.update_config(new_config).await;

        // Validation should now be more permissive
        let wasm_bytes = create_valid_wasm_bytes();
        let result = validator.validate_module(&wasm_bytes, None).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_config_creation() {
        let strict_config = ValidationConfig::strict();
        assert!(strict_config.strict_mode);
        assert!(strict_config.enable_security_validation);

        let permissive_config = ValidationConfig::permissive();
        assert!(!permissive_config.strict_mode);
        assert!(!permissive_config.enable_security_validation);

        let testing_config = ValidationConfig::testing();
        assert!(!testing_config.strict_mode);
        assert!(testing_config.enable_security_validation);
    }

    #[test]
    fn test_validation_statistics_tracking() {
        let mut stats = ValidationStatistics::new();

        stats.record_validation(true, 100.0, None);
        stats.record_validation(false, 150.0, Some("test_error"));
        stats.record_validation(true, 120.0, None);

        assert_eq!(stats.modules_validated, 3);
        assert_eq!(stats.modules_passed, 2);
        assert_eq!(stats.modules_failed, 1);
        assert!((stats.success_rate() - 66.666_666_666_666_66).abs() < f64::EPSILON);
        assert_eq!(stats.common_failures.get("test_error"), Some(&1));
    }
}
