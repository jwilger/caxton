# ADR-0045: Configuration State Management

## Status

Accepted (2025-09-16)

## Context

Following ADR-0043 (Manual Deployment Commands) and ADR-0044 (Incremental Deployment Strategy), we need to define how Caxton manages configuration state across multiple environments and handles potential conflicts between different developers deploying from their local workspaces.

Key requirements include:

- Supporting multiple developers working on different configurations
- Detecting conflicts when deployments would overwrite each other's changes
- Maintaining deployment audit trail for compliance and debugging
- Ensuring workspace deployments are traceable to source control
- Providing clear visibility into deployed vs workspace state

The challenge is balancing developer autonomy with production safety while maintaining clear traceability of all configuration changes.

## Decision

We will implement a **hybrid state management model** combining workspace development flexibility with server-side deployed state tracking, using **explicit versioning with conflict detection** and requiring **git commit tracking** for all deployments.

### Technical Specification

#### 1. Hybrid State Model

**Workspace State (Development):**

- Developers edit TOML configurations locally
- Changes tested and validated in workspace
- **REQUIRED:** Workspace must be in a git repository
- Git commit hash captured at deployment time

**Server State (Deployed):**

- Current active configuration on server
- Versioned with monotonic counter per configuration item
- Deployment metadata including deployer and git reference

#### 2. Explicit Versioning System

**Configuration Versions:**

```rust
struct ConfigVersion {
    item_path: String,        // e.g., "agents/coordinator.toml"
    version: u64,             // Monotonic counter
    checksum: String,         // SHA-256 of content
    deployed_by: String,      // User identifier
    deployed_at: DateTime,    // Timestamp
    git_commit: String,       // Git commit hash (required)
    git_branch: String,       // Git branch name
    git_dirty: bool,         // Whether workspace had uncommitted changes
}
```

**Conflict Detection Protocol:**

```rust
enum DeploymentConflict {
    VersionMismatch {
        item: String,
        workspace_base_version: u64,
        current_server_version: u64,
        conflicting_deployer: String,
    },
    MissingGitCommit {
        reason: String,  // "not a git repository" or "uncommitted changes"
    }
}
```

#### 3. Git Integration Requirements

**Pre-deployment Validation:**

```rust
struct GitValidation {
    is_git_repository: bool,
    current_commit: String,
    current_branch: String,
    has_uncommitted_changes: bool,
    remote_url: Option<String>,
}

// Deployment blocked if:
// - Not a git repository
// - Cannot determine current commit
// - (Warning only) Uncommitted changes present
```

**Git Information Capture:**

```bash
# Required git commands during deployment
git rev-parse HEAD           # Current commit hash
git symbolic-ref HEAD         # Current branch
git status --porcelain       # Check for uncommitted changes
git remote get-url origin    # Optional: remote repository
```

#### 4. Deployment Log Format

**Persistent Deployment Log (not configuration snapshots):**

```rust
struct DeploymentLogEntry {
    deployment_id: Uuid,
    timestamp: DateTime,
    deployer: String,              // User identifier
    workspace_path: String,
    git_commit: String,            // Required: full commit hash
    git_branch: String,            // Branch name at deployment
    git_dirty: bool,              // Had uncommitted changes
    git_remote: Option<String>,   // Remote URL if available
    changes: Vec<ConfigChange>,    // What changed
    result: DeploymentResult,      // Success/failure
}

struct ConfigChange {
    path: String,
    operation: ChangeOperation,    // Added, Modified, Deleted
    old_version: Option<u64>,
    new_version: u64,
    old_checksum: Option<String>,
    new_checksum: String,
}
```

**Storage Implementation:**

```sql
-- Deployment log (append-only)
CREATE TABLE deployment_log (
    deployment_id TEXT PRIMARY KEY,
    timestamp DATETIME NOT NULL,
    deployer TEXT NOT NULL,
    workspace_path TEXT NOT NULL,
    git_commit TEXT NOT NULL,       -- Required field
    git_branch TEXT NOT NULL,
    git_dirty BOOLEAN NOT NULL,
    git_remote TEXT,
    changes_json TEXT NOT NULL,     -- JSON array of changes
    result TEXT NOT NULL,

    INDEX idx_timestamp (timestamp),
    INDEX idx_deployer (deployer),
    INDEX idx_git_commit (git_commit)
);

-- Current configuration versions (mutable)
CREATE TABLE config_versions (
    item_path TEXT PRIMARY KEY,
    version INTEGER NOT NULL,
    checksum TEXT NOT NULL,
    deployed_by TEXT NOT NULL,
    deployed_at DATETIME NOT NULL,
    deployment_id TEXT REFERENCES deployment_log(deployment_id)
);
```

#### 5. Multiple Workspace Support

**Workspace Independence:**

- Each workspace maintains its own local state cache
- Developers can work on different configurations simultaneously
- No workspace "locking" or exclusive access

**Deployment Coordination:**

```rust
// Before deployment, check for conflicts
async fn check_deployment_conflicts(
    local_changes: &[ConfigChange],
    server_versions: &HashMap<String, ConfigVersion>
) -> Result<(), Vec<DeploymentConflict>> {
    let mut conflicts = Vec::new();

    for change in local_changes {
        if let Some(server_version) = server_versions.get(&change.path) {
            // Check if someone else modified since our last sync
            if change.base_version < server_version.version {
                conflicts.push(DeploymentConflict::VersionMismatch {
                    item: change.path.clone(),
                    workspace_base_version: change.base_version,
                    current_server_version: server_version.version,
                    conflicting_deployer: server_version.deployed_by.clone(),
                });
            }
        }
    }

    if conflicts.is_empty() {
        Ok(())
    } else {
        Err(conflicts)
    }
}
```

#### 6. Forward-Only Deployment Model

**No Rollback Support:**

- Deployments are always forward-moving
- No built-in rollback to previous configurations
- Recovery through new forward deployment only

**Rationale:**

- Simplifies version tracking (monotonic counters)
- Encourages proper testing before deployment
- Rollback can be achieved by deploying previous config from git history

#### 7. Conflict Resolution Strategies

**When Conflicts Detected:**

```
Developer A deploys agents/coordinator.toml (version 5 → 6)
Developer B attempts deployment based on version 5
→ CONFLICT: Version mismatch detected

Resolution options:
1. Pull latest configuration from server
2. Merge changes locally
3. Redeploy with updated base version
```

**User Workflow:**

```bash
# Check current deployed state
caxton status

# Attempt deployment
caxton deploy
> Error: Configuration conflict detected
> agents/coordinator.toml has been modified by alice@example.com
> Your version: 5, Server version: 6
>
> Suggested actions:
> 1. caxton pull agents/coordinator.toml
> 2. Merge your changes
> 3. caxton deploy

# Pull server configuration
caxton pull agents/coordinator.toml

# Merge changes (manual process)
# Redeploy
caxton deploy
> Success: Deployed 1 configuration change
> Git commit: abc123def456
> Deployment ID: 550e8400-e29b-41d4-a716-446655440000
```

## Consequences

### Positive

- **Git traceability:** Every deployment linked to exact source control state
- **Conflict prevention:** Explicit versioning prevents accidental overwrites
- **Multi-developer support:** Teams can work independently with clear coordination
- **Audit compliance:** Complete deployment history with who, what, when, and from which commit
- **Simple mental model:** Forward-only deployments are easier to reason about
- **Source of truth:** Git commits provide definitive configuration history

### Negative

- **Git dependency:** Deployments require workspace to be in git repository
- **Manual conflict resolution:** No automatic merge strategies
- **No instant rollback:** Must redeploy previous configuration
- **Storage overhead:** Deployment logs grow over time (mitigated by log rotation)
- **Extra validation step:** Git status check adds ~50ms to deployment

## References

- ADR-0043: Manual Deployment Commands (defines deployment trigger mechanism)
- ADR-0044: Incremental Deployment Strategy (defines deployment protocol)
- EVENT_MODEL.md v2.1: Deployment workflow specification
- STORY-003: Agent Configuration System requirements

## Implementation Notes

1. Use `git2` crate for git operations (libgit2 bindings)
2. Add deployment log rotation after 10,000 entries or 90 days
3. Provide `caxton log` command to view deployment history
4. Consider adding `caxton diff` to preview conflicts before deployment
5. Warning (not error) for uncommitted changes allows hot-fix deployments
6. Future enhancement: Integration with git hooks for automatic deployment on push
