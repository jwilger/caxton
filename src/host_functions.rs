//! Host function management for WebAssembly agents

use std::collections::HashMap;

/// Registry for managing host functions exposed to WebAssembly agents
pub struct HostFunctions {
    registered_functions: HashMap<String, FunctionMetadata>,
}

/// Metadata for a host function
#[derive(Clone)]
pub struct FunctionMetadata {
    /// Function name
    pub name: String,
    /// Module name containing the function
    pub module: String,
    /// Human-readable description
    pub description: String,
    /// Required permission to access this function
    pub required_permission: Option<String>,
}

impl Default for HostFunctions {
    fn default() -> Self {
        Self::new()
    }
}

impl HostFunctions {
    /// Creates a new host function registry with default functions
    pub fn new() -> Self {
        let mut functions = HashMap::new();

        functions.insert(
            "log".to_string(),
            FunctionMetadata {
                name: "log".to_string(),
                module: "env".to_string(),
                description: "Log a message from the agent".to_string(),
                required_permission: None,
            },
        );

        functions.insert(
            "get_time".to_string(),
            FunctionMetadata {
                name: "get_time".to_string(),
                module: "env".to_string(),
                description: "Get current Unix timestamp".to_string(),
                required_permission: None,
            },
        );

        functions.insert(
            "send_message".to_string(),
            FunctionMetadata {
                name: "send_message".to_string(),
                module: "env".to_string(),
                description: "Send a message to another agent".to_string(),
                required_permission: Some("messaging".to_string()),
            },
        );

        functions.insert(
            "receive_message".to_string(),
            FunctionMetadata {
                name: "receive_message".to_string(),
                module: "env".to_string(),
                description: "Receive messages from other agents".to_string(),
                required_permission: Some("messaging".to_string()),
            },
        );

        Self {
            registered_functions: functions,
        }
    }

    /// Returns a list of all available host function names
    pub fn get_available_functions(&self) -> Vec<String> {
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
    pub fn get_function_metadata(&self, name: &str) -> Option<&FunctionMetadata> {
        self.registered_functions.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_functions_new() {
        let registry = HostFunctions::new();
        let functions = registry.get_available_functions();

        assert!(functions.contains(&"log".to_string()));
        assert!(functions.contains(&"get_time".to_string()));
        assert!(functions.contains(&"send_message".to_string()));
        assert!(functions.contains(&"receive_message".to_string()));
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

        let log_metadata = registry.get_function_metadata("log");
        assert!(log_metadata.is_some());
        let metadata = log_metadata.unwrap();
        assert_eq!(metadata.name, "log");
        assert_eq!(metadata.module, "env");
        assert!(metadata.required_permission.is_none());

        let send_metadata = registry.get_function_metadata("send_message");
        assert!(send_metadata.is_some());
        let metadata = send_metadata.unwrap();
        assert_eq!(metadata.name, "send_message");
        assert_eq!(metadata.required_permission, Some("messaging".to_string()));

        let unknown_metadata = registry.get_function_metadata("unknown");
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
