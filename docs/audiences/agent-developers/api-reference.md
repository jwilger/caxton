---
title: "API Reference for Agent Developers"
date: 2025-01-14
layout: page
categories: [Agent Developers, API]
difficulty: intermediate
audience: agent-developers
---

**Implementation Status**: See
[Implementation Status](../../api/implementation-status.md) for details on
what's
currently available.

Complete API documentation for Caxton management and control relevant to agent
developers.

> **Note**: This reference includes both implemented endpoints (marked with ✅)
> and planned endpoints (marked with ⏳). Only endpoints marked with ✅ are
> currently functional in Story 006 implementation.

## API Overview

Caxton provides a REST/HTTP API for all management operations, ensuring
simplicity and compatibility with standard HTTP tooling.

### Agent Types

Caxton supports two agent types:

- **Configuration agents**: Defined in TOML configuration files
  (primary experience) - **Beginner Level**
- **WebAssembly agents**: Compiled modules for advanced use cases requiring
  custom algorithms - **Advanced Level**

### API Base URL

The API is available at `http://localhost:3000` by default (configurable).

### Authentication

All API endpoints use simple bearer token authentication:

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  http://localhost:3000/api/agents
```

## Core Endpoints for Agent Developers

### Agent Management

#### List Agents ✅

```text
GET /api/agents
```

Returns all agents with their current status and capabilities.

**Response:**

```json
{
  "agents": [
    {
      "id": "agent-001",
      "name": "DataAnalyzer",
      "type": "configuration",
      "status": "running",
      "capabilities": ["data_analysis", "csv_processing"],
      "health": "healthy"
    }
  ]
}
```

#### Get Agent Details ✅

```text
GET /api/agents/{agent_id}
```

Returns detailed information about a specific agent.

##### Parameters

- `agent_id` (string): Unique identifier for the agent

**Response:**

```json
{
  "id": "agent-001",
  "name": "DataAnalyzer",
  "type": "configuration",
  "status": "running",
  "capabilities": ["data_analysis", "csv_processing"],
  "health": "healthy",
  "created_at": "2025-01-14T10:00:00Z",
  "last_activity": "2025-01-14T11:30:00Z",
  "configuration": {
    "description": "Analyzes CSV data and generates reports",
    "tools": ["csv_reader", "chart_generator"]
  }
}
```

#### Deploy Configuration Agent ✅

```text
POST /api/agents
```

Deploy a new configuration agent from a TOML definition.

**Request Body:**

```json
{
  "type": "configuration",
  "definition": "name = \"MyAgent\"\ndescription = \"Sample agent\"\n\n
    documentation = '''\n# Agent Instructions\n...\n'''"
}
```

**Response:**

```json
{
  "id": "agent-002",
  "status": "deployed",
  "message": "Agent deployed successfully"
}
```

#### Update Configuration Agent ⏳

```text
PUT /api/agents/{agent_id}
```

Update an existing configuration agent with a new definition.

#### Remove Agent ✅

```text
DELETE /api/agents/{agent_id}
```

Gracefully removes an agent and cleans up its resources.

### Capability Management

#### List Available Capabilities ✅

```text
GET /api/capabilities
```

Returns all capabilities available to agents.

**Response:**

```json
{
  "capabilities": [
    {
      "name": "data_analysis",
      "description": "Analyze structured data",
      "category": "data",
      "complexity": "intermediate"
    },
    {
      "name": "web_scraping",
      "description": "Extract data from web pages",
      "category": "web",
      "complexity": "beginner"
    }
  ]
}
```

#### Register Custom Capability ⏳

```text
POST /api/capabilities
```

Register a new capability for use by agents.

### Agent Communication

#### Send Message to Agent ✅

```text
POST /api/agents/{agent_id}/messages
```

Send a direct message to an agent.

**Request Body:**

```json
{
  "content": "Analyze the sales data for Q4",
  "conversation_id": "conv-123",
  "sender": "user-456"
}
```

#### Get Agent Conversation History ⏳

```text
GET /api/agents/{agent_id}/conversations/{conversation_id}
```

Retrieve message history for a specific conversation.

### Configuration Validation

#### Validate Agent Configuration ✅

```text
POST /api/validate
```

Validate an agent configuration before deployment.

**Request Body:**

```json
{
  "definition": "name = \"TestAgent\"\ndescription = \"Test\"\n\n
    documentation = '''\n# Instructions\n...\n'''"
}
```

**Response:**

```json
{
  "valid": true,
  "errors": [],
  "warnings": ["Description should be more detailed"]
}
```

## Development Workflow APIs

### Hot Reload and Testing

#### Reload Agent Configuration ⏳

```text
POST /api/agents/{agent_id}/reload
```

Hot reload an agent with updated configuration during development.

#### Test Agent Capability ⏳

```text
POST /api/agents/{agent_id}/test
```

Run predefined tests against an agent to verify functionality.

### Performance Monitoring

#### Get Agent Metrics ⏳

```text
GET /api/agents/{agent_id}/metrics
```

Retrieve performance metrics for debugging and optimization.

## Error Handling

All API endpoints return standard HTTP status codes:

- `200 OK`: Successful operation
- `201 Created`: Resource created successfully
- `400 Bad Request`: Invalid request data
- `404 Not Found`: Resource not found
- `500 Internal Server Error`: Server error

Error responses include detailed information:

```json
{
  "error": "validation_failed",
  "message": "Agent configuration is invalid",
  "details": {
    "field": "capabilities",
    "issue": "Unknown capability 'invalid_tool'"
  }
}
```

## Rate Limiting

API requests are rate limited to prevent abuse:

- **Configuration agents**: 100 requests per minute
- **WebAssembly agents**: 500 requests per minute
- **Validation endpoints**: 50 requests per minute

Rate limit headers are included in responses:

```text
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1641998400
```

## SDK Support

Official SDKs are available for popular languages:

- **JavaScript/TypeScript**: `@caxton/client`
- **Python**: `caxton-client`
- **Rust**: `caxton-sdk`

## Related Documentation

- [Building Agents Guide](building-agents.md) - **Beginner**
- [Configuration Agent Format](config-agents/agent-format.md) - **Beginner**
- [Security Guide](security.md) - **Intermediate**
- [Performance Specifications](../../api/performance-specifications.md) -
  **Advanced**
