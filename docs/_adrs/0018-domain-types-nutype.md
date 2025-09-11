---
title: "ADR-0018: Domain Types"
date: 2025-08-09
status: accepted
layout: adr
categories: [Architecture]
---


## Status

Accepted

## Context

The codebase was using primitive types (String, usize, u64, etc.) throughout for
domain concepts like agent IDs, memory sizes, CPU fuel, etc. This led to:

- Type confusion risks (mixing up different IDs or counts)
- No validation at type boundaries
- Unclear semantics in function signatures
- Potential for invalid states

## Decision

We will use the `nutype` crate to create strongly-typed domain types for all
domain concepts, following the "Parse Don't Validate" paradigm.

### Domain Types Adopted

Strong typing replaces primitive obsession for core domain concepts:

- **Identity Types**: Agent IDs and names with validation constraints
- **Resource Types**: Memory sizes, CPU units, and message sizes with limits
- **Configuration Types**: Agent counts and function limits with boundaries
- **Operational Types**: Counters and execution times with appropriate semantics

### Type Construction Patterns

Domain types use consistent construction patterns:

- **Validated Types**: Construction may fail, requiring error handling
- **Simple Types**: Direct construction for types without validation constraints
- **Trait Derivation**: Common traits (Debug, Clone, serialization) derived
  automatically
- **Conversion Helpers**: Convenience methods for common unit conversions

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
- [Parse, Don't Validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)
