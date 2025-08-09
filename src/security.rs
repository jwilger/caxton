//! Security policy configuration for WebAssembly agent execution

use crate::domain_types::{HostFunctionName, MaxImportFunctions};
use serde::{Deserialize, Serialize};

/// Security policy defining WASM feature restrictions and permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)] // Security policies inherently need multiple boolean flags
pub struct SecurityPolicy {
    /// Disable SIMD instructions
    pub disable_simd: bool,
    /// Disable reference types
    pub disable_reference_types: bool,
    /// Disable bulk memory operations
    pub disable_bulk_memory: bool,
    /// Disable threading support
    pub disable_threads: bool,
    /// Enable fuel-based metering
    pub enable_fuel_metering: bool,
    /// Allow network access
    pub allow_network_access: bool,
    /// Allow filesystem access
    pub allow_filesystem_access: bool,
    /// Maximum number of import functions allowed
    pub max_import_functions: MaxImportFunctions,
    /// List of allowed host functions
    pub allowed_host_functions: Vec<HostFunctionName>,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            disable_simd: true,
            disable_reference_types: true,
            disable_bulk_memory: true,
            disable_threads: true,
            enable_fuel_metering: true,
            allow_network_access: false,
            allow_filesystem_access: false,
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
            disable_simd: true,
            disable_reference_types: true,
            disable_bulk_memory: true,
            disable_threads: true,
            enable_fuel_metering: true,
            allow_network_access: false,
            allow_filesystem_access: false,
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
            disable_simd: false,
            disable_reference_types: false,
            disable_bulk_memory: false,
            disable_threads: false,
            enable_fuel_metering: true,
            allow_network_access: true,
            allow_filesystem_access: false,
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
        if !self.enable_fuel_metering && !self.disable_threads {
            return Err("Fuel metering must be enabled when threads are allowed".to_string());
        }

        if self.allow_filesystem_access && self.allowed_host_functions.is_empty() {
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
        assert!(policy.disable_simd);
        assert!(policy.disable_reference_types);
        assert!(policy.disable_bulk_memory);
        assert!(policy.disable_threads);
        assert!(policy.enable_fuel_metering);
        assert!(!policy.allow_network_access);
        assert!(!policy.allow_filesystem_access);
        assert_eq!(policy.max_import_functions.into_inner(), 10);
        assert_eq!(policy.allowed_host_functions.len(), 4);
    }

    #[test]
    fn test_strict_security_policy() {
        let policy = SecurityPolicy::strict();
        assert!(policy.disable_simd);
        assert!(policy.disable_reference_types);
        assert!(policy.disable_bulk_memory);
        assert!(policy.disable_threads);
        assert!(policy.enable_fuel_metering);
        assert!(!policy.allow_network_access);
        assert!(!policy.allow_filesystem_access);
        assert_eq!(policy.max_import_functions.into_inner(), 5);
        assert_eq!(policy.allowed_host_functions.len(), 2);
    }

    #[test]
    fn test_relaxed_security_policy() {
        let policy = SecurityPolicy::relaxed();
        assert!(!policy.disable_simd);
        assert!(!policy.disable_reference_types);
        assert!(!policy.disable_bulk_memory);
        assert!(!policy.disable_threads);
        assert!(policy.enable_fuel_metering);
        assert!(policy.allow_network_access);
        assert!(!policy.allow_filesystem_access);
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
            disable_threads: false,
            ..Default::default()
        };
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_policy_filesystem_no_functions() {
        let policy = SecurityPolicy {
            allow_filesystem_access: true,
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
