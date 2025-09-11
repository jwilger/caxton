---
title: "REST API Quick Start Guide"
date: 2025-09-10
layout: page
categories: [Getting Started]
---

## Interact with configuration agents via REST API in 5 minutes

This guide provides working examples for Caxton's REST API, focused on
**configuration agents** and **capability-based messaging**. All examples use
real endpoints you can test immediately.

## Prerequisites

1. Start Caxton server:

   ```bash
   caxton server start
   # Server starts on http://localhost:8080
   ```

2. Verify server is running:

```bash
curl http://localhost:8080/api/v1/health
```

Expected response:

```json
{
  "status": "healthy",
  "version": "1.0.0",
  "memory_backend": "embedded",
  "agents_count": 0,
  "uptime": "5s"
}
```

## Configuration Agent API Patterns

### 1. Deploy a Configuration Agent

Create and deploy an agent from a markdown configuration:

```bash
# Create agent configuration file
cat > data-analyzer.json << EOF
{
  "name": "DataAnalyzer",
  "version": "1.0.0",
  "capabilities": ["data-analysis", "report-generation"],
  "tools": ["http_client", "csv_parser"],
  "memory": {
    "enabled": true,
    "scope": "workspace"
  },
  "system_prompt": "You are a data analysis expert who helps users understand their data. When you receive analysis requests, check your memory for similar patterns, use appropriate tools to fetch and parse data, then provide clear insights.",
  "user_prompt_template": "Analyze this request: {{request}}\n\nMemory context: {{memory_context}}\nData source: {{data_source}}"
}
EOF

# Deploy via REST API
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d @data-analyzer.json
```

Expected response:

```json
{
  "agent_id": "DataAnalyzer",
  "status": "deployed",
  "capabilities": ["data-analysis", "report-generation"],
  "memory_enabled": true,
  "tools": ["http_client", "csv_parser"],
  "deployment_time": "2025-09-10T15:30:00Z"
}
```

### 2. List Configuration Agents

```bash
curl http://localhost:8080/api/v1/agents
```

Expected response:

```json
{
  "agents": [
    {
      "name": "DataAnalyzer",
      "type": "configuration",
      "status": "running",
      "capabilities": ["data-analysis", "report-generation"],
      "memory": {
        "enabled": true,
        "scope": "workspace",
        "entities": 0
      },
      "uptime": "2m15s",
      "last_activity": "2025-09-10T15:30:00Z"
    }
  ],
  "total": 1
}
```

### 3. Send Capability-Based Messages

Instead of targeting specific agents, send messages to **capabilities**:

```bash
# Request data analysis capability
curl -X POST http://localhost:8080/api/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "capability": "data-analysis",
    "performative": "request",
    "content": {
      "request": "Analyze Q3 sales trends",
      "data_source": "https://example.com/sales.csv",
      "requirements": "Focus on growth patterns and seasonality"
    },
    "conversation_id": "conv-001"
  }'
```

Expected response:

```json
{
  "message_id": "msg-001",
  "conversation_id": "conv-001",
  "routed_to": "DataAnalyzer",
  "routing_strategy": "best_match",
  "status": "delivered",
  "timestamp": "2025-09-10T15:35:00Z"
}
```

### 4. Follow Agent Response

```bash
# Get agent response
curl http://localhost:8080/api/v1/conversations/conv-001/messages
```

Expected response:

```json
{
  "conversation_id": "conv-001",
  "messages": [
    {
      "id": "msg-001",
      "performative": "request",
      "sender": "api_client",
      "capability_target": "data-analysis",
      "routed_to": "DataAnalyzer",
      "content": {
        "request": "Analyze Q3 sales trends",
        "data_source": "https://example.com/sales.csv"
      },
      "timestamp": "2025-09-10T15:35:00Z"
    },
    {
      "id": "msg-002",
      "performative": "inform",
      "sender": "DataAnalyzer",
      "receiver": "api_client",
      "content": {
        "analysis": "Q3 shows 15% growth with strong seasonal patterns in September...",
        "insights": ["Peak sales in September", "Steady growth trend", "Inventory recommendations"],
        "memory_used": ["similar_q3_analysis", "seasonal_patterns"]
      },
      "timestamp": "2025-09-10T15:35:30Z"
    }
  ]
}
```

## Capability Registry API

### 5. Discover Available Capabilities

```bash
curl http://localhost:8080/api/v1/capabilities
```

Expected response:

```json
{
  "capabilities": [
    {
      "name": "data-analysis",
      "agents": ["DataAnalyzer"],
      "description": "Analyze datasets and provide insights",
      "load": "low",
      "avg_response_time": "2.3s"
    },
    {
      "name": "report-generation",
      "agents": ["DataAnalyzer"],
      "description": "Generate reports from analysis results",
      "load": "low",
      "avg_response_time": "1.8s"
    }
  ]
}
```

### 6. Query Specific Capability

```bash
curl http://localhost:8080/api/v1/capabilities/data-analysis
```

Expected response:

```json
{
  "capability": "data-analysis",
  "description": "Analyze datasets and provide insights",
  "agents": [
    {
      "name": "DataAnalyzer",
      "confidence": 0.95,
      "load": "low",
      "last_used": "2025-09-10T15:35:00Z",
      "success_rate": 0.98
    }
  ],
  "routing_strategy": "best_match",
  "total_requests": 12,
  "avg_response_time": "2.3s"
}
```

## Memory System API

### 7. Agent Memory Operations

```bash
# View agent's learned knowledge
curl http://localhost:8080/api/v1/agents/DataAnalyzer/memory
```

Expected response:

```json
{
  "agent": "DataAnalyzer",
  "memory": {
    "scope": "workspace",
    "entities": 15,
    "relationships": 8,
    "last_updated": "2025-09-10T15:35:30Z"
  },
  "recent_memories": [
    {
      "entity": "q3_sales_analysis",
      "type": "analysis_pattern",
      "confidence": 0.92,
      "created": "2025-09-10T15:35:30Z"
    },
    {
      "entity": "seasonal_sales_pattern",
      "type": "insight",
      "confidence": 0.87,
      "created": "2025-09-10T15:35:30Z"
    }
  ]
}
```

### 8. Semantic Memory Search

```bash
# Search agent memory for similar patterns
curl -X POST http://localhost:8080/api/v1/agents/DataAnalyzer/memory/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "sales trends analysis",
    "limit": 5,
    "min_similarity": 0.7
  }'
```

Expected response:

```json
{
  "query": "sales trends analysis",
  "matches": [
    {
      "entity": "q3_sales_analysis",
      "similarity": 0.95,
      "content": "Analysis of Q3 sales showing 15% growth with September peak",
      "metadata": {
        "type": "analysis_pattern",
        "created": "2025-09-10T15:35:30Z"
      }
    },
    {
      "entity": "seasonal_sales_pattern",
      "similarity": 0.84,
      "content": "Historical pattern: sales peak in Q3, especially September",
      "metadata": {
        "type": "insight",
        "created": "2025-09-10T15:35:30Z"
      }
    }
  ]
}
```

## Multi-Agent Workflow API

### 9. Deploy Multiple Cooperating Agents

Deploy a report generator that works with the data analyzer:

```bash
# Deploy report generator agent
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "ReportGenerator",
    "version": "1.0.0",
    "capabilities": ["report-generation", "document-creation"],
    "tools": ["pdf_generator", "template_engine"],
    "memory": {
      "enabled": true,
      "scope": "workspace"
    },
    "system_prompt": "You create professional reports from analysis results. Listen for messages from data-analysis agents and automatically generate comprehensive reports."
  }'
```

### 10. Trigger Multi-Agent Workflow

Send a request that activates multiple agents:

```bash
# Request comprehensive analysis + report
curl -X POST http://localhost:8080/api/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "capability": "data-analysis",
    "performative": "request",
    "content": {
      "request": "Create comprehensive Q3 sales report",
      "data_source": "https://example.com/sales.csv",
      "requirements": "Include analysis, charts, and executive summary"
    },
    "conversation_id": "workflow-001",
    "follow_conversation": true
  }'
```

### 11. Monitor Workflow Progress

```bash
# Watch conversation progress
curl http://localhost:8080/api/v1/conversations/workflow-001?follow=true
```

This streams real-time updates showing:

1. DataAnalyzer processes the data
2. DataAnalyzer sends results to ReportGenerator via capability routing
3. ReportGenerator creates the final report
4. User receives completed report

## Agent Configuration Management

### 12. Update Agent Configuration

```bash
# Update agent system prompt or capabilities
curl -X PUT http://localhost:8080/api/v1/agents/DataAnalyzer \
  -H "Content-Type: application/json" \
  -d '{
    "version": "1.1.0",
    "capabilities": ["data-analysis", "report-generation", "trend-prediction"],
    "system_prompt": "Enhanced data analyst with trend prediction capabilities..."
  }'
```

### 13. Agent Health and Metrics

```bash
# Check agent health
curl http://localhost:8080/api/v1/agents/DataAnalyzer/health
```

Expected response:

```json
{
  "agent": "DataAnalyzer",
  "status": "healthy",
  "uptime": "1h23m",
  "metrics": {
    "messages_processed": 47,
    "avg_response_time": "2.1s",
    "success_rate": 0.97,
    "memory_usage": "45MB",
    "last_activity": "2025-09-10T16:15:00Z"
  },
  "capabilities": {
    "data-analysis": {
      "requests": 32,
      "success_rate": 0.96,
      "avg_time": "2.3s"
    },
    "report-generation": {
      "requests": 15,
      "success_rate": 0.98,
      "avg_time": "1.8s"
    }
  }
}
```

## Conversation Management API

### 14. List Active Conversations

```bash
curl http://localhost:8080/api/v1/conversations
```

Expected response:

```json
{
  "conversations": [
    {
      "id": "conv-001",
      "participants": ["api_client", "DataAnalyzer"],
      "message_count": 4,
      "status": "active",
      "created": "2025-09-10T15:35:00Z",
      "last_activity": "2025-09-10T15:42:00Z"
    },
    {
      "id": "workflow-001",
      "participants": ["api_client", "DataAnalyzer", "ReportGenerator"],
      "message_count": 7,
      "status": "completed",
      "created": "2025-09-10T15:45:00Z",
      "last_activity": "2025-09-10T15:47:30Z"
    }
  ]
}
```

### 15. Archive Completed Conversations

```bash
# Archive old conversations
curl -X POST http://localhost:8080/api/v1/conversations/archive \
  -H "Content-Type: application/json" \
  -d '{
    "older_than": "24h",
    "status": "completed"
  }'
```

## Server Management API

### 16. Server Configuration

```bash
# View current server configuration
curl http://localhost:8080/api/v1/config
```

Expected response:

```json
{
  "server": {
    "host": "0.0.0.0",
    "port": 8080,
    "version": "1.0.0"
  },
  "runtime": {
    "max_agents": 1000,
    "agent_timeout": "30s",
    "llm_provider": "anthropic"
  },
  "memory": {
    "backend": "embedded",
    "max_entities": 100000,
    "entities_count": 23
  },
  "capabilities": {
    "registered": 4,
    "routing_strategy": "best_match"
  }
}
```

### 17. System Metrics

```bash
# Get Prometheus metrics
curl http://localhost:9090/metrics | grep caxton
```

Sample metrics:

```text
caxton_agents_total{type="configuration"} 2
caxton_messages_total{status="delivered"} 47
caxton_capability_requests_total{capability="data-analysis"} 32
caxton_memory_entities_total{scope="workspace"} 23
caxton_response_time_seconds{capability="data-analysis"} 2.1
```

### 18. System Health Dashboard

```bash
# Get dashboard data
curl http://localhost:8080/api/v1/dashboard
```

Expected response:

```json
{
  "system": {
    "status": "healthy",
    "uptime": "2h15m",
    "version": "1.0.0"
  },
  "agents": {
    "total": 2,
    "running": 2,
    "config_agents": 2,
    "wasm_agents": 0
  },
  "capabilities": {
    "total": 4,
    "active": 4,
    "avg_response_time": "2.0s"
  },
  "memory": {
    "backend": "embedded",
    "entities": 23,
    "relationships": 12,
    "usage": "12MB"
  },
  "conversations": {
    "active": 1,
    "total_today": 8,
    "avg_duration": "3m20s"
  }
}
```

## Real-World API Usage Patterns

### 19. Batch Operations

Process multiple requests efficiently:

```bash
# Batch message sending
curl -X POST http://localhost:8080/api/v1/messages/batch \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      {
        "capability": "data-analysis",
        "performative": "request",
        "content": {"dataset": "sales-jan.csv"}
      },
      {
        "capability": "data-analysis",
        "performative": "request",
        "content": {"dataset": "sales-feb.csv"}
      },
      {
        "capability": "data-analysis",
        "performative": "request",
        "content": {"dataset": "sales-mar.csv"}
      }
    ],
    "batch_id": "q1-analysis"
  }'
```

### 20. Webhook Notifications

Set up webhooks for agent responses:

```bash
# Register webhook for capability responses
curl -X POST http://localhost:8080/api/v1/webhooks \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://your-app.com/webhooks/caxton",
    "events": ["message.response", "conversation.complete"],
    "filter": {
      "capabilities": ["data-analysis", "report-generation"]
    }
  }'
```

### 21. Agent Templates

Create agents from templates:

```bash
# List available templates
curl http://localhost:8080/api/v1/templates

# Create agent from template
curl -X POST http://localhost:8080/api/v1/agents/from-template \
  -H "Content-Type: application/json" \
  -d '{
    "template": "data-analyst",
    "name": "SalesAnalyzer",
    "parameters": {
      "specialization": "sales",
      "tools": ["crm_integration", "excel_parser"]
    }
  }'
```

## Error Handling

### Common HTTP Status Codes

- **200 OK**: Request successful
- **201 Created**: Agent deployed successfully
- **400 Bad Request**: Invalid request data
- **404 Not Found**: Agent or capability not found
- **409 Conflict**: Agent name already exists
- **422 Unprocessable Entity**: Invalid agent configuration
- **500 Internal Server Error**: Server error

### Error Response Format

```json
{
  "error": {
    "code": "INVALID_CAPABILITY",
    "message": "Capability 'nonexistent-capability' not found",
    "details": {
      "available_capabilities": ["data-analysis", "report-generation"],
      "suggestion": "Use /api/v1/capabilities to list available capabilities"
    },
    "timestamp": "2025-09-10T15:50:00Z"
  }
}
```

## API Client Libraries

### JavaScript/Node.js

```javascript
const caxton = require('@caxton/client');

const client = new caxton.Client('http://localhost:8080');

// Deploy configuration agent
const agent = await client.agents.deploy({
  name: 'DataAnalyzer',
  capabilities: ['data-analysis'],
  system_prompt: 'You analyze data...'
});

// Send capability-based message
const response = await client.messages.send({
  capability: 'data-analysis',
  content: { dataset: 'sales.csv' }
});
```

### Python

```python
from caxton import CaxtonClient

client = CaxtonClient('http://localhost:8080')

# Deploy agent
agent = client.agents.deploy({
    'name': 'DataAnalyzer',
    'capabilities': ['data-analysis'],
    'system_prompt': 'You analyze data...'
})

# Send message
response = client.messages.send(
    capability='data-analysis',
    content={'dataset': 'sales.csv'}
)
```

### cURL Scripts

Save common operations as scripts:

```bash
#!/bin/bash
# deploy-agent.sh
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d @"$1"

# send-message.sh
curl -X POST http://localhost:8080/api/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "capability": "'$1'",
    "performative": "request",
    "content": '"$2"'
  }'
```

## Next Steps

You now understand Caxton's REST API! Continue exploring:

- **[Configuration Guide](configuration.md)** - Advanced agent configuration
  options
- **[Agent Patterns](../developer-guide/agent-patterns.md)** - Multi-agent
  orchestration patterns
- **[Memory System](../developer-guide/memory-system.md)** - Deep dive into
  agent learning
- **[Production API](../operations/production-api.md)** - Security, rate
  limiting, monitoring

**Ready to integrate?** The REST API makes it easy to build configuration
agents into any application or workflow!
