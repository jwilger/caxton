---
title: "ADR-0021: Atomic Primitives"
date: 2025-01-31
status: accepted
layout: adr
categories: [Architecture]
---

## Status

Accepted

## Context

During the domain types refactoring, we identified that atomic counters
(AtomicUsize, AtomicU64) were still using primitive types internally. We needed
to decide whether to wrap these in domain types or keep them as primitives.

## Decision

Keep atomic primitives (AtomicUsize, AtomicU64) unwrapped for internal state
tracking.

### Rationale

1. **Implementation Details**: Atomics are private fields, never exposed in
   public APIs
2. **Performance Critical**: Atomic operations are in hot paths, wrapper
   overhead matters
3. **Already Encapsulated**: Atomics exist within domain-aware structs
4. **Public API Protected**: All public methods use domain types (MemoryBytes,
   CpuFuel)

### Usage Pattern

Atomic primitives remain as private implementation details within domain-aware
structures, while all public APIs continue to use validated domain types. This
preserves both performance and type safety.

## Consequences

### Positive

- **Performance**: No wrapper overhead for high-frequency atomic operations
- **Simplicity**: Standard Rust atomic patterns unchanged
- **Encapsulation**: Implementation details hidden from public API
- **Type Safety**: Public API still fully type-safe with domain types

### Negative

- **Inconsistency**: Exception to "no primitives" rule
- **Potential Confusion**: Developers might wonder why atomics are exempt

### Alternative Considered

We considered creating atomic wrapper types that would enforce domain validation
at the atomic level. This was rejected because:

- Added complexity without significant safety benefit
- Performance overhead in critical paths where atomics are used
- Atomics are never exposed publicly anyway, so domain boundaries remain intact

## Guidelines

- Atomic primitives are allowed ONLY for internal counters
- Must be private fields
- Public API must convert to/from domain types
- Document why atomics are used as primitives

## References

- ADR-0018: Domain Types with nutype
- ADR-0019: Primitives Only at System Boundaries
- [Rust Atomics and Locks](https://marabos.nl/atomics/)
