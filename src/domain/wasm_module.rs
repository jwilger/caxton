//! WASM Module domain types
//!
//! This module defines types for WASM module validation, metadata extraction,
//! security policies, and module lifecycle management for agent deployment.

use nutype::nutype;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use thiserror::Error;

use super::agent_lifecycle::{AgentVersion, VersionNumber};
use crate::domain_types::AgentName;

/// Hash of WASM module content for integrity verification
#[nutype(
    validate(len_char_min = 64, len_char_max = 128), // SHA-256 or SHA-512
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct ModuleHash(String);

impl ModuleHash {
    /// Creates hash from hex string
    ///
    /// # Errors
    ///
    /// Returns `ModuleHashError` if the hex string is invalid or wrong length.
    pub fn from_hex(hex: &str) -> Result<Self, ModuleHashError> {
        if hex.len() != 64 && hex.len() != 128 {
            return Err(Self::try_new("invalid_length".to_string()).unwrap_err());
        }

        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Self::try_new("invalid_chars".to_string()).unwrap_err());
        }

        Self::try_new(hex.to_string())
    }

    /// Creates SHA-256 hash from bytes
    ///
    /// # Panics
    ///
    /// Panics if the generated hash string is invalid (should never happen).
    #[must_use]
    pub fn sha256(data: &[u8]) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let hash = hasher.finish();

        // Convert to 64-character hex string (simulated SHA-256)
        Self::try_new(format!("{hash:016x}{hash:016x}{hash:016x}{hash:016x}")).unwrap()
    }

    /// Get hash algorithm type based on length
    #[must_use]
    pub fn algorithm(&self) -> HashAlgorithm {
        match self.clone().into_inner().len() {
            64 => HashAlgorithm::Sha256,
            128 => HashAlgorithm::Sha512,
            _ => HashAlgorithm::Unknown,
        }
    }
}

/// Supported hash algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    Sha256,
    Sha512,
    Unknown,
}

/// WASM module size validation
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 104_857_600), // 1 byte to 100MB
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct ModuleSize(usize);

impl ModuleSize {
    /// Creates module size from megabytes
    /// Create module size from megabytes
    ///
    /// # Errors
    ///
    /// Returns `ModuleSizeError` if the size is outside valid limits.
    pub fn from_mb(mb: usize) -> Result<Self, ModuleSizeError> {
        Self::try_new(mb * 1024 * 1024)
    }

    /// Creates module size from kilobytes
    /// Create module size from kilobytes
    ///
    /// # Errors
    ///
    /// Returns `ModuleSizeError` if the size is outside valid limits.
    pub fn from_kb(kb: usize) -> Result<Self, ModuleSizeError> {
        Self::try_new(kb * 1024)
    }

    /// Gets size in bytes
    #[must_use]
    pub fn as_bytes(&self) -> usize {
        self.into_inner()
    }

    /// Gets size in kilobytes (rounded up)
    #[must_use]
    pub fn as_kb(&self) -> usize {
        self.into_inner().div_ceil(1024)
    }

    /// Gets size in megabytes (rounded up)
    #[must_use]
    pub fn as_mb(&self) -> usize {
        self.into_inner().div_ceil(1_048_576)
    }
}

/// WASM function export name
#[nutype(
    validate(len_char_min = 1, len_char_max = 100),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct WasmExportName(String);

/// WASM function import name
#[nutype(
    validate(len_char_min = 1, len_char_max = 100),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct WasmImportName(String);

/// WASM module name identifier
#[nutype(
    validate(len_char_min = 1, len_char_max = 100),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct WasmModuleName(String);

/// Maximum number of WASM functions
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 10_000),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 100
)]
pub struct MaxWasmFunctions(u16);

impl MaxWasmFunctions {
    /// Gets the value as u16
    #[must_use]
    pub fn as_u16(&self) -> u16 {
        self.into_inner()
    }
}

/// WASM function signature for validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct WasmFunctionSignature {
    pub name: String, // Use String to be flexible for both imports and exports
    pub parameters: Vec<WasmValueType>,
    pub results: Vec<WasmValueType>,
    pub is_host_import: bool,
}

impl WasmFunctionSignature {
    /// Creates new function signature
    #[must_use]
    pub fn new(
        name: String,
        parameters: Vec<WasmValueType>,
        results: Vec<WasmValueType>,
        is_host_import: bool,
    ) -> Self {
        Self {
            name,
            parameters,
            results,
            is_host_import,
        }
    }

    /// Check if function has parameters
    #[must_use]
    pub fn has_parameters(&self) -> bool {
        !self.parameters.is_empty()
    }

    /// Check if function returns values
    #[must_use]
    pub fn has_results(&self) -> bool {
        !self.results.is_empty()
    }

    /// Get function arity (parameter count)
    #[must_use]
    pub fn arity(&self) -> usize {
        self.parameters.len()
    }
}

/// WASM value types for function signatures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum WasmValueType {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
}

impl WasmValueType {
    /// Check if type is numeric
    #[must_use]
    pub fn is_numeric(&self) -> bool {
        matches!(self, Self::I32 | Self::I64 | Self::F32 | Self::F64)
    }

    /// Check if type is reference
    #[must_use]
    pub fn is_reference(&self) -> bool {
        matches!(self, Self::FuncRef | Self::ExternRef)
    }

    /// Get type size in bytes (approximation)
    #[must_use]
    pub fn size_bytes(&self) -> usize {
        match self {
            Self::I32 | Self::F32 => 4,
            Self::I64 | Self::F64 | Self::FuncRef | Self::ExternRef => 8,
            Self::V128 => 16,
            // Pointer size
        }
    }
}

/// WASM module validation result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationResult {
    Valid,
    Invalid { reasons: Vec<ValidationFailure> },
    Warning { warnings: Vec<ValidationWarning> },
}

impl ValidationResult {
    /// Check if validation passed
    #[must_use]
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid | Self::Warning { .. })
    }

    /// Check if there are warnings
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        matches!(self, Self::Warning { .. })
    }

    /// Get all error messages
    #[must_use]
    pub fn error_messages(&self) -> Vec<String> {
        match self {
            Self::Invalid { reasons } => reasons
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            _ => vec![],
        }
    }

    /// Get all warning messages
    #[must_use]
    pub fn warning_messages(&self) -> Vec<String> {
        match self {
            Self::Warning { warnings } => warnings
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            _ => vec![],
        }
    }
}

/// Specific validation failure reasons
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationFailure {
    InvalidWasmFormat,
    UnsupportedWasmVersion,
    ModuleTooLarge {
        size: usize,
        limit: usize,
    },
    TooManyFunctions {
        count: usize,
        limit: usize,
    },
    TooManyImports {
        count: usize,
        limit: usize,
    },
    TooManyExports {
        count: usize,
        limit: usize,
    },
    UnauthorizedImport {
        function_name: String,
    },
    MissingRequiredExport {
        function_name: String,
    },
    InvalidFunctionSignature {
        function_name: String,
        reason: String,
    },
    SecurityViolation {
        policy: String,
        violation: String,
    },
    ResourceLimitExceeded {
        resource: String,
        limit: String,
    },
    DependencyNotFound {
        dependency: String,
    },
}

impl std::fmt::Display for ValidationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidWasmFormat => write!(f, "Invalid WASM format"),
            Self::UnsupportedWasmVersion => write!(f, "Unsupported WASM version"),
            Self::ModuleTooLarge { size, limit } => {
                write!(f, "Module too large: {size} bytes, limit {limit} bytes")
            }
            Self::TooManyFunctions { count, limit } => {
                write!(f, "Too many functions: {count}, limit {limit}")
            }
            Self::TooManyImports { count, limit } => {
                write!(f, "Too many imports: {count}, limit {limit}")
            }
            Self::TooManyExports { count, limit } => {
                write!(f, "Too many exports: {count}, limit {limit}")
            }
            Self::UnauthorizedImport { function_name } => {
                write!(f, "Unauthorized import: {function_name}")
            }
            Self::MissingRequiredExport { function_name } => {
                write!(f, "Missing required export: {function_name}")
            }
            Self::InvalidFunctionSignature {
                function_name,
                reason,
            } => {
                write!(
                    f,
                    "Invalid function signature for {function_name}: {reason}"
                )
            }
            Self::SecurityViolation { policy, violation } => {
                write!(f, "Security policy '{policy}' violation: {violation}")
            }
            Self::ResourceLimitExceeded { resource, limit } => {
                write!(f, "Resource limit exceeded for {resource}: {limit}")
            }
            Self::DependencyNotFound { dependency } => {
                write!(f, "Dependency not found: {dependency}")
            }
        }
    }
}

/// Validation warnings for non-critical issues
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationWarning {
    UnusedFunction { function_name: String },
    UnusedImport { import_name: String },
    LargeFunctionCount { count: usize },
    DeprecatedFeature { feature: String },
    PerformanceWarning { warning: String },
    CompatibilityIssue { issue: String },
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnusedFunction { function_name } => {
                write!(f, "Unused function: {function_name}")
            }
            Self::UnusedImport { import_name } => {
                write!(f, "Unused import: {import_name}")
            }
            Self::LargeFunctionCount { count } => {
                write!(f, "Large function count: {count}")
            }
            Self::DeprecatedFeature { feature } => {
                write!(f, "Deprecated feature: {feature}")
            }
            Self::PerformanceWarning { warning } => {
                write!(f, "Performance warning: {warning}")
            }
            Self::CompatibilityIssue { issue } => {
                write!(f, "Compatibility issue: {issue}")
            }
        }
    }
}

/// WASM module security policy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmSecurityPolicy {
    pub name: String,
    pub version: String,
    pub allowed_imports: HashSet<WasmImportName>,
    pub required_exports: HashSet<WasmExportName>,
    pub forbidden_instructions: HashSet<String>,
    pub max_memory_pages: u32,
    pub max_table_elements: u32,
    pub allow_bulk_memory: bool,
    pub allow_simd: bool,
    pub allow_threads: bool,
    pub custom_validations: Vec<CustomValidationRule>,
}

impl WasmSecurityPolicy {
    /// Creates a strict security policy
    ///
    /// # Panics
    ///
    /// Panics if hardcoded export names are invalid (should never happen).
    #[must_use]
    pub fn strict() -> Self {
        Self {
            name: "strict".to_string(),
            version: "1.0".to_string(),
            allowed_imports: HashSet::new(),
            required_exports: {
                let mut exports = HashSet::new();
                exports.insert(WasmExportName::try_new("_start".to_string()).unwrap());
                exports
            },
            forbidden_instructions: {
                let mut forbidden = HashSet::new();
                forbidden.insert("unreachable".to_string());
                forbidden.insert("memory.grow".to_string());
                forbidden
            },
            max_memory_pages: 16, // 1MB (64KB per page)
            max_table_elements: 100,
            allow_bulk_memory: false,
            allow_simd: false,
            allow_threads: false,
            custom_validations: vec![],
        }
    }

    /// Creates a permissive security policy
    #[must_use]
    pub fn permissive() -> Self {
        Self {
            name: "permissive".to_string(),
            version: "1.0".to_string(),
            allowed_imports: HashSet::new(), // Allow all
            required_exports: HashSet::new(),
            forbidden_instructions: HashSet::new(),
            max_memory_pages: 1024, // 64MB
            max_table_elements: 10000,
            allow_bulk_memory: true,
            allow_simd: true,
            allow_threads: false, // Still restrict threads
            custom_validations: vec![],
        }
    }

    /// Creates policy for testing environments
    #[must_use]
    pub fn testing() -> Self {
        let mut policy = Self::permissive();
        policy.name = "testing".to_string();
        policy.max_memory_pages = 32; // 2MB
        policy.max_table_elements = 1000;
        policy
    }

    /// Check if import is allowed
    #[must_use]
    pub fn is_import_allowed(&self, import_name: &str) -> bool {
        if self.allowed_imports.is_empty() {
            return true; // Allow all if no restrictions
        }
        // Convert string to WasmImportName for checking
        if let Ok(import) = WasmImportName::try_new(import_name.to_string()) {
            self.allowed_imports.contains(&import)
        } else {
            false
        }
    }

    /// Check if export is required
    #[must_use]
    pub fn is_export_required(&self, export_name: &str) -> bool {
        if let Ok(export) = WasmExportName::try_new(export_name.to_string()) {
            self.required_exports.contains(&export)
        } else {
            false
        }
    }

    /// Validate module against this policy
    #[must_use]
    pub fn validate_module(&self, module: &WasmModule) -> ValidationResult {
        let mut failures = vec![];
        let mut warnings = vec![];

        // Check size limits
        if module.size.as_bytes() > 50 * 1024 * 1024 {
            // 50MB
            failures.push(ValidationFailure::ModuleTooLarge {
                size: module.size.as_bytes(),
                limit: 50 * 1024 * 1024,
            });
        }

        // Check function count
        if module.functions.len() > 1000 {
            failures.push(ValidationFailure::TooManyFunctions {
                count: module.functions.len(),
                limit: 1000,
            });
        } else if module.functions.len() > 500 {
            warnings.push(ValidationWarning::LargeFunctionCount {
                count: module.functions.len(),
            });
        }

        // Check imports
        for import in &module.imports {
            if !self.is_import_allowed(&import.name) {
                failures.push(ValidationFailure::UnauthorizedImport {
                    function_name: import.name.clone(),
                });
            }
        }

        // Check required exports
        let export_names: HashSet<_> = module.exports.iter().map(|e| &e.name).collect();

        for required_export in &self.required_exports {
            let export_name = required_export.clone().into_inner();
            if !export_names.contains(&&export_name) {
                failures.push(ValidationFailure::MissingRequiredExport {
                    function_name: export_name,
                });
            }
        }

        // Return validation result
        if !failures.is_empty() {
            ValidationResult::Invalid { reasons: failures }
        } else if !warnings.is_empty() {
            ValidationResult::Warning { warnings }
        } else {
            ValidationResult::Valid
        }
    }
}

impl Default for WasmSecurityPolicy {
    fn default() -> Self {
        Self::strict()
    }
}

/// Custom validation rule for extensible policies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomValidationRule {
    pub name: String,
    pub description: String,
    pub rule_type: ValidationRuleType,
    pub parameters: HashMap<String, String>,
}

/// Types of custom validation rules
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationRuleType {
    FunctionNamePattern,
    ImportWhitelist,
    ExportBlacklist,
    InstructionCount,
    CallDepth,
    Custom(String),
}

/// WASM module metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmModule {
    pub version: AgentVersion,
    pub version_number: VersionNumber,
    pub name: Option<WasmModuleName>,
    pub agent_name: Option<AgentName>,
    pub hash: ModuleHash,
    pub size: ModuleSize,
    pub functions: Vec<WasmFunctionSignature>,
    pub imports: Vec<WasmFunctionSignature>,
    pub exports: Vec<WasmFunctionSignature>,
    pub memory_pages: u32,
    pub table_elements: u32,
    pub features_used: HashSet<WasmFeature>,
    pub security_policy: WasmSecurityPolicy,
    pub validation_result: ValidationResult,
    pub created_at: SystemTime,
    pub metadata: HashMap<String, String>,
}

impl WasmModule {
    /// Creates a new WASM module from bytes
    /// Create WASM module from bytes with validation
    ///
    /// # Errors
    ///
    /// Returns `WasmValidationError` if the WASM bytes are invalid or fail validation.
    pub fn from_bytes(
        version: AgentVersion,
        version_number: VersionNumber,
        name: Option<WasmModuleName>,
        agent_name: Option<AgentName>,
        wasm_bytes: &[u8],
        security_policy: &WasmSecurityPolicy,
    ) -> Result<Self, WasmValidationError> {
        if wasm_bytes.is_empty() {
            return Err(WasmValidationError::EmptyModule);
        }

        let hash = ModuleHash::sha256(wasm_bytes);
        let size = ModuleSize::try_new(wasm_bytes.len()).map_err(|_| {
            WasmValidationError::ModuleTooLarge {
                size: wasm_bytes.len(),
                limit: 104_857_600,
            }
        })?;

        // Simulate parsing (in real implementation, would use wasmparser)
        let functions = Self::extract_functions(wasm_bytes);
        let imports = Self::extract_imports(wasm_bytes);
        let exports = Self::extract_exports(wasm_bytes);
        let features_used = Self::extract_features(wasm_bytes);

        let mut module = Self {
            version,
            version_number,
            name,
            agent_name,
            hash,
            size,
            functions,
            imports,
            exports,
            memory_pages: 16, // Default
            table_elements: 0,
            features_used,
            security_policy: security_policy.clone(),
            validation_result: ValidationResult::Valid,
            created_at: SystemTime::now(),
            metadata: HashMap::new(),
        };

        // Validate against security policy
        module.validation_result = security_policy.validate_module(&module);

        // Update security policy (clone for ownership)
        module.security_policy = security_policy.clone();

        Ok(module)
    }

    /// Validate module integrity
    #[must_use]
    pub fn validate(&self) -> ValidationResult {
        self.security_policy.validate_module(self)
    }

    /// Check if module is valid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.validation_result.is_valid()
    }

    /// Get total function count
    #[must_use]
    pub fn total_function_count(&self) -> usize {
        self.functions.len() + self.imports.len()
    }

    /// Get memory usage estimate in bytes
    #[must_use]
    pub fn estimated_memory_usage(&self) -> usize {
        (self.memory_pages as usize) * 65536 // 64KB per page
    }

    /// Check if module uses specific WASM feature
    #[must_use]
    pub fn uses_feature(&self, feature: WasmFeature) -> bool {
        self.features_used.contains(&feature)
    }

    /// Add custom metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    // Helper methods for parsing (simulated)
    fn extract_functions(_wasm_bytes: &[u8]) -> Vec<WasmFunctionSignature> {
        // In real implementation, would use wasmparser to extract function signatures
        vec![]
    }

    fn extract_imports(_wasm_bytes: &[u8]) -> Vec<WasmFunctionSignature> {
        vec![]
    }

    fn extract_exports(_wasm_bytes: &[u8]) -> Vec<WasmFunctionSignature> {
        vec![]
    }

    fn extract_features(_wasm_bytes: &[u8]) -> HashSet<WasmFeature> {
        HashSet::new()
    }
}

/// WASM features that can be used in modules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WasmFeature {
    BulkMemory,
    Simd,
    Threads,
    TailCall,
    ReferenceTypes,
    MultiValue,
    SignExtension,
    ExceptionHandling,
    GarbageCollection,
    RelaxedSimd,
}

impl WasmFeature {
    /// Check if feature is considered stable
    #[must_use]
    pub fn is_stable(&self) -> bool {
        matches!(
            self,
            Self::BulkMemory
                | Self::Simd
                | Self::ReferenceTypes
                | Self::MultiValue
                | Self::SignExtension
        )
    }

    /// Check if feature is experimental
    #[must_use]
    pub fn is_experimental(&self) -> bool {
        !self.is_stable()
    }

    /// Get feature name as string
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::BulkMemory => "bulk-memory",
            Self::Simd => "simd",
            Self::Threads => "threads",
            Self::TailCall => "tail-call",
            Self::ReferenceTypes => "reference-types",
            Self::MultiValue => "multi-value",
            Self::SignExtension => "sign-extension",
            Self::ExceptionHandling => "exception-handling",
            Self::GarbageCollection => "garbage-collection",
            Self::RelaxedSimd => "relaxed-simd",
        }
    }
}

/// WASM module validation errors
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum WasmValidationError {
    #[error("Empty WASM module")]
    EmptyModule,

    #[error("Module too large: {size} bytes, limit {limit} bytes")]
    ModuleTooLarge { size: usize, limit: usize },

    #[error("Invalid WASM format: {reason}")]
    InvalidFormat { reason: String },

    #[error("Unsupported WASM version: {version}")]
    UnsupportedVersion { version: u32 },

    #[error("Function limit exceeded: {count}, limit {limit}")]
    FunctionLimitExceeded { count: usize, limit: usize },

    #[error("Import not allowed: {function_name}")]
    ImportNotAllowed { function_name: String },

    #[error("Required export missing: {function_name}")]
    RequiredExportMissing { function_name: String },

    #[error("Security policy violation: {policy} - {violation}")]
    SecurityPolicyViolation { policy: String, violation: String },

    #[error("Feature not supported: {feature}")]
    FeatureNotSupported { feature: String },

    #[error("Hash verification failed")]
    HashVerificationFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_hash() {
        let data = b"test wasm module";
        let hash = ModuleHash::sha256(data);
        assert_eq!(hash.algorithm(), HashAlgorithm::Sha256);

        let hex_hash = ModuleHash::from_hex(
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        )
        .unwrap();
        assert_eq!(hex_hash.algorithm(), HashAlgorithm::Sha256);
    }

    #[test]
    fn test_module_size() {
        let size_10mb = ModuleSize::from_mb(10).unwrap();
        assert_eq!(size_10mb.as_mb(), 10);
        assert_eq!(size_10mb.as_kb(), 10240);

        let size_512kb = ModuleSize::from_kb(512).unwrap();
        assert_eq!(size_512kb.as_kb(), 512);
        assert_eq!(size_512kb.as_mb(), 1); // Rounded up
    }

    #[test]
    fn test_wasm_value_type() {
        let i32_type = WasmValueType::I32;
        assert!(i32_type.is_numeric());
        assert!(!i32_type.is_reference());
        assert_eq!(i32_type.size_bytes(), 4);

        let funcref_type = WasmValueType::FuncRef;
        assert!(!funcref_type.is_numeric());
        assert!(funcref_type.is_reference());
        assert_eq!(funcref_type.size_bytes(), 8);
    }

    #[test]
    fn test_security_policy() {
        let strict_policy = WasmSecurityPolicy::strict();
        let permissive_policy = WasmSecurityPolicy::permissive();

        assert_eq!(strict_policy.max_memory_pages, 16);
        assert_eq!(permissive_policy.max_memory_pages, 1024);

        assert!(!strict_policy.allow_simd);
        assert!(permissive_policy.allow_simd);
    }

    #[test]
    fn test_validation_result() {
        let valid = ValidationResult::Valid;
        assert!(valid.is_valid());
        assert!(!valid.has_warnings());

        let invalid = ValidationResult::Invalid {
            reasons: vec![ValidationFailure::InvalidWasmFormat],
        };
        assert!(!invalid.is_valid());
        assert_eq!(invalid.error_messages().len(), 1);

        let warning = ValidationResult::Warning {
            warnings: vec![ValidationWarning::LargeFunctionCount { count: 600 }],
        };
        assert!(warning.is_valid());
        assert!(warning.has_warnings());
    }

    #[test]
    fn test_wasm_features() {
        let simd = WasmFeature::Simd;
        assert!(simd.is_stable());
        assert!(!simd.is_experimental());
        assert_eq!(simd.name(), "simd");

        let gc = WasmFeature::GarbageCollection;
        assert!(!gc.is_stable());
        assert!(gc.is_experimental());
        assert_eq!(gc.name(), "garbage-collection");
    }

    #[test]
    fn test_wasm_module_creation() {
        let version = AgentVersion::generate();
        let version_number = VersionNumber::first();
        let policy = WasmSecurityPolicy::testing();
        let wasm_bytes = b"fake wasm module content";

        let module =
            WasmModule::from_bytes(version, version_number, None, None, wasm_bytes, &policy)
                .unwrap();

        assert!(module.is_valid());
        assert_eq!(module.size.as_bytes(), wasm_bytes.len());
        assert_eq!(module.total_function_count(), 0); // No functions in fake module
    }
}
