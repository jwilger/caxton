---
title: "ADR-0042: Single Binary CLI Consolidation"
date: 2025-09-15
status: accepted
layout: adr
categories: [Architecture, User Experience]
---

## Status

Accepted

## Context

Currently, Caxton distributes two separate binaries:

- `caxton`: Main binary that currently outputs "Hello, world!" (placeholder)
- `caxton-cli`: CLI binary with actual functionality (--version, --help, serve subcommand)

This creates several user experience and maintenance problems:

### User Experience Issues

**Discoverability Problems**: Users are confused about which binary to use for what purpose. Running `caxton --help` (the expected entry point) provides no useful information.

**Cognitive Load**: Users must learn and remember two different binary names, increasing mental overhead and reducing the intuitive nature of the CLI.

**Industry Pattern Violation**: Successful CLI tools (Docker, kubectl, Git, Cargo) use single binaries with subcommands rather than multiple binaries.

### Technical Issues

**Maintenance Overhead**: Two binaries require separate build configurations, testing, and distribution management.

**Code Duplication**: Shared CLI logic, error handling, and configuration parsing may be duplicated across binaries.

**Deployment Complexity**: Multiple binaries complicate packaging, installation, and PATH management.

### Conflicts with Strategic Goals

**ADR-0041 5-10 Minute Promise**: The configuration-first architecture pivot emphasizes rapid onboarding. User confusion about binary selection works directly against this crucial goal.

**ADR-0027 Concerns**: While ADR-0027 supported multiple binary targets, it identified "potential bloat" as a concern. Single binary eliminates this issue by removing redundant dependencies.

## Decision

We will **consolidate to a single `caxton` binary** with subcommands, following industry-standard CLI patterns.

### Architecture Changes

1. **Eliminate `caxton-cli` binary**: Remove separate CLI binary target from Cargo.toml
2. **Enhance main binary**: Move CLI functionality from `src/bin/caxton-cli.rs` to `src/main.rs`
3. **Subcommand structure**: Maintain existing `serve` subcommand with future mode flags
4. **Preserve interface**: All existing CLI functionality remains identical for users

### Mode Flags (Future Enhancement)

The consolidated binary will support operational mode differentiation:

```bash
# Production deployment
caxton serve --release --config production.toml

# Development server for end users
caxton serve --dev --config dev.toml

# Contributor development with debug features
caxton serve --dev --debug --reload
```

## Decision Drivers

### User Experience Excellence

- **Single Entry Point**: `caxton --help` reveals all available functionality
- **Industry Patterns**: Follows Docker (`docker run`, `docker build`), kubectl, Git conventions
- **Reduced Confusion**: One binary eliminates "which tool do I use?" questions

### Technical Benefits

- **Simplified Distribution**: One binary to package, install, and manage
- **Reduced Test Complexity**: Single binary reduces CI/CD and integration test overhead
- **Code Consolidation**: Shared argument parsing, error handling, and utilities

### Strategic Alignment

- **5-10 Minute Onboarding**: Removes friction from ADR-0041's core promise
- **Development Velocity**: Faster iteration without binary coordination overhead

## Alternatives Considered

### Keep Separate Binaries

- **Advantages**: Conceptual separation between CLI and server
- **Rejected**: User confusion outweighs theoretical benefits; industry patterns prove unified approach works

### Create Symlinks/Aliases

- **Advantages**: Maintains both interfaces temporarily
- **Rejected**: Adds complexity without addressing root cause; confuses users further

### Rename to caxton-server and caxton-cli

- **Advantages**: Makes binary purposes explicit
- **Rejected**: Still requires users to know which binary to use; violates industry patterns

## Consequences

### Positive

- **Improved Discoverability**: Users naturally run `caxton --help` and see everything
- **Faster Onboarding**: Eliminates binary selection confusion from critical first experience
- **Reduced Maintenance**: Single binary target, build configuration, and test suite
- **Industry Alignment**: Follows proven patterns from successful CLI tools
- **Future-Proof**: Mode flags provide clean path for operational differentiation

### Negative

- **Breaking Change**: Existing users running `caxton-cli` commands must update scripts
- **Single Binary Size**: Slightly larger than separate binaries (minimal impact)
- **Conceptual Merge**: Some developers prefer clear separation between client/server binaries

### Risk Mitigation

**Breaking Change Management**:

- Clear migration documentation with exact command mappings
- Deprecation notice period if needed for major users
- Simple find-and-replace migration (`caxton-cli` → `caxton`)

**Binary Size Concerns**:

- Modern CLIs routinely include both client and server functionality
- Benefits of consolidated UX outweigh minimal size increase
- Conditional compilation can optimize if needed

## Implementation Plan

### Phase 1: Consolidation

1. Move CLI logic from `src/bin/caxton-cli.rs` to `src/main.rs`
2. Remove `[[bin]]` configuration for `caxton-cli` from `Cargo.toml`
3. Update integration tests to use `caxton` instead of `caxton-cli`

### Phase 2: Verification

1. Verify all existing acceptance criteria still pass
2. Confirm CLI functionality remains identical
3. Test installation and distribution processes

### Phase 3: Enhancement Foundation

1. Add mode flag framework to serve subcommand
2. Prepare infrastructure for future operational modes
3. Update documentation and examples

## Related Decisions

- **Refines ADR-0027**: Maintains single codebase but optimizes binary distribution
- **Supports ADR-0041**: Removes friction from 5-10 minute onboarding promise
- **Aligns with ADR-0009**: Preserves all CLI UX design principles

## References

- [Command Line Interface Guidelines](https://clig.dev/)
- [Docker CLI Architecture](https://docs.docker.com/engine/reference/commandline/cli/)
- [Kubernetes kubectl Design](https://kubernetes.io/docs/reference/kubectl/)
- [The Rust Book: Binary Projects](https://doc.rust-lang.org/book/ch12-00-an-io-project.html)
