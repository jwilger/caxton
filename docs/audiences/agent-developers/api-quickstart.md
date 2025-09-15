---
title: "REST API Quick Start Guide"
date: 2025-09-10
layout: page
audience: agent-developers
navigation_order: 1
categories: [Agent Developers, API Reference]
---

## Build agent integrations with Caxton's REST API in 5 minutes

This guide provides working examples for Caxton's REST API, focused on
**configuration agents** and **capability-based messaging**. Perfect for
developers building applications that integrate with multi-agent systems.

All examples use real endpoints you can test immediately and are designed
for production integration scenarios.

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

### Port Usage Reference

Caxton uses specific ports for different services:

- **Port 8080**: Main REST API server (all agent management and messaging)
- **Port 9090**: Prometheus metrics endpoint (monitoring and observability)

All examples in this guide use port 8080 for the main API unless
specifically noted.

## Configuration Agent API Patterns

### 1. Deploy a Configuration Agent

Create and deploy an agent from JSON configuration:

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
  "system_prompt": "You are a data analysis expert who helps users
    understand their data. When you receive analysis requests, check your
    memory for similar patterns, use appropriate tools to fetch and parse
    data, then provide clear insights.",
  "user_prompt_template": "Analyze this request: {{request}}\n\n
    Memory context: {{memory_context}}\nData source: {{data_source}}"
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
        "analysis": "Q3 shows 15% growth with strong seasonal patterns
          in September...",
        "insights": ["Peak sales in September", "Steady growth trend",
          "Inventory recommendations"],
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
    "system_prompt": "You create professional reports from analysis
      results. Listen for messages from data-analysis agents and
      automatically generate comprehensive reports."
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
    "system_prompt": "Enhanced data analyst with trend prediction
      capabilities..."
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

## Production Integration Patterns

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
    },
    "authentication": {
      "type": "bearer",
      "token": "your-webhook-token"
    }
  }'
```

### 21. Agent Templates

Create agents from templates for consistent deployment:

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

## Authentication and Security

### 22. API Authentication

```bash
# Obtain JWT token
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "developer",
    "password": "your-password"
  }'
```

Response:

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3600,
  "refresh_token": "refresh_token_here"
}
```

### 23. Authenticated Requests

```bash
# Use JWT token in subsequent requests
curl -X GET http://localhost:8080/api/v1/agents \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### 24. API Key Authentication

```bash
# Alternative: Use API key authentication
curl -X GET http://localhost:8080/api/v1/agents \
  -H "X-API-Key: your-api-key"
```

## Error Handling

### Common HTTP Status Codes

- **200 OK**: Request successful
- **201 Created**: Agent deployed successfully
- **400 Bad Request**: Invalid request data
- **401 Unauthorized**: Authentication required
- **403 Forbidden**: Insufficient permissions
- **404 Not Found**: Agent or capability not found
- **409 Conflict**: Agent name already exists
- **422 Unprocessable Entity**: Invalid agent configuration
- **429 Too Many Requests**: Rate limit exceeded
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
    "timestamp": "2025-09-10T15:50:00Z",
    "request_id": "req-123456"
  }
}
```

### Retry Logic Example

```bash
# Bash retry function
retry_request() {
  local url=$1
  local max_attempts=3
  local delay=1

  for i in $(seq 1 $max_attempts); do
    response=$(curl -s -w "HTTPSTATUS:%{http_code}" "$url")
    http_code=$(echo "$response" | grep "HTTPSTATUS:" | cut -d: -f2)

    if [ "$http_code" -eq 200 ]; then
      echo "$response" | sed -E 's/HTTPSTATUS:[0-9]+$//'
      return 0
    fi

    if [ "$i" -lt "$max_attempts" ]; then
      sleep "$delay"
      delay=$((delay * 2))
    fi
  done

  echo "Request failed after $max_attempts attempts"
  return 1
}
```

## API Client Libraries

### JavaScript/Node.js

```javascript
const CaxtonClient = require("@caxton/client");

class CaxtonIntegration {
  constructor(baseUrl, apiKey) {
    this.client = new CaxtonClient(baseUrl, { apiKey });
  }

  async deployAgent(config) {
    try {
      const result = await this.client.agents.deploy(config);
      console.log("Agent deployed:", result.agent_id);
      return result;
    } catch (error) {
      console.error("Deployment failed:", error.message);
      throw error;
    }
  }

  async sendMessage(capability, content, conversationId = null) {
    const message = {
      capability,
      performative: "request",
      content,
      conversation_id: conversationId || this.generateConversationId(),
    };

    const response = await this.client.messages.send(message);
    return this.waitForResponse(response.conversation_id);
  }

  async waitForResponse(conversationId, timeout = 30000) {
    return new Promise((resolve, reject) => {
      const startTime = Date.now();

      const checkResponse = async () => {
        try {
          const conversation =
            await this.client.conversations.get(conversationId);
          const lastMessage =
            conversation.messages[conversation.messages.length - 1];

          if (lastMessage.sender !== "api_client") {
            resolve(lastMessage);
            return;
          }

          if (Date.now() - startTime > timeout) {
            reject(new Error("Response timeout"));
            return;
          }

          setTimeout(checkResponse, 1000);
        } catch (error) {
          reject(error);
        }
      };

      checkResponse();
    });
  }

  generateConversationId() {
    return `conv-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }
}

// Usage example
const caxton = new CaxtonIntegration("http://localhost:8080", "your-api-key");

// Deploy an agent
await caxton.deployAgent({
  name: "DocumentAnalyzer",
  capabilities: ["document-analysis"],
  system_prompt: "Analyze documents and extract key information...",
});

// Send a message and wait for response
const response = await caxton.sendMessage("document-analysis", {
  document_url: "https://example.com/document.pdf",
  analysis_type: "summary",
});

console.log("Analysis result:", response.content);
```

### Python

```python
import asyncio
import aiohttp
from typing import Dict, List, Optional

class CaxtonClient:
    def __init__(self, base_url: str, api_key: Optional[str] = None):
        self.base_url = base_url.rstrip('/')
        self.api_key = api_key
        self.session = None

    async def __aenter__(self):
        self.session = aiohttp.ClientSession()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    def _headers(self) -> Dict[str, str]:
        headers = {'Content-Type': 'application/json'}
        if self.api_key:
            headers['X-API-Key'] = self.api_key
        return headers

    async def deploy_agent(self, config: Dict) -> Dict:
        """Deploy a configuration agent."""
        async with self.session.post(
            f'{self.base_url}/api/v1/agents',
            json=config,
            headers=self._headers()
        ) as response:
            response.raise_for_status()
            return await response.json()

    async def send_message(self, capability: str, content: Dict,
                          conversation_id: Optional[str] = None) -> Dict:
        """Send a capability-based message."""
        message = {
            'capability': capability,
            'performative': 'request',
            'content': content,
            'conversation_id': (
                conversation_id or self._generate_conversation_id()
            )
        }

        async with self.session.post(
            f'{self.base_url}/api/v1/messages',
            json=message,
            headers=self._headers()
        ) as response:
            response.raise_for_status()
            return await response.json()

    async def get_conversation(self, conversation_id: str) -> Dict:
        """Get conversation messages."""
        async with self.session.get(
            f'{self.base_url}/api/v1/conversations/{conversation_id}/messages',
            headers=self._headers()
        ) as response:
            response.raise_for_status()
            return await response.json()

    async def wait_for_response(self, conversation_id: str,
                               timeout: int = 30) -> Dict:
        """Wait for agent response in conversation."""
        import time
        start_time = time.time()

        while time.time() - start_time < timeout:
            conversation = await self.get_conversation(conversation_id)
            messages = conversation['messages']

            if len(messages) > 1:  # Original message + response
                last_message = messages[-1]
                if last_message['sender'] != 'api_client':
                    return last_message

            await asyncio.sleep(1)

        raise TimeoutError('No response received within timeout period')

    def _generate_conversation_id(self) -> str:
        import time
        import random
        import string
        timestamp = int(time.time())
        random_suffix = ''.join(random.choices(string.ascii_lowercase, k=6))
        return f'conv-{timestamp}-{random_suffix}'

# Usage example
async def main():
    async with CaxtonClient('http://localhost:8080', 'your-api-key') as client:
        # Deploy agent
        agent_config = {
            'name': 'DataProcessor',
            'capabilities': ['data-processing'],
            'system_prompt': 'Process and analyze data efficiently...'
        }

        deployment = await client.deploy_agent(agent_config)
        print(f"Agent deployed: {deployment['agent_id']}")

        # Send message and wait for response
        message_response = await client.send_message(
            'data-processing',
            {'data': 'sample data', 'operation': 'analyze'}
        )

        conversation_id = message_response['conversation_id']
        agent_response = await client.wait_for_response(conversation_id)

        print(f"Agent response: {agent_response['content']}")

# Run the example
asyncio.run(main())
```

### cURL Scripts for CI/CD

Save common operations as reusable scripts:

```bash
#!/bin/bash
# deploy-agent.sh - Deploy agent from configuration file

set -euo pipefail

CAXTON_URL="${CAXTON_URL:-http://localhost:8080}"
CONFIG_FILE="$1"

if [ ! -f "$CONFIG_FILE" ]; then
    echo "Error: Configuration file $CONFIG_FILE not found"
    exit 1
fi

echo "Deploying agent from $CONFIG_FILE..."

response=$(curl -s -w "HTTPSTATUS:%{http_code}" \
    -X POST "$CAXTON_URL/api/v1/agents" \
    -H "Content-Type: application/json" \
    -H "X-API-Key: ${CAXTON_API_KEY}" \
    -d @"$CONFIG_FILE")

http_code=$(echo "$response" | grep "HTTPSTATUS:" | cut -d: -f2)
body=$(echo "$response" | sed -E 's/HTTPSTATUS:[0-9]+$//')

if [ "$http_code" -eq 201 ]; then
    echo "✅ Agent deployed successfully"
    echo "$body" | jq '.'
else
    echo "❌ Deployment failed (HTTP $http_code)"
    echo "$body" | jq '.'
    exit 1
fi
```

```bash
#!/bin/bash
# send-message.sh - Send capability-based message

set -euo pipefail

CAXTON_URL="${CAXTON_URL:-http://localhost:8080}"
CAPABILITY="$1"
CONTENT="$2"
CONVERSATION_ID="${3:-conv-$(date +%s)-$(openssl rand -hex 4)}"

message=$(jq -n \
    --arg capability "$CAPABILITY" \
    --argjson content "$CONTENT" \
    --arg conversation_id "$CONVERSATION_ID" \
    '{
        capability: $capability,
        performative: "request",
        content: $content,
        conversation_id: $conversation_id
    }')

echo "Sending message to capability: $CAPABILITY"
echo "Conversation ID: $CONVERSATION_ID"

response=$(curl -s \
    -X POST "$CAXTON_URL/api/v1/messages" \
    -H "Content-Type: application/json" \
    -H "X-API-Key: ${CAXTON_API_KEY}" \
    -d "$message")

echo "$response" | jq '.'

# Optionally wait for response
if [ "${WAIT_FOR_RESPONSE:-false}" = "true" ]; then
    echo "Waiting for agent response..."
    sleep 2

    curl -s "$CAXTON_URL/api/v1/conversations/$CONVERSATION_ID/messages" \
        -H "X-API-Key: ${CAXTON_API_KEY}" | \
        jq '.messages[] | select(.sender != "api_client") | .content'
fi
```

## Production Integration Best Practices

### 1. Connection Management

Use connection pooling and keep-alive:

```python
# Configure session with connection pooling
connector = aiohttp.TCPConnector(
    limit=100,  # Total connection pool size
    limit_per_host=20,  # Connections per host
    keepalive_timeout=30,
    enable_cleanup_closed=True
)

session = aiohttp.ClientSession(
    connector=connector,
    timeout=aiohttp.ClientTimeout(total=30)
)
```

### 2. Error Handling and Retries

Implement exponential backoff:

```python
import asyncio
import random

async def retry_with_backoff(func, max_retries=3, base_delay=1):
    for attempt in range(max_retries):
        try:
            return await func()
        except aiohttp.ClientError as e:
            if attempt == max_retries - 1:
                raise

            delay = base_delay * (2 ** attempt) + random.uniform(0, 1)
            await asyncio.sleep(delay)
```

### 3. Rate Limiting

Respect API rate limits:

```python
import asyncio
from asyncio import Semaphore

class RateLimiter:
    def __init__(self, max_concurrent=10, rate_per_second=5):
        self.semaphore = Semaphore(max_concurrent)
        self.rate_per_second = rate_per_second
        self.last_request_time = 0

    async def acquire(self):
        await self.semaphore.acquire()

        current_time = asyncio.get_event_loop().time()
        time_since_last = current_time - self.last_request_time
        min_interval = 1.0 / self.rate_per_second

        if time_since_last < min_interval:
            await asyncio.sleep(min_interval - time_since_last)

        self.last_request_time = asyncio.get_event_loop().time()

    def release(self):
        self.semaphore.release()
```

### 4. Monitoring and Observability

Track API usage and performance:

```python
import time
from dataclasses import dataclass
from typing import Dict

@dataclass
class APIMetrics:
    total_requests: int = 0
    successful_requests: int = 0
    failed_requests: int = 0
    total_response_time: float = 0.0

class MonitoredCaxtonClient(CaxtonClient):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.metrics = APIMetrics()

    async def _make_request(self, method, url, **kwargs):
        start_time = time.time()
        self.metrics.total_requests += 1

        try:
            response = await super()._make_request(method, url, **kwargs)
            self.metrics.successful_requests += 1
            return response
        except Exception as e:
            self.metrics.failed_requests += 1
            raise
        finally:
            self.metrics.total_response_time += time.time() - start_time

    def get_metrics(self) -> Dict:
        return {
            'total_requests': self.metrics.total_requests,
            'success_rate': (
                self.metrics.successful_requests /
                max(self.metrics.total_requests, 1)
            ),
            'avg_response_time': (
                self.metrics.total_response_time /
                max(self.metrics.total_requests, 1)
            )
        }
```

## Next Steps for Agent Developers

You now understand Caxton's REST API! Continue building:

- **[Configuration Reference](../operators/configuration.md)** - Advanced
  server configuration options
- **[Agent Format Reference](../../config-agents/agent-format.md)** - Complete
  agent configuration schema
- **[API Reference](../../developer-guide/api-reference.md)** - Complete API
  documentation
- **[Memory System](../../memory-system/overview.md)** - Deep dive into agent
  learning capabilities
- **[Production Integration](../../operations/production-api.md)** - Security,
  scaling, and monitoring

### Integration Patterns

Explore these common integration patterns:

1. **Event-Driven Architecture**: Use webhooks for real-time responses
2. **Batch Processing**: Process multiple requests efficiently
3. **Pipeline Integration**: Chain agents for complex workflows
4. **Monitoring Integration**: Track performance and costs
5. **A/B Testing**: Compare agent versions in production

**Ready to integrate?** The REST API makes it easy to build configuration
agents into any application or workflow!
