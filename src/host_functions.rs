//! Host function management for WebAssembly agents

use crate::domain_types::{
    FunctionDescription, FunctionModuleName, HostFunctionName, PermissionName,
};
use std::collections::HashMap;

/// Registry for managing host functions exposed to WebAssembly agents
pub struct HostFunctions {
    registered_functions: HashMap<HostFunctionName, FunctionMetadata>,
}

/// Metadata for a host function
#[derive(Clone)]
pub struct FunctionMetadata {
    /// Function name
    pub name: HostFunctionName,
    /// Module name containing the function
    pub module: FunctionModuleName,
    /// Human-readable description
    pub description: FunctionDescription,
    /// Required permission to access this function
    pub required_permission: Option<PermissionName>,
}

impl Default for HostFunctions {
    fn default() -> Self {
        Self::new()
    }
}

impl HostFunctions {
    /// Creates a new host function registry with default functions
    ///
    /// # Panics
    /// Panics if any of the hardcoded function names are invalid for domain types
    pub fn new() -> Self {
        let mut functions = HashMap::new();

        let log_name = HostFunctionName::try_new("log".to_string()).unwrap();
        functions.insert(
            log_name.clone(),
            FunctionMetadata {
                name: log_name,
                module: FunctionModuleName::try_new("env".to_string()).unwrap(),
                description: FunctionDescription::try_new(
                    "Log a message from the agent".to_string(),
                )
                .unwrap(),
                required_permission: None,
            },
        );

        let get_time_name = HostFunctionName::try_new("get_time".to_string()).unwrap();
        functions.insert(
            get_time_name.clone(),
            FunctionMetadata {
                name: get_time_name,
                module: FunctionModuleName::try_new("env".to_string()).unwrap(),
                description: FunctionDescription::try_new("Get current Unix timestamp".to_string())
                    .unwrap(),
                required_permission: None,
            },
        );

        let send_message_name = HostFunctionName::try_new("send_message".to_string()).unwrap();
        functions.insert(
            send_message_name.clone(),
            FunctionMetadata {
                name: send_message_name,
                module: FunctionModuleName::try_new("env".to_string()).unwrap(),
                description: FunctionDescription::try_new(
                    "Send a message to another agent".to_string(),
                )
                .unwrap(),
                required_permission: Some(
                    PermissionName::try_new("messaging".to_string()).unwrap(),
                ),
            },
        );

        let receive_message_name =
            HostFunctionName::try_new("receive_message".to_string()).unwrap();
        functions.insert(
            receive_message_name.clone(),
            FunctionMetadata {
                name: receive_message_name,
                module: FunctionModuleName::try_new("env".to_string()).unwrap(),
                description: FunctionDescription::try_new(
                    "Receive messages from other agents".to_string(),
                )
                .unwrap(),
                required_permission: Some(
                    PermissionName::try_new("messaging".to_string()).unwrap(),
                ),
            },
        );

        Self {
            registered_functions: functions,
        }
    }

    /// Returns a list of all available host function names
    pub fn get_available_functions(&self) -> Vec<HostFunctionName> {
        self.registered_functions.keys().cloned().collect()
    }

    /// Checks if a function name is considered safe for execution
    pub fn is_function_safe(&self, function_name: &str) -> bool {
        !matches!(
            function_name,
            "file_system_access" | "network_raw_socket" | "process_spawn" | "memory_direct_access"
        )
    }

    /// Retrieves metadata for a specific function
    pub fn get_function_metadata(&self, name: &HostFunctionName) -> Option<&FunctionMetadata> {
        self.registered_functions.get(name)
    }

    /// Retrieves metadata for a specific function by string name
    pub fn get_function_metadata_by_name(&self, name: &str) -> Option<&FunctionMetadata> {
        let function_name = HostFunctionName::try_new(name.to_string()).ok()?;
        self.registered_functions.get(&function_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_functions_new() {
        let registry = HostFunctions::new();
        let functions = registry.get_available_functions();

        let log_name = HostFunctionName::try_new("log".to_string()).unwrap();
        let get_time_name = HostFunctionName::try_new("get_time".to_string()).unwrap();
        let send_message_name = HostFunctionName::try_new("send_message".to_string()).unwrap();
        let receive_message_name =
            HostFunctionName::try_new("receive_message".to_string()).unwrap();

        assert!(functions.contains(&log_name));
        assert!(functions.contains(&get_time_name));
        assert!(functions.contains(&send_message_name));
        assert!(functions.contains(&receive_message_name));
    }

    #[test]
    fn test_is_function_safe() {
        let registry = HostFunctions::new();

        assert!(registry.is_function_safe("log"));
        assert!(registry.is_function_safe("get_time"));
        assert!(registry.is_function_safe("send_message"));
        assert!(registry.is_function_safe("unknown_function"));

        assert!(!registry.is_function_safe("file_system_access"));
        assert!(!registry.is_function_safe("network_raw_socket"));
        assert!(!registry.is_function_safe("process_spawn"));
        assert!(!registry.is_function_safe("memory_direct_access"));
    }

    #[test]
    fn test_get_function_metadata() {
        let registry = HostFunctions::new();

        let log_metadata = registry.get_function_metadata_by_name("log");
        assert!(log_metadata.is_some());
        let metadata = log_metadata.unwrap();
        assert_eq!(metadata.name.to_string(), "log");
        assert_eq!(metadata.module.to_string(), "env");
        assert!(metadata.required_permission.is_none());

        let send_metadata = registry.get_function_metadata_by_name("send_message");
        assert!(send_metadata.is_some());
        let metadata = send_metadata.unwrap();
        assert_eq!(metadata.name.to_string(), "send_message");
        assert_eq!(
            metadata
                .required_permission
                .as_ref()
                .map(std::string::ToString::to_string),
            Some("messaging".to_string())
        );

        let unknown_metadata = registry.get_function_metadata_by_name("unknown");
        assert!(unknown_metadata.is_none());
    }

    #[test]
    fn test_default_impl() {
        let registry1 = HostFunctions::new();
        let registry2 = HostFunctions::default();

        assert_eq!(
            registry1.get_available_functions().len(),
            registry2.get_available_functions().len()
        );
    }
}
