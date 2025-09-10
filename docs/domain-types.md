# Domain Types in Caxton

This document describes the domain types used throughout the Caxton WebAssembly
runtime to eliminate primitive obsession and improve type safety.

## Overview

Caxton uses the `nutype` crate to create strongly-typed domain values that
prevent primitive obsession throughout the codebase. These domain types provide:

- **Compile-time validation**: Invalid values are caught at compile time
- **Type safety**: Operations on the wrong type are prevented
- **Clear intent**: Code is more readable and self-documenting
- **Reduced bugs**: Eliminates many common errors from primitive mixing

## Core Domain Types

### Agent Management

#### `AgentId`

- **Type**: `Uuid` wrapper
- **Purpose**: Unique identifier for agents
- **Usage**: `AgentId::generate()` creates a new random ID
- **Derives**: Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize,
  Display

#### `AgentName`

- **Type**: `String` wrapper with validation
- **Validation**: 1-255 characters
- **Purpose**: Human-readable name for agents
- **Usage**: `AgentName::try_new("my-agent".to_string())?`

### Resource Management

#### `MemoryBytes`

- **Type**: `usize` wrapper with validation
- **Validation**: ≤ 1GB (1,073,741,824 bytes)
- **Purpose**: Memory size in bytes
- **Usage**: `MemoryBytes::from_mb(10)?` for 10MB
- **Helper methods**: `zero()`, `from_mb()`, `as_usize()`

#### `CpuFuel`

- **Type**: `u64` wrapper with validation
- **Validation**: ≤ 1 billion fuel units
- **Purpose**: CPU execution fuel for WebAssembly
- **Usage**: `CpuFuel::try_new(1_000_000)?`
- **Helper methods**: `zero()`, `saturating_add()`, `as_u64()`

#### `MaxAgentMemory`

- **Type**: `usize` wrapper with validation
- **Validation**: 0-10MB per agent
- **Purpose**: Maximum memory allocation per agent
- **Usage**: `MaxAgentMemory::try_new(1048576)?` for 1MB
- **Default**: 1MB

#### `MaxTotalMemory`

- **Type**: `usize` wrapper with validation
- **Validation**: 0-100MB total
- **Purpose**: Maximum total memory across all agents
- **Usage**: `MaxTotalMemory::try_new(104_857_600)?` for 100MB
- **Default**: 100MB

#### `MaxTableEntries`

- **Type**: `usize` wrapper with validation
- **Validation**: 1-100,000 entries
- **Purpose**: Maximum WASM table entries
- **Default**: 10,000 entries

### Message Routing

#### `MessageSize`

- **Type**: `usize` wrapper with validation
- **Validation**: ≤ 10MB
- **Purpose**: Size of messages in bytes
- **Usage**: `MessageSize::from_kb(100)?` for 100KB

#### `MessageCount`

- **Type**: `usize` wrapper
- **Purpose**: Count of messages processed
- **Usage**: `MessageCount::zero().increment()`
- **Helper methods**: `zero()`, `increment()`, `as_usize()`

#### `ChannelCapacity`

- **Type**: `usize` wrapper with validation
- **Validation**: 1-1,000,000
- **Purpose**: Capacity for message channels
- **Default**: 1,000

#### `MaxRetries`

- **Type**: `u8` wrapper with validation
- **Validation**: 1-10 retries
- **Purpose**: Maximum retry attempts for failed operations
- **Default**: 3 retries

#### `RetryDelayMs`

- **Type**: `u64` wrapper with validation
- **Validation**: 100ms - 5 minutes
- **Purpose**: Delay between retry attempts
- **Usage**: `delay.as_duration()` converts to `std::time::Duration`
- **Default**: 1 second

### Host Functions

#### `HostFunctionName`

- **Type**: `String` wrapper with validation
- **Validation**: 1-100 characters
- **Purpose**: Name of host functions exposed to WASM

#### `FunctionModuleName`

- **Type**: `String` wrapper with validation
- **Validation**: 1-100 characters
- **Purpose**: Module name containing the function

#### `FunctionDescription`

- **Type**: `String` wrapper with validation
- **Validation**: 1-1,000 characters
- **Purpose**: Human-readable description of function

#### `PermissionName`

- **Type**: `String` wrapper with validation
- **Validation**: 1-100 characters
- **Purpose**: Required permission to access host function

### Configuration Types

#### `ConnectionPoolSize`

- **Type**: `usize` wrapper with validation
- **Validation**: 1-1,000
- **Purpose**: Size of connection pools
- **Default**: 10

#### `StorageCleanupIntervalMs`

- **Type**: `u64` wrapper with validation
- **Validation**: 1 minute - 24 hours
- **Purpose**: Interval for storage cleanup operations
- **Default**: 1 hour

#### `RateLimitPerSecond`

- **Type**: `usize` wrapper with validation
- **Validation**: 1-100,000
- **Purpose**: Rate limit for messages per second
- **Default**: 1,000

## Usage Patterns

### Creating Domain Types

```rust
// Types WITH validation use try_new() -> Result<T, TError>
let agent_name = AgentName::try_new("my-agent".to_string())?;
let memory = MemoryBytes::from_mb(10)?;

// Types WITHOUT validation use new() -> T
let agent_id = AgentId::generate();
let timestamp = MessageTimestamp::now();
```

### Accessing Values

```rust
// Use into_inner() only at system boundaries
let raw_bytes: usize = memory.into_inner();

// Use helper methods for common operations
let bytes: usize = memory.as_usize();
let duration: Duration = timeout.as_duration();
```

### System Boundaries

Domain types should be used throughout internal APIs. Only extract primitive
values when interfacing with external systems:

```rust
// ✅ Good: Internal APIs use domain types
pub fn allocate_memory(&self, agent_id: AgentId, bytes: MemoryBytes) -> Result<()>

// ❌ Bad: Internal APIs use primitives
pub fn allocate_memory(&self, agent_id: Uuid, bytes: usize) -> Result<()>
```

### Error Handling

All validation errors are handled at creation time:

```rust
// Handle validation errors appropriately
match AgentName::try_new(input) {
    Ok(name) => { /* use valid name */ },
    Err(e) => return Err(anyhow!("Invalid agent name: {}", e)),
}
```

## Benefits

1. **Type Safety**: Prevents mixing different numeric types
2. **Validation**: Ensures all values are within valid ranges
3. **Documentation**: Types are self-documenting
4. **IDE Support**: Better autocompletion and error messages
5. **Refactoring Safety**: Changes to types are caught at compile time
6. **Testing**: Domain types have built-in validation tests

## Testing

The `nutype` crate automatically generates tests for all domain types:

- Boundary validation tests
- Default value tests
- Serialization/deserialization tests

Custom tests can be added for business logic:

```rust
#[test]
fn test_memory_helper_methods() {
    let memory = MemoryBytes::from_mb(1).unwrap();
    assert_eq!(memory.as_usize(), 1_048_576);
}
```

## Migration Guidelines

When adding new domain types:

1. Identify primitive obsession instances
2. Create domain type with appropriate validation
3. Update all function signatures
4. Add helper methods as needed
5. Update tests to use domain types
6. Document the new type

This systematic approach to domain modeling helps create more maintainable and
bug-free code.
