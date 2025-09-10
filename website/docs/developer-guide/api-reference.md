---
title: "API Reference"
layout: documentation
description: "Complete API documentation for Caxton management and control"
date: 2025-09-10
categories: [Website]
---

Complete API documentation for Caxton management and control.

## Quick Start

The three most common operations with configuration agents (5-10 minute setup):

### 1. Deploy a Configuration Agent

```bash
# Set your environment
export CAXTON_TOKEN="your-token-here"
export CAXTON_API="http://localhost:8080/api/v1"

# Deploy configuration agent (primary experience)
curl -X POST "$CAXTON_API/agents/config" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "data-analyzer",
    "config": "---\nname: DataAnalyzer\nversion: \"1.0.0\"\ncapabilities:\n  - data-analysis\ntools:\n  - http_client\n  - csv_parser\nsystem_prompt: |\n  You are a data analysis expert.\n---\n\n# DataAnalyzer Agent\n\nSpecializes in data analysis tasks.",
    "memory_enabled": true
  }'

# Deploy WebAssembly agent (advanced use case)
curl -X POST "$CAXTON_API/agents/wasm" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: multipart/form-data" \
  -F "wasm=@agent.wasm" \
  -F 'config={"name":"my-wasm-agent","resources":{"memory":"50MB"}}'
```

### 2. Send a Message to Capability

```bash
# Send message via capability-based routing (recommended)
curl -X POST "$CAXTON_API/messages" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "performative": "request",
    "sender": "client_001",
    "capability": "data-analysis",
    "content": {"action": "analyze", "data": {"dataset": "sales_q3.csv"}}
  }'

# Send message to specific agent (still supported)
curl -X POST "$CAXTON_API/messages" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "performative": "request",
    "sender": "client_001",
    "receiver": "agent_123",
    "content": {"action": "process", "data": {"key": "value"}}
  }'
```

### 3. List All Agents

```bash
# Get all agents (configuration and WebAssembly)
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/agents"

# Filter by type
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/agents?type=configuration&status=running"
```

## Environment Setup

Set these environment variables for all examples:

```bash
# Required - Your authentication token
export CAXTON_TOKEN="your-token-here"

# API endpoints
export CAXTON_API="http://localhost:8080/api/v1"
export CAXTON_WS="ws://localhost:8080/ws"

# Optional - Default values
export CAXTON_TIMEOUT="30"
export CAXTON_RETRY_COUNT="3"
```

## Error Codes

All APIs return consistent error responses with these codes:

| Code | HTTP Status | Description | |------|-------------|-------------| |
`AGENT_NOT_FOUND` | 404 | Agent does not exist | | `AGENT_ALREADY_EXISTS` | 409
| Agent name already in use | | `INVALID_WASM` | 400 | Invalid WebAssembly
module | | `RESOURCE_LIMIT_EXCEEDED` | 429 | Resource limits exceeded | |
`DEPLOYMENT_FAILED` | 500 | Deployment operation failed | |
`MESSAGE_DELIVERY_FAILED` | 500 | Message could not be delivered | |
`UNAUTHORIZED` | 401 | Authentication required | | `FORBIDDEN` | 403 | Operation
not permitted | | `RATE_LIMITED` | 429 | Rate limit exceeded | |
`INTERNAL_ERROR` | 500 | Internal server error |

**Error Response Format:**

```json
{
  "error": {
    "code": "AGENT_NOT_FOUND",
    "message": "Agent with ID 'agent_999' not found",
    "trace_id": "trace_abc123"
  }
}
```

## API Overview

Caxton provides a REST/HTTP API for all management operations, ensuring
simplicity and compatibility with standard HTTP tooling.

### Base URLs

- **REST API**: `http://localhost:8080/api/v1`
- **WebSocket**: `ws://localhost:8080/ws`

### Authentication

Include authentication token in requests:

```bash
curl -H "X-Caxton-Token: $CAXTON_TOKEN" "$CAXTON_API/agents"
```

## Agent Management API

Caxton supports two agent types:

- **Configuration agents**: Primary experience, markdown-based (5-10 minute deployment)
- **WebAssembly agents**: Advanced use case for custom algorithms

### Deploy Configuration Agent

Deploy a configuration agent defined in markdown with YAML frontmatter.

```bash
# Deploy configuration agent (primary experience)
curl -X POST "$CAXTON_API/agents/config" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "data-analyzer",
    "config": "---\nname: DataAnalyzer\nversion: \"1.0.0\"\ncapabilities:\n  - data-analysis\ntools:\n  - http_client\n  - csv_parser\nsystem_prompt: |\n  You are a data analysis expert.\n---\n\n# DataAnalyzer Agent\n\nSpecializes in data analysis tasks.",
    "memory_enabled": true,
    "memory_scope": "workspace"
  }'

# Response
{
  "agent_id": "agent_123",
  "name": "data-analyzer",
  "type": "configuration",
  "status": "deployed",
  "capabilities": ["data-analysis"],
  "deployed_at": "2024-01-15T10:30:00Z"
}
```

### Deploy WebAssembly Agent

Deploy a WebAssembly agent for advanced use cases.

```bash
# Deploy WASM agent (advanced use case)
curl -X POST "$CAXTON_API/agents/wasm" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: multipart/form-data" \
  -F "wasm=@agent.wasm" \
  -F 'config={"name":"my-wasm-agent","resources":{"memory":"50MB"}}'

# Response
{
  "agent_id": "agent_456",
  "name": "my-wasm-agent",
  "type": "webassembly",
  "status": "running",
  "deployed_at": "2024-01-15T10:30:00Z"
}
```

### List Agents

Get a list of all deployed agents.

```bash
# List all agents
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/agents"

# List running agents with pagination
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/agents?status=running&limit=10&offset=0"

# Response (Mixed agent types)
{
  "agents": [
    {
      "agent_id": "agent_123",
      "name": "data-analyzer",
      "type": "configuration",
      "status": "running",
      "capabilities": ["data-analysis"],
      "memory_enabled": true
    },
    {
      "agent_id": "agent_456",
      "name": "my-wasm-agent",
      "type": "webassembly",
      "status": "running",
      "memory_used": "25MB",
      "cpu_usage": 0.15
    }
  ],
  "total": 42
}
```

### Get Agent Details

Retrieve detailed information about a specific agent.

```bash
# Get agent details
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/agents/agent_123"

# Response
{
  "agent_id": "agent_123",
  "name": "my-agent",
  "status": "running",
  "deployed_at": "2024-01-15T10:30:00Z",
  "resources": {
    "memory": "25MB/50MB",
    "cpu": "50m/100m"
  },
  "metrics": {
    "messages_processed": 1542,
    "messages_failed": 3,
    "uptime_seconds": 3600
  }
}
```

### Update Agent

Update an agent's configuration or code.

```bash
# Update agent configuration
curl -X PUT "$CAXTON_API/agents/agent_123" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "config": {
      "resources": {"memory": "100MB"}
    },
    "strategy": "blue_green"
  }'

# Response
{
  "agent_id": "agent_123",
  "status": "updating",
  "deployment_id": "deploy_456"
}
```

### Stop Agent

Stop a running agent gracefully.

```bash
# Stop agent with grace period
curl -X POST "$CAXTON_API/agents/agent_123/stop" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "grace_period_seconds": 30,
    "drain_messages": true
  }'

# Stop immediately
curl -X POST "$CAXTON_API/agents/agent_123/stop" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -d '{}'

# Response
{
  "agent_id": "agent_123",
  "status": "stopping"
}
```

### Remove Agent

Remove an agent from the system.

```bash
# Remove agent
curl -X DELETE "$CAXTON_API/agents/agent_123" \
  -H "X-Caxton-Token: $CAXTON_TOKEN"

# Force remove (skip graceful shutdown)
curl -X DELETE "$CAXTON_API/agents/agent_123?force=true" \
  -H "X-Caxton-Token: $CAXTON_TOKEN"

# Response
{
  "agent_id": "agent_123",
  "status": "removed"
}
```

## Message API

### Send Message

Send a FIPA message using capability-based routing or direct agent addressing.

```bash
# Send message via capability-based routing (recommended)
curl -X POST "$CAXTON_API/messages" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "performative": "request",
    "sender": "client_001",
    "capability": "data-analysis",
    "content": {"action": "analyze", "data": {"dataset": "sales_q3.csv"}}
  }'

# Send message to specific agent (still supported)
curl -X POST "$CAXTON_API/messages" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "performative": "request",
    "sender": "client_001",
    "receiver": "agent_123",
    "content": {"action": "process", "data": {"key": "value"}}
  }'

# Send with conversation tracking
curl -X POST "$CAXTON_API/messages" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "performative": "request",
    "receiver": "agent_123",
    "content": {"action": "process"},
    "conversation_id": "conv_789"
  }'

# Response
{
  "message_id": "msg_abc123",
  "status": "delivered"
}
```

### Subscribe to Messages

Subscribe to message streams via WebSocket.

```javascript
// Connect to WebSocket
const ws = new WebSocket(process.env.CAXTON_WS);

// Subscribe to agent messages
ws.send(JSON.stringify({
  type: 'subscribe',
  agents: ['agent_123'],
  events: ['message.*']
}));

// Receive messages
ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Message received:', message.content);
};
```

### Query Message History

Retrieve historical messages.

```bash
# Get conversation messages
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/messages?conversation_id=conv_789"

# Get agent messages with pagination
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/messages?sender=agent_123&limit=50&offset=0"

# Get recent messages
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/messages?since=2024-01-15T10:00:00Z"

# Response
{
  "messages": [
    {
      "message_id": "msg_001",
      "performative": "request",
      "sender": "agent_123",
      "receiver": "agent_456",
      "content": {"action": "process"},
      "timestamp": "2024-01-15T10:30:00Z"
    }
  ],
  "total": 150
}
```

## Memory API

Configuration agents with embedded memory system support.

### Search Agent Memory

Semantically search agent memory for relevant context.

```bash
# Search agent memory
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/agents/agent_123/memory/search?query=customer%20patterns&limit=10"

# Search with filters
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/agents/agent_123/memory/search?query=sales&entity_type=insight&limit=5"

# Response
{
  "results": [
    {
      "entity_name": "Q3SalesInsight",
      "entity_type": "business_insight",
      "similarity": 0.92,
      "observations": [
        "Q3 sales increased 23% over Q2",
        "Mobile channel drove 67% of growth"
      ],
      "created_at": "2024-01-15T10:30:00Z"
    }
  ],
  "total": 25,
  "query_time_ms": 15
}
```

### Add to Agent Memory

Store new knowledge in agent memory.

```bash
# Add memory entity
curl -X POST "$CAXTON_API/agents/agent_123/memory/entities" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "entities": [
      {
        "name": "Q4Forecast",
        "entityType": "business_forecast",
        "observations": [
          "Projected 15% growth in Q4",
          "Holiday season expected to boost sales",
          "New product launch planned for December"
        ]
      }
    ]
  }'

# Response
{
  "entities_created": 1,
  "entity_ids": ["ent_abc123"]
}
```

### Get Memory Statistics

Get agent memory usage statistics.

```bash
# Memory stats
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/agents/agent_123/memory/stats"

# Response
{
  "total_entities": 1247,
  "total_relations": 3891,
  "memory_scope": "workspace",
  "storage_used_mb": 12.5,
  "last_updated": "2024-01-15T10:30:00Z",
  "embedding_model": "all-MiniLM-L6-v2"
}
```

## Task API

### Create Task

Create a task for distribution among agents.

```bash
# Create simple task
curl -X POST "$CAXTON_API/tasks" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "process_data",
    "description": "Process customer data batch",
    "data": {"batch_id": "batch_001"},
    "timeout_seconds": 300
  }'

# Create task with specific agents
curl -X POST "$CAXTON_API/tasks" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "process_data",
    "data": {"batch_id": "batch_001"},
    "participants": ["agent_1", "agent_2"],
    "protocol": "contract_net"
  }'

# Response
{
  "task_id": "task_999",
  "status": "created"
}
```

### Distribute Task

Distribute a task using Contract Net Protocol.

```bash
# Distribute task to available agents
curl -X POST "$CAXTON_API/tasks/task_999/distribute" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "strategy": "best_bid",
    "max_wait_seconds": 30
  }'

# Distribute with immediate assignment
curl -X POST "$CAXTON_API/tasks/task_999/distribute" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -d '{}'

# Response
{
  "task_id": "task_999",
  "status": "distributed",
  "assigned_to": "agent_456",
  "proposals_received": 3
}
```

### Get Task Status

Check the status of a task.

```bash
# Get task status
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/tasks/task_999"

# List all tasks
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/tasks"

# List tasks by status
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/tasks?status=in_progress"

# Response
{
  "task_id": "task_999",
  "status": "in_progress",
  "assigned_to": "agent_456",
  "progress": 0.65,
  "started_at": "2024-01-15T10:31:00Z"
}
```

## Deployment API

### Create Deployment

Create a new deployment with specific strategy.

```bash
# Simple deployment
curl -X POST "$CAXTON_API/deployments" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: multipart/form-data" \
  -F "agent_id=agent_123" \
  -F "wasm=@agent-v2.wasm" \
  -F "version=2.0.0"

# Canary deployment
curl -X POST "$CAXTON_API/deployments" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "agent_123",
    "version": "2.0.0",
    "strategy": {
      "type": "canary",
      "stages": [10, 50, 100]
    }
  }'

# Response
{
  "deployment_id": "deploy_888",
  "status": "initializing"
}
```

### Monitor Deployment

Monitor deployment progress.

```bash
# Check deployment status
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/deployments/deploy_888"

# List all deployments
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/deployments"

# List active deployments
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/deployments?status=in_progress"

# Response
{
  "deployment_id": "deploy_888",
  "status": "in_progress",
  "current_stage": 2,
  "current_percentage": 50,
  "success_rate": 0.98,
  "error_count": 5
}
```

### Rollback Deployment

Rollback a deployment to previous version.

```bash
# Rollback deployment
curl -X POST "$CAXTON_API/deployments/deploy_888/rollback" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"reason": "High error rate detected"}'

# Emergency rollback
curl -X POST "$CAXTON_API/deployments/deploy_888/rollback" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -d '{}'

# Response
{
  "deployment_id": "deploy_888",
  "status": "rolling_back",
  "rollback_to_version": "1.0.0"
}
```

## Metrics API

### Get System Metrics

Retrieve system-wide metrics.

```bash
# Current system metrics
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/metrics/system"

# Historical metrics
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/metrics/system?period=1h"

# Response
{
  "timestamp": "2024-01-15T10:30:00Z",
  "agents": {
    "total": 42,
    "running": 40,
    "failed": 2
  },
  "messages": {
    "rate_per_second": 1500,
    "queue_size": 250
  },
  "resources": {
    "cpu_usage_percent": 45.2,
    "memory_used_mb": 2048
  }
}
```

### Get Agent Metrics

Get metrics for a specific agent.

```bash
# Current agent metrics
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/metrics/agents/agent_123"

# Historical metrics (1 hour)
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/metrics/agents/agent_123?period=1h"

# Performance metrics only
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/metrics/agents/agent_123?type=performance"

# Response
{
  "agent_id": "agent_123",
  "messages_processed": 5432,
  "messages_failed": 12,
  "average_latency_ms": 15.3,
  "cpu_usage": 0.25,
  "memory_usage_mb": 32
}
```

## WebSocket Events

Real-time event streaming via WebSocket.

### Connect and Subscribe

```bash
# Test WebSocket connection
wscat -c "$CAXTON_WS" -H "X-Caxton-Token: $CAXTON_TOKEN"
```

```javascript
// Connect and subscribe to agent events
const ws = new WebSocket(process.env.CAXTON_WS);

ws.onopen = () => {
  // Subscribe to all events for specific agents
  ws.send(JSON.stringify({
    type: 'subscribe',
    agents: ['agent_123', 'agent_456'],
    events: ['*']
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log(`${data.type}: ${data.message}`);
};
```

### Common Event Types

```javascript
// Agent events
{"type": "agent.deployed", "agent_id": "agent_123"}
{"type": "agent.failed", "agent_id": "agent_123", "error": "timeout"}

// Message events
{"type": "message.sent", "from": "agent_123", "to": "agent_456"}

// Task events
{"type": "task.completed", "task_id": "task_999", "result": "success"}

// System alerts
{"type": "system.alert", "message": "High memory usage"}
```

## Quick Examples

### Complete Agent Workflow (Configuration Agent)

```bash
# 1. Deploy configuration agent (5-10 minute setup)
AGENT_ID=$(curl -s -X POST "$CAXTON_API/agents/config" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "data-processor",
    "config": "---\nname: DataProcessor\nversion: \"1.0.0\"\ncapabilities:\n  - data-analysis\ntools:\n  - csv_parser\nsystem_prompt: |\n  You process data efficiently.\n---\n\n# DataProcessor Agent",
    "memory_enabled": true
  }' | jq -r '.agent_id')

# 2. Send work request via capability routing
MSG_ID=$(curl -s -X POST "$CAXTON_API/messages" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "performative": "request",
    "capability": "data-analysis",
    "content": {"action": "analyze", "data": "sample_data.csv"}
  }' | jq -r '.message_id')

# 3. Check agent status
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/agents/$AGENT_ID"

# 4. Check memory usage
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/agents/$AGENT_ID/memory/stats"

# 5. Get metrics
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/metrics/agents/$AGENT_ID"
```

### Complete Agent Workflow (WebAssembly Agent)

```bash
# 1. Deploy WASM agent (for advanced use cases)
AGENT_ID=$(curl -s -X POST "$CAXTON_API/agents/wasm" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -F "wasm=@agent.wasm" \
  -F 'config={"name":"math-worker","resources":{"memory":"50MB"}}' \
  | jq -r '.agent_id')

# 2. Send work request
MSG_ID=$(curl -s -X POST "$CAXTON_API/messages" \
  -H "X-Caxton-Token: $CAXTON_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"performative\": \"request\",
    \"receiver\": \"$AGENT_ID\",
    \"content\": {\"operation\": \"fibonacci\", \"n\": 10}
  }" | jq -r '.message_id')

# 3. Check agent status
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/agents/$AGENT_ID"

# 4. Get metrics
curl -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/metrics/agents/$AGENT_ID"
```

### Batch Operations

```bash
# Deploy multiple configuration agents
for i in {1..3}; do
  curl -X POST "$CAXTON_API/agents/config" \
    -H "X-Caxton-Token: $CAXTON_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
      \"name\": \"worker-$i\",
      \"config\": \"---\\nname: Worker$i\\nversion: \\\"1.0.0\\\"\\ncapabilities:\\n  - task-processing\\ntools:\\n  - http_client\\nsystem_prompt: |\\n  You are worker $i.\\n---\\n\\n# Worker Agent $i\",
      \"memory_enabled\": true
    }"
done

# Send messages via capability routing (load balanced)
for i in {1..10}; do
  curl -X POST "$CAXTON_API/messages" \
    -H "X-Caxton-Token: $CAXTON_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
      \"performative\": \"request\",
      \"capability\": \"task-processing\",
      \"content\": {\"task_id\": \"task_$i\", \"data\": \"sample_$i\"}
    }"
done

# Send messages to specific agents
for agent in $(curl -s -H "X-Caxton-Token: $CAXTON_TOKEN" \
  "$CAXTON_API/agents?type=configuration" | jq -r '.agents[].agent_id'); do
  curl -X POST "$CAXTON_API/messages" \
    -H "X-Caxton-Token: $CAXTON_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"performative\":\"request\",\"receiver\":\"$agent\",\"content\":{\"ping\":\"hello\"}}"
done
```

## Rate Limits & Headers

**Rate Limits:**

- Agent operations: 10/min
- Message sending: 1000/sec
- Metrics queries: 100/min

**Check Rate Limits:**

```bash
# Headers show remaining quota
curl -I -H "X-Caxton-Token: $CAXTON_TOKEN" "$CAXTON_API/agents"
# Returns:
# X-RateLimit-Limit: 1000
# X-RateLimit-Remaining: 950
```

## Next Steps

- \[Message Protocols\]({{ '/docs/developer-guide/message-protocols/' |
  relative_url }})
  - FIPA protocol details
- \[WebAssembly Integration\]({{ '/docs/developer-guide/wasm-integration/' |
  relative_url }})
  - WASM specifics
- \[Testing Guide\]({{ '/docs/developer-guide/testing/' | relative_url }}) -
  Testing strategies
