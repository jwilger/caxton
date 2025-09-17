# ADR-0043: Manual Deployment Command Pattern

**Date**: 2025-09-16
**Status**: Accepted
**Deciders**: Technical Architect, User

## Context

The deployment model for Caxton agents requires careful balance between developer control and system responsiveness. Initial implementations assumed automatic file watching and hot reload patterns similar to web development frameworks. However, user feedback and requirements analysis revealed that production-oriented deployment requires explicit control over when changes go live.

The EVENT_MODEL.md v2.1 consensus between product-manager, technical-architect, and ux-ui-design-expert established that developers need:

1. **Explicit deployment control** - Changes should only go live when explicitly commanded
2. **Fast incremental updates** - Deployments should be sub-second for single agent changes
3. **Offline status checking** - Ability to verify deployment state without server connectivity
4. **Atomic deployment semantics** - All-or-nothing updates with rollback capability

The misinterpretation of "hot reload" as automatic file watching led to CI test failures and unwanted automatic behavior. The clarified requirement is for manual deployment commands that provide fast, incremental updates to a running server.

## Decision

We will implement a **manual deployment command pattern** where:

1. **Server runs continuously** without watching workspace files
2. **`caxton deploy` command** explicitly pushes changes from workspace to server
3. **Incremental deployment** transmits only changed configurations using checksums
4. **Deployment atomicity** ensures all-or-nothing semantics with automatic rollback on failure
5. **Status visibility** through `caxton status` command works offline with cached state

The deployment workflow follows:

```bash
# Start server once (long-running)
caxton serve

# Edit configurations in workspace
vim agents/my-agent/agent.toml

# Explicitly deploy changes
caxton deploy

# Check deployment status (works offline)
caxton status
```

## Rationale

### Why Not Automatic File Watching?

Automatic file watching creates several problems:

1. **Unintended deployments** - Saving intermediate work triggers unwanted updates
2. **Resource overhead** - File system watchers consume CPU/memory continuously
3. **Platform inconsistencies** - File watching behavior varies across OS platforms
4. **CI/test complexity** - File watching tests are inherently flaky and timing-dependent
5. **Production mismatch** - Production deployments should never auto-trigger from file changes

### Industry Precedents

This pattern follows established deployment models:

- **Docker**: `docker build` → `docker push` → `docker run`
- **Kubernetes**: `kubectl apply -f config.yaml`
- **Terraform**: `terraform plan` → `terraform apply`
- **Ansible**: `ansible-playbook deploy.yml`

All these tools require explicit commands to apply changes, providing predictability and control.

### Performance Requirements

The deployment command must meet strict performance targets:

- **< 500ms** for single agent deployment
- **< 2s** for 5-10 agent workspace
- **< 5s** for 50+ agent workspace

These are achieved through:

- Checksum-based change detection (only deploy what changed)
- Binary diff transmission (minimal network overhead)
- Parallel validation and deployment operations
- Pre-validated configuration caching

### Offline Capability

The `caxton status` command maintains a local cache of:

- Last deployed configuration checksums
- Deployment timestamps and versions
- Server endpoint and connection status
- Recent deployment history

This enables developers to:

- Work offline and queue deployments
- Verify what's deployed without server access
- Diagnose deployment issues from local state
- Plan changes based on known deployed state

## Consequences

### Positive

1. **Predictable deployments** - Changes only go live when explicitly commanded
2. **Fast feedback** - Sub-second deployments maintain developer flow
3. **Reduced complexity** - No file watching reduces platform-specific code
4. **Better testing** - Explicit commands are deterministic and easy to test
5. **Production parity** - Development workflow matches production deployment
6. **Offline development** - Status checking works without server connectivity
7. **Atomic updates** - All-or-nothing deployments prevent partial states

### Negative

1. **Extra command** - Developers must remember to run `caxton deploy`
2. **No immediate feedback** - File saves don't automatically reflect in running system
3. **Manual sync burden** - Developer responsible for keeping server synchronized

### Mitigations

To address the negative consequences:

1. **Clear feedback** - Deploy command shows what changed and deployment result
2. **Status indicators** - CLI warns when workspace has undeployed changes
3. **Deployment shortcuts** - Shell aliases and editor integration for quick deploys
4. **Watch mode option** - Future `--watch` flag for development scenarios only

## Implementation Notes

The implementation requires:

1. **Workspace state tracking** - Computing checksums for all agent configurations
2. **Diff generation** - Comparing workspace to deployed state
3. **Atomic deployment protocol** - Two-phase commit with validation and application
4. **Status caching** - Local SQLite database for deployment history
5. **Rollback mechanism** - Preserving previous state for recovery

This aligns with STORY-003's revised acceptance criteria and supports the 5-10 minute onboarding experience while providing production-grade deployment control.

## References

- EVENT_MODEL.md v2.1 - Phase 1.5: Deployment Model
- REQUIREMENTS_ANALYSIS.md v1.1 - Epic 5: Agent Deployment and Management
- ADR-0006: Application Server Architecture
- ADR-0001: Observability First Architecture
