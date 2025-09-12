---
title: "Capability Registration & Discovery API"
date: 2025-09-10
layout: page
categories: [api, capabilities]
---

## Overview

The Capability Registration & Discovery API enables capability-based routing
as defined in ADR-0029. Instead of addressing specific agents, messages
target capabilities, and the system routes requests to appropriate agents
based on their declared capabilities.

**Key Benefits**:

- **Decoupled communication**: Agents don't need to know about each other
  specifically
- **Load balancing**: Distribute requests across multiple agents providing the
  same capability
- **Service discovery**: Automatically find agents that can handle specific tasks
- **Dynamic scaling**: Add/remove capability providers without changing
  consumers

## Core Concepts

### Capabilities

**Capability**: A named service or functionality that agents can provide (e.g.,
"data-analysis", "image-processing", "web-search")

**Capability Provider**: An agent that declares it can handle requests for a
specific capability

**Capability Consumer**: An agent or client that sends requests to
capabilities rather than specific agents

### Routing Strategies

- **Single recipient**: Route to the best-matching agent for the capability
- **Broadcast**: Send to all agents that provide the capability
- **Load balancing**: Distribute requests across capable agents
- **Failover**: Try backup agents if primary providers are unavailable

## API Endpoints

### Register Agent Capability

**POST** `/api/v1/capabilities`

Register that an agent provides a specific capability.

#### Request Body

```json
{
  "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
  "capability": "data-analysis",
  "version": "1.0.0",
  "priority": 100,
  "metadata": {
    "supported_formats": ["csv", "json", "xlsx"],
    "max_file_size": "10MB",
    "processing_time": "fast"
  },
  "health_check_url": "/api/v1/config-agents/{agent_id}/health"
}
```

#### Request Fields

- `agent_id` (string, required): Agent identifier that provides the capability
- `capability` (string, required): Capability name (lowercase,
  hyphen-separated)
- `version` (string, optional): Capability version (semantic versioning,
  default: "1.0.0")
- `priority` (integer, optional): Routing priority (higher = preferred,
  default: 50)
- `metadata` (object, optional): Capability-specific information for routing decisions
- `health_check_url` (string, optional): Endpoint to verify agent health

#### Response (201 Created)

```json
{
  "registration_id": "cap-reg-abc123",
  "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
  "capability": "data-analysis",
  "version": "1.0.0",
  "priority": 100,
  "status": "active",
  "registered_at": "2025-09-10T14:30:00Z",
  "last_health_check": "2025-09-10T14:30:00Z",
  "metadata": {
    "supported_formats": ["csv", "json", "xlsx"],
    "max_file_size": "10MB",
    "processing_time": "fast"
  }
}
```

#### Error Responses

- **400 Bad Request**: Invalid capability name or agent ID
- **404 Not Found**: Agent does not exist
- **409 Conflict**: Agent already registered for this capability

### Discover Available Capabilities

**GET** `/api/v1/capabilities`

List all available capabilities and their providers.

#### Query Parameters

- `capability` (string, optional): Filter by specific capability name
- `agent_id` (string, optional): Filter by specific agent
- `status` (string, optional): Filter by registration status (`active`,
  `inactive`, `unhealthy`)
- `version` (string, optional): Filter by capability version
- `include_metadata` (boolean, optional): Include capability metadata
  (default: false)
- `limit` (integer, optional): Maximum results (default: 50, max: 200)
- `offset` (integer, optional): Pagination offset (default: 0)

#### Response (200 OK)

```json
{
  "capabilities": [
    {
      "capability": "data-analysis",
      "providers": [
        {
          "registration_id": "cap-reg-abc123",
          "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
          "agent_name": "DataAnalyzer",
          "version": "1.0.0",
          "priority": 100,
          "status": "active",
          "load_score": 0.3,
          "response_time_avg": "1.2s",
          "success_rate": 0.95
        },
        {
          "registration_id": "cap-reg-def456",
          "agent_id": "config-660f9511-f30c-52e5-b827-557766551111",
          "agent_name": "AdvancedAnalyzer",
          "version": "2.0.0",
          "priority": 80,
          "status": "active",
          "load_score": 0.7,
          "response_time_avg": "2.1s",
          "success_rate": 0.98
        }
      ]
    },
    {
      "capability": "image-processing",
      "providers": [
        {
          "registration_id": "cap-reg-ghi789",
          "agent_id": "config-770fA622-g41d-63f6-c938-668877662222",
          "agent_name": "ImageProcessor",
          "version": "1.0.0",
          "priority": 50,
          "status": "active",
          "load_score": 0.1,
          "response_time_avg": "3.5s",
          "success_rate": 0.92
        }
      ]
    }
  ],
  "total": 2,
  "limit": 50,
  "offset": 0
}
```

#### With Metadata (include_metadata=true)

```json
{
  "capabilities": [
    {
      "capability": "data-analysis",
      "providers": [
        {
          "registration_id": "cap-reg-abc123",
          "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
          "agent_name": "DataAnalyzer",
          "version": "1.0.0",
          "priority": 100,
          "status": "active",
          "metadata": {
            "supported_formats": ["csv", "json", "xlsx"],
            "max_file_size": "10MB",
            "processing_time": "fast",
            "specializations": ["sales_analysis", "financial_reporting"]
          },
          "performance_metrics": {
            "load_score": 0.3,
            "response_time_avg": "1.2s",
            "success_rate": 0.95,
            "requests_last_hour": 45
          }
        }
      ]
    }
  ]
}
```

### Find Agents for Capability

**GET** `/api/v1/capabilities/{capability}`

Find all agents that provide a specific capability, sorted by routing
preference.

#### Path Parameters

- `capability` (string, required): Capability name to search for

#### Query Parameters

- `version` (string, optional): Specific capability version
- `routing_strategy` (string, optional): Routing preference (`priority`,
  `load_balance`, `least_loaded`, `fastest_response`)
- `include_unhealthy` (boolean, optional): Include unhealthy providers
  (default: false)
- `metadata_filter` (object, optional): Filter by metadata fields (JSON object)

#### Response (200 OK)

```json
{
  "capability": "data-analysis",
  "providers": [
    {
      "registration_id": "cap-reg-abc123",
      "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
      "agent_name": "DataAnalyzer",
      "version": "1.0.0",
      "priority": 100,
      "status": "active",
      "routing_score": 0.85,
      "metadata": {
        "supported_formats": ["csv", "json", "xlsx"],
        "max_file_size": "10MB"
      },
      "health": {
        "status": "healthy",
        "last_check": "2025-09-10T15:30:00Z",
        "response_time": "1.2s"
      }
    }
  ],
  "routing_strategy": "priority",
  "total_providers": 1,
  "healthy_providers": 1
}
```

#### Error Responses

- **404 Not Found**: Capability has no providers
- **422 Unprocessable Entity**: Invalid routing strategy or metadata filter

### Unregister Agent Capability

**DELETE** `/api/v1/capabilities/{agent_id}/{capability}`

Remove capability registration for a specific agent.

#### Path Parameters

- `agent_id` (string, required): Agent identifier
- `capability` (string, required): Capability name to unregister

#### Response (204 No Content)

Capability registration successfully removed.

#### Error Responses

- **404 Not Found**: Agent or capability registration does not exist

### Update Capability Registration

**PUT** `/api/v1/capabilities/{registration_id}`

Update capability registration metadata or priority.

#### Path Parameters

- `registration_id` (string, required): Capability registration identifier

#### Request Body

```json
{
  "priority": 120,
  "metadata": {
    "supported_formats": ["csv", "json", "xlsx", "parquet"],
    "max_file_size": "50MB",
    "processing_time": "fast",
    "new_feature": "streaming_analysis"
  },
  "health_check_url": "/api/v1/config-agents/{agent_id}/health/detailed"
}
```

#### Response (200 OK)

```json
{
  "registration_id": "cap-reg-abc123",
  "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
  "capability": "data-analysis",
  "version": "1.0.0",
  "priority": 120,
  "status": "active",
  "updated_at": "2025-09-10T16:45:00Z",
  "metadata": {
    "supported_formats": ["csv", "json", "xlsx", "parquet"],
    "max_file_size": "50MB",
    "processing_time": "fast",
    "new_feature": "streaming_analysis"
  }
}
```

## Health Monitoring

### Capability Health Check

**GET** `/api/v1/capabilities/{capability}/health`

Check health status of all providers for a capability.

#### Path Parameters

- `capability` (string, required): Capability name

#### Response (200 OK)

```json
{
  "capability": "data-analysis",
  "overall_health": "healthy",
  "healthy_providers": 2,
  "total_providers": 3,
  "providers": [
    {
      "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
      "agent_name": "DataAnalyzer",
      "status": "healthy",
      "last_check": "2025-09-10T15:30:00Z",
      "response_time": "1.2s",
      "success_rate": 0.95
    },
    {
      "agent_id": "config-660f9511-f30c-52e5-b827-557766551111",
      "agent_name": "AdvancedAnalyzer",
      "status": "healthy",
      "last_check": "2025-09-10T15:29:45Z",
      "response_time": "2.1s",
      "success_rate": 0.98
    },
    {
      "agent_id": "config-770fA622-g41d-63f6-c938-668877662222",
      "agent_name": "BackupAnalyzer",
      "status": "unhealthy",
      "last_check": "2025-09-10T15:25:00Z",
      "error": "Connection timeout after 30s",
      "consecutive_failures": 3
    }
  ]
}
```

### Bulk Health Check

**GET** `/api/v1/capabilities/health`

Check health status of all capability registrations.

#### Query Parameters

- `include_healthy` (boolean, optional): Include healthy providers (default: true)
- `include_unhealthy` (boolean, optional): Include unhealthy providers
  (default: true)

#### Response (200 OK)

```json
{
  "overall_health": "degraded",
  "healthy_capabilities": 4,
  "degraded_capabilities": 2,
  "unhealthy_capabilities": 1,
  "capabilities": [
    {
      "capability": "data-analysis",
      "health": "healthy",
      "healthy_providers": 2,
      "total_providers": 2
    },
    {
      "capability": "image-processing",
      "health": "degraded",
      "healthy_providers": 1,
      "total_providers": 2
    },
    {
      "capability": "web-search",
      "health": "unhealthy",
      "healthy_providers": 0,
      "total_providers": 1
    }
  ]
}
```

## Routing Strategies

### Priority-Based Routing

Routes to the agent with highest priority score. Falls back to next
highest if primary is unavailable.

**Algorithm**: `routing_score = priority * health_multiplier * (1 - load_score)`

### Load-Balanced Routing

Distributes requests evenly across all healthy providers using weighted
round-robin.

**Weight Calculation**: `weight = priority / current_load`

### Least-Loaded Routing

Routes to the agent with lowest current load score.

**Selection**: Agent with minimum `load_score` among healthy providers

### Fastest-Response Routing

Routes to the agent with best average response time.

**Selection**: Agent with minimum `response_time_avg` among healthy providers

## Capability Naming Conventions

### Standard Capabilities

Common capability names follow these patterns:

- `data-analysis` - Analyze structured data
- `data-processing` - Transform or clean data
- `image-processing` - Manipulate images
- `text-analysis` - NLP and text processing
- `web-search` - Search web content
- `content-generation` - Create text/documents
- `translation` - Language translation
- `code-analysis` - Source code processing
- `report-generation` - Create formatted reports

### Custom Capabilities

Custom capability names should follow these guidelines:

- Use lowercase with hyphens (kebab-case)
- Be descriptive and specific (`sales-forecasting` not `forecasting`)
- Include scope prefix for domain-specific capabilities (`finance-risk-analysis`)
- Avoid version numbers in names (use version field instead)

## WebSocket Events

Real-time updates for capability changes:

**Connection**: `ws://localhost:8080/ws/capabilities`

**Event Types**:

```json
{
  "type": "capability_registered",
  "data": {
    "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
    "capability": "data-analysis",
    "priority": 100
  },
  "timestamp": "2025-09-10T16:45:00Z"
}
```

```json
{
  "type": "capability_health_changed",
  "data": {
    "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
    "capability": "data-analysis",
    "old_status": "healthy",
    "new_status": "unhealthy",
    "error": "Connection timeout"
  },
  "timestamp": "2025-09-10T16:45:00Z"
}
```

**Event Types**: `capability_registered`, `capability_unregistered`,
`capability_updated`, `capability_health_changed`

## Performance Metrics

The system tracks performance metrics for capability providers:

### Response Time Metrics

- `response_time_avg` - Average response time over last 1000 requests
- `response_time_p95` - 95th percentile response time
- `response_time_p99` - 99th percentile response time

### Load Metrics

- `load_score` - Current load from 0.0 (idle) to 1.0 (overloaded)
- `concurrent_requests` - Number of active requests
- `requests_per_minute` - Request rate over last minute
- `queue_depth` - Number of queued requests

### Reliability Metrics

- `success_rate` - Percentage of successful requests (last 1000 requests)
- `error_rate` - Percentage of failed requests
- `consecutive_failures` - Number of consecutive failed health checks
- `uptime_percentage` - Percentage uptime over last 24 hours

## Domain Types

### CapabilityRegistration

```typescript
interface CapabilityRegistration {
  registration_id: string;       // Unique registration identifier
  agent_id: string;             // Agent providing the capability
  capability: string;           // Capability name
  version: string;              // Capability version
  priority: number;             // Routing priority (0-1000)
  status: RegistrationStatus;   // Current status
  metadata: Record<string, any>; // Capability-specific metadata
  health_check_url?: string;    // Health check endpoint
  registered_at: string;        // ISO 8601 registration timestamp
  last_updated: string;         // ISO 8601 last update timestamp
  last_health_check: string;    // ISO 8601 last health check
}
```

### RegistrationStatus

- `active` - Registration is active and routing traffic
- `inactive` - Registration exists but not routing traffic
- `unhealthy` - Provider failed health checks
- `draining` - Provider is being gracefully removed

### CapabilityProvider

```typescript
interface CapabilityProvider {
  registration_id: string;
  agent_id: string;
  agent_name: string;
  version: string;
  priority: number;
  status: RegistrationStatus;
  routing_score: number;        // Calculated routing preference score
  metadata?: Record<string, any>;
  performance_metrics: PerformanceMetrics;
  health: HealthStatus;
}
```

## Integration Examples

### Automatic Registration (Configuration Agent)

Configuration agents automatically register their declared capabilities:

```yaml
---
name: DataAnalyzer
capabilities:
  - data-analysis
  - report-generation
capability_metadata:
  data-analysis:
    supported_formats: ["csv", "json", "xlsx"]
    max_file_size: "10MB"
    processing_time: "fast"
  report-generation:
    formats: ["pdf", "html", "json"]
    template_support: true
---
```

### Manual Registration (External Agent)

External systems can register capabilities via API:

```javascript
// Register capability for external agent
const response = await fetch('/api/v1/capabilities', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    agent_id: 'external-python-analyzer',
    capability: 'data-analysis',
    priority: 90,
    metadata: {
      supported_formats: ['csv', 'parquet', 'arrow'],
      max_file_size: '1GB',
      processing_time: 'medium'
    },
    health_check_url: 'https://analyzer.example.com/health'
  })
});
```

### Capability Discovery in Agent

```javascript
// Find agents that can handle image processing
const response = await fetch(
  '/api/v1/capabilities/image-processing?routing_strategy=fastest_response'
);
const { providers } = await response.json();

// Route request to best provider
const bestProvider = providers[0];
const result = await sendAgentMessage(bestProvider.agent_id, {
  performative: 'REQUEST',
  content: { action: 'resize_image', params: { width: 800, height: 600 } }
});
```

## Error Handling

### Common Error Responses

```json
{
  "error": "Capability not found",
  "code": "CAPABILITY_NOT_FOUND",
  "details": {
    "capability": "non-existent-capability",
    "available_capabilities": ["data-analysis", "image-processing"]
  },
  "timestamp": "2025-09-10T16:45:00Z",
  "request_id": "req_cap_abc123"
}
```

### Error Codes

- `CAPABILITY_NOT_FOUND` - Requested capability has no providers
- `AGENT_NOT_FOUND` - Agent in registration does not exist
- `REGISTRATION_NOT_FOUND` - Capability registration does not exist
- `INVALID_CAPABILITY_NAME` - Capability name violates naming conventions
- `DUPLICATE_REGISTRATION` - Agent already registered for capability
- `HEALTH_CHECK_FAILED` - Provider failed health verification
- `ROUTING_STRATEGY_INVALID` - Unknown routing strategy specified

## Related Documentation

- [Configuration Agent API](config-agents.md) - Configuration agents
  automatically register capabilities
- [Agent Messaging API](agent-messaging.md) - Send messages to capabilities
- [Memory System API](memory-integration.md) - Store capability usage patterns
- [ADR-0029](../adrs/0029-fipa-acl-lightweight-messaging.md) -
  Capability-based routing design
- [ADR-0011](../adrs/0011-capability-registration-in-code.md) - Original
  capability registration approach
