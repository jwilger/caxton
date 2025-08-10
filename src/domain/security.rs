use crate::domain_types::{MaxExports, MaxImportFunctions};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[nutype(
    validate(len_char_min = 1, len_char_max = 255),
    derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display)
)]
pub struct SafeFunctionName(String);

impl SafeFunctionName {
    pub fn is_messaging_function(&self) -> bool {
        self.to_string().starts_with("agent_message_")
    }

    pub fn is_standard_function(&self) -> bool {
        const STANDARD_FUNCTIONS: &[&str] = &[
            "agent_get_id",
            "agent_get_timestamp",
            "agent_log",
            "agent_message_send",
            "agent_message_receive",
        ];
        STANDARD_FUNCTIONS.contains(&self.to_string().as_str())
    }
}

#[nutype(
    validate(len_char_min = 1, len_char_max = 255),
    derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display)
)]
pub struct UnsafeFunctionName(String);

impl UnsafeFunctionName {
    pub fn is_memory_function(&self) -> bool {
        self.to_string().starts_with("memory_")
    }

    pub fn is_system_function(&self) -> bool {
        self.to_string().starts_with("system_") || self.to_string().starts_with("process_")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FunctionName {
    Safe(SafeFunctionName),
    Unsafe(UnsafeFunctionName),
}

impl FunctionName {
    pub fn from_str(name: &str) -> Self {
        const UNSAFE_FUNCTIONS: &[&str] = &[
            "memory_grow",
            "memory_copy",
            "table_grow",
            "table_copy",
            "process_exit",
            "system_call",
            "fd_write",
            "fd_read",
            "environ_get",
            "environ_sizes_get",
        ];

        if UNSAFE_FUNCTIONS.contains(&name) {
            FunctionName::Unsafe(UnsafeFunctionName::try_new(name.to_string()).unwrap())
        } else {
            SafeFunctionName::try_new(name.to_string())
                .map(FunctionName::Safe)
                .unwrap_or_else(|_| {
                    FunctionName::Unsafe(UnsafeFunctionName::try_new(name.to_string()).unwrap())
                })
        }
    }

    pub fn is_safe(&self) -> bool {
        matches!(self, FunctionName::Safe(_))
    }

    pub fn as_str(&self) -> String {
        match self {
            FunctionName::Safe(name) => name.to_string(),
            FunctionName::Unsafe(name) => name.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrictSecurityPolicy {
    pub max_import_functions: MaxImportFunctions,
    pub max_exports: MaxExports,
    pub allowed_functions: Vec<SafeFunctionName>,
}

impl StrictSecurityPolicy {
    pub fn new(max_import_functions: MaxImportFunctions, max_exports: MaxExports) -> Self {
        let allowed_functions = vec![
            SafeFunctionName::try_new("agent_get_id".to_string()).unwrap(),
            SafeFunctionName::try_new("agent_get_timestamp".to_string()).unwrap(),
            SafeFunctionName::try_new("agent_log".to_string()).unwrap(),
        ];

        Self {
            max_import_functions,
            max_exports,
            allowed_functions,
        }
    }

    pub fn enable_networking(&self) -> bool {
        false
    }

    pub fn enable_threads(&self) -> bool {
        false
    }

    pub fn enable_fuel_metering(&self) -> bool {
        true
    }

    pub fn is_function_allowed(&self, function: &FunctionName) -> bool {
        match function {
            FunctionName::Safe(name) => self.allowed_functions.contains(name),
            FunctionName::Unsafe(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelaxedSecurityPolicy {
    pub max_import_functions: MaxImportFunctions,
    pub max_exports: MaxExports,
    pub allowed_functions: HashSet<FunctionName>,
    pub enable_threads: bool,
    pub enable_networking: bool,
}

impl RelaxedSecurityPolicy {
    pub fn new(
        max_import_functions: MaxImportFunctions,
        max_exports: MaxExports,
        enable_threads: bool,
        enable_networking: bool,
    ) -> Self {
        let mut allowed_functions = HashSet::new();

        allowed_functions.insert(FunctionName::Safe(
            SafeFunctionName::try_new("agent_get_id".to_string()).unwrap(),
        ));
        allowed_functions.insert(FunctionName::Safe(
            SafeFunctionName::try_new("agent_get_timestamp".to_string()).unwrap(),
        ));
        allowed_functions.insert(FunctionName::Safe(
            SafeFunctionName::try_new("agent_log".to_string()).unwrap(),
        ));
        allowed_functions.insert(FunctionName::Safe(
            SafeFunctionName::try_new("agent_message_send".to_string()).unwrap(),
        ));
        allowed_functions.insert(FunctionName::Safe(
            SafeFunctionName::try_new("agent_message_receive".to_string()).unwrap(),
        ));

        if enable_networking {
            allowed_functions.insert(FunctionName::Safe(
                SafeFunctionName::try_new("network_connect".to_string()).unwrap(),
            ));
            allowed_functions.insert(FunctionName::Safe(
                SafeFunctionName::try_new("network_send".to_string()).unwrap(),
            ));
            allowed_functions.insert(FunctionName::Safe(
                SafeFunctionName::try_new("network_receive".to_string()).unwrap(),
            ));
        }

        Self {
            max_import_functions,
            max_exports,
            allowed_functions,
            enable_threads,
            enable_networking,
        }
    }

    pub fn enable_fuel_metering(&self) -> bool {
        !self.enable_threads
    }

    pub fn is_function_allowed(&self, function: &FunctionName) -> bool {
        self.allowed_functions.contains(function)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityLevel {
    Strict(StrictSecurityPolicy),
    Relaxed(RelaxedSecurityPolicy),
}

impl SecurityLevel {
    pub fn validate(&self) -> bool {
        match self {
            SecurityLevel::Strict(policy) => {
                policy.max_import_functions.into_inner() > 0 && policy.max_exports.into_inner() > 0
            }
            SecurityLevel::Relaxed(policy) => {
                policy.max_import_functions.into_inner() > 0
                    && policy.max_exports.into_inner() > 0
                    && !(policy.enable_threads && !policy.enable_fuel_metering())
            }
        }
    }

    pub fn is_function_allowed(&self, function: &FunctionName) -> bool {
        match self {
            SecurityLevel::Strict(policy) => policy.is_function_allowed(function),
            SecurityLevel::Relaxed(policy) => policy.is_function_allowed(function),
        }
    }
}

pub struct ValidatedSecurityPolicy {
    level: SecurityLevel,
}

impl ValidatedSecurityPolicy {
    pub fn new(level: SecurityLevel) -> Option<Self> {
        if level.validate() {
            Some(Self { level })
        } else {
            None
        }
    }

    pub fn level(&self) -> &SecurityLevel {
        &self.level
    }
}
