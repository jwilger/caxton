---
title: "ADR-0019: Primitives at Boundaries"
date: 2025-08-09
status: accepted
layout: adr
categories: [Architecture]
---

## Status

Accepted

## Context

After introducing domain types throughout the codebase, we needed to decide
where primitive types are acceptable and where they must be avoided.

## Decision

Primitive types are ONLY allowed at system boundaries:

1. External API interactions (wasmtime, system calls)
2. Serialization/deserialization boundaries
3. User input/output interfaces
4. Internal atomic operations for performance-critical counters

All internal APIs must use domain types exclusively.

### Boundary Examples

#### Acceptable Primitive Usage

- **System Integration**: External library APIs requiring primitive types
- **Performance-Critical Counters**: Atomic operations with primitives for
  efficiency
- **Serialization**: Automatic conversion to primitives during data marshaling
- **User Input/Output**: Interface boundaries with external systems

#### Unacceptable Primitive Usage

- **Internal APIs**: All internal function signatures must use domain types
- **Business Logic**: Domain operations should never operate on raw primitives
- **Configuration**: Internal configuration should use validated domain types

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

We explicitly allow atomic primitives (AtomicUsize, AtomicU64) for internal
counters because:

1. They're implementation details, not part of public API
2. Performance is critical for these operations
3. They're already encapsulated within domain-aware structs
4. The public API still uses domain types exclusively

## References

- ADR-0018: Domain Types with nutype
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
