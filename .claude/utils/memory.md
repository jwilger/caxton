# Agent Memory Management System

The Caxton agent memory system provides persistent knowledge sharing across SPARC workflow executions. This system enables agents to remember decisions, learnings, and context between stories while maintaining privacy boundaries.

## Architecture Overview

### Directory Structure
```
.claude/memories/
├── shared/           # Cross-agent accessible memories
│   ├── decisions/    # Architecture decisions, design choices
│   ├── learnings/    # Patterns, anti-patterns, best practices
│   ├── context/      # Codebase context, domain knowledge
│   └── index.json    # Fast lookup index
└── private/          # Agent-specific memories
    ├── researcher/   # Research findings, external docs
    ├── implementer/  # Implementation patterns, technical notes
    ├── planner/      # Planning strategies, estimation patterns
    ├── expert/       # Code review findings, quality insights
    ├── type-architect/ # Type design decisions, domain modeling
    ├── test-hardener/ # Testing strategies, edge cases
    ├── pr-manager/   # PR patterns, review feedback
    └── index.json    # Private agent index
```

### Memory Structure
Each memory is a JSON file with:
- **Unique ID**: Generated timestamp-based identifier
- **Metadata**: Creation time, agent, tags, category
- **Content**: The actual information to remember
- **Search Keywords**: For efficient retrieval
- **Relationships**: Links to related memories

## Memory Operations

### 1. Save Memory

Agents use this pattern to save important information:

```markdown
MEMORY_SAVE: {
  "scope": "private|shared",
  "category": "decisions|learnings|context|general",
  "title": "Brief descriptive title",
  "content": "Detailed information to remember",
  "tags": ["relevant", "searchable", "tags"],
  "priority": "low|medium|high",
  "story_context": "story-051-ci-cd-pipeline"
}
```

**Implementation Process:**
1. Generate unique ID: `{timestamp}-{agent}-{hash}`
2. Create memory file in appropriate directory
3. Update index.json with metadata
4. Add search keywords for retrieval

**Privacy Rules:**
- `private` scope: Only accessible to creating agent
- `shared` scope: Accessible to all agents
- Validation prevents cross-agent private access

### 2. Search Memories

Find relevant past information:

```markdown
MEMORY_SEARCH: {
  "query": "search terms or pattern",
  "scope": "private|shared|all",
  "tags": ["optional", "filter", "tags"],
  "category": "decisions|learnings|context|general",
  "agent": "optional-agent-filter",
  "limit": 10,
  "sort": "date|relevance"
}
```

**Search Methods:**
1. **Content Search**: Use Grep tool across memory files
2. **Tag Search**: Filter by index.json tags
3. **Metadata Search**: Filter by category, agent, date
4. **Combined Search**: Multiple criteria intersection

### 3. List Recent Memories

Get overview of recent activity:

```markdown
MEMORY_LIST: {
  "scope": "private|shared|all",
  "limit": 10,
  "category": "optional-category-filter",
  "since_days": 7
}
```

### 4. Update Memory

Modify existing memory (same agent only for private):

```markdown
MEMORY_UPDATE: {
  "id": "memory-identifier",
  "updates": {
    "content": "new content",
    "tags": ["updated", "tags"],
    "title": "updated title"
  }
}
```

### 5. Delete Memory

Remove memory (same agent only for private):

```markdown
MEMORY_DELETE: {
  "id": "memory-identifier",
  "reason": "cleanup|outdated|error"
}
```

## Memory Categories

### Decisions
- Architecture decisions and rationale
- Design choices and trade-offs
- Rejected approaches and why
- Technology selection reasoning

**Example:**
```json
{
  "title": "Using nutype for domain types",
  "category": "decisions",
  "content": "Decided to use nutype crate for eliminating primitive obsession. Provides validation, sanitization, and type safety with minimal boilerplate.",
  "tags": ["architecture", "types", "nutype", "domain-modeling"]
}
```

### Learnings
- Patterns that work well in this codebase
- Anti-patterns to avoid
- Performance insights
- Testing strategies

**Example:**
```json
{
  "title": "TDD with cargo nextest pattern",
  "category": "learnings",
  "content": "Always use 'cargo nextest run --nocapture' for test execution. Provides better output and parallel execution. Red-Green-Refactor cycle works best with single failing test focus.",
  "tags": ["tdd", "testing", "nextest", "workflow"]
}
```

### Context
- Codebase architecture understanding
- Domain knowledge and business rules
- Integration patterns
- System boundaries

**Example:**
```json
{
  "title": "WASM agent isolation model",
  "category": "context",
  "content": "Agents run in WebAssembly sandboxes with resource limits. Agent<State> phantom types track lifecycle. Security policies control host function access.",
  "tags": ["wasm", "security", "agents", "isolation"]
}
```

## Best Practices

### When to Save Memories

**Always Save:**
- Important decisions and their rationale
- Patterns that work well or should be avoided
- Complex domain knowledge discoveries
- Integration insights and gotchas
- Performance bottlenecks and solutions

**Don't Save:**
- Temporary implementation details
- Obvious code patterns
- One-time fixes without broader application
- Personal preferences without technical merit

### Memory Scope Guidelines

**Use Private Scope For:**
- Agent-specific implementation patterns
- Work-in-progress research findings
- Personal notes and reminders
- Intermediate analysis steps

**Use Shared Scope For:**
- Architecture decisions affecting multiple agents
- Best practices all agents should follow
- Domain knowledge valuable to the team
- Patterns other agents should reuse

### Tagging Strategy

**Effective Tags:**
- **Domain**: `agents`, `wasm`, `messaging`, `security`
- **Technical**: `rust`, `types`, `testing`, `performance`
- **Process**: `tdd`, `review`, `deployment`, `ci-cd`
- **Quality**: `patterns`, `anti-patterns`, `gotchas`

**Tag Guidelines:**
- Use 3-6 relevant tags per memory
- Prefer existing tags for consistency
- Include both domain and technical tags
- Add process tags for workflow memories

### Search Patterns

**Common Searches:**
```markdown
# Find architecture decisions
MEMORY_SEARCH: {"query": "architecture decision", "category": "decisions"}

# Get testing patterns
MEMORY_SEARCH: {"tags": ["testing", "patterns"], "scope": "shared"}

# Find agent-specific learnings
MEMORY_SEARCH: {"agent": "implementer", "category": "learnings"}

# Search recent context
MEMORY_SEARCH: {"category": "context", "since_days": 30}
```

## Memory Lifecycle

### Automatic Cleanup
- Memories with `expires_after` are automatically considered stale
- Low priority memories older than 90 days flagged for review
- Orphaned memories (no related memories) reviewed monthly

### Manual Maintenance
- Agents should periodically review their private memories
- Shared memories reviewed during story retrospectives
- Duplicate or outdated memories should be consolidated

### Version Management
- Memory updates preserve original creation date
- Version history tracked in metadata
- Major updates create new memories with references

## Implementation Notes

### File Naming Convention
```
{timestamp}-{agent}-{category}-{hash}.json
```
Example: `1691779380-implementer-decisions-a7b2c9.json`

### Index Maintenance
- Indexes updated atomically with memory operations
- Full index rebuild available for corruption recovery
- Search optimization through keyword extraction

### Error Handling
- Invalid memory operations return descriptive errors
- Partial failures in batch operations are logged
- File system errors gracefully degrade to search-only mode

### Performance Considerations
- Index files enable fast metadata queries
- Content search uses efficient Grep tool
- Memory files kept small (< 10KB recommended)
- Automatic compression for large content

## Integration Examples

### Researcher Agent Memory Usage
```markdown
# Save external documentation reference
MEMORY_SAVE: {
  "scope": "private",
  "category": "context",
  "title": "FIPA Message Structure Reference",
  "content": "FIPA messages require performative, sender, receiver fields. Content-language defaults to JSON. Conversation-id tracks multi-turn exchanges.",
  "tags": ["fipa", "messaging", "protocol", "external-docs"]
}

# Search for similar protocols
MEMORY_SEARCH: {
  "query": "protocol message structure",
  "tags": ["protocol", "messaging"]
}
```

### Implementer Agent Memory Usage
```markdown
# Save implementation pattern
MEMORY_SAVE: {
  "scope": "shared",
  "category": "learnings",
  "title": "WASM resource limit pattern",
  "content": "Use ResourceManager::with_limits() builder. Always validate limits before agent creation. CPU fuel and memory bytes use nutype validation.",
  "tags": ["wasm", "patterns", "resource-management", "validation"]
}

# Find related patterns
MEMORY_SEARCH: {
  "tags": ["patterns", "wasm"],
  "category": "learnings"
}
```

### Expert Agent Memory Usage
```markdown
# Save code review insight
MEMORY_SAVE: {
  "scope": "shared",
  "category": "learnings",
  "title": "Error type design anti-pattern",
  "content": "Avoid generic Error types. Use specific error variants for each failure mode. Enables better error handling and user experience.",
  "tags": ["error-handling", "anti-patterns", "rust", "types"]
}
```

This memory system enables continuous learning and knowledge accumulation across SPARC workflow executions while maintaining appropriate privacy boundaries and efficient search capabilities.
