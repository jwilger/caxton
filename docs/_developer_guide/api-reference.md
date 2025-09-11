---
title: "API Reference"
date: 2025-01-14
layout: page
categories: [Developer Guide]
---

**Implementation Status**: See
[Implementation Status](../api/implementation-status.md) for details on what's
currently available.

Complete API documentation for Caxton management and control.

> **Note**: This reference includes both implemented endpoints (marked with ✅)
> and planned endpoints (marked with ⏳). Only endpoints marked with ✅ are
> currently functional in Story 006 implementation.

## API Overview

Caxton provides a REST/HTTP API for all management operations, ensuring
simplicity and compatibility with standard HTTP tooling.

### Agent Types

Caxton uses configuration agents defined in TOML configuration files
for rapid agent deployment and iteration.

### Base URLs

- **REST API**: `http://localhost:8080/api/v1`
- **WebSocket**: `ws://localhost:8080/ws` ⏳ **PLANNED**

### Authentication ⏳ **PLANNED**

Include authentication token in requests:

```bash
curl -H "X-Caxton-Token: your-token" http://localhost:8080/api/v1/agents
```

## System API

### Health Check ✅ **IMPLEMENTED**

Check if the server is running and responsive.

#### REST

```bash
GET /api/v1/health

# Request
curl http://localhost:8080/api/v1/health

# Response
{
  "status": "healthy"
}
```

## Agent Management API

### Deploy Configuration Agent ✅ **IMPLEMENTED**

Deploy a new configuration-driven agent defined in TOML format.

> **Primary Experience**: Configuration agents provide the fastest path to agent
> deployment with a 5-10 minute onboarding experience.

#### REST

```bash
POST /api/v1/agents/config

# Request
curl -X POST http://localhost:8080/api/v1/agents/config \
  -H "Content-Type: application/json" \
  -d '{
    "name": "data-analyzer",
    "config": "name = \"DataAnalyzer\"\nversion = \"1.0.0\"\ncapabilities = [\"data-analysis\", \"report-generation\"]\ntools = [\"http_client\", \"csv_parser\"]\n\nsystem_prompt = '''\nYou are a data analysis expert.\n'''\n\ndocumentation = '''\n# DataAnalyzer Agent\n\nSpecializes in data analysis tasks.\n'''",
    "memory_enabled": true,
    "memory_scope": "workspace"
  }'

# Response
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "data-analyzer",
  "type": "configuration",
  "status": "deployed",
  "capabilities": ["data-analysis", "report-generation"],
  "tools": ["http_client", "csv_parser"],
  "memory_enabled": true,
  "deployed_at": "2024-01-15T10:30:00Z"
}
```

#### Deployment Strategies

Available deployment strategies:

- **`immediate`**: Replace agent instantly (brief interruption)
- **`rolling`**: Gradual replacement with configurable batch size
- **`blue_green`**: Deploy to parallel environment, switch traffic
- **`canary`**: Deploy to subset, gradually increase traffic

### Hot Reload Agent ⏳ **PLANNED**

Update an existing agent without downtime:

```bash
PUT /api/v1/agents/{agent_id}/reload

# Request
curl -X PUT http://localhost:8080/api/v1/agents/agent_123/reload \
  -H "Content-Type: multipart/form-data" \
  -F "config=@agent-v2.toml" \
  -F "config={\"strategy\":\"graceful\",\"traffic_split\":10}"

# Response
{
  "hot_reload_id": "reload_789",
  "status": "in_progress",
  "from_version": "1.0.0",
  "to_version": "2.0.0",
  "strategy": "graceful",
  "started_at": "2024-01-15T10:45:00Z"
}
```

### Agent Lifecycle States

#### Configuration Agent States

- **`deployed`**: Configuration parsed and agent ready
- **`running`**: Actively processing messages via LLM orchestration
- **`suspended`**: Temporarily paused
- **`failed`**: Configuration error or runtime failure

### List Agents ✅ **IMPLEMENTED**

Get a list of all deployed configuration agents.

> **Current Implementation**: Returns all agents as a simple array. Pagination
> and filtering are planned for future releases.

#### REST

```bash
GET /api/v1/agents

# Request (Current Implementation)
curl http://localhost:8080/api/v1/agents

# Response (Current Implementation - Empty)
[]

# Response (Current Implementation - With Agents)
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "data-analyzer",
    "type": "configuration",
    "status": "running",
    "capabilities": ["data-analysis", "report-generation"],
    "memory_enabled": true
  },
  {
    "id": "660e8400-e29b-41d4-a716-446655440001",
    "name": "report-generator",
    "type": "configuration",
    "status": "running",
    "capabilities": ["report-generation", "data-visualization"]
  }
]

# Future Request Format (Planned with filtering)
GET /api/v1/agents?status=running&limit=10&offset=0
```

### Get Agent Details ✅ **IMPLEMENTED**

Retrieve detailed information about a specific agent.

> **Current Implementation**: Returns basic agent information. Metrics and usage
> statistics are planned for future releases.

#### REST

```bash
GET /api/v1/agents/{agent_id}

# Request (Current Implementation)
curl http://localhost:8080/api/v1/agents/550e8400-e29b-41d4-a716-446655440000

# Response (Configuration Agent)
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "data-analyzer",
  "type": "configuration",
  "status": "running",
  "capabilities": ["data-analysis", "report-generation"],
  "tools": ["http_client", "csv_parser"],
  "memory_enabled": true,
  "memory_scope": "workspace",
  "config": "name = \"DataAnalyzer\"\n..."
}

# Response (Current Implementation - Not Found)
{
  "error": "Agent not found",
  "details": {
    "agent_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}

# Future Response Format (Planned with metrics)
{
  "agent_id": "agent_123",
  "name": "my-agent",
  "status": "running",
  "metrics": {
    "messages_processed": 1542,
    "uptime_seconds": 3600
  }
}
```

### Update Configuration Agent ⏳ **PLANNED**

Update a configuration agent's definition.

#### REST

```bash
PUT /api/v1/agents/config/{agent_id}

# Request
{
  "config": "name = \"DataAnalyzer\"\nversion = \"1.1.0\"\n...",
  "memory_enabled": true,
  "strategy": "immediate"
}

# Response
{
  "agent_id": "agent_123",
  "status": "updated",
  "version": "1.1.0"
}
```

### Stop Agent ⏳ **PLANNED**

Stop a running agent gracefully.

#### REST

```bash
POST /api/v1/agents/{agent_id}/stop

# Request
{
  "grace_period_seconds": 30,
  "drain_messages": true
}

# Response
{
  "agent_id": "agent_123",
  "status": "draining",
  "estimated_completion": "2024-01-15T10:31:00Z"
}
```

### Remove Agent ⏳ **PLANNED**

Remove an agent from the system.

#### REST

```bash
DELETE /api/v1/agents/{agent_id}

# Response
{
  "agent_id": "agent_123",
  "status": "removed",
  "removed_at": "2024-01-15T10:35:00Z"
}
```

## Message API ⏳ **PLANNED**

### Send Message to Capability

Send an agent message to agents with specific capabilities (capability-based
routing).

#### REST

```bash
POST /api/v1/messages

# Request (Capability-based routing)
{
  "performative": "request",
  "sender": "client_001",
  "capability": "data-analysis",
  "content": {
    "action": "analyze",
    "data": {"dataset": "sales_q3.csv"}
  },
  "conversation_id": "conv_789",
  "reply_with": "msg_001"
}

# Request (Direct agent addressing - still supported)
{
  "performative": "request",
  "sender": "client_001",
  "receiver": "agent_123",
  "content": {
    "action": "process",
    "data": {"key": "value"}
  },
  "conversation_id": "conv_789",
  "reply_with": "msg_001"
}

# Response
{
  "message_id": "msg_abc123",
  "status": "delivered",
  "delivered_to": ["agent_456"],
  "delivered_at": "2024-01-15T10:30:00.123Z"
}
```

### Subscribe to Messages

Subscribe to message streams via WebSocket.

#### WebSocket

```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:8080/ws');

// Subscribe to agent messages
ws.send(JSON.stringify({
  type: 'subscribe',
  filter: {
    agents: ['agent_123', 'agent_456'],
    performatives: ['inform', 'request']
  }
}));

// Receive messages
ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Received:', message);
};
```

### Query Message History

Retrieve historical messages.

#### REST

```bash
GET /api/v1/messages?conversation_id=conv_789&limit=50

# Response
{
  "messages": [
    {
      "message_id": "msg_001",
      "performative": "request",
      "sender": "agent_123",
      "receiver": "agent_456",
      "content": {...},
      "timestamp": "2024-01-15T10:30:00Z"
    }
  ],
  "total": 150,
  "next_cursor": "cursor_abc"
}
```

## Task API ⏳ **PLANNED**

### Create Task

Create a task for distribution among agents.

#### REST

```bash
POST /api/v1/tasks

# Request
{
  "name": "process_data",
  "description": "Process customer data batch",
  "data": {...},
  "protocol": "contract_net",
  "participants": ["agent_1", "agent_2", "agent_3"],
  "timeout_seconds": 300,
  "requirements": {
    "capabilities": ["data_processing"],
    "min_performance_score": 0.8
  }
}

# Response
{
  "task_id": "task_999",
  "status": "created",
  "created_at": "2024-01-15T10:30:00Z"
}
```

### Distribute Task

Distribute a task using Contract Net Protocol.

#### REST

```bash
POST /api/v1/tasks/{task_id}/distribute

# Request
{
  "strategy": "best_bid",
  "max_wait_seconds": 30
}

# Response
{
  "task_id": "task_999",
  "status": "distributed",
  "assigned_to": "agent_456",
  "proposals_received": 3,
  "winning_bid": {
    "agent_id": "agent_456",
    "estimated_time": 45,
    "confidence": 0.95
  }
}
```

### Get Task Status

Check the status of a task.

#### REST

```bash
GET /api/v1/tasks/{task_id}

# Response
{
  "task_id": "task_999",
  "status": "in_progress",
  "assigned_to": "agent_456",
  "progress": 0.65,
  "started_at": "2024-01-15T10:31:00Z",
  "estimated_completion": "2024-01-15T10:32:00Z",
  "subtasks": [
    {
      "id": "subtask_001",
      "status": "completed"
    },
    {
      "id": "subtask_002",
      "status": "in_progress"
    }
  ]
}
```

## Deployment API ⏳ **PLANNED**

### Create Deployment

Create a new deployment with specific strategy.

#### REST

```bash
POST /api/v1/deployments

# Request
{
  "agent_id": "agent_123",
  "config": "updated_agent_config_toml",
  "version": "2.0.0",
  "strategy": {
    "type": "canary",
    "stages": [
      {"percentage": 10, "duration": "5m"},
      {"percentage": 50, "duration": "10m"},
      {"percentage": 100, "duration": "0"}
    ],
    "rollback_on_error": true,
    "error_threshold": 0.05
  }
}

# Response
{
  "deployment_id": "deploy_888",
  "status": "initializing",
  "created_at": "2024-01-15T10:30:00Z"
}
```

### Monitor Deployment

Monitor deployment progress.

#### REST

```bash
GET /api/v1/deployments/{deployment_id}

# Response
{
  "deployment_id": "deploy_888",
  "status": "in_progress",
  "current_stage": 2,
  "current_percentage": 50,
  "metrics": {
    "success_rate": 0.98,
    "error_count": 5,
    "latency_p99": 125.5
  },
  "stages": [
    {
      "stage": 1,
      "percentage": 10,
      "status": "completed",
      "completed_at": "2024-01-15T10:35:00Z"
    },
    {
      "stage": 2,
      "percentage": 50,
      "status": "in_progress",
      "started_at": "2024-01-15T10:35:00Z"
    }
  ]
}
```

### Rollback Deployment

Rollback a deployment to previous version.

#### REST

```bash
POST /api/v1/deployments/{deployment_id}/rollback

# Request
{
  "reason": "High error rate detected"
}

# Response
{
  "deployment_id": "deploy_888",
  "status": "rolling_back",
  "rollback_to_version": "1.0.0",
  "estimated_completion": "2024-01-15T10:37:00Z"
}
```

## Metrics API ⏳ **PLANNED**

### Get System Metrics

Retrieve system-wide metrics.

#### REST

```bash
GET /api/v1/metrics/system

# Response
{
  "timestamp": "2024-01-15T10:30:00Z",
  "agents": {
    "total": 42,
    "running": 40,
    "failed": 2
  },
  "messages": {
    "total_processed": 1000000,
    "rate_per_second": 1500,
    "queue_size": 250
  },
  "resources": {
    "cpu_usage_percent": 45.2,
    "memory_used_mb": 2048,
    "memory_available_mb": 6144,
    "disk_used_gb": 10.5
  },
  "performance": {
    "message_latency_p50": 10.5,
    "message_latency_p99": 125.3,
    "agent_spawn_time_ms": 85.2
  }
}
```

### Get Agent Metrics

Get metrics for a specific agent.

#### REST

```bash
GET /api/v1/metrics/agents/{agent_id}?period=1h

# Response
{
  "agent_id": "agent_123",
  "period": "1h",
  "metrics": {
    "messages_processed": 5432,
    "messages_failed": 12,
    "average_latency_ms": 15.3,
    "cpu_usage": {
      "average": 0.25,
      "peak": 0.85
    },
    "memory_usage": {
      "average_mb": 32,
      "peak_mb": 48
    },
    "errors": [
      {
        "timestamp": "2024-01-15T10:15:00Z",
        "error": "timeout",
        "count": 3
      }
    ]
  }
}
```

## WebSocket Events

Real-time event streaming via WebSocket.

### Event Types

```javascript
// Agent lifecycle events
{
  "type": "agent.deployed",
  "agent_id": "agent_123",
  "timestamp": "2024-01-15T10:30:00Z"
}

// Message events
{
  "type": "message.sent",
  "message_id": "msg_456",
  "from": "agent_123",
  "to": "agent_456",
  "performative": "inform"
}

// Task events
{
  "type": "task.completed",
  "task_id": "task_999",
  "agent_id": "agent_456",
  "result": "success"
}

// System events
{
  "type": "system.alert",
  "severity": "warning",
  "message": "High memory usage detected",
  "details": {
    "memory_percent": 85
  }
}
```

### Subscription Management

```javascript
// Subscribe to specific event types
ws.send(JSON.stringify({
  type: 'subscribe',
  events: ['agent.*', 'task.completed'],
  filters: {
    agent_ids: ['agent_123']
  }
}));

// Unsubscribe
ws.send(JSON.stringify({
  type: 'unsubscribe',
  subscription_id: 'sub_123'
}));
```

## Error Responses

All APIs use consistent error responses:

### Current Implementation ✅

```json
{
  "error": "Error message",
  "details": {
    "field": "field_name",
    "reason": "validation reason"
  }
}
```

### Implemented Error Codes

| Error Type | HTTP Status | Description | Example |
|------------|-------------|-------------|---------|
| Validation Error | 400 | Invalid input data | Empty name, zero limits |
| Invalid JSON | 400 | Malformed JSON request | Syntax errors |
| Agent Not Found | 404 | Agent ID doesn't exist | Non-existent UUID |

### Future Error Format ⏳ **PLANNED**

```json
{
  "error": {
    "code": "AGENT_NOT_FOUND",
    "message": "Agent with ID 'agent_999' not found",
    "details": {
      "agent_id": "agent_999",
      "searched_at": "2024-01-15T10:30:00Z"
    },
    "trace_id": "trace_abc123"
  }
}
```

### Planned Error Codes ⏳

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `AGENT_ALREADY_EXISTS` | 409 | Agent name already in use |
| `RESOURCE_LIMIT_EXCEEDED` | 429 | Resource limits exceeded |
| `DEPLOYMENT_FAILED` | 500 | Deployment operation failed |
| `MESSAGE_DELIVERY_FAILED` | 500 | Message could not be delivered |
| `UNAUTHORIZED` | 401 | Authentication required |
| `FORBIDDEN` | 403 | Operation not permitted |
| `RATE_LIMITED` | 429 | Rate limit exceeded |
| `INTERNAL_ERROR` | 500 | Internal server error |

## SDK Examples

### JavaScript/TypeScript

```typescript
import { CaxtonClient } from '@caxton/sdk';

const client = new CaxtonClient({
  endpoint: 'http://localhost:8080',
  apiKey: 'your-api-key'
});

// Deploy a configuration agent
const agent = await client.deployConfigAgent({
  config: fs.readFileSync('agent.toml', 'utf8'),
  memoryEnabled: true,
  memoryScope: 'workspace'
});

// Send a message
const response = await client.sendMessage({
  performative: 'request',
  receiver: agent.id,
  content: { action: 'process' }
});

// Subscribe to events
client.on('agent.failed', (event) => {
  console.log('Agent failed:', event);
});
```

### Python

```python
from caxton import CaxtonClient

client = CaxtonClient(
    endpoint='http://localhost:8080',
    api_key='your-api-key'
)

# Deploy a configuration agent
with open('agent.toml', 'r') as f:
    agent = client.deploy_config_agent(
        config=f.read(),
        memory_enabled=True,
        memory_scope='workspace'
    )

# Send and wait for reply
reply = client.send_message_and_wait(
    performative='request',
    receiver=agent.id,
    content={'action': 'process'},
    timeout=30
)

# Query metrics
metrics = client.get_agent_metrics(
    agent_id=agent.id,
    period='1h'
)
```

### Go

```go
package main

import (
    "github.com/caxton/caxton-go"
)

func main() {
    client := caxton.NewClient(
        caxton.WithEndpoint("http://localhost:8080"),
        caxton.WithAPIKey("your-api-key"),
    )

    // Deploy configuration agent
    agent, err := client.DeployConfigAgent(ctx, &caxton.ConfigAgentRequest{
        Config: configTOML,
        MemoryEnabled: true,
        MemoryScope: "workspace",
    })

    // Send message
    resp, err := client.SendMessage(ctx, &caxton.Message{
        Performative: "request",
        Receiver: agent.ID,
        Content: map[string]interface{}{
            "action": "process",
        },
    })
}
```

## Rate Limiting

API rate limits per endpoint:

| Endpoint | Rate Limit | Burst |
|----------|------------|-------|
| Agent deployment | 10/min | 20 |
| Message sending | 1000/sec | 2000 |
| Metrics queries | 100/min | 200 |
| System operations | 50/min | 100 |

Rate limit headers:

```text
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 950
X-RateLimit-Reset: 1642248000
```

## Memory API ⏳ **PLANNED**

Memory operations for configuration agents with embedded memory system.

### Search Memory

Semantically search agent memory for relevant context.

#### REST

```bash
GET /api/v1/agents/{agent_id}/memory/search?query=customer%20data%20patterns&\
    limit=10

# Response
{
  "results": [
    {
      "entity_name": "CustomerAnalysisPattern",
      "entity_type": "analysis_pattern",
      "similarity": 0.92,
      "observations": [
        "Customers with repeat purchases show 85% higher lifetime value",
        "Geographic clustering indicates regional preferences"
      ],
      "created_at": "2024-01-15T10:30:00Z"
    }
  ],
  "total": 25,
  "query_time_ms": 15
}
```

### Add Memory

Store new knowledge in agent memory.

#### REST

```bash
POST /api/v1/agents/{agent_id}/memory/entities

# Request
{
  "entities": [
    {
      "name": "Q3SalesInsight",
      "entityType": "business_insight",
      "observations": [
        "Q3 sales increased 23% over Q2",
        "Mobile channel drove 67% of growth",
        "Customer acquisition cost decreased by 15%"
      ]
    }
  ]
}

# Response
{
  "entities_created": 1,
  "entity_ids": ["ent_abc123"],
  "status": "success"
}
```

### Get Memory Stats

Get statistics about agent memory usage.

#### REST

```bash
GET /api/v1/agents/{agent_id}/memory/stats

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

## Next Steps

- [Building Agents](building-agents.md) - Configuration agent development guide
- [Message Protocols](message-protocols.md) - Agent messaging protocol details
- [Security Guide](security-guide.md) - Configuration agent security model
- [Testing Guide](testing.md) - Testing strategies
