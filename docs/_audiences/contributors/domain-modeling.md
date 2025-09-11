---
title: "Domain-Driven Design with Rust Types"
description: "Comprehensive guide to domain modeling in Caxton using Scott Wlaschin's principles"
audience: contributors
categories: [Development, Architecture, Types]
layout: page
---

## Domain Modeling Philosophy

Caxton follows Scott Wlaschin's "Domain Modeling Made Functional"
principles, with the core philosophy:

> **"Make Illegal States Unrepresentable"**

This approach uses Rust's type system to encode business rules and
prevent entire classes of bugs at compile time.

## Core Domain Principles

### 1. Type-Driven Design

Use Rust's type system to encode business rules and domain concepts:

```rust
// ❌ Bad: Primitive obsession allows invalid states
fn process_agent(id: u64, name: String, memory: usize) -> Result<(), String>

// ✅ Good: Domain types make invalid states impossible
fn process_agent(
    id: AgentId,
    name: AgentName,
    memory: MemoryBytes
) -> Result<(), ProcessingError>
```

### 2. Parse, Don't Validate

Transform unstructured data into structured types at system boundaries:

```rust
// ❌ Bad: Validation scattered throughout codebase
fn create_agent(raw_name: &str) -> Result<(), ValidationError> {
    if raw_name.is_empty() || raw_name.len() > 64 {
        return Err(ValidationError::InvalidName);
    }
    // Use raw string throughout...
}

// ✅ Good: Parse once, use everywhere safely
fn create_agent(raw_name: &str) -> Result<Agent, ValidationError> {
    let name = AgentName::try_new(raw_name.to_string())?;
    Agent::new(name)  // name is guaranteed valid
}
```

### 3. Algebraic Data Types

Use sum types (enums) for OR relationships and product types (structs)
for AND relationships:

```rust
// Sum type: Agent can be in exactly one state
#[derive(Debug, Clone, PartialEq)]
pub enum AgentState {
    Unloaded,
    Loaded { module: WasmModule },
    Running { instance: WasmInstance },
    Failed { error: AgentError },
}

// Agent operations enumerated for type safety
#[derive(Debug, Clone, PartialEq)]
pub enum AgentOperation {
    Load,
    Start,
    Stop,
    Restart,
    Deploy,
    Undeploy,
    SendMessage,
    GetStatus,
}

// Product type: Deployment has all these properties
#[derive(Debug, Clone)]
pub struct Deployment {
    pub id: DeploymentId,          // AND
    pub agent_id: AgentId,         // AND
    pub strategy: DeploymentStrategy, // AND
    pub created_at: SystemTime,    // AND
}
```

### 4. Total Functions Over Partial

Prefer functions that work for all inputs of their type:

```rust
// ❌ Partial function: Can panic or fail unexpectedly
fn calculate_fuel_remaining(budget: u64, consumed: u64) -> u64 {
    budget - consumed  // Can underflow!
}

// ✅ Total function: Safe for all valid inputs
impl CpuFuel {
    pub fn subtract(self, consumed: CpuFuelAmount) -> Result<Self, FuelError> {
        let remaining = self.0.checked_sub(consumed.0)
            .ok_or(FuelError::InsufficientFuel {
                available: self,
                requested: consumed
            })?;
        Ok(Self(remaining))
    }

    pub fn saturating_subtract(self, consumed: CpuFuelAmount) -> Self {
        Self(self.0.saturating_sub(consumed.0))
    }
}
```

## Domain Types with nutype

### Creating Domain Primitives

Caxton uses the `nutype` crate to eliminate primitive obsession:

```rust
#[nutype(
    sanitize(trim),                    // Clean input data
    validate(len(min = 1, max = 64)),  // Business rules
    derive(Clone, Debug, Eq, PartialEq, Display, Serialize, Deserialize)
)]
pub struct AgentName(String);

#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 1_073_741_824), // 1GB max
    derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)
)]
pub struct MemoryBytes(usize);

#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 1_000_000_000),
    derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)
)]
pub struct CpuFuel(u64);
```

### Domain Type Patterns

**Resource Management Types:**

```rust
// Memory with validation and helper methods
#[nutype(validate(less_or_equal = 10_485_760))] // 10MB max per agent
pub struct MaxAgentMemory(usize);

impl MaxAgentMemory {
    pub fn from_mb(mb: usize) -> Result<Self, ValidationError> {
        Self::try_new(mb * 1024 * 1024)
    }

    pub fn as_mb(&self) -> usize {
        self.into_inner() / (1024 * 1024)
    }

    pub const DEFAULT: Self = Self(1_048_576); // 1MB
}

// CPU with safe arithmetic operations
impl CpuFuel {
    pub fn saturating_add(self, additional: CpuFuelAmount) -> Self {
        let sum = self.0.saturating_add(additional.0);
        Self(sum.min(Self::MAX_VALUE))
    }

    pub fn is_exhausted(&self) -> bool {
        self.0 == 0
    }
}
```

**Agent Identity Types:**

```rust
// Guaranteed unique agent identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentId(NonZeroU64);

impl AgentId {
    pub fn generate() -> Self {
        Self(NonZeroU64::new(rand::random()).expect("random u64 is non-zero"))
    }

    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let id = s.parse::<u64>()
            .map_err(|_| ParseError::InvalidFormat)?;
        let non_zero = NonZeroU64::new(id)
            .ok_or(ParseError::ZeroId)?;
        Ok(Self(non_zero))
    }

    pub fn system() -> Self {
        Self(NonZeroU64::new(1).unwrap())
    }
}

// Human-readable names with validation
impl AgentName {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_kebab_case(&self) -> String {
        self.0.to_lowercase().replace('_', "-")
    }
}
```

## State Machines with Phantom Types

### Compile-Time State Enforcement

Use phantom types to make invalid state transitions impossible:

```rust
use std::marker::PhantomData;

// Agent with compile-time state tracking
#[derive(Debug)]
pub struct Agent<State> {
    id: AgentId,
    name: AgentName,
    wasm_module: Option<WasmModule>,
    instance: Option<WasmInstance>,
    resources: ResourceLimits,
    _state: PhantomData<State>,
}

// States as zero-sized types
pub struct Unloaded;
pub struct Loaded;
pub struct Running;
pub struct Draining;
pub struct Failed;

// State transitions enforced at compile time
impl Agent<Unloaded> {
    pub fn new(id: AgentId, name: AgentName) -> Self {
        Self {
            id, name,
            wasm_module: None,
            instance: None,
            resources: ResourceLimits::default(),
            _state: PhantomData,
        }
    }

    pub fn load(
        mut self,
        module: WasmModule
    ) -> Result<Agent<Loaded>, LoadError> {
        // Validate WASM module
        module.validate()?;

        self.wasm_module = Some(module);
        Ok(Agent {
            id: self.id,
            name: self.name,
            wasm_module: self.wasm_module,
            instance: None,
            resources: self.resources,
            _state: PhantomData,
        })
    }
}

impl Agent<Loaded> {
    pub fn start(mut self) -> Result<Agent<Running>, StartError> {
        let module = self.wasm_module
            .take()
            .ok_or(StartError::NoModule)?;

        let instance = WasmInstance::new(module, &self.resources)?;
        self.instance = Some(instance);

        Ok(Agent {
            id: self.id,
            name: self.name,
            wasm_module: None,
            instance: self.instance,
            resources: self.resources,
            _state: PhantomData,
        })
    }
}

impl Agent<Running> {
    pub fn handle_message(&self, msg: FipaMessage) -> Result<(), ProcessingError> {
        let instance = self.instance
            .as_ref()
            .ok_or(ProcessingError::NoInstance)?;

        instance.process_message(msg)
    }

    pub fn drain(self) -> Result<Agent<Draining>, DrainError> {
        // Begin graceful shutdown process
        Ok(Agent {
            id: self.id,
            name: self.name,
            wasm_module: None,
            instance: self.instance,
            resources: self.resources,
            _state: PhantomData,
        })
    }
}

// Only running agents can process messages
impl Agent<Draining> {
    pub fn complete_shutdown(self) -> Agent<Unloaded> {
        Agent::new(self.id, self.name)
    }
}
```

## Smart Constructors

### Validation at Creation Time

Use smart constructors to ensure only valid data exists:

```rust
impl MessageSize {
    pub fn try_new(bytes: usize) -> Result<Self, ValidationError> {
        if bytes > 10 * 1024 * 1024 {  // 10MB limit
            return Err(ValidationError::TooLarge {
                value: bytes,
                limit: 10 * 1024 * 1024,
            });
        }
        Ok(Self(bytes))
    }

    pub fn from_kb(kb: usize) -> Result<Self, ValidationError> {
        Self::try_new(kb * 1024)
    }

    pub fn zero() -> Self {
        Self(0)
    }
}

impl ConversationId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let uuid = Uuid::parse_str(s)
            .map_err(|_| ParseError::InvalidUuid)?;
        Ok(Self(uuid))
    }
}
```

## Error Types as Documentation

### Domain-Specific Error Hierarchies

Create error types that guide proper handling:

```rust
// Top-level error type
#[derive(Debug, Error)]
pub enum CaxtonError {
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),

    #[error("Message routing error: {0}")]
    Routing(#[from] RoutingError),

    #[error("Resource error: {0}")]
    Resource(#[from] ResourceError),

    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
}

// Domain enums for error context
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceType {
    Memory,
    CpuFuel,
    ImportCount,
    ExecutionTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WasmExecutionFailure {
    OutOfFuel,
    OutOfMemory,
    InvalidInstruction,
    HostFunctionError,
    ModuleCompilationFailed,
    ModuleInstantiationFailed,
}

// Domain-specific error types with context
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Agent {agent_id} not found")]
    NotFound { agent_id: AgentId },

    #[error("Agent {agent_id} in invalid state {current_state} for operation {operation}")]
    InvalidState {
        agent_id: AgentId,
        current_state: AgentState,
        operation: AgentOperation,
    },

    #[error("Agent {agent_id} exceeded {resource_type:?} limit: {details}")]
    ResourceLimitExceeded {
        agent_id: AgentId,
        resource_type: ResourceType,
        details: String,
    },

    #[error("WASM execution failed for agent {agent_id}: {reason:?}")]
    WasmExecutionFailed {
        agent_id: AgentId,
        reason: WasmExecutionFailure,
    },
}

// Domain enums for routing error context
#[derive(Debug, Clone, PartialEq)]
pub enum DeliveryFailureReason {
    AgentUnavailable,
    NetworkTimeout,
    MessageTooLarge,
    ConversationClosed,
    SecurityPolicyViolation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageField {
    Sender,
    Receiver,
    Performative,
    ContentType,
    Body,
    ConversationId,
}

#[derive(Debug, Error)]
pub enum RoutingError {
    #[error("No route found for capability {capability}")]
    NoCapabilityProviders { capability: Capability },

    #[error("Message {message_id} delivery failed: {reason:?}")]
    DeliveryFailed {
        message_id: MessageId,
        reason: DeliveryFailureReason,
    },

    #[error("Invalid message format in field {field:?}: {details}")]
    InvalidMessageFormat {
        field: MessageField,
        details: String,
    },

    #[error("Conversation {conversation_id} not found or expired")]
    ConversationNotFound { conversation_id: ConversationId },
}
```

## Railway-Oriented Programming

### Result Chain Patterns

Model workflows as Result chains for clear error handling:

```rust
pub type CaxtonResult<T> = Result<T, CaxtonError>;

// Extension trait for additional context
pub trait CaxtonResultExt<T> {
    fn with_agent_context(self, agent_id: AgentId) -> CaxtonResult<T>;
    fn with_message_context(self, message_id: MessageId) -> CaxtonResult<T>;
}

impl<T, E> CaxtonResultExt<T> for Result<T, E>
where E: Into<CaxtonError>
{
    fn with_agent_context(self, agent_id: AgentId) -> CaxtonResult<T> {
        self.map_err(|e| {
            let error = e.into();
            tracing::error!(%agent_id, %error, "Operation failed for agent");
            error
        })
    }

    fn with_message_context(self, message_id: MessageId) -> CaxtonResult<T> {
        self.map_err(|e| {
            let error = e.into();
            tracing::error!(%message_id, %error, "Message processing failed");
            error
        })
    }
}

// Workflow as Result chain
impl AgentRuntime {
    pub async fn deploy_agent(
        &mut self,
        config: AgentConfig
    ) -> CaxtonResult<AgentId> {
        let agent_id = AgentId::generate();

        self.validate_agent_config(&config)
            .with_agent_context(agent_id)?;

        let module = self.load_wasm_module(&config.wasm_bytes)
            .with_agent_context(agent_id)?;

        let agent = Agent::new(agent_id, config.name)
            .load(module)
            .map_err(AgentError::from)
            .with_agent_context(agent_id)?
            .start()
            .map_err(AgentError::from)
            .with_agent_context(agent_id)?;

        self.register_agent(agent)
            .with_agent_context(agent_id)?;

        Ok(agent_id)
    }
}
```

## Domain Operations

### Business Logic at the Type Level

Encode business operations as type methods:

```rust
impl CpuFuel {
    pub fn subtract(self, consumed: CpuFuelAmount) -> Result<Self, FuelError> {
        let remaining = self.0.checked_sub(consumed.0)
            .ok_or(FuelError::InsufficientFuel {
                available: self,
                requested: consumed,
            })?;
        Self::try_new(remaining).map_err(|_| FuelError::InvalidAmount)
    }

    pub fn is_sufficient_for(&self, operation: &Operation) -> bool {
        self.0 >= operation.fuel_cost().0
    }

    pub fn allocate_for(&mut self, operation: &Operation) -> Result<CpuFuelAmount, FuelError> {
        let cost = operation.fuel_cost();
        if self.0 < cost.0 {
            return Err(FuelError::InsufficientFuel {
                available: *self,
                requested: cost,
            });
        }
        self.0 -= cost.0;
        Ok(cost)
    }
}

impl MemoryBytes {
    pub fn can_allocate(&self, requested: MemoryBytes) -> bool {
        self.0 >= requested.0
    }

    pub fn allocate(&mut self, requested: MemoryBytes) -> Result<(), AllocationError> {
        if !self.can_allocate(requested) {
            return Err(AllocationError::InsufficientMemory {
                available: *self,
                requested,
            });
        }
        self.0 -= requested.0;
        Ok(())
    }

    pub fn deallocate(&mut self, amount: MemoryBytes) {
        self.0 = self.0.saturating_add(amount.0);
    }
}
```

## Testing Domain Types

### Property-Based Testing

Use proptest to verify domain invariants:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn cpu_fuel_operations_maintain_invariants(
        initial in 0u64..1_000_000,
        consumed in 0u64..1_000_000
    ) {
        let fuel = CpuFuel::try_new(initial).unwrap();
        let consumption = CpuFuelAmount::try_new(consumed).unwrap();

        // saturating_subtract never underflows
        let remaining = fuel.saturating_subtract(consumption);
        assert!(remaining.as_u64() <= initial);

        // subtract returns appropriate errors
        if consumed > initial {
            assert!(fuel.subtract(consumption).is_err());
        } else {
            let result = fuel.subtract(consumption).unwrap();
            assert_eq!(result.as_u64(), initial - consumed);
        }
    }

    #[test]
    fn memory_allocation_is_reversible(
        initial in 1usize..10_485_760,  // Max agent memory
        allocated in 1usize..1024
    ) {
        let mut memory = MemoryBytes::try_new(initial).unwrap();
        let to_allocate = MemoryBytes::try_new(allocated).unwrap();

        let can_allocate = memory.can_allocate(to_allocate);

        if can_allocate {
            let before = memory;
            memory.allocate(to_allocate).unwrap();
            memory.deallocate(to_allocate);
            assert_eq!(memory.as_usize(), before.as_usize());
        }
    }

    #[test]
    fn agent_id_parsing_roundtrip(
        id in 1u64..u64::MAX
    ) {
        let original = AgentId::from_u64(id);
        let serialized = original.to_string();
        let parsed = AgentId::parse(&serialized).unwrap();
        assert_eq!(original, parsed);
    }
}
```

### Unit Testing Domain Logic

Test business rules directly:

```rust
#[cfg(test)]
mod agent_name_tests {
    use super::*;

    #[test]
    fn valid_agent_names_are_accepted() {
        let valid_names = vec![
            "web-scraper",
            "data_analyzer",
            "chatbot-v2",
            "a",                    // Minimum length
            "a".repeat(64),         // Maximum length
        ];

        for name in valid_names {
            assert!(
                AgentName::try_new(name.clone()).is_ok(),
                "Should accept valid name: {}",
                name
            );
        }
    }

    #[test]
    fn invalid_agent_names_are_rejected() {
        let invalid_names = vec![
            "",                     // Empty
            " ",                    // Whitespace only
            "a".repeat(65),         // Too long
            "agent with spaces",    // Contains spaces
            "agent@domain.com",     // Invalid characters
        ];

        for name in invalid_names {
            assert!(
                AgentName::try_new(name.clone()).is_err(),
                "Should reject invalid name: {}",
                name
            );
        }
    }

    #[test]
    fn agent_name_sanitization_works() {
        let name = AgentName::try_new("  web-scraper  ".to_string()).unwrap();
        assert_eq!(name.as_str(), "web-scraper");
    }
}
```

## Anti-Patterns to Avoid

### 1. Primitive Obsession

```rust
// ❌ Bad: Raw primitives allow invalid states
struct Agent {
    id: u64,           // Could be zero or duplicate
    name: String,      // Could be empty or too long
    memory: usize,     // Could exceed limits
    fuel: u64,         // Could underflow during operations
}

// ✅ Good: Domain types enforce invariants
struct Agent {
    id: AgentId,       // Guaranteed unique and non-zero
    name: AgentName,   // Validated length and format
    memory: MemoryBytes, // Validated within limits
    fuel: CpuFuel,     // Protected arithmetic operations
}
```

### 2. Stringly Typed Programming

```rust
// ❌ Bad: String-based error handling
fn process_message(msg: Message) -> Result<(), String> {
    if msg.sender.is_empty() {
        return Err("Missing sender".to_string());
    }
    // Error handling scattered, unclear recovery
}

// ✅ Good: Structured error types
fn process_message(msg: Message) -> Result<(), MessageError> {
    if msg.sender.is_empty() {
        return Err(MessageError::MissingSender {
            message_id: msg.id
        });
    }
    // Clear error context and recovery paths
}
```

### 3. Validation Everywhere

```rust
// ❌ Bad: Validation scattered throughout codebase
fn create_agent(name: &str) -> Result<Agent, Error> {
    if name.len() > 64 { return Err(Error::NameTooLong); }
    // ... validation logic repeated everywhere
}

fn update_agent_name(agent: &mut Agent, name: &str) -> Result<(), Error> {
    if name.len() > 64 { return Err(Error::NameTooLong); }
    // ... same validation repeated
}

// ✅ Good: Validation at boundaries only
fn create_agent(raw_name: &str) -> Result<Agent, Error> {
    let name = AgentName::try_new(raw_name.to_string())?;  // Parse once
    Agent::new(name)  // Use validated type everywhere
}

fn update_agent_name(agent: &mut Agent, name: AgentName) {
    agent.set_name(name);  // No validation needed - type guarantees validity
}
```

## Advanced Patterns

### 1. Capabilities as Types

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Capability {
    name: CapabilityName,
    version: CapabilityVersion,
    requirements: Vec<Requirement>,
}

impl Capability {
    pub fn data_analysis() -> Self {
        Self {
            name: CapabilityName::try_new("data-analysis".to_string()).unwrap(),
            version: CapabilityVersion::v1(),
            requirements: vec![
                Requirement::tool("csv_parser"),
                Requirement::tool("chart_generator"),
            ],
        }
    }

    pub fn can_handle(&self, request: &CapabilityRequest) -> bool {
        self.name == request.capability_name &&
        self.version.is_compatible_with(&request.min_version) &&
        self.requirements.iter().all(|req| req.is_satisfied())
    }
}
```

### 2. Workflow State Machines

```rust
#[derive(Debug)]
pub struct Deployment<State> {
    id: DeploymentId,
    config: DeploymentConfig,
    _state: PhantomData<State>,
}

pub struct Planning;
pub struct Validating;
pub struct Executing;
pub struct Monitoring;
pub struct Completed;
pub struct Failed;

impl Deployment<Planning> {
    pub fn validate(self) -> Result<Deployment<Validating>, ValidationError> {
        // Validation logic here
        Ok(Deployment {
            id: self.id,
            config: self.config,
            _state: PhantomData,
        })
    }
}

impl Deployment<Validating> {
    pub fn execute(self) -> Result<Deployment<Executing>, ExecutionError> {
        // Begin deployment execution
        Ok(Deployment {
            id: self.id,
            config: self.config,
            _state: PhantomData,
        })
    }
}
```

### 3. Resource Phantom Types

```rust
pub struct ResourceAllocation<T> {
    amount: T,
    allocated_at: SystemTime,
    owner: AgentId,
}

pub type MemoryAllocation = ResourceAllocation<MemoryBytes>;
pub type CpuAllocation = ResourceAllocation<CpuFuel>;

impl<T> ResourceAllocation<T> {
    fn age(&self) -> Duration {
        SystemTime::now().duration_since(self.allocated_at)
            .unwrap_or_default()
    }
}

impl MemoryAllocation {
    pub fn can_expand_by(&self, additional: MemoryBytes) -> bool {
        // Memory-specific logic
        self.amount.as_usize() + additional.as_usize() <= MAX_AGENT_MEMORY
    }
}

impl CpuAllocation {
    pub fn is_exhausted(&self) -> bool {
        self.amount.as_u64() == 0
    }
}
```

## Migration Guide

### Eliminating Primitive Obsession

### Step 1: Identify Primitives

- Look for `String`, `usize`, `u64`, `u32` in business logic
- Find repeated validation patterns
- Locate error-prone arithmetic operations

### Step 2: Create Domain Types

```rust
// Before
fn allocate_memory(agent_id: u64, bytes: usize) -> Result<(), String>

// After
fn allocate_memory(agent_id: AgentId, bytes: MemoryBytes) -> Result<(), AllocationError>
```

### Step 3: Update Function Signatures

- Replace all primitive parameters with domain types
- Update return types to use domain-specific errors
- Add smart constructors for validation

### Step 4: Add Helper Methods

```rust
impl MemoryBytes {
    pub fn from_mb(mb: usize) -> Result<Self, ValidationError> { /* ... */ }
    pub fn as_mb(&self) -> usize { /* ... */ }
    pub fn is_within_limit(&self, limit: MemoryBytes) -> bool { /* ... */ }
}
```

### Step 5: Update Tests

```rust
// Before
#[test]
fn test_memory_allocation() {
    assert!(allocate_memory(123, 1000000).is_ok());
}

// After
#[test]
fn test_memory_allocation() {
    let agent_id = AgentId::generate();
    let memory = MemoryBytes::from_mb(1).unwrap();
    assert!(allocate_memory(agent_id, memory).is_ok());
}
```

### Best Practices

1. **Start Small**: Begin with one domain type and expand
2. **Validate Once**: Use smart constructors at system boundaries
3. **Test Properties**: Use proptest for invariant verification
4. **Document Invariants**: Make business rules explicit in types
5. **Iterate**: Refine domain model based on usage patterns

This domain-driven approach to Rust development creates more
maintainable, safer, and self-documenting code by encoding business
rules directly in the type system.
