---
title: "ADR-0009: CLI Tool Design"
date: 2025-08-03
status: accepted
layout: adr
categories: [Technology]
---

## Status

Accepted

## Context

With Caxton as an application server (ADR-0006), users need a command-line
interface for operational tasks, debugging, and development workflows. The CLI
must be intuitive for developers who have never used Rust while providing power
users with advanced capabilities.

The CLI serves multiple audiences:

- **Developers**: Quick iteration during agent development
- **Operators**: Production deployment and monitoring
- **Debuggers**: Troubleshooting distributed agent interactions

## Decision Drivers

- **Zero Rust requirement**: Users should never see or know about Rust
- **Discoverability**: Commands should be intuitive and self-documenting
- **Progressive disclosure**: Simple tasks simple, complex tasks possible
- **Speed**: Sub-second response for common operations
- **Integration**: Work seamlessly with existing tools (kubectl, docker, etc.)

## Decision

We will implement a noun-verb CLI structure with progressive disclosure:

### 1. Command Structure

**Noun-Verb Pattern**:

The CLI will follow a noun-verb structure where users specify the resource type
(noun) followed by the action (verb). This pattern aligns with established CLI
tools and matches user mental models from tools like kubectl, docker, and git.

### 2. Core Command Categories

**Agent Management**: Deploy, list, monitor, and remove agents from the system

**Message Operations**: Send messages between agents, trace message flows, and
monitor communication

**Server Operations**: Check system health, manage configuration, and access
system-level logging

**Development Workflow**: Support iterative development with testing,
validation, and auto-reload capabilities

### 3. Progressive Disclosure

The CLI design follows progressive disclosure principles, where:

**Level 1 - Getting Started**: Simple commands with clear guidance for
first-time users and common workflows

**Level 2 - Common Tasks**: Expanded options and detailed help for typical
operational tasks

**Level 3 - Power User**: Advanced configuration options, complex deployment
strategies, and expert-level features

This approach allows new users to be productive quickly while providing the
depth that experienced operators require.

### 4. Output Formats

**Human-Friendly Default**: Tabular format with clear headings, aligned columns,
and summary information for easy reading

**Machine-Readable Options**: JSON output for scripting and automation, wide
format with additional columns, and customizable column selection for specific
use cases

This dual approach supports both interactive use and programmatic integration.

### 5. Interactive Features

**Auto-completion**: Shell completion support for commands, subcommands, and
dynamic resource names (agent IDs, message types, etc.)

**Interactive Mode**: Optional REPL-style interface for exploration and
experimentation, with context-aware prompts and built-in help

These features reduce cognitive load and enable faster, more accurate command
execution.

### 6. Error Handling

**Clear, Actionable Errors**: Error messages that explain what went wrong, why
it happened, and how to fix it, with specific remediation steps

**Intelligent Suggestions**: Detection of common typos and mistakes with helpful
suggestions for correct commands

**Context-Aware Help**: Error messages that link to relevant help topics and
diagnostic commands

### 7. Development Workflow Integration

**Watch Mode**: Automatic redeployment when agent files change, enabling rapid
iteration during development

**Testing Workflow**: Built-in testing capabilities with scenario-based test
execution and coverage reporting

**Validation Pipeline**: Pre-deployment validation tools to catch issues early
in the development cycle

These features support modern development practices and reduce the feedback loop
between code changes and deployment.

## Consequences

### Positive

- **Intuitive**: Noun-verb structure matches user expectations
- **Discoverable**: Help at every level guides users
- **Powerful**: Advanced features available when needed
- **Fast**: Built on gRPC client for sub-second operations
- **Scriptable**: JSON output and proper exit codes
- **Integrated**: Works with standard Unix tools

### Negative

- **Binary size**: ~10MB due to gRPC and CLI framework
- **Installation**: Requires separate download/install
- **Learning curve**: Many commands to learn
- **Maintenance**: CLI must stay synchronized with API

### Mitigation Strategies

**Binary Size**:

- Provide lightweight "caxton-lite" for CI/CD
- Web-based UI alternative for some users

**Installation**:

- One-line installers for all platforms
- Package managers: brew, apt, yum
- Docker image with CLI included

**Learning Curve**:

- Interactive tutorial: `caxton tutorial`
- Command suggestions for mistakes
- Extensive examples in help text

## Related Decisions

- ADR-0006: Application Server Architecture - Established need for CLI
- ADR-0007: Management API Design - CLI uses gRPC API
- ADR-0008: Agent Deployment Model - CLI implements deployment strategies

## References

- [Command Line Interface Guidelines](https://clig.dev/)
- [The Unix Philosophy](http://www.catb.org/~esr/writings/taoup/html/ch01s06.html)
- [Kubernetes CLI Design](https://kubernetes.io/docs/reference/kubectl/)
- Nielsen's Usability Heuristics for CLIs
