# Memory Operations Verification Checklist

## UUID-Based Memory Protocol Enforcement

This checklist ensures agents correctly use the dual-memory system with
UUID-based linking between qdrant and sparc-memory.

### ✅ CORRECT Memory Storage Pattern

```python
# Step 1: Generate UUID
uuid = mcp__uuid__generateUuid()  # e.g., "550e8400-e29b-41d4-a716-446655440000"

# Step 2: Store in Qdrant
mcp__qdrant__qdrant-store(
    information="Research finding about Rust async patterns...[UUID: 550e8400-e29b-41d4-a716-446655440000]"
)

# Step 3: Create Graph Node with UUID as name
mcp__sparc-memory__create_entities([{
    "name": "550e8400-e29b-41d4-a716-446655440000",  # ✅ UUID as name
    "entityType": "research-finding",  # Descriptive type
    "observations": ["Details about the finding"]
}])

# Step 4: Link UUIDs
mcp__sparc-memory__create_relations([{
    "from": "550e8400-e29b-41d4-a716-446655440000",  # ✅ UUID
    "to": "660e8400-e29b-41d4-a716-446655440001",     # ✅ UUID
    "relationType": "informs-implementation"
}])
```

### ❌ INCORRECT Patterns to AVOID

```python
# ❌ WRONG: Using descriptive names in sparc-memory
mcp__sparc-memory__create_entities([{
    "name": "rust-async-patterns",  # ❌ Descriptive name instead of UUID
    "entityType": "research",
    "observations": ["..."]
}])

# ❌ WRONG: Searching sparc-memory by descriptive names
mcp__sparc-memory__search_nodes("rust patterns")  # ❌ Semantic search

# ❌ WRONG: Not including UUID in qdrant content
mcp__qdrant__qdrant-store("Research finding...")  # ❌ Missing [UUID: xxx]
```

### ✅ CORRECT Search Pattern

```python
# Step 1: Semantic search in qdrant
results = mcp__qdrant__qdrant-find("async Rust patterns")

# Step 2: Extract UUIDs from results
uuids = []
for result in results:
    # Parse [UUID: xxx] tags
    uuid = extract_uuid_from_text(result)
    uuids.append(uuid)

# Step 3: Open specific nodes by UUID
for uuid in uuids:
    node = mcp__sparc-memory__open_nodes(names=[uuid])  # ✅ UUID lookup

# Step 4: Traverse relationships
    for relation in node.relations:
        related_uuid = relation.to  # or relation.from

# Step 5: Secondary qdrant search with related UUIDs
        related_memory = mcp__qdrant__qdrant-find(f"[UUID: {related_uuid}]")
```

## Agent Compliance Verification

### For SPARC Coordinator

- [ ] Verify each agent generates UUID before storing
- [ ] Check qdrant content includes `[UUID: xxx]` at END
- [ ] Confirm sparc-memory entities use UUID as name field
- [ ] Validate relations link UUID to UUID, not names

### For Each Agent

#### Storage Verification

- [ ] `mcp__uuid__generateUuid` called first
- [ ] Qdrant content includes descriptive text + `[UUID: {uuid}]`
- [ ] sparc-memory entity name = UUID string
- [ ] entityType is descriptive (not UUID)
- [ ] observations describe the memory content

#### Search Verification

- [ ] Starts with `mcp__qdrant__qdrant-find` (semantic)
- [ ] Extracts UUIDs from results
- [ ] Uses `mcp__sparc-memory__open_nodes` with UUID names
- [ ] NEVER uses `search_nodes` with descriptive text
- [ ] Follows graph relations by UUID

## Common Mistakes to Watch For

1. **Entity Name Confusion**: The `name` field in sparc-memory MUST be the UUID,
   not a descriptive name
2. **Search Method Mix-up**: Use `open_nodes` with UUIDs, not `search_nodes`
   with text
3. **Missing UUID Tags**: Every qdrant memory MUST end with `[UUID: xxx]`
4. **Relation Endpoints**: Both `from` and `to` in relations MUST be UUIDs
5. **UUID Generation Timing**: Generate UUID BEFORE any storage operations

## Enforcement Protocol

If an agent violates the UUID protocol:

1. **First Violation**: Coordinator corrects and reminds agent of protocol
2. **Second Violation**: Agent must demonstrate understanding by explaining
   correct process
3. **Third Violation**: Agent marked as non-compliant, work rejected

## Example Agent Memory Operation

```markdown
## Storing a research finding (researcher agent)

1. Generate UUID: `7f3b9c4d-2e1a-4f6b-8d3c-9a7e5b2c1d8f`

2. Store in qdrant: "Found that Rust's async runtime tokio provides excellent
   performance for I/O-bound operations with work-stealing scheduler.
   Documentation at https://tokio.rs shows 100K+ requests/sec possible. [UUID:
   7f3b9c4d-2e1a-4f6b-8d3c-9a7e5b2c1d8f]"

3. Create graph node:
   - name: "7f3b9c4d-2e1a-4f6b-8d3c-9a7e5b2c1d8f"
   - entityType: "research-finding"
   - observations: ["tokio async runtime", "100K+ req/sec", "work-stealing"]

4. Link to related UUID (if exists):
   - from: "7f3b9c4d-2e1a-4f6b-8d3c-9a7e5b2c1d8f"
   - to: "8a4c0d5e-3f2b-5g7c-9e4d-0b8f6c3e2f9g" (previous async research)
   - relationType: "extends-research"
```

## Validation Script (Conceptual)

```python
def validate_memory_operation(agent_action):
    # Check UUID generation
    assert "mcp__uuid__generateUuid" in agent_action

    # Check qdrant storage
    qdrant_call = find_qdrant_store(agent_action)
    assert "[UUID:" in qdrant_call.information

    # Check sparc-memory entity
    entity_call = find_create_entities(agent_action)
    uuid_pattern = (r'^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}'
                     r'-[0-9a-f]{4}-[0-9a-f]{12}$')
    assert re.match(uuid_pattern, entity_call.name)

    # Check relations use UUIDs
    relation_call = find_create_relations(agent_action)
    if relation_call:
        assert re.match(uuid_pattern, relation_call.from)
        assert re.match(uuid_pattern, relation_call.to)
```

## Summary

The dual-memory system MUST maintain UUID-based linking:

- **qdrant**: Stores semantic content with UUID tag
- **sparc-memory**: Stores graph structure with UUID as primary key
- **Search**: Semantic → Extract UUIDs → Graph lookup → Follow relations
- **Never**: Use descriptive names as sparc-memory entity names
