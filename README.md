<div align="center">
  <img src="logo.svg" alt="Caxton Logo" width="200" height="200">
</div>

# Caxton

**Build production-ready multi-agent systems in minutes, not months.**

> **🚧 Implementation Status**
>
> This documentation represents the intended system design and serves as
> acceptance criteria for development. Caxton is currently in active
> development with core functionality being implemented according to the
> architectural vision defined in ADRs 28-30.
>
> **Current State**: Domain modeling and type system foundation
> (see `domain-modeling-experiment` branch)
> **Target**: Configuration-driven agents with 5-10 minute onboarding experience
>
> Features described below represent the planned architecture and user
> experience goals.

Caxton is a production-ready server that orchestrates multi-agent systems.
Deploy agents written in any WebAssembly-compatible language - JavaScript,
Python, Go, or Rust - with built-in message routing, fault tolerance, and
observability.

> ⚠️ **Important**: Caxton is a standalone server application, not a Rust
> library. You install and run it like any other server (Redis, Nginx, etc.) and
> interact with it via CLI or API. Unlike traditional databases, Caxton requires
> **no external dependencies** - not even PostgreSQL.

## What is Caxton?

Caxton is a multi-agent orchestration server - like Redis for caching or
PostgreSQL for data, but for coordinating intelligent agents.

You install Caxton, create agent configurations in markdown files, and it handles
all the complex distributed systems challenges: message routing, fault
tolerance, observability, and scaling.

✅ **5-10 minute agent creation** - Simple markdown + YAML configuration files
✅ **Embedded memory system** - Built-in SQLite + vector search, no external
databases
✅ **Production-ready** - Built-in observability, fault tolerance, and
horizontal scaling
✅ **Zero compilation** - Configuration agents run immediately, no toolchain
setup
✅ **Advanced options available** - WebAssembly agents for power users needing
custom algorithms

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

## From Zero to Running Agents in 5 Minutes

```bash
# 1. Start the server (10 seconds)
caxton server start
# ✓ Server running at http://localhost:8080
# ✓ Dashboard available at http://localhost:8080/dashboard
# ✓ Embedded memory system initialized

# 2. Create your first agent (2 minutes)
cat > data-analyzer.md << EOF
---
name: DataAnalyzer
capabilities:
  - data-analysis
  - report-generation
tools:
  - http_client
  - csv_parser
memory_enabled: true
system_prompt: |
  You are a data analysis expert. You can fetch CSV data from URLs
  and provide insights and summaries.
---
# DataAnalyzer Agent
I analyze data and create reports from CSV files.
EOF

# 3. Deploy the configuration agent (5 seconds)
caxton deploy data-analyzer.md
# ✓ Agent 'DataAnalyzer' deployed and ready
# ✓ Memory system connected
# ✓ Tools available: http_client, csv_parser

# 4. Test it immediately (30 seconds)
caxton chat DataAnalyzer "Analyze the sales data at https://example.com/sales.csv"
# [DataAnalyzer] Fetching CSV data...
# [DataAnalyzer] Found 1,247 sales records from Q3 2024
# [DataAnalyzer] Key insights: Revenue up 23%, top product is Widget Pro...
# [DataAnalyzer] Storing analysis patterns in memory for future use

# That's it! You have an intelligent agent that learns and remembers.
# No compilation. No external databases. No complex setup.
```

## Architecture

Caxton is a standalone application server that hosts and orchestrates
agents using a hybrid architecture:

```text
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
│  │  │    • REST/HTTP API (port 8080)                │ │ │
│  │  │    • Authentication • Authorization           │ │ │
│  │  └────────────────┬────────────────────────────┘ │ │
│  │                   │                                │ │
│  │  ┌────────────────▼────────────────────────────┐  │ │
│  │  │         Agent Runtime Environment           │  │ │
│  │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐     │  │ │
│  │  │  │ Agent A │ │ Agent B │ │ Agent C │ ... │  │ │
│  │  │  │(Config) │ │(Config) │ │ (WASM)  │     │  │ │
│  │  │  └────┬────┘ └────┬────┘ └────┬────┘     │  │ │
│  │  │       └───────────┼───────────┘           │  │ │
│  │  │                   │                        │  │ │
│  │  │  ┌────────────────▼────────────────────┐  │  │ │
│  │  │  │ FIPA Message Bus + Memory System    │  │  │ │
│  │  │  │ (SQLite + Vector Search)            │  │  │ │
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
- **Hybrid Agent Model**: Configuration agents (primary) + WebAssembly agents (advanced)
- **Embedded Memory**: SQLite + vector search with no external dependencies
- **Capability-Based Messaging**: Agents communicate via lightweight FIPA messaging
- **Management API**: Control plane for deploying and managing agents
- **Observable by Design**: Built-in logging, metrics, and distributed tracing

Unlike traditional libraries, Caxton runs independently from your application
code. You create agent configurations and deploy them - no compilation or
Rust knowledge required for most use cases.

## What Caxton Provides

| Capability | Description |
|------------|-------------|
| **Configuration Agents** | Create agents in 5-10 minutes using markdown + YAML files |
| **Embedded Memory System** | Built-in SQLite + vector search, no external databases required |
| **Message Routing** | Capability-based routing with FIPA-compliant messaging |
| **WebAssembly Support** | Advanced agents with custom algorithms (for power users) |
| **Fault Isolation** | Agent crashes don't affect other agents or the server |
| **Resource Management** | CPU and memory limits per agent with built-in monitoring |
| **Observability** | Logs, metrics, and traces out of the box |
| **Hot Deployment** | Deploy/update agents without server restart |
| **API Access** | Full control via REST/HTTP API |

## Building Agents

### Configuration Agents (Recommended)

Most agents can be created using simple markdown files with YAML configuration:

```yaml
---
name: ChatBot
version: "1.0.0"
capabilities:
  - conversation
  - customer-support
tools:
  - knowledge_base
  - ticket_system
memory_enabled: true
system_prompt: |
  You are a helpful customer support agent. You can access our knowledge base
  and create support tickets when needed.
user_prompt_template: |
  Customer inquiry: {{message}}
  Previous conversation: {{conversation_history}}
  Knowledge base context: {{relevant_knowledge}}
---

# ChatBot Agent

I provide customer support by accessing our knowledge base and
creating support tickets when needed.
```

### WebAssembly Agents (Advanced)

For custom algorithms or performance-critical code, you can create WebAssembly agents:

```rust
// Example agent in Rust
#[no_mangle]
pub extern "C" fn handle_message(msg_ptr: *const u8, msg_len: usize) -> i32 {
    // Your custom algorithm logic here
}
```

For detailed examples and language-specific guides, see the
[Building Agents Guide](docs/developer-guide/building-agents.md).

## Documentation

📖 **[Full Documentation](docs/)** - Complete guide to Caxton

### Quick Links

- 🚀 [Installation](docs/getting-started/installation.md) - Get Caxton installed
- ⚡ [Quick Start](docs/getting-started/quickstart.md) - Configuration agents in 5
  minutes
- 🎯 [First Agent](docs/getting-started/first-agent.md) - Create your first
  configuration agent
- 🔧 [Configuration](docs/getting-started/configuration.md) - Configure Caxton
- 📚 [API Reference](docs/developer-guide/api-reference.md) - Complete API docs
- 🏭 [Production Guide](docs/operations/production-deployment.md) - Deploy to
  production

## The Problem

Most agent frameworks either:

- Require 2-4 hours to create your first working agent
- Lock you into specific AI/LLM providers or programming languages
- Hide communication complexity (making debugging impossible)
- Require external databases and complex infrastructure setup

Caxton takes a different approach:

- **5-10 minute onboarding**: Create agents with simple configuration files
- **No external dependencies**: Embedded memory and messaging systems
- **Observable by design**: Comprehensive logging and OpenTelemetry tracing
- **Progressive complexity**: Start with configs, upgrade to WASM when needed

## What Caxton Does

Caxton is a multi-agent orchestration server that handles:

1. **Configuration Agent Runtime**: Deploy and run agents from markdown files
2. **Embedded Memory System**: SQLite + vector search with automatic knowledge management
3. **Capability-Based Messaging**: FIPA-compliant routing between agent capabilities
4. **Production Observability**: Structured logging, tracing, and metrics

Caxton runs as a standalone server (like PostgreSQL or Redis) and manages all
agent coordination for you.

## Memory and State Management

Caxton includes an **embedded memory system** that works out of the box:

- **Zero external dependencies** - Built-in SQLite + vector search using All-MiniLM-L6-v2
- **Automatic knowledge management** - Agents learn from interactions and store patterns
- **Semantic search capabilities** - Find relevant context using vector similarity
- **Pluggable backends** - Upgrade to Neo4j or Qdrant for high-scale deployments

Configuration agents automatically use the memory system to provide context-aware
responses and learn from successful interactions. See
[ADR-0030](docs/_adrs/0030-embedded-memory-system.md) for technical details.

## External Tools via MCP

Agents can access external tools through the Model Context Protocol, including
state persistence:

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

- **Configuration-driven agent runtime** with 5-10 minute onboarding experience
- **Embedded memory system** with SQLite + vector search for knowledge management
- **Capability-based messaging** using lightweight FIPA protocols
- **WebAssembly support** for advanced agents requiring custom
  algorithms
- **Observable agent communications** with full tracing and debugging support

## What's Out of Scope

We're intentionally NOT building:

- AI/LLM providers or model hosting (agents use external providers)
- Complex orchestration languages or workflow engines
- Agent hierarchies or permissions systems beyond basic capability routing
- **Infrastructure-level consensus protocols** (Raft, Paxos, PBFT) - use external
  coordination services
- Built-in code compilation or language toolchains

These can all be built as libraries or external services that integrate with Caxton.

## Development

### Claude Code Development Environment

Caxton is developed using [Claude Code](https://claude.ai/code), Anthropic's
official CLI for Claude. This provides:

- **AI-Assisted Development**: Most code is written with Claude Code's
  assistance using the SPARC workflow
- **Type-Driven Design**: Emphasis on making illegal states unrepresentable
  through Rust's type system
- **Test-Driven Development**: Strict Red-Green-Refactor discipline following
  Kent Beck's practices
- **Systematic Knowledge Building**: AI agents accumulate knowledge across
  development sessions

### MCP Server Integration

The development environment uses Model Context Protocol (MCP) servers for tool
access:

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

All AI agents store knowledge in MCP memory, building institutional knowledge
that improves development quality over time.

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

Caxton is inspired by the Actor model, the BEAM VM's approach to fault
tolerance, and decades of research in multi-agent systems. Special thanks to the
WebAssembly and Rust async communities.
