# ADR-0019: Primitives Only at System Boundaries

## Status
Accepted

## Context
After introducing domain types throughout the codebase, we needed to decide where primitive types are acceptable and where they must be avoided.

## Decision
Primitive types are ONLY allowed at system boundaries:
1. External API interactions (wasmtime, system calls)
2. Serialization/deserialization boundaries
3. User input/output interfaces
4. Internal atomic operations for performance-critical counters

All internal APIs must use domain types exclusively.

### Boundary Examples

#### ✅ Acceptable Primitive Usage
```rust
// System boundary - wasmtime API
store.set_fuel(max_fuel.into_inner())

// Atomic counters - internal implementation detail
total_memory: Arc<AtomicUsize>

// Serialization boundary
#[derive(Serialize)]
pub struct Config {
    max_agents: MaxAgents, // Serializes to primitive
}
```

#### ❌ Unacceptable Primitive Usage
```rust
// Internal API - must use domain types
pub fn allocate_memory(&self, agent_id: Uuid, bytes: usize) // WRONG
pub fn allocate_memory(&self, agent_id: AgentId, bytes: MemoryBytes) // CORRECT
```

## Consequences

### Positive
- **Clear Boundaries**: Obvious where type conversion happens
- **Type Safety**: Internal APIs cannot be misused
- **Performance**: No unnecessary conversions in hot paths
- **Maintenance**: Easy to identify integration points

### Negative
- **Conversion Code**: Need explicit conversions at boundaries
- **Cognitive Load**: Must remember where primitives are allowed

### Atomic Operations Exception
We explicitly allow atomic primitives (AtomicUsize, AtomicU64) for internal counters because:
1. They're implementation details, not part of public API
2. Performance is critical for these operations
3. They're already encapsulated within domain-aware structs
4. The public API still uses domain types exclusively

## References
- ADR-0018: Domain Types with nutype
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
