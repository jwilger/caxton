<div align="center">
  <img src="logo.svg" alt="Caxton Logo" width="200" height="200">
</div>

# Caxton

**Build production-ready multi-agent systems in minutes, not months.**

Caxton is a production-ready server that orchestrates multi-agent systems. Deploy agents written in any WebAssembly-compatible language - JavaScript, Python, Go, or Rust - with built-in message routing, fault tolerance, and observability.

> ⚠️ **Important**: Caxton is a standalone server application, not a Rust library. You install and run it like any other server (Redis, Nginx, etc.) and interact with it via CLI or API. Unlike traditional databases, Caxton requires **no external dependencies** - not even PostgreSQL.

## What is Caxton?

Caxton is a multi-agent orchestration server - like Redis for caching or PostgreSQL for data, but for coordinating intelligent agents.

You install Caxton, deploy your agents (written in any language), and it handles all the complex distributed systems challenges: message routing, fault tolerance, observability, and scaling.

✅ **Install in seconds** - Single binary, no external dependencies
✅ **Deploy any language** - If it compiles to WebAssembly, it runs on Caxton
✅ **Production-ready** - Built-in observability, fault tolerance, and horizontal scaling
✅ **Zero boilerplate** - Message routing and coordination handled for you
✅ **Truly lightweight** - No databases required, uses coordination protocols instead

## Installation

Caxton runs as a server on your infrastructure:

**Quick Install (Linux/macOS):**
```bash
curl -sSL https://caxton.io/install.sh | sh
```

**Package Managers:**
```bash
# macOS
brew install caxton

# Ubuntu/Debian
sudo apt install caxton

# Docker
docker run -d -p 8080:8080 caxton/caxton:latest
```

**Verify Installation:**
```bash
caxton version
caxton server status
```

## From Zero to Running Agents in 3 Minutes

```bash
# 1. Start the server (10 seconds)
caxton server start
# ✓ Server running at http://localhost:8080
# ✓ Dashboard available at http://localhost:8080/dashboard

# 2. Deploy agents that talk to each other (20 seconds)
caxton deploy examples/ping.wasm --name ping
caxton deploy examples/pong.wasm --name pong
# ✓ Agent 'ping' deployed and healthy
# ✓ Agent 'pong' deployed and healthy

# 3. Watch them communicate (immediate gratification)
caxton logs --agents ping,pong --follow
# [ping] Sending ping to pong
# [pong] Received ping, sending pong back
# [ping] Received pong, sending ping to pong
# ...

# That's it! You have a working multi-agent system.
# No configuration files. No infrastructure setup. No distributed systems PhD required.
```

## Architecture

Caxton is a standalone application server that hosts and orchestrates WebAssembly agents:

```
┌─────────────────────────────────────────────────────────────┐
│                     Your Infrastructure                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐         ┌────────────────────────────┐   │
│  │   CLI Tool  │         │   Management Dashboard    │   │
│  │  (caxton)   │         │    (Web UI - Future)     │   │
│  └──────┬──────┘         └──────────┬─────────────────┘   │
│         │                           │                       │
│         │         Network           │                       │
│         └───────────┬───────────────┘                       │
│                     │                                       │
│  ┌──────────────────▼────────────────────────────────────┐ │
│  │              Caxton Server Process                    │ │
│  │  ┌─────────────────────────────────────────────────┐ │ │
│  │  │          Management API Layer                   │ │ │
│  │  │    • gRPC (primary) • REST (gateway)          │ │ │
│  │  │    • Authentication • Authorization           │ │ │
│  │  └────────────────┬────────────────────────────┘ │ │
│  │                   │                                │ │
│  │  ┌────────────────▼────────────────────────────┐  │ │
│  │  │         Agent Runtime Environment           │  │ │
│  │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐     │  │ │
│  │  │  │ Agent A │ │ Agent B │ │ Agent C │ ... │  │ │
│  │  │  │ (WASM)  │ │ (WASM)  │ │ (WASM)  │     │  │ │
│  │  │  └────┬────┘ └────┬────┘ └────┬────┘     │  │ │
│  │  │       └───────────┼───────────┘           │  │ │
│  │  │                   │                        │  │ │
│  │  │  ┌────────────────▼────────────────────┐  │  │ │
│  │  │  │    FIPA Message Bus (Internal)      │  │  │ │
│  │  │  └─────────────────────────────────────┘  │  │ │
│  │  └─────────────────────────────────────────────┘  │ │
│  │                                                    │ │
│  │  ┌─────────────────────────────────────────────┐  │ │
│  │  │         Observability Layer                │  │ │
│  │  │  • Structured Logs • Metrics (Prometheus)  │  │ │
│  │  │  • Distributed Traces (OpenTelemetry)      │  │ │
│  │  └─────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────┘ │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Key Points:**
- **Server Process**: Runs as a system service (systemd, Docker, or Kubernetes)
- **Agent Isolation**: Each agent runs in its own WebAssembly sandbox
- **Message Passing**: Agents communicate via FIPA-compliant message bus
- **Management API**: Control plane for deploying and managing agents
- **Observable by Design**: Built-in logging, metrics, and distributed tracing

Unlike traditional libraries, Caxton runs independently from your application code. You deploy agents to it and interact via API - no Rust knowledge required.

## What Caxton Provides

| Capability | Description |
|------------|-------------|
| **Agent Hosting** | Deploy and run WebAssembly agents from any language |
| **Message Routing** | Automatic message delivery between agents |
| **Fault Isolation** | Agent crashes don't affect other agents or the server |
| **Resource Management** | CPU and memory limits per agent |
| **Observability** | Logs, metrics, and traces out of the box |
| **Hot Deployment** | Deploy/update agents without server restart |
| **API Access** | Full control via gRPC/REST API |

## Building Agents

Agents are WebAssembly modules that can be written in any language that compiles to WASM. Here's the basic structure:

```rust
// Example agent in Rust
#[no_mangle]
pub extern "C" fn handle_message(msg_ptr: *const u8, msg_len: usize) -> i32 {
    // Your agent logic here
}
```

For language-specific examples, see the [Building Agents Guide](docs/developer-guide/building-agents.md).

## Documentation

📖 **[Full Documentation](docs/)** - Complete guide to Caxton

### Quick Links
- 🚀 [Installation](docs/getting-started/installation.md) - Get Caxton installed
- ⚡ [Quick Start](docs/getting-started/quickstart.md) - Running in 3 minutes
- 🎯 [First Agent](docs/getting-started/first-agent.md) - Build your first agent
- 🔧 [Configuration](docs/getting-started/configuration.md) - Configure Caxton
- 📚 [API Reference](docs/developer-guide/api-reference.md) - Complete API docs
- 🏭 [Production Guide](docs/operations/production-deployment.md) - Deploy to production

## The Problem

Most agent frameworks either:
- Lock you into specific AI/LLM providers
- Hide communication complexity (making debugging impossible)
- Impose rigid architectural patterns
- Require complex distributed systems knowledge

Caxton takes a different approach:
- **Agent-agnostic**: Works with any agent implementation
- **Observable by design**: Comprehensive logging and OpenTelemetry tracing
- **Minimal core**: Just enough to be useful, not enough to be constraining
- **Progressive complexity**: Start simple, add sophistication as needed

## What Caxton Does

Caxton is a multi-agent orchestration server that handles:

1. **Agent Management**: Deploy, run, and monitor WebAssembly agents
2. **Message Orchestration**: FIPA-compliant routing between agents
3. **Production Observability**: Structured logging, tracing, and metrics

Caxton runs as a standalone server (like PostgreSQL or Redis) and manages all agent coordination for you.

## State Management Philosophy

Caxton uses a **coordination-first architecture** instead of shared databases:

- **No PostgreSQL/MySQL required** - Each instance uses embedded SQLite for local state
- **Cluster coordination via SWIM protocol** - Lightweight gossip for agent discovery
- **Agent state is YOUR responsibility** - Use MCP tools to persist to your preferred backend

This means Caxton can scale horizontally without database bottlenecks. See [ADR-0014](docs/adr/0014-coordination-first-architecture.md) for details.

## External Tools via MCP

Agents can access external tools through the Model Context Protocol, including state persistence:

```javascript
// In your agent (JavaScript example)
// Search the web
const result = await mcp_call("web_search", {
    query: "latest news on quantum computing"
});

// Persist state (business provides the backend)
await mcp_call("state_tool", {
    action: "store",
    key: "agent_checkpoint",
    value: currentState
});
```

## What's In Scope

Caxton provides:
- **FIPA agent interaction protocols** including Contract Net Protocol (CNP) for task delegation
- **Agent negotiation and coordination** through typed message passing
- **Observable agent communications** with full tracing support
- **WebAssembly isolation** for secure multi-tenant agent hosting

## What's Out of Scope

We're intentionally NOT building:
- Complex orchestration languages
- Built-in workflow engines
- Agent hierarchies or permissions systems
- **Infrastructure-level consensus protocols** (Raft, Paxos, PBFT) - use etcd/Consul for distributed state
- Message transformation pipelines

These can all be built as libraries on top of Caxton's simple primitives.

## Development

### Claude Code Development Environment

Caxton is developed using [Claude Code](https://claude.ai/code), Anthropic's official CLI for Claude. This provides:

- **AI-Assisted Development**: Most code is written with Claude Code's assistance using the SPARC workflow
- **Type-Driven Design**: Emphasis on making illegal states unrepresentable through Rust's type system
- **Test-Driven Development**: Strict Red-Green-Refactor discipline following Kent Beck's practices
- **Systematic Knowledge Building**: AI agents accumulate knowledge across development sessions

### MCP Server Integration

The development environment uses Model Context Protocol (MCP) servers for tool access:

**Cargo MCP Server** (`cargo` namespace):
- `cargo_test` - Run tests (replaces `cargo nextest run`)
- `cargo_check`, `cargo_clippy`, `cargo_build` - Code quality and compilation
- `cargo_add`, `cargo_remove`, `cargo_update` - Dependency management
- `cargo_run` - Execute project binaries

**Git MCP Server** (`git` namespace):
- `git_status`, `git_diff`, `git_log` - Repository state
- `git_add`, `git_commit`, `git_push` - Version control operations
- `git_branch`, `git_checkout`, `git_merge` - Branch management

**GitHub MCP Server** (`github` namespace):
- `create_pull_request`, `update_pull_request` - PR management
- `get_pull_request_status`, `list_pull_requests` - PR queries
- `add_issue_comment`, `create_and_submit_pull_request_review` - Code reviews


### Development Workflow (SPARC)

1. **Story Selection**: Pick from PLANNING.md backlog
2. **Research**: AI agent researches unknowns and documents findings
3. **Planning**: AI agent creates TDD implementation plan
4. **Implementation**: Three specialized AI agents handle TDD phases:
   - `red-implementer`: Writes failing tests
   - `green-implementer`: Implements minimal passing code
   - `refactor-implementer`: Improves code structure
5. **Expert Review**: AI agent validates architecture and type safety
6. **PR Creation**: AI agent creates draft PRs for human review

### Setting Up Development Environment

```bash
# Install Claude Code
curl -sSL https://claude.ai/install | sh

# Clone the repository
git clone https://github.com/caxton-ai/caxton.git
cd caxton

# The MCP servers are already configured in .claude/
# Start development with SPARC workflow
claude /sparc
```

All AI agents store knowledge in MCP memory, building institutional knowledge that improves development quality over time.

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Key areas where we need help:
- Example agents and patterns
- Performance optimizations
- Language bindings for agent development
- Debugging and visualization tools

## License

Caxton is dual-licensed under Apache 2.0 and MIT licenses.

## Acknowledgments

Caxton is inspired by the Actor model, the BEAM VM's approach to fault tolerance, and decades of research in multi-agent systems. Special thanks to the WebAssembly and Rust async communities.
