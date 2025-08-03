# 0009. CLI Tool Design

Date: 2025-08-03

## Status

Proposed

## Context

With Caxton as an application server (ADR-0006), users need a command-line interface for operational tasks, debugging, and development workflows. The CLI must be intuitive for developers who have never used Rust while providing power users with advanced capabilities.

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
```bash
caxton <noun> <verb> [options]

# Examples:
caxton agent deploy processor.wasm
caxton agent list
caxton message send --to processor --type task
caxton server status
```

**Rationale**: Matches user mental models (kubectl, docker, git)

### 2. Core Commands

```bash
# Agent Management
caxton agent deploy <file>      # Deploy new agent
caxton agent list              # List running agents  
caxton agent logs <id>         # Stream agent logs
caxton agent delete <id>       # Remove agent
caxton agent inspect <id>      # Detailed agent info

# Message Operations  
caxton message send            # Send message to agent
caxton message trace <id>      # Trace message flow
caxton message stream          # Stream all messages

# Server Operations
caxton server status           # Health and metrics
caxton server config           # View/edit configuration
caxton server logs            # Server-level logs

# Development Workflow
caxton dev watch <file>        # Auto-reload on changes
caxton dev test <file>         # Test agent locally
caxton dev validate <file>     # Pre-deployment checks
```

### 3. Progressive Disclosure

**Level 1 - Getting Started**:
```bash
$ caxton
Caxton Multi-Agent Orchestration (v1.0.0)

Quick Start:
  caxton agent deploy my-agent.wasm    Deploy your first agent
  caxton agent list                    See running agents
  caxton help                          Get detailed help

Examples:
  caxton agent deploy processor.wasm --name my-processor
  caxton message send --to my-processor --type task --data '{"id": 123}'

Run 'caxton help <command>' for more information.
```

**Level 2 - Common Tasks**:
```bash
$ caxton agent deploy --help
Deploy a WebAssembly agent to the server

Usage: caxton agent deploy <file> [options]

Options:
  -n, --name <name>           Agent name (default: from file)
  -c, --capabilities <list>   Comma-separated capabilities
  -r, --replicas <count>      Number of instances (default: 1)
  --canary                    Use canary deployment
  --shadow                    Deploy as shadow for testing

Examples:
  # Simple deployment
  caxton agent deploy processor.wasm

  # Production deployment with canary
  caxton agent deploy processor.wasm --canary --name prod-processor

  # High-availability deployment  
  caxton agent deploy processor.wasm --replicas 3
```

**Level 3 - Power User**:
```bash
$ caxton agent deploy processor.wasm \
  --strategy canary \
  --canary-stages "5:5m,25:10m,50:10m" \
  --rollback-on "error_rate>1%,p99>100ms" \
  --resource-limits "memory=100Mi,cpu=1000m" \
  --env-from secrets/prod.env \
  --trace
```

### 4. Output Formats

**Human-Friendly Default**:
```bash
$ caxton agent list
AGENT ID        NAME            STATUS    UPTIME    MESSAGES
proc-7f8d9      processor       Running   2h30m     45,231
calc-2a4e1      calculator      Running   1h15m     12,054  
filter-9b3c2    filter          Failed    -         0

3 agents (2 running, 1 failed)
```

**Machine-Readable Options**:
```bash
# JSON output for scripting
$ caxton agent list --output json

# Wide output with more columns
$ caxton agent list --output wide

# Custom columns
$ caxton agent list --output custom-columns=NAME:.name,MEM:.resources.memory
```

### 5. Interactive Features

**Auto-completion**:
```bash
# Bash/Zsh completion
$ caxton agent delete proc<TAB>
proc-7f8d9  proc-2a4e1  proc-9b3c2

# Dynamic completion for agent names
$ caxton message send --to <TAB>
processor  calculator  filter
```

**Interactive Mode**:
```bash
$ caxton interactive
caxton> agent list
[agent list output]
caxton> message send --to processor
Message type: task
Message data (JSON): {"work": "process_order", "id": 123}
✓ Message sent (trace: 7f8d9a2b)
caxton> trace 7f8d9a2b
[shows message flow through agents]
```

### 6. Error Handling

**Clear, Actionable Errors**:
```bash
$ caxton agent deploy broken.wasm
✗ Deployment failed: Validation error

The WebAssembly module failed validation:
  - Missing required export: 'handle_message'
  - Memory limit exceeds maximum (requested: 500MB, max: 100MB)

To fix:
  1. Ensure your agent exports 'handle_message' function
  2. Reduce memory usage or request limit increase

Run 'caxton dev validate broken.wasm' for detailed analysis.
```

**Suggestions for Common Mistakes**:
```bash
$ caxton agents list
✗ Unknown command: 'agents'

Did you mean?
  caxton agent list

Run 'caxton help' to see all commands.
```

### 7. Development Workflow Integration

**Watch Mode**:
```bash
$ caxton dev watch processor.wasm
👁  Watching processor.wasm for changes...
✓ Initial deployment successful
⟳ File changed, redeploying...
✓ Validation passed
✓ Agent updated (0.3s)
```

**Testing Workflow**:
```bash
$ caxton dev test processor.wasm --scenario order-processing
Running test scenario: order-processing
  ✓ Agent initialized
  ✓ Received order message
  ✓ Sent confirmation
  ✓ State correctly updated
  
All tests passed! (4/4)
Coverage: 92% of message handlers
```

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