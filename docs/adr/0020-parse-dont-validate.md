# ADR-0020: Parse Don't Validate Paradigm

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

### Implementation Strategy

#### Construction Patterns

```rust
// Types with validation - can fail
let name = AgentName::try_new(input)?; // Validates 1-255 chars

// Types without validation - always succeed
let id = AgentId::new(uuid); // UUID is always valid

// After construction, the value is guaranteed valid
process_agent(name); // No validation needed here
```

#### Validation Rules in Types

```rust
#[nutype(
    validate(len_char_min = 1, len_char_max = 255),
    derive(Debug, Clone, Serialize, Deserialize),
)]
pub struct AgentName(String);
// Invalid AgentName cannot exist after construction
```

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

### Examples

#### Before (Validate)

```rust
fn process_memory(bytes: usize) -> Result<()> {
    if bytes > MAX_MEMORY {
        return Err("Invalid memory size");
    }
    // Use bytes...
}
```

#### After (Parse)

```rust
fn process_memory(bytes: MemoryBytes) -> Result<()> {
    // bytes is guaranteed valid, just use it
    allocate(bytes);
}
```

## References

- [Parse, Don't Validate - Alexis King](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)
- ADR-0018: Domain Types with nutype
- [Making Invalid States Unrepresentable](https://yoric.github.io/post/rust-typestate/)
