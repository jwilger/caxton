---
title: "Building Agents Guide"
date: 2025-01-15
layout: page
categories: [Developer Guide]
---

**Implementation Status**: Configuration agents are the primary development
experience. WebAssembly agents are available for advanced use cases.

Complete guide for building agents on the Caxton platform, from simple
configuration-driven agents to advanced WebAssembly implementations.

## Quick Start: Configuration Agents

Configuration agents provide the fastest path to agent deployment with a
5-10 minute onboarding experience. They are defined using markdown files with
YAML frontmatter.

### Your First Configuration Agent

Create a simple data analysis agent:

```yaml
---
name: QuickAnalyzer
version: "1.0.0"
description: "Analyzes data and provides insights"

capabilities:
  - data-analysis
  - report-generation

tools:
  - http_client
  - csv_parser

memory:
  enabled: true
  scope: workspace

system_prompt: |
  You are a data analysis expert who helps users understand their data.
  You can fetch data from URLs, parse various formats, and create insights.

  Always provide clear, actionable insights and visualizations when possible.

user_prompt_template: |
  Analyze the following data request: {{request}}

  Available context: {{context}}
  User requirements: {{requirements}}
---

# QuickAnalyzer Agent

This agent specializes in quick data analysis tasks and can:

- Fetch data from HTTP endpoints using the `http_client` tool
- Parse CSV files using the `csv_parser` tool
- Store analysis results in workspace memory
- Generate clear, actionable insights

## Usage Examples

Ask the agent to:
- "Analyze the sales data at https://example.com/sales.csv"
- "What are the key trends in this quarterly data?"
- "Create a summary of the most important metrics"

## Memory Features

The agent maintains memory of past analyses in the workspace scope, enabling:
- Pattern recognition across datasets
- Contextual insights based on previous work
- Improved recommendations over time
```

Save this as `quick-analyzer.md` and deploy:

```bash
# Deploy the configuration agent
caxton deploy quick-analyzer.md

# Check deployment status
caxton agents list

# Test the agent
caxton message send \
  --capability data-analysis \
  --content '{"request": "analyze trends", "data": "..."}'
```

## Configuration Agent Architecture

Configuration agents use a multi-layered architecture for security and functionality:

```text
┌─────────────────────────────────────────┐
│           Configuration Agent            │
│  (Markdown + YAML Frontmatter)         │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│           LLM Orchestration             │
│  (Prompt Templates + Tool Calls)       │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│           MCP Server Tools              │
│    (WebAssembly Sandboxed)             │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│         External Services               │
│  (Databases, APIs, File Systems)       │
└─────────────────────────────────────────┘
```

### Security Model

- **Configuration Layer**: Schema validation and capability checking
- **LLM Orchestration**: Prompt injection protection and output validation
- **MCP Tools**: WebAssembly sandboxing provides security isolation
- **External Services**: Authentication and rate limiting

## Configuration Agent Schema

### Required Fields

```yaml
---
name: AgentName              # Unique identifier
version: "1.0.0"            # Semantic version
description: "Agent purpose" # Human-readable description
---
```

### Capabilities System

Declare what your agent can do:

```yaml
capabilities:
  - message-send       # Send FIPA messages
  - message-receive    # Receive FIPA messages
  - data-analysis      # Analyze data
  - report-generation  # Create reports
  - file-processing    # Handle files
  - api-integration    # External API calls
```

### Tool Integration

Specify which tools your agent needs:

```yaml
tools:
  - http_client        # HTTP requests
  - database_reader    # Database queries
  - file_processor     # File operations
  - email_sender       # Email integration
  - csv_parser         # CSV handling
  - json_processor     # JSON manipulation
  - chart_generator    # Data visualization
```

### Memory Configuration

Configure persistent memory for your agent:

```yaml
memory:
  enabled: true
  scope: workspace      # Options: agent, workspace, global
  max_entities: 10000   # Prevent memory exhaustion
  max_relations: 50000  # Limit relationship complexity

  # Memory search settings
  search:
    similarity_threshold: 0.7
    max_results: 20
```

### Prompt Engineering

Craft effective prompts for your agent:

```yaml
system_prompt: |
  You are an expert in {{domain}} with the following capabilities:
  {{#each capabilities}}
  - {{this}}
  {{/each}}

  Always be helpful, accurate, and efficient.
  Use available tools when needed to gather information.

user_prompt_template: |
  User Request: {{request}}

  {{#if context}}
  Context from memory: {{context}}
  {{/if}}

  {{#if conversation_history}}
  Previous messages: {{conversation_history}}
  {{/if}}

  Please help the user with their request.
```

## Advanced Configuration Patterns

### Multi-Capability Agent

Create agents that handle multiple types of requests:

```yaml
---
name: BusinessAnalyst
version: "1.0.0"
description: "Comprehensive business analysis agent"

capabilities:
  - data-analysis
  - report-generation
  - market-research
  - financial-modeling

tools:
  - http_client
  - database_reader
  - csv_parser
  - excel_processor
  - chart_generator
  - pdf_generator

memory:
  enabled: true
  scope: workspace

system_prompt: |
  You are a senior business analyst with expertise in:
  - Data analysis and visualization
  - Financial modeling and forecasting
  - Market research and competitive analysis
  - Report generation and presentation

  Use appropriate tools based on the request type:
  - For data analysis: csv_parser, chart_generator
  - For research: http_client, database_reader
  - For reports: pdf_generator, excel_processor

routing_rules:
  - condition: "request contains 'financial'"
    tools: ["excel_processor", "chart_generator"]
  - condition: "request contains 'market'"
    tools: ["http_client", "database_reader"]
  - condition: "request contains 'report'"
    tools: ["pdf_generator", "chart_generator"]
---

# BusinessAnalyst Agent

Comprehensive business analysis capabilities including financial modeling, market research, and data visualization.
```

### Workflow Agent

Create agents that handle complex multi-step workflows:

```yaml
---
name: OrderProcessor
version: "1.0.0"
description: "Processes customer orders end-to-end"

capabilities:
  - order-processing
  - inventory-management
  - payment-processing
  - notification-sending

tools:
  - database_reader
  - database_writer
  - payment_gateway
  - email_sender
  - inventory_checker

memory:
  enabled: true
  scope: workspace

workflows:
  order_processing:
    steps:
      - name: validate_order
        tools: ["database_reader"]
        validation: "order has required fields"

      - name: check_inventory
        tools: ["inventory_checker"]
        condition: "inventory > order.quantity"

      - name: process_payment
        tools: ["payment_gateway"]
        rollback_on_failure: true

      - name: update_inventory
        tools: ["database_writer", "inventory_checker"]

      - name: send_confirmation
        tools: ["email_sender"]

error_handling:
  payment_failure:
    action: "rollback"
    notify: ["admin", "customer"]

  inventory_insufficient:
    action: "suggest_alternatives"
    tools: ["database_reader"]
---

# OrderProcessor Agent

Handles complete order processing workflows with built-in error handling and rollback capabilities.
```

## Agent Development Best Practices

### 1. Start Simple

Begin with minimal functionality and expand:

```yaml
# v1.0.0 - Basic functionality
capabilities:
  - data-analysis

tools:
  - csv_parser

# v1.1.0 - Add visualization
tools:
  - csv_parser
  - chart_generator

# v1.2.0 - Add external data
tools:
  - csv_parser
  - chart_generator
  - http_client
```

### 2. Use Descriptive Names

Make your agent's purpose clear:

```yaml
# Good
name: SalesDataAnalyzer
description: "Analyzes sales data and generates trend reports"

# Avoid
name: Agent1
description: "Does analysis"
```

### 3. Implement Progressive Enhancement

Layer capabilities for different user skill levels:

```yaml
user_prompt_templates:
  beginner: |
    I can help you analyze data. Just upload a CSV file or share a URL.

  intermediate: |
    I can analyze data from various sources:
    - Upload CSV/Excel files
    - Fetch data from URLs
    - Query databases
    - Generate visualizations

  expert: |
    Available analysis methods: {{methods}}
    Supported data sources: {{sources}}
    Custom parameters: {{parameters}}
```

### 4. Error Handling

Implement robust error handling:

```yaml
error_templates:
  tool_unavailable: |
    I need the {{tool}} tool to complete this task, but it's not currently available.
    Please contact your administrator or try a different approach.

  data_invalid: |
    The data you provided doesn't match the expected format.
    Expected: {{expected_format}}
    Received: {{actual_format}}

  permission_denied: |
    I don't have permission to access {{resource}}.
    Required capability: {{required_capability}}
```

## Testing Configuration Agents

### Unit Testing

Test individual agent components:

```bash
# Test schema validation
caxton validate quick-analyzer.md

# Test capability declarations
caxton check-capabilities quick-analyzer.md

# Test tool availability
caxton verify-tools quick-analyzer.md
```

### Integration Testing

Test agent behavior in realistic scenarios:

```bash
# Deploy to test environment
caxton deploy quick-analyzer.md --env test

# Send test messages
caxton test-suite run \
  --agent quick-analyzer \
  --scenarios test/scenarios.yaml

# Check memory functionality
caxton memory test \
  --agent quick-analyzer \
  --operations crud
```

### Performance Testing

Ensure agents handle load appropriately:

```bash
# Load testing
caxton load-test \
  --agent quick-analyzer \
  --concurrent 10 \
  --duration 60s

# Memory usage testing
caxton monitor memory \
  --agent quick-analyzer \
  --alert-threshold 100MB
```

## WebAssembly Agents (Advanced)

For use cases requiring custom algorithms, performance optimization, or
strict security isolation, WebAssembly agents provide maximum flexibility.

### When to Use WASM Agents

Consider WASM agents when you need:

- **Custom Algorithms**: Mathematical models, ML inference, complex logic
- **Performance**: Computationally intensive operations
- **Strict Isolation**: Maximum security for untrusted code
- **Legacy Integration**: Existing code in C/C++/Rust/Go
- **Binary Distribution**: Compiled modules for proprietary algorithms

### WASM Agent Development

Create a simple WASM agent in Rust:

```rust
// src/lib.rs
use caxton_sdk::{Agent, Message, Response, Capability};
use serde_json::Value;

#[derive(Default)]
pub struct MathAgent;

impl Agent for MathAgent {
    fn name(&self) -> &str {
        "advanced-math-processor"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::Custom("mathematical-modeling".to_string()),
            Capability::Custom("statistical-analysis".to_string()),
        ]
    }

    fn handle_message(&mut self, message: Message) -> Result<Response, Box<dyn std::error::Error>> {
        let content: Value = serde_json::from_str(&message.content)?;

        match content["operation"].as_str() {
            Some("fibonacci") => {
                let n = content["n"].as_u64().unwrap_or(0);
                let result = fibonacci(n);
                Ok(Response::new(serde_json::json!({
                    "result": result,
                    "operation": "fibonacci",
                    "input": n
                })))
            },
            Some("prime_factors") => {
                let n = content["n"].as_u64().unwrap_or(0);
                let factors = prime_factors(n);
                Ok(Response::new(serde_json::json!({
                    "result": factors,
                    "operation": "prime_factors",
                    "input": n
                })))
            },
            _ => Ok(Response::error("Unknown mathematical operation"))
        }
    }
}

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn prime_factors(mut n: u64) -> Vec<u64> {
    let mut factors = Vec::new();
    let mut d = 2;

    while d * d <= n {
        while n % d == 0 {
            factors.push(d);
            n /= d;
        }
        d += 1;
    }

    if n > 1 {
        factors.push(n);
    }

    factors
}

caxton_sdk::export_agent!(MathAgent);
```

Build and deploy:

```bash
# Build WASM module
cargo build --target wasm32-wasi --release

# Optimize the binary
wasm-opt target/wasm32-wasi/release/math_agent.wasm \
  -O3 -o math_agent_optimized.wasm

# Deploy WASM agent
caxton deploy math_agent_optimized.wasm \
  --name advanced-math-processor \
  --memory-limit 10MB \
  --cpu-limit 5s
```

### Hybrid Development Approach

Combine configuration and WASM agents for optimal flexibility:

```yaml
---
name: DataScienceWorkflow
version: "1.0.0"
description: "Complete data science workflow combining config and WASM agents"

capabilities:
  - data-ingestion
  - data-processing
  - statistical-modeling
  - visualization

# Configuration agent handles orchestration
tools:
  - http_client
  - database_reader
  - chart_generator

# WASM agents handle heavy computation
wasm_delegates:
  - name: statistical-modeler
    capability: statistical-modeling
    wasm_module: "stats_engine.wasm"

  - name: ml-processor
    capability: machine-learning
    wasm_module: "ml_models.wasm"

workflow:
  - name: ingest_data
    type: config
    tools: ["http_client", "database_reader"]

  - name: statistical_analysis
    type: wasm
    delegate: statistical-modeler

  - name: generate_report
    type: config
    tools: ["chart_generator"]
---

# DataScienceWorkflow Agent

Combines the ease of configuration agents with the power of WebAssembly for computationally intensive tasks.
```

## Deployment Strategies

### Development Deployment

```bash
# Quick development deployment
caxton deploy my-agent.md \
  --env development \
  --hot-reload \
  --debug-mode

# Watch for changes
caxton watch my-agent.md \
  --auto-deploy \
  --test-on-deploy
```

### Staging Deployment

```bash
# Staging with validation
caxton deploy my-agent.md \
  --env staging \
  --validate-schema \
  --check-dependencies \
  --run-tests

# Blue-green deployment
caxton deploy my-agent.md \
  --strategy blue-green \
  --health-check-timeout 30s
```

### Production Deployment

```bash
# Production deployment with full validation
caxton deploy my-agent.md \
  --env production \
  --validate-schema \
  --security-scan \
  --performance-test \
  --strategy canary \
  --canary-percentage 10 \
  --rollback-threshold 5%
```

## Monitoring and Observability

### Agent Metrics

Monitor agent performance:

```bash
# Real-time metrics
caxton metrics watch --agent my-agent

# Performance dashboard
caxton dashboard --agent my-agent \
  --metrics "requests,latency,errors,memory"

# Alert configuration
caxton alerts create \
  --agent my-agent \
  --metric error_rate \
  --threshold 5% \
  --action email
```

### Memory Monitoring

Track agent memory usage:

```bash
# Memory statistics
caxton memory stats --agent my-agent

# Memory growth tracking
caxton memory monitor --agent my-agent \
  --alert-growth 10MB/hour

# Memory optimization suggestions
caxton memory analyze --agent my-agent \
  --suggest-optimizations
```

### Conversation Analytics

Understand agent interactions:

```bash
# Conversation patterns
caxton conversations analyze \
  --agent my-agent \
  --period 7d

# User satisfaction metrics
caxton feedback analyze \
  --agent my-agent \
  --breakdown capability

# Performance by capability
caxton performance breakdown \
  --agent my-agent \
  --group-by capability
```

## Migration Guide: WASM to Configuration

Convert existing WASM agents to configuration agents for easier maintenance:

### 1. Identify Orchestration Logic

Extract business logic from implementation details:

```rust
// WASM agent (before)
impl Agent for DataProcessor {
    fn handle_message(&mut self, msg: Message) -> Result<Response> {
        // Parse request
        let request: DataRequest = serde_json::from_str(&msg.content)?;

        // Fetch data (can be moved to MCP tool)
        let data = fetch_from_url(&request.url)?;

        // Parse CSV (can be moved to MCP tool)
        let parsed = parse_csv(&data)?;

        // Business logic (stays in agent)
        let analysis = analyze_trends(&parsed);

        // Generate response
        Ok(Response::new(serde_json::to_string(&analysis)?))
    }
}
```

### 2. Create Configuration Agent

Convert the business logic to prompts:

```yaml
---
name: DataProcessor
version: "2.0.0"
description: "Migrated from WASM to configuration agent"

capabilities:
  - data-analysis

tools:
  - http_client    # Replaces fetch_from_url
  - csv_parser     # Replaces parse_csv

system_prompt: |
  You are a data processor that analyzes trends in CSV data.

  Process:
  1. Use http_client to fetch data from provided URLs
  2. Use csv_parser to parse the CSV data
  3. Analyze the data for trends, patterns, and insights
  4. Provide clear, actionable analysis results

  Focus on trend analysis including:
  - Growth patterns
  - Seasonal variations
  - Anomaly detection
  - Statistical summaries
---
```

### 3. Migration Checklist

- [ ] Identify which logic can be moved to MCP tools
- [ ] Convert algorithms to prompt-based reasoning
- [ ] Test equivalent functionality
- [ ] Performance comparison
- [ ] Update deployment scripts
- [ ] Monitor post-migration metrics

## Best Practices Summary

1. **Start with Configuration Agents**: Use the primary development
   experience for 90% of use cases
2. **Use WASM for Algorithms**: Reserve WebAssembly for custom computation or
   strict isolation
3. **Design for Observability**: Include logging, metrics, and debugging from
   the start
4. **Implement Gradual Rollout**: Use canary deployments for production
   changes
5. **Plan for Scale**: Design memory and tool usage with growth in mind
6. **Security First**: Follow the principle of least privilege for tools and capabilities
7. **Test Thoroughly**: Use automated testing for both functionality and performance
8. **Monitor Continuously**: Track agent behavior and user satisfaction metrics

## Next Steps

- [API Reference](api-reference.md) - Complete API documentation
- [Security Guide](security-guide.md) - Security best practices
- [Message Protocols](message-protocols.md) - FIPA protocol details
- [Testing Guide](testing.md) - Testing strategies and tools
- [WebAssembly Integration](wasm-integration.md) - Advanced WASM development
