---
title: "ADR-0020: Parse Don't Validate"
date: 2025-01-31
status: accepted
layout: adr
categories: [Architecture]
---


## Status

Accepted

## Context

The codebase previously validated values at usage sites, leading to:

- Repeated validation logic
- Possibility of forgetting validation
- Invalid states being possible throughout the system
- Runtime errors deep in the call stack

## Decision

Adopt the "Parse Don't Validate" paradigm:

1. Parse and validate data at system boundaries
2. Use types that make invalid states unrepresentable
3. Once parsed into a domain type, the value is guaranteed valid

This paradigm shift moves validation from usage sites to construction time,
ensuring that invalid data cannot exist within the system once it passes the
initial parsing phase.

## Consequences

### Positive

- **Guaranteed Validity**: Once constructed, values are always valid
- **Single Validation Point**: Validation happens once at construction
- **Compile-Time Safety**: Invalid states are impossible to represent
- **Simplified Logic**: Functions can assume inputs are valid
- **Error Locality**: Validation errors occur at system boundaries

### Negative

- **Construction Complexity**: Must handle potential construction failures
- **Type Proliferation**: Need many specific types instead of primitives
- **Learning Curve**: Developers must understand the paradigm

### Conceptual Change

**Before**: Functions validated inputs at every usage site, leading to scattered
validation logic and the possibility of invalid states existing throughout the
system.

**After**: Data is parsed into validated domain types at system boundaries. Once
constructed, these types guarantee validity, eliminating the need for repeated
validation checks.

## References

- [Parse, Don't Validate - Alexis King](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)
- ADR-0018: Domain Types with nutype
- [Making Invalid States Unrepresentable](https://yoric.github.io/post/rust-typestate/)
