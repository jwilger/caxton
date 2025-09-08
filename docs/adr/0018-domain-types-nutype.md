# ADR-0018: Use Domain Types with nutype to Eliminate Primitive Obsession

## Status

Accepted

## Context

The codebase was using primitive types (String, usize, u64, etc.) throughout
for domain concepts like agent IDs, memory sizes, CPU fuel, etc. This led to:

- Type confusion risks (mixing up different IDs or counts)
- No validation at type boundaries
- Unclear semantics in function signatures
- Potential for invalid states

## Decision

We will use the `nutype` crate to create strongly-typed domain types for all
domain concepts, following the "Parse Don't Validate" paradigm.

### Domain Types Created

- `AgentId` - Wraps UUID for agent identification
- `AgentName` - Validated string (1-255 chars)
- `HostFunctionName` - Validated string (1-100 chars)
- `MemoryBytes` - Validated memory size (max 1GB)
- `CpuFuel` - Validated CPU units (max 1 billion)
- `MessageSize` - Validated message size (max 10MB)
- `MaxAgents` - Validated agent count (1-10000)
- `MaxImportFunctions` - Validated import count (1-1000)
- `MessageCount` - Counter with increment operations
- `ExecutionTime` - Duration wrapper for execution times

### Implementation Patterns

1. Types with validation use `try_new()` returning `Result<T, TError>`
2. Types without validation use `new()` returning `T`
3. All types derive appropriate traits: Debug, Clone, Serialize, Deserialize
4. Helper methods provided for common conversions (e.g.,
   `MemoryBytes::from_mb()`)

## Consequences

### Positive

- **Type Safety**: Impossible to mix different types of IDs, sizes, or counts
- **Validation**: All values validated at construction, preventing invalid
  states
- **Self-Documenting**: Types clearly express intent and constraints
- **Zero-Cost**: nutype generates efficient wrapper types with no runtime
  overhead
- **Serialization**: All types support serde for persistence

### Negative

- **Learning Curve**: Developers need to understand nutype patterns
- **Boilerplate**: Some additional code for type construction
- **Conversion**: Need to use `into_inner()` at system boundaries

### Trade-offs

- Slightly more verbose code in exchange for complete type safety
- Construction may fail for validated types, requiring error handling
- Must remember to use correct construction method (`new` vs `try_new`)

## References

- [nutype crate documentation](https://docs.rs/nutype)
- [Parse, Don't Validate](
  https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)
