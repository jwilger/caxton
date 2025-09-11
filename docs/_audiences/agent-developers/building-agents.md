---
title: "Building Agents Guide"
date: 2025-01-15
layout: page
categories: [Agent Developers, Building]
difficulty: beginner
audience: agent-developers
---

**Implementation Status**: Configuration agents are the primary and recommended
development experience for Caxton 1.0.

Complete guide for building configuration agents on the Caxton platform,
providing rapid deployment with a 5-10 minute onboarding experience.

## Quick Start: Configuration Agents - **Beginner**

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
  - data_analysis
  - csv_processing

tools:
  - csv_reader
  - chart_generator

llm:
  provider: openai
  model: gpt-4
  temperature: 0.1
---

# Data Analysis Agent

You are an expert data analyst. Analyze provided data and create meaningful
insights.

## Instructions

1. When given CSV data, first examine the structure
2. Identify key patterns and trends
3. Generate visualizations when appropriate
4. Provide actionable recommendations

## Example Workflows

### CSV Analysis
- Load CSV file using csv_reader tool
- Examine columns and data types
- Calculate summary statistics
- Create charts with chart_generator
- Present findings in markdown format
```

Save this as `agents/quick-analyzer.md` and deploy:

```bash
curl -X POST http://localhost:3000/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "type": "configuration",
    "definition": "'$(cat agents/quick-analyzer.md | sed 's/"/\\"/g' | tr '\n' '\r' | sed 's/\r/\\n/g')'"
  }'
```

## Configuration Agent Structure - **Beginner**

### YAML Frontmatter

The YAML frontmatter defines agent metadata and capabilities:

```yaml
---
name: AgentName                    # Required: Unique identifier
version: "1.0.0"                   # Required: Semantic version
description: "Agent description"   # Required: Brief description

capabilities:                      # Required: List of capabilities
  - capability_name

tools:                            # Optional: Available tools
  - tool_name

llm:                              # Required: LLM configuration
  provider: openai|anthropic|local
  model: model_name
  temperature: 0.1
  max_tokens: 2000

permissions:                      # Optional: Resource permissions
  file_access: readonly|readwrite|none
  network_access: restricted|full|none

schedule:                         # Optional: Automated execution
  cron: "0 9 * * MON-FRI"
  timezone: "UTC"
---
```

### Markdown Instructions

The markdown content provides detailed instructions for the agent:

```markdown
# Agent Role and Purpose

Define the agent's primary role and objectives.

## Instructions

Detailed step-by-step instructions for the agent.

## Example Workflows

Specific examples of how the agent should handle different scenarios.
```

## Agent Development Workflow - **Intermediate**

### 1. Design Phase

Before writing code, plan your agent:

1. **Define the problem**: What specific task will the agent solve?
2. **Identify capabilities**: What tools and skills does it need?
3. **Map the workflow**: How should it process inputs and generate outputs?
4. **Design configuration**: Define YAML schema and capabilities

### 2. Configuration Agent Development

For most use cases, start with configuration agents:

```bash
# 1. Create agent directory
mkdir -p agents/my-agent

# 2. Write configuration
vim agents/my-agent/agent.md

# 3. Validate configuration
curl -X POST http://localhost:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{"definition": "'$(cat agents/my-agent/agent.md)'"}'

# 4. Deploy agent
curl -X POST http://localhost:3000/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "type": "configuration",
    "definition": "'$(cat agents/my-agent/agent.md)'"
  }'

# 5. Test agent
curl -X POST http://localhost:3000/api/agents/my-agent/messages \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Test message",
    "conversation_id": "test-conv"
  }'
```

### 3. Iterative Development

Use hot reload for rapid iteration:

```bash
# Edit configuration
vim agents/my-agent/agent.md

# Reload without redeployment
curl -X POST http://localhost:3000/api/agents/my-agent/reload
```

## Testing Strategies - **Intermediate**

### Unit Testing Configuration

Test agent responses without deployment:

```bash
# Create test cases
cat > test-cases.json << EOF
{
  "cases": [
    {
      "input": "Analyze sales data",
      "expected_capabilities": ["data_analysis"],
      "expected_tools": ["csv_reader"]
    }
  ]
}
EOF

# Run tests
curl -X POST http://localhost:3000/api/agents/my-agent/test \
  -H "Content-Type: application/json" \
  -d @test-cases.json
```

### Integration Testing

Test complete workflows:

```python
import requests
import json

def test_agent_workflow():
    # Deploy agent
    with open('agents/test-agent.md', 'r') as f:
        definition = f.read()

    response = requests.post(
        'http://localhost:3000/api/agents',
        json={'type': 'configuration', 'definition': definition}
    )
    agent_id = response.json()['id']

    # Send test message
    response = requests.post(
        f'http://localhost:3000/api/agents/{agent_id}/messages',
        json={
            'content': 'Process this data: 1,2,3,4,5',
            'conversation_id': 'test-conv'
        }
    )

    # Verify response
    assert response.status_code == 200
    result = response.json()
    assert 'processed' in result['content'].lower()

    # Cleanup
    requests.delete(f'http://localhost:3000/api/agents/{agent_id}')

if __name__ == '__main__':
    test_agent_workflow()
```

## Performance Optimization - **Advanced**

### Configuration Agent Optimization

1. **Minimize tool usage**: Only include necessary tools
2. **Optimize prompts**: Clear, concise instructions
3. **Set appropriate limits**: Temperature, max_tokens
4. **Use caching**: For repeated operations

```yaml
---
name: OptimizedAgent
# ... other config ...

llm:
  provider: openai
  model: gpt-4-turbo  # Faster than gpt-4
  temperature: 0      # Deterministic responses
  max_tokens: 1000    # Limit response length

cache:
  enabled: true
  ttl: 3600           # 1 hour cache
---
```

### WASM Agent Optimization

1. **Minimize memory allocation**: Use stack allocation when possible
2. **Optimize algorithms**: Choose efficient data structures
3. **Profile performance**: Use Caxton's built-in profiling

```rust
#[caxton_agent]
pub struct OptimizedAgent {
    // Pre-allocate buffers
    buffer: Vec<u8>,
    cache: HashMap<String, ProcessedData>,
}

impl Agent for OptimizedAgent {
    fn handle_message(&mut self, message: Message) -> Result<Response> {
        // Check cache first
        if let Some(cached) = self.cache.get(&message.id) {
            return Ok(Response::from_cached(cached));
        }

        // Process efficiently
        self.buffer.clear();
        let result = self.process_streaming(&message.content, &mut self.buffer)?;

        // Cache result
        self.cache.insert(message.id, result.clone());

        Ok(Response::new().with_content(result))
    }
}
```

## Error Handling and Debugging - **Intermediate**

### Common Configuration Errors

1. **Invalid YAML**: Use YAML validators
2. **Unknown capabilities**: Check available capabilities
3. **Tool permission errors**: Verify tool access

```bash
# Validate before deploy
curl -X POST http://localhost:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{"definition": "'$(cat agent.md)'"}'
```

### Debug Configuration Agents

```bash
# Check agent logs
curl http://localhost:3000/api/agents/my-agent/logs

# Monitor performance
curl http://localhost:3000/api/agents/my-agent/metrics

# Test specific capabilities
curl -X POST http://localhost:3000/api/agents/my-agent/test \
  -H "Content-Type: application/json" \
  -d '{"capability": "data_analysis", "input": "test data"}'
```

### Debug WASM Agents

```rust
use caxton_sdk::debug::*;

impl AdvancedAgent {
    fn debug_process(&self, input: &str) -> Result<String> {
        debug_log!("Processing input: {}", input);

        let start = debug_timer_start!("processing");
        let result = self.complex_algorithm(input)?;
        debug_timer_end!(start, "processing completed");

        debug_memory_stats!();

        Ok(result)
    }
}
```

## Deployment Strategies - **Advanced**

### Blue-Green Deployment

Deploy new versions without downtime:

```bash
# Deploy new version as blue
curl -X POST http://localhost:3000/api/agents/my-agent/deploy \
  -H "Content-Type: application/json" \
  -d '{
    "strategy": "blue-green",
    "definition": "'$(cat agent-v2.md)'"
  }'

# Test blue version
curl -X POST http://localhost:3000/api/agents/my-agent-blue/test

# Switch traffic to blue
curl -X POST http://localhost:3000/api/agents/my-agent/promote \
  -H "Content-Type: application/json" \
  -d '{"version": "blue"}'
```

### Canary Deployment

Gradually shift traffic to new version:

```bash
curl -X POST http://localhost:3000/api/agents/my-agent/deploy \
  -H "Content-Type: application/json" \
  -d '{
    "strategy": "canary",
    "traffic_percentage": 10,
    "definition": "'$(cat agent-v2.md)'"
  }'
```

## Related Documentation

- [Configuration Agent Format](config-agents/agent-format.md) - **Beginner**
- [Configuration Best Practices](config-agents/best-practices.md) -
  **Intermediate**
- [LLM Provider Configuration](config-agents/llm-providers.md) - **Beginner**
- [Security Guide](security.md) - **Intermediate**
- [API Reference](api-reference.md) - **Intermediate**
- [Performance Specifications](../../api/performance-specifications.md) -
  **Advanced**

## Quick Reference

### Essential Commands

```bash
# Validate configuration
curl -X POST http://localhost:3000/api/validate -d '{"definition": "..."}'

# Deploy configuration agent
curl -X POST http://localhost:3000/api/agents -d '{"type": "configuration", "definition": "..."}'

# Send message to agent
curl -X POST http://localhost:3000/api/agents/{id}/messages -d '{"content": "..."}'

# Get agent status
curl http://localhost:3000/api/agents/{id}

# Hot reload agent
curl -X POST http://localhost:3000/api/agents/{id}/reload

# Remove agent
curl -X DELETE http://localhost:3000/api/agents/{id}
```

### Configuration Template

```yaml
---
name: MyAgent
version: "1.0.0"
description: "Brief description"
capabilities: [capability1, capability2]
tools: [tool1, tool2]
llm:
  provider: openai
  model: gpt-4
  temperature: 0.1
---

# Agent Instructions

Your role and objectives here.

## Instructions

Step-by-step guidance.
```
