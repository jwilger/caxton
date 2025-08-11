# Memory System Usage Examples

This document provides concrete examples of how agents should use the memory system during SPARC workflows.

## Example Memory Operations

### 1. Researcher Agent Saving External Documentation

```markdown
MEMORY_SAVE: {
  "scope": "shared",
  "category": "context",
  "title": "FIPA Message Protocol Structure Requirements",
  "content": "FIPA (Foundation for Intelligent Physical Agents) messages require specific fields: performative (ACL message type), sender (agent identifier), receiver (agent identifier), content (message payload), content-language (defaults to JSON), conversation-id (tracks multi-turn exchanges). Reference: http://www.fipa.org/specs/fipa00061/SC00061G.html - 'FIPA ACL Message Structure Specification'",
  "tags": ["fipa", "messaging", "protocol", "external-docs", "acl"],
  "priority": "high",
  "story_context": "story-023-fipa-message-router"
}
```

### 2. Type Architect Saving Domain Design Decision

```markdown
MEMORY_SAVE: {
  "scope": "shared",
  "category": "decisions",
  "title": "AgentId newtype with validation prevents ID confusion",
  "content": "Created AgentId newtype using nutype with UUID validation. Prevents bugs where resource limits were passed where agent IDs expected. Pattern: #[nutype(sanitize(trim), validate(not_empty, len(min=1)), derive(Clone, Debug, Display, Eq, PartialEq))] pub struct AgentId(String);",
  "tags": ["types", "agent-id", "nutype", "validation", "uuid", "domain-modeling"],
  "priority": "medium",
  "story_context": "story-018-type-safety-improvements"
}
```

### 3. Implementer Saving Technical Pattern

```markdown
MEMORY_SAVE: {
  "scope": "shared",
  "category": "learnings",
  "title": "WASM resource limit enforcement pattern",
  "content": "Use ResourceManager::with_limits() builder pattern. Always validate limits before agent creation. CPU fuel uses FuelLimit newtype, memory uses MemoryBytes newtype. Both implement TryFrom for validation. Pattern: ResourceManager::new().with_cpu_fuel(fuel).with_memory_limit(memory).build()? - Returns ValidationError if limits exceed system constraints.",
  "tags": ["wasm", "resource-limits", "validation", "builder-pattern", "implementation"],
  "priority": "high",
  "story_context": "story-012-wasm-resource-management"
}
```

### 4. Expert Agent Saving Cross-Cutting Analysis

```markdown
MEMORY_SAVE: {
  "scope": "shared",
  "category": "decisions",
  "title": "Error propagation strategy: no panics, railway-oriented programming",
  "content": "Established error handling strategy: use Result<T, CaxtonError> throughout. Never panic except for truly unreachable states. CaxtonError enum covers all domain error cases. Use ? operator for error propagation. thiserror for error trait implementation. Pattern enables clean error recovery and user-friendly error messages.",
  "tags": ["error-handling", "railway-oriented", "thiserror", "architecture", "no-panics"],
  "priority": "high",
  "story_context": "story-007-error-handling-strategy"
}
```

### 5. Test Hardener Recording Failure Mode Elimination

```markdown
MEMORY_SAVE: {
  "scope": "shared",
  "category": "learnings",
  "title": "Converted runtime message validation to compile-time types",
  "content": "Original test: validate message performative is valid FIPA type. Solution: Created Performative enum with all FIPA types. Eliminated entire class of runtime validation errors. Before: String validation, After: enum Performative { Inform, Request, Agree, Refuse, ... }. All message construction now type-safe at compile time.",
  "tags": ["testing", "compile-time-safety", "message-validation", "enum-types", "fipa"],
  "priority": "medium",
  "story_context": "story-023-fipa-message-router"
}
```

## Memory Search Examples

### 1. Find Related Type Design Patterns

```markdown
MEMORY_SEARCH: {
  "query": "nutype validation",
  "scope": "shared",
  "tags": ["types", "validation"],
  "category": "decisions",
  "limit": 5
}
```

**Expected Results:**
- AgentId newtype design decision
- Resource limit type patterns
- Domain validation strategies

### 2. Find Testing-Related Learnings

```markdown
MEMORY_SEARCH: {
  "query": "test.*compile.*time",
  "scope": "shared",
  "tags": ["testing", "compile-time-safety"],
  "limit": 10
}
```

**Expected Results:**
- Type-driven test elimination examples
- Compile-time guarantee implementations
- Property-based testing patterns

### 3. Agent-Specific Pattern Search

```markdown
MEMORY_SEARCH: {
  "query": "builder pattern",
  "scope": "all",
  "agent": "implementer",
  "category": "learnings",
  "limit": 5
}
```

**Expected Results:**
- ResourceManager builder pattern
- Configuration builder implementations
- API design patterns

### 4. Recent Architecture Decisions

```markdown
MEMORY_LIST: {
  "scope": "shared",
  "category": "decisions",
  "limit": 10,
  "since_days": 30
}
```

**Expected Results:**
- Recent architecture decisions
- Type system changes
- Error handling strategies

## Memory Integration Workflow Examples

### Implementer Agent Workflow

1. **Before Implementation**: Search for similar patterns
```markdown
MEMORY_SEARCH: {
  "query": "wasm agent lifecycle",
  "scope": "shared",
  "tags": ["wasm", "agents"],
  "limit": 5
}
```

2. **During Implementation**: Record new patterns discovered
```markdown
MEMORY_SAVE: {
  "scope": "private",
  "category": "learnings",
  "title": "Agent state transition requires explicit drop",
  "content": "When transitioning Agent<Loaded> to Agent<Running>, must explicitly drop previous state. WASM module keeps references. Pattern: let running = loaded.start()?; mem::drop(loaded);",
  "tags": ["implementation", "wasm", "state-transitions", "memory-management"],
  "priority": "medium"
}
```

3. **After Success**: Share broadly applicable patterns
```markdown
MEMORY_SAVE: {
  "scope": "shared",
  "category": "learnings",
  "title": "WASM agent graceful shutdown pattern",
  "content": "Use async shutdown with timeout. Pattern: tokio::time::timeout(Duration::from_secs(5), agent.shutdown()).await?? - Prevents hanging on unresponsive agents.",
  "tags": ["wasm", "shutdown", "async", "timeout", "graceful"],
  "priority": "medium"
}
```

### Researcher Agent Workflow

1. **Research Phase**: Check existing knowledge
```markdown
MEMORY_SEARCH: {
  "query": "github actions matrix",
  "scope": "all",
  "tags": ["ci-cd", "github-actions"],
  "limit": 5
}
```

2. **Save Research Findings**:
```markdown
MEMORY_SAVE: {
  "scope": "shared",
  "category": "context",
  "title": "GitHub Actions matrix strategy for Rust cross-platform",
  "content": "GitHub Actions matrix builds support multiple OS and Rust versions. Pattern: strategy: matrix: os: [ubuntu-latest, windows-latest, macos-latest], rust: [stable, beta, nightly]. Use exclude: to skip problematic combinations. Reference: https://docs.github.com/en/actions/using-jobs/using-a-matrix-for-your-jobs",
  "tags": ["github-actions", "matrix", "cross-platform", "rust", "ci-cd"],
  "priority": "medium"
}
```

### Type Architect Agent Workflow

1. **Check Existing Type Patterns**:
```markdown
MEMORY_SEARCH: {
  "query": "phantom types state",
  "scope": "shared",
  "tags": ["types", "phantom", "state-machine"],
  "limit": 5
}
```

2. **Design New Types Based on Patterns**:
```markdown
MEMORY_SAVE: {
  "scope": "shared",
  "category": "decisions",
  "title": "DeploymentStatus phantom types prevent invalid transitions",
  "content": "Created typestate machine for deployment: Deployment<Pending>, Deployment<Active>, Deployment<Completed>. Prevents invalid operations like canceling completed deployments. Pattern uses PhantomData<Status> with sealed trait. Only valid transitions compile.",
  "tags": ["types", "phantom", "deployment", "state-machine", "typestate"],
  "priority": "high"
}
```

## Privacy Boundary Examples

### Private Memories (Agent-Specific)

**Use For:**
- Work-in-progress research notes
- Personal implementation reminders
- Agent-specific workflow improvements
- Experimental approaches being tested

**Example:**
```markdown
MEMORY_SAVE: {
  "scope": "private",
  "category": "general",
  "title": "Planning heuristic: complex features need 3x time estimate",
  "content": "Personal observation: when estimating complex multi-agent features, initial estimate is usually 3x too low. Account for integration complexity, testing edge cases, and cross-cutting concerns.",
  "tags": ["planning", "estimation", "heuristic", "personal"],
  "priority": "low"
}
```

### Shared Memories (Cross-Agent)

**Use For:**
- Architecture decisions affecting multiple agents
- Proven patterns others should reuse
- Domain knowledge valuable to the team
- Best practices and anti-patterns

**Example:**
```markdown
MEMORY_SAVE: {
  "scope": "shared",
  "category": "learnings",
  "title": "Anti-pattern: generic Error types hide failure context",
  "content": "Avoid using generic Error or anyhow::Error in domain code. Use specific error enums with thiserror. Enables better error handling, user messages, and debugging. Generic errors acceptable only at application boundaries.",
  "tags": ["error-handling", "anti-patterns", "thiserror", "domain"],
  "priority": "medium"
}
```

## Maintenance Examples

### Memory Cleanup

```markdown
MEMORY_DELETE: {
  "id": "1723402800-researcher-general-outdated",
  "reason": "outdated - replaced by newer research findings"
}
```

### Memory Updates

```markdown
MEMORY_UPDATE: {
  "id": "1723402600-type-architect-decisions-a7b2c9",
  "updates": {
    "content": "Updated nutype usage: now includes serde traits for serialization...",
    "tags": ["types", "nutype", "validation", "serde", "serialization"],
    "related_memories": ["1723402700-implementer-learnings-serde-integration"]
  }
}
```

These examples demonstrate how the memory system enables continuous learning and knowledge accumulation across SPARC workflow executions while maintaining appropriate privacy boundaries.
