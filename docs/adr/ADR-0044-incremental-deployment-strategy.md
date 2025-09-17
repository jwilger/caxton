# ADR-0044: Incremental Deployment Strategy

## Status

Accepted (2025-09-16)

## Context

Following ADR-0043's decision on manual deployment commands, we need to define how the `caxton deploy` command efficiently transmits configuration changes from a workspace to a running Caxton server. The deployment mechanism must support:

- Fast incremental updates (< 500ms for single agent changes)
- Minimal network overhead for large workspaces
- Reliable change detection and validation
- Atomic deployment with rollback capabilities
- Offline status inspection

The challenge is balancing deployment speed with reliability while maintaining clear visibility into what changes are being deployed.

## Decision

We will implement a **checksum-based incremental deployment strategy** using SHA-256 hashes organized in a Merkle tree structure for efficient change detection and minimal data transfer.

### Technical Specification

#### 1. Change Detection Protocol

**Workspace Checksums:**

- Each file gets SHA-256 checksum: `sha256(file_contents)`
- Each directory gets composite checksum: `sha256(sorted(file_checksums + subdir_checksums))`
- Root workspace checksum: `sha256(all_top_level_checksums)`

**Merkle Tree Structure:**

```
workspace_root/
  checksum: abc123...
  ├── agents/
  │   checksum: def456...
  │   ├── coordinator.toml (checksum: 111...)
  │   └── researcher.toml (checksum: 222...)
  └── tools/
      checksum: ghi789...
      └── mcp_servers.toml (checksum: 333...)
```

#### 2. Deployment Protocol

**Phase 1: Discovery (50ms target)**

```rust
// Client sends workspace Merkle tree root
DeployRequest {
    workspace_root: "abc123...",
    workspace_path: "/path/to/workspace"
}

// Server responds with diff request
DiffRequest {
    missing_nodes: ["agents/", "agents/coordinator.toml"],
    unchanged_nodes: ["tools/"]
}
```

**Phase 2: Validation (100ms target)**

```rust
// Client sends only changed content with checksums
DeployPayload {
    changes: [
        FileChange {
            path: "agents/coordinator.toml",
            checksum: "111...",
            content: Binary,  // Using binary diff if file exists
            operation: Modified
        }
    ]
}

// Server validates all checksums match
ValidationResponse {
    valid: true,
    errors: []
}
```

**Phase 3: Apply (350ms target)**

```rust
// Server applies changes atomically
DeploymentResult {
    success: true,
    deployed_checksum: "abc123...",
    changes_applied: 1,
    timestamp: "2025-09-16T22:35:59Z"
}
```

#### 3. Binary Diff Protocol

For modified files, use binary diff to minimize transfer:

- Small files (< 10KB): Send full content
- Large files (≥ 10KB): Send xdelta3 binary diff
- Compression: Zstandard for all payloads

#### 4. Atomic Deployment

**Two-Phase Commit:**

1. **Prepare Phase:** Validate all changes, stage in temporary location
2. **Commit Phase:** Atomically swap staged changes with current configuration

**Rollback on Failure:**

- Keep previous configuration until new one fully validated
- Automatic rollback if any validation fails
- Transaction log for debugging failed deployments

#### 5. Local State Tracking

**Client-side cache (SQLite):**

```sql
CREATE TABLE deployments (
    id INTEGER PRIMARY KEY,
    workspace_path TEXT,
    server_url TEXT,
    deployed_checksum TEXT,
    deployment_time TIMESTAMP,
    changes_count INTEGER
);

CREATE TABLE file_checksums (
    path TEXT PRIMARY KEY,
    checksum TEXT,
    last_modified TIMESTAMP,
    deployment_id INTEGER REFERENCES deployments(id)
);
```

**Benefits:**

- `caxton status` works offline by comparing local vs cached deployed state
- Skip unchanged file scanning on subsequent deploys
- Track deployment history for debugging

### Performance Targets

| Scenario            | Target Time | Breakdown                                        |
| ------------------- | ----------- | ------------------------------------------------ |
| Single agent change | 500ms       | Discovery: 50ms, Validation: 100ms, Apply: 350ms |
| 5-10 agent changes  | < 2s        | Parallel validation, batched apply               |
| 50+ agent changes   | < 5s        | Streaming protocol, progressive apply            |
| No changes          | < 100ms     | Fast checksum comparison only                    |

### Network Efficiency

- **Bandwidth optimization:** Only transmit changed bytes using binary diffs
- **Round-trip minimization:** Maximum 3 round-trips for any deployment
- **Compression:** Zstandard compression reduces payload by ~70% for TOML files
- **Caching:** Server caches parsed configurations to avoid re-parsing

## Consequences

### Positive

- **Minimal data transfer:** Only changed content transmitted
- **Fast change detection:** O(log n) comparisons with Merkle tree
- **Reliable deployments:** Checksums ensure data integrity
- **Atomic updates:** All-or-nothing deployment semantics
- **Offline capability:** Local state enables offline status checking
- **Debugging support:** Clear visibility into what changed
- **Incremental scaling:** Performance degrades gracefully with workspace size

### Negative

- **Checksum computation overhead:** Initial scan requires hashing all files
- **Cache maintenance:** SQLite cache must be kept in sync
- **Binary diff complexity:** Requires xdelta3 dependency
- **Storage overhead:** Local cache duplicates some server state

## References

- ADR-0043: Manual Deployment Commands (prerequisite for this deployment model)
- EVENT_MODEL.md v2.1: Deployment workflow specification
- STORY-003: Agent Configuration System requirements

## Implementation Notes

1. Use `sha2` crate for checksum computation (already in dependencies)
2. Consider `xdelta3` or `bsdiff` for binary diff implementation
3. Zstandard compression via `zstd` crate
4. SQLite cache via `rusqlite` (consistent with memory system)
5. Consider adding deployment progress callbacks for UI feedback
