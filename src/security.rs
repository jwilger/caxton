//! Security policy configuration for WebAssembly agent execution

use crate::domain_types::{HostFunctionName, MaxImportFunctions};
use serde::{Deserialize, Serialize};

/// Feature enablement state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureState {
    /// Feature is enabled
    Enabled,
    /// Feature is disabled
    Disabled,
}

/// WebAssembly features that can be enabled or disabled
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmFeatures {
    /// Enable SIMD (Single Instruction, Multiple Data) instructions
    pub simd: FeatureState,
    /// Enable reference types (anyref, funcref)
    pub reference_types: FeatureState,
    /// Enable bulk memory operations
    pub bulk_memory: FeatureState,
    /// Enable threading support
    pub threads: FeatureState,
}

impl WasmFeatures {
    /// Strict security: all advanced features disabled
    pub fn strict() -> Self {
        Self {
            simd: FeatureState::Disabled,
            reference_types: FeatureState::Disabled,
            bulk_memory: FeatureState::Disabled,
            threads: FeatureState::Disabled,
        }
    }

    /// Relaxed security: all features enabled
    pub fn relaxed() -> Self {
        Self {
            simd: FeatureState::Enabled,
            reference_types: FeatureState::Enabled,
            bulk_memory: FeatureState::Enabled,
            threads: FeatureState::Enabled,
        }
    }

    /// Development: enable common features but keep threads disabled
    pub fn development() -> Self {
        Self {
            simd: FeatureState::Enabled,
            reference_types: FeatureState::Enabled,
            bulk_memory: FeatureState::Enabled,
            threads: FeatureState::Disabled,
        }
    }
}

impl Default for WasmFeatures {
    fn default() -> Self {
        Self::strict()
    }
}

/// System access permissions for agents
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessPermissions {
    /// Allow network access (HTTP, TCP, etc.)
    pub network: bool,
    /// Allow filesystem access (read/write files)
    pub filesystem: bool,
}

impl AccessPermissions {
    /// No external access permitted
    pub fn none() -> Self {
        Self {
            network: false,
            filesystem: false,
        }
    }

    /// Network access only
    pub fn network_only() -> Self {
        Self {
            network: true,
            filesystem: false,
        }
    }

    /// Full system access
    pub fn full() -> Self {
        Self {
            network: true,
            filesystem: true,
        }
    }
}

impl Default for AccessPermissions {
    fn default() -> Self {
        Self::none()
    }
}

/// Security policy defining WASM feature restrictions and permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// WebAssembly features configuration
    pub wasm_features: WasmFeatures,
    /// System access permissions
    pub access_permissions: AccessPermissions,
    /// Enable fuel-based metering
    pub enable_fuel_metering: bool,
    /// Maximum number of import functions allowed
    pub max_import_functions: MaxImportFunctions,
    /// List of allowed host functions
    pub allowed_host_functions: Vec<HostFunctionName>,
}

impl SecurityPolicy {
    /// Backward compatibility: check if SIMD is disabled
    pub fn disable_simd(&self) -> bool {
        self.wasm_features.simd == FeatureState::Disabled
    }

    /// Backward compatibility: check if reference types are disabled
    pub fn disable_reference_types(&self) -> bool {
        self.wasm_features.reference_types == FeatureState::Disabled
    }

    /// Backward compatibility: check if bulk memory is disabled
    pub fn disable_bulk_memory(&self) -> bool {
        self.wasm_features.bulk_memory == FeatureState::Disabled
    }

    /// Backward compatibility: check if threads are disabled
    pub fn disable_threads(&self) -> bool {
        self.wasm_features.threads == FeatureState::Disabled
    }

    /// Backward compatibility: check if network access is allowed
    pub fn allow_network_access(&self) -> bool {
        self.access_permissions.network
    }

    /// Backward compatibility: check if filesystem access is allowed
    pub fn allow_filesystem_access(&self) -> bool {
        self.access_permissions.filesystem
    }
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            wasm_features: WasmFeatures::default(),
            access_permissions: AccessPermissions::default(),
            enable_fuel_metering: true,
            max_import_functions: MaxImportFunctions::try_new(10).unwrap(),
            allowed_host_functions: vec![
                HostFunctionName::try_new("log".to_string()).unwrap(),
                HostFunctionName::try_new("get_time".to_string()).unwrap(),
                HostFunctionName::try_new("send_message".to_string()).unwrap(),
                HostFunctionName::try_new("receive_message".to_string()).unwrap(),
            ],
        }
    }
}

impl SecurityPolicy {
    /// Creates a strict security policy with minimal permissions
    ///
    /// # Panics
    ///
    /// Panics if the domain type validation fails (should not happen with hardcoded values)
    pub fn strict() -> Self {
        Self {
            wasm_features: WasmFeatures::strict(),
            access_permissions: AccessPermissions::none(),
            enable_fuel_metering: true,
            max_import_functions: MaxImportFunctions::try_new(5).unwrap(),
            allowed_host_functions: vec![
                HostFunctionName::try_new("log".to_string()).unwrap(),
                HostFunctionName::try_new("get_time".to_string()).unwrap(),
            ],
        }
    }

    /// Creates a relaxed security policy for trusted environments
    ///
    /// # Panics
    ///
    /// Panics if the domain type validation fails (should not happen with hardcoded values)
    pub fn relaxed() -> Self {
        Self {
            wasm_features: WasmFeatures::relaxed(),
            access_permissions: AccessPermissions::network_only(),
            enable_fuel_metering: true,
            max_import_functions: MaxImportFunctions::try_new(20).unwrap(),
            allowed_host_functions: vec![
                HostFunctionName::try_new("log".to_string()).unwrap(),
                HostFunctionName::try_new("get_time".to_string()).unwrap(),
                HostFunctionName::try_new("send_message".to_string()).unwrap(),
                HostFunctionName::try_new("receive_message".to_string()).unwrap(),
                HostFunctionName::try_new("http_request".to_string()).unwrap(),
                HostFunctionName::try_new("http_response".to_string()).unwrap(),
            ],
        }
    }

    /// Checks if a host function is allowed by this policy
    pub fn is_function_allowed(&self, function_name: &str) -> bool {
        let name = HostFunctionName::try_new(function_name.to_string());
        if let Ok(name) = name {
            self.allowed_host_functions.contains(&name)
        } else {
            false
        }
    }

    /// Validates the security policy for consistency
    ///
    /// # Errors
    ///
    /// Returns an error if the policy configuration is invalid
    pub fn validate(&self) -> Result<(), String> {
        if !self.enable_fuel_metering && self.wasm_features.threads == FeatureState::Enabled {
            return Err("Fuel metering must be enabled when threads are allowed".to_string());
        }

        if self.access_permissions.filesystem && self.allowed_host_functions.is_empty() {
            return Err(
                "Filesystem access requires at least one allowed host function".to_string(),
            );
        }

        if self.max_import_functions.into_inner() == 0 {
            return Err("At least one import function must be allowed".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_security_policy() {
        let policy = SecurityPolicy::default();
        assert!(policy.disable_simd());
        assert!(policy.disable_reference_types());
        assert!(policy.disable_bulk_memory());
        assert!(policy.disable_threads());
        assert!(policy.enable_fuel_metering);
        assert!(!policy.allow_network_access());
        assert!(!policy.allow_filesystem_access());
        assert_eq!(policy.max_import_functions.into_inner(), 10);
        assert_eq!(policy.allowed_host_functions.len(), 4);
    }

    #[test]
    fn test_strict_security_policy() {
        let policy = SecurityPolicy::strict();
        assert!(policy.disable_simd());
        assert!(policy.disable_reference_types());
        assert!(policy.disable_bulk_memory());
        assert!(policy.disable_threads());
        assert!(policy.enable_fuel_metering);
        assert!(!policy.allow_network_access());
        assert!(!policy.allow_filesystem_access());
        assert_eq!(policy.max_import_functions.into_inner(), 5);
        assert_eq!(policy.allowed_host_functions.len(), 2);
    }

    #[test]
    fn test_relaxed_security_policy() {
        let policy = SecurityPolicy::relaxed();
        assert!(!policy.disable_simd());
        assert!(!policy.disable_reference_types());
        assert!(!policy.disable_bulk_memory());
        assert!(!policy.disable_threads());
        assert!(policy.enable_fuel_metering);
        assert!(policy.allow_network_access());
        assert!(!policy.allow_filesystem_access());
        assert_eq!(policy.max_import_functions.into_inner(), 20);
        assert_eq!(policy.allowed_host_functions.len(), 6);
    }

    #[test]
    fn test_is_function_allowed() {
        let policy = SecurityPolicy::default();
        assert!(policy.is_function_allowed("log"));
        assert!(policy.is_function_allowed("get_time"));
        assert!(policy.is_function_allowed("send_message"));
        assert!(!policy.is_function_allowed("unknown_function"));
        assert!(!policy.is_function_allowed("file_read"));
    }

    #[test]
    fn test_validate_valid_policy() {
        let policy = SecurityPolicy::default();
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_policy_threads_without_fuel() {
        let policy = SecurityPolicy {
            enable_fuel_metering: false,
            wasm_features: WasmFeatures {
                threads: FeatureState::Enabled,
                ..WasmFeatures::default()
            },
            ..Default::default()
        };
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_policy_filesystem_no_functions() {
        let policy = SecurityPolicy {
            access_permissions: AccessPermissions {
                filesystem: true,
                ..AccessPermissions::default()
            },
            allowed_host_functions: vec![],
            ..Default::default()
        };
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_policy_zero_imports() {
        // This test is no longer valid since MaxImportFunctions has a minimum of 1
        // The type itself prevents this invalid state
        // Let's test with the minimum value instead
        let policy = SecurityPolicy {
            max_import_functions: MaxImportFunctions::try_new(1).unwrap(),
            ..Default::default()
        };
        assert!(policy.validate().is_ok());
    }
}
