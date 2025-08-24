# Memory Operations Verification Checklist

## Qdrant Memory System Enforcement

This checklist ensures agents correctly use the qdrant memory system for
knowledge storage and retrieval.

### ✅ CORRECT Memory Storage Pattern

```python
# Store in Qdrant with clear context
mcp__qdrant__qdrant-store(
    information="""Research finding about Rust async patterns. The tokio
    runtime provides excellent performance for I/O-bound operations with
    work-stealing scheduler. Documentation at https://tokio.rs shows
    100K+ requests/sec possible.""",
    metadata={
        "category": "research-finding",
        "topic": "async-runtime",
        "story": "story-001",
        "agent": "researcher"
    }
)
```

### ✅ CORRECT Search Pattern

```python
# Semantic search in qdrant
results = mcp__qdrant__qdrant-find("async Rust patterns")

# Process results
for result in results:
    # Use the retrieved knowledge
    apply_knowledge(result)
```

### ❌ INCORRECT Patterns to AVOID

```python
# ❌ WRONG: Storing without context
mcp__qdrant__qdrant-store("tokio is fast")  # Too vague

# ❌ WRONG: Not categorizing knowledge
mcp__qdrant__qdrant-store(
    information="Finding about async",  # Missing metadata
)
```

## Agent Compliance Verification

### For SPARC Coordinator

- [ ] Verify each agent stores knowledge after significant actions
- [ ] Check stored content includes clear context and descriptions
- [ ] Confirm metadata properly categorizes the knowledge
- [ ] Validate knowledge is searchable and retrievable

### For Each Agent

#### Storage Requirements

- [ ] Clear, descriptive content that provides value
- [ ] Proper context about when/why/how the knowledge applies
- [ ] Metadata for categorization (category, topic, story, agent)
- [ ] Relationships to other concepts described in text

#### Search Requirements

- [ ] Uses semantic search with relevant query terms
- [ ] Retrieves and applies previous knowledge
- [ ] Stores new insights discovered during work

## Common Patterns by Agent Type

### Researcher

- Research findings with sources
- API documentation summaries
- Best practices and patterns
- Tool capabilities and limitations

### Planner

- Planning strategies and templates
- Task breakdown patterns
- Architectural decisions
- Acceptance criteria patterns

### Implementers (Red/Green/Refactor)

- Test patterns and techniques
- Minimal implementation strategies
- Refactoring patterns
- Code quality improvements

### Type Architect

- Domain type designs
- Validation patterns
- State machine approaches
- Type safety strategies

### Expert

- Architectural insights
- Cross-cutting concerns
- Quality patterns
- Safety analysis results

### PR Manager

- Workflow patterns
- Commit strategies
- PR best practices
- Review response patterns

## Enforcement Protocol

If an agent fails to store knowledge:

1. **First Failure**: Coordinator requests immediate storage
2. **Second Failure**: Agent must explain what should have been stored
3. **Third Failure**: Work rejected until proper storage completed

## Example Memory Operations

### Storing a Research Finding (Researcher Agent)

```python
mcp__qdrant__qdrant-store(
    information="""
    Rust Async Runtime Research:
    - Tokio provides excellent performance for I/O-bound operations
    - Uses work-stealing scheduler for efficient task distribution
    - Documentation: https://tokio.rs
    - Benchmarks show 100K+ requests/sec possible
    - Best for: Web servers, network services, concurrent I/O
    - Trade-offs: Higher memory usage than single-threaded alternatives
    """,
    metadata={
        "category": "research-finding",
        "topic": "async-runtime",
        "story": "story-001-wasm-runtime",
        "agent": "researcher",
        "sources": ["https://tokio.rs", "benchmarks"],
        "date": "2025-01-10"
    }
)
```

### Searching for Related Knowledge

```python
# Search for async patterns
results = mcp__qdrant__qdrant-find("Rust async runtime performance tokio")

# Apply the knowledge
for finding in results:
    if "tokio" in finding and "performance" in finding:
        # Use this knowledge for implementation decisions
        apply_to_current_context(finding)
```

## Summary

The qdrant memory system requirements:

- **Storage**: Clear, contextual, categorized knowledge
- **Search**: Semantic queries to find relevant patterns
- **Quality**: Knowledge must be valuable for future reference
- **Enforcement**: All agents must participate in knowledge management
