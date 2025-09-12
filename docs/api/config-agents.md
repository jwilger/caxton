---
title: "Configuration Agent API"
date: 2025-09-10
layout: page
categories: [api, config-agents]
---

> **🚧 Implementation Status**
>
> This API specification represents the intended configuration agent
> management interface from ADR-28. The REST endpoints and YAML schema
> documented here serve as acceptance criteria for the implementation
> currently in development.
>
> **Target**: Full configuration agent lifecycle management API
> **Status**: API design complete, implementation in progress

## Overview

Configuration agents are the primary user experience for Caxton, providing
5-10 minute onboarding through TOML configuration files. This API
enables creation, management, and orchestration of configuration-driven agents
without requiring WebAssembly compilation.

**Architecture**: Based on ADR-0032, configuration agents are defined as
TOML configuration files with embedded documentation, executed through LLM
orchestration in
the host runtime while actual functionality is provided by WebAssembly MCP
servers in isolated sandboxes.

## Configuration Agent Structure

Configuration agents consist of three parts:

1. **TOML Configuration**: Agent metadata, capabilities, and configuration
2. **System Instructions**: Embedded documentation and prompts
3. **Runtime Integration**: Automatic capability registration and memory integration

### Example Configuration Agent

```toml
name = "DataAnalyzer"
version = "1.0.0"
capabilities = ["data-analysis", "report-generation"]
tools = ["http_client", "csv_parser", "chart_generator"]

[parameters]
max_file_size = "10MB"
supported_formats = ["csv", "json", "xlsx"]

[memory]
enabled = true
scope = "workspace"

system_prompt = '''
You are a data analysis expert who helps users understand their data.
You can fetch data from URLs, parse various formats, and create visualizations.
'''

user_prompt_template = '''
Analyze the following data request: {{request}}

Available data: {{context}}
User requirements: {{requirements}}
'''

documentation = '''
# DataAnalyzer Agent

This agent specializes in data analysis tasks and can:
- Fetch data from HTTP endpoints
- Parse CSV, JSON, and Excel files
- Generate charts and visualizations
- Provide statistical summaries

## Usage Examples

Ask me to:
- "Analyze the sales data at https://example.com/sales.csv"
- "Create a chart showing monthly trends"
- "Summarize the key metrics in this dataset"
'''
```

## Core API Endpoints

### Deploy Configuration Agent

**POST** `/api/v1/config-agents`

Deploy a new configuration agent from TOML content.

#### Request Body

```json
{
  "name": "DataAnalyzer",
  "content": "name = \"DataAnalyzer\"\nversion = \"1.0.0\"\n...",
  "auto_start": true,
  "workspace": "project-alpha"
}
```

#### Request Fields

- `name` (string, required): Agent identifier, must be unique within workspace
- `content` (string, required): Complete TOML configuration file
- `auto_start` (boolean, optional): Start agent immediately after
  deployment (default: true)
- `workspace` (string, optional): Workspace scope for agent deployment
  (default: "default")

#### Response (201 Created)

```json
{
  "id": "config-550e8400-e29b-41d4-a716-446655440000",
  "name": "DataAnalyzer",
  "status": "running",
  "capabilities": ["data-analysis", "report-generation"],
  "workspace": "project-alpha",
  "deployed_at": "2025-09-10T14:30:00Z",
  "memory_enabled": true,
  "tools": ["http_client", "csv_parser", "chart_generator"]
}
```

#### Error Responses

- **400 Bad Request**: Invalid TOML format or missing required fields
- **409 Conflict**: Agent name already exists in workspace
- **422 Unprocessable Entity**: Configuration validation failed

### List Configuration Agents

**GET** `/api/v1/config-agents`

Retrieve all deployed configuration agents.

#### Query Parameters

- `workspace` (string, optional): Filter by workspace
- `capability` (string, optional): Filter by declared capability
- `status` (string, optional): Filter by status (`running`, `stopped`, `error`)
- `limit` (integer, optional): Maximum number of results (default: 50, max: 200)
- `offset` (integer, optional): Pagination offset (default: 0)

#### Response (200 OK)

```json
{
  "agents": [
    {
      "id": "config-550e8400-e29b-41d4-a716-446655440000",
      "name": "DataAnalyzer",
      "status": "running",
      "capabilities": ["data-analysis", "report-generation"],
      "workspace": "project-alpha",
      "deployed_at": "2025-09-10T14:30:00Z",
      "last_activity": "2025-09-10T15:45:23Z"
    }
  ],
  "total": 1,
  "limit": 50,
  "offset": 0
}
```

### Get Configuration Agent

**GET** `/api/v1/config-agents/{id}`

Retrieve detailed information about a specific configuration agent.

#### Path Parameters

- `id` (string, required): Configuration agent identifier

#### Response (200 OK)

```json
{
  "id": "config-550e8400-e29b-41d4-a716-446655440000",
  "name": "DataAnalyzer",
  "version": "1.0.0",
  "status": "running",
  "capabilities": ["data-analysis", "report-generation"],
  "tools": ["http_client", "csv_parser", "chart_generator"],
  "parameters": {
    "max_file_size": "10MB",
    "supported_formats": ["csv", "json", "xlsx"]
  },
  "workspace": "project-alpha",
  "memory_enabled": true,
  "memory_scope": "workspace",
  "deployed_at": "2025-09-10T14:30:00Z",
  "last_activity": "2025-09-10T15:45:23Z",
  "execution_stats": {
    "total_requests": 45,
    "successful_requests": 43,
    "failed_requests": 2,
    "average_response_time": "1.2s"
  },
  "content": "name = \"DataAnalyzer\"\nversion = \"1.0.0\"\n..."
}
```

#### Error Responses

- **404 Not Found**: Configuration agent does not exist

### Update Configuration Agent

**PUT** `/api/v1/config-agents/{id}`

Update an existing configuration agent's definition.

#### Path Parameters

- `id` (string, required): Configuration agent identifier

#### Request Body

```json
{
  "content": "---\nname: DataAnalyzer\nversion: \"1.1.0\"\n...",
  "restart": true
}
```

#### Request Fields

- `content` (string, required): Updated TOML configuration content
- `restart` (boolean, optional): Restart agent after update (default: true)

#### Response (200 OK)

```json
{
  "id": "config-550e8400-e29b-41d4-a716-446655440000",
  "name": "DataAnalyzer",
  "version": "1.1.0",
  "status": "running",
  "updated_at": "2025-09-10T16:30:00Z",
  "changes": ["version", "parameters", "system_prompt"]
}
```

#### Error Responses

- **400 Bad Request**: Invalid TOML format
- **404 Not Found**: Configuration agent does not exist
- **422 Unprocessable Entity**: Configuration validation failed

### Delete Configuration Agent

**DELETE** `/api/v1/config-agents/{id}`

Remove a configuration agent and clean up its resources.

#### Path Parameters

- `id` (string, required): Configuration agent identifier

#### Query Parameters

- `force` (boolean, optional): Force deletion even if agent has active
  conversations (default: false)

#### Response (204 No Content)

Agent successfully deleted.

#### Error Responses

- **404 Not Found**: Configuration agent does not exist
- **409 Conflict**: Agent has active conversations (use `force=true` to override)

### Restart Configuration Agent

**POST** `/api/v1/config-agents/{id}/restart`

Restart a configuration agent, reloading its configuration.

#### Path Parameters

- `id` (string, required): Configuration agent identifier

#### Response (200 OK)

```json
{
  "id": "config-550e8400-e29b-41d4-a716-446655440000",
  "name": "DataAnalyzer",
  "status": "running",
  "restarted_at": "2025-09-10T16:45:00Z"
}
```

#### Error Responses

- **404 Not Found**: Configuration agent does not exist
- **422 Unprocessable Entity**: Agent configuration is invalid

### Stream Agent Logs

**GET** `/api/v1/config-agents/{id}/logs`

Stream real-time execution logs from a configuration agent.

#### Path Parameters

- `id` (string, required): Configuration agent identifier

#### Query Parameters

- `follow` (boolean, optional): Follow log stream in real-time (default: false)
- `tail` (integer, optional): Number of recent log lines to include
  (default: 100, max: 1000)
- `level` (string, optional): Minimum log level (`debug`, `info`, `warn`, `error`)

#### Response (200 OK)

```text
Content-Type: text/plain; charset=utf-8
Content-Encoding: chunked

2025-09-10T16:45:01Z [INFO] Agent DataAnalyzer started
2025-09-10T16:45:02Z [INFO] Registered capabilities: data-analysis, report-generation
2025-09-10T16:45:05Z [DEBUG] Received request: analyze sales data
2025-09-10T16:45:06Z [INFO] Fetching data from https://example.com/sales.csv
2025-09-10T16:45:07Z [INFO] Processing 1,250 rows of sales data
2025-09-10T16:45:08Z [INFO] Generated analysis report with 3 charts
```

#### Error Responses

- **404 Not Found**: Configuration agent does not exist

## Configuration Validation

### Validate Configuration

**POST** `/api/v1/config-agents/validate`

Validate configuration agent TOML configuration without deploying.

#### Request Body

```json
{
  "content": "name = \"DataAnalyzer\"\nversion = \"1.0.0\"\n..."
}
```

#### Response (200 OK)

```json
{
  "valid": true,
  "parsed_config": {
    "name": "DataAnalyzer",
    "version": "1.0.0",
    "capabilities": ["data-analysis", "report-generation"],
    "tools": ["http_client", "csv_parser", "chart_generator"]
  },
  "warnings": [
    "Tool 'advanced_analytics' not available, requests will fail"
  ]
}
```

#### Validation Errors (422 Unprocessable Entity)

```json
{
  "valid": false,
  "errors": [
    {
      "field": "name",
      "message": "Agent name is required",
      "location": "frontmatter"
    },
    {
      "field": "capabilities",
      "message": "At least one capability must be declared",
      "location": "frontmatter.capabilities"
    }
  ]
}
```

## Configuration Templates

### List Configuration Templates

**GET** `/api/v1/config-agents/templates`

Retrieve available configuration agent templates.

#### Query Parameters

- `category` (string, optional): Filter by template category
- `capability` (string, optional): Filter by required capability

#### Response (200 OK)

```json
{
  "templates": [
    {
      "id": "data-analyzer",
      "name": "Data Analyzer",
      "description": "Analyzes CSV, JSON, and Excel files with visualization",
      "category": "data-processing",
      "capabilities": ["data-analysis", "report-generation"],
      "required_tools": ["http_client", "csv_parser", "chart_generator"],
      "example_use_cases": [
        "Sales data analysis",
        "Financial reporting",
        "Survey result processing"
      ]
    },
    {
      "id": "web-researcher",
      "name": "Web Researcher",
      "description": "Searches web content and compiles research reports",
      "category": "research",
      "capabilities": ["web-search", "content-analysis"],
      "required_tools": ["web_search", "html_parser", "pdf_generator"]
    }
  ],
  "total": 2
}
```

### Get Configuration Template

**GET** `/api/v1/config-agents/templates/{template_id}`

Retrieve a specific configuration template with full content.

#### Path Parameters

- `template_id` (string, required): Template identifier

#### Response (200 OK)

```json
{
  "id": "data-analyzer",
  "name": "Data Analyzer",
  "description": "Analyzes CSV, JSON, and Excel files with visualization",
  "category": "data-processing",
  "content": "name = \"{{AGENT_NAME}}\"\nversion = \"1.0.0\"\ncapabilities = [\"data-analysis\", \"report-generation\"]\n...",
  "parameters": [
    {
      "name": "AGENT_NAME",
      "description": "Name for your data analysis agent",
      "required": true,
      "example": "SalesAnalyzer"
    },
    {
      "name": "MAX_FILE_SIZE",
      "description": "Maximum file size to process",
      "required": false,
      "default": "10MB"
    }
  ]
}
```

#### Error Responses

- **404 Not Found**: Template does not exist

## Domain Types

### ConfigurationAgent

Core configuration agent representation:

```typescript
interface ConfigurationAgent {
  id: string;                    // Unique identifier (config-{uuid})
  name: string;                  // Agent name from YAML
  version: string;               // Agent version from YAML
  status: AgentStatus;           // Current runtime status
  capabilities: string[];        // Declared capabilities
  tools: string[];              // Required MCP tools
  parameters: Record<string, any>; // Custom parameters
  workspace: string;             // Workspace scope
  memory_enabled: boolean;       // Memory system integration
  memory_scope: MemoryScope;     // Memory sharing scope
  deployed_at: string;          // ISO 8601 deployment timestamp
  last_activity: string;        // ISO 8601 last activity timestamp
  content: string;              // Full TOML content
}
```

### AgentStatus

Agent execution status:

- `starting` - Agent is being initialized
- `running` - Agent is active and accepting requests
- `stopped` - Agent has been stopped manually
- `error` - Agent encountered a runtime error
- `restarting` - Agent is being restarted

### MemoryScope

Memory sharing configuration:

- `agent-only` - Private memory per agent instance
- `workspace` - Shared memory within workspace
- `global` - System-wide shared knowledge base

## Error Handling

All endpoints follow consistent error response format:

```json
{
  "error": "Configuration validation failed",
  "code": "VALIDATION_ERROR",
  "details": {
    "field": "capabilities",
    "message": "At least one capability must be declared"
  },
  "timestamp": "2025-09-10T16:45:00Z",
  "request_id": "req_abc123"
}
```

### Common Error Codes

- `VALIDATION_ERROR` - Invalid TOML or configuration
- `AGENT_NOT_FOUND` - Specified agent does not exist
- `NAME_CONFLICT` - Agent name already exists in workspace
- `RUNTIME_ERROR` - Agent execution failed
- `PERMISSION_DENIED` - Insufficient permissions for operation
- `WORKSPACE_NOT_FOUND` - Specified workspace does not exist

## Rate Limiting

Configuration agent endpoints are subject to rate limiting:

- **Deployment operations**: 10 requests per minute
- **Management operations**: 100 requests per minute
- **Validation operations**: 200 requests per minute
- **Log streaming**: 5 concurrent streams per user

Rate limit headers are included in all responses:

```text
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1694356800
```

## WebSocket Integration

Real-time updates for configuration agent events:

**Connection**: `ws://localhost:8080/ws/config-agents`

**Authentication**: Include API token as query parameter or Authorization header

**Event Format**:

```json
{
  "type": "agent_status_changed",
  "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
  "data": {
    "old_status": "starting",
    "new_status": "running"
  },
  "timestamp": "2025-09-10T16:45:00Z"
}
```

**Event Types**:

- `agent_deployed` - New configuration agent deployed
- `agent_status_changed` - Agent status transition
- `agent_updated` - Agent configuration updated
- `agent_deleted` - Agent removed
- `execution_error` - Agent runtime error occurred

## Performance Considerations

- **Deployment time**: Configuration agents start in 2-5 seconds vs
  30-60 seconds for WASM agents
- **Memory usage**: ~50-100MB per config agent vs ~200-500MB per WASM
  agent
- **Update speed**: Hot reloads complete in <1 second for configuration
  changes
- **Concurrent agents**: System supports 100+ configuration agents on
  standard hardware

## Migration from WASM Agents

Existing WASM agents can be migrated to configuration agents:

1. **Extract capabilities**: Identify what the WASM agent does
2. **Map to tools**: Determine which MCP tools provide equivalent functionality
3. **Create configuration**: Write YAML frontmatter and instructions
4. **Test behavior**: Validate equivalent functionality
5. **Deploy alongside**: Run both versions during transition
6. **Switch traffic**: Gradually move requests to config agent
7. **Deprecate WASM**: Remove WASM agent once config agent is stable

## Related Documentation

- [Capability Registration API](capability-registration.md) - Register and
  discover agent capabilities
- [Memory System API](memory-integration.md) - Agent memory and knowledge
  management
- [Agent Messaging API](fipa-messaging.md) - Inter-agent communication
- [ADR-0028](../adrs/0028-configuration-driven-agent-architecture.md) -
  Architectural decision rationale
