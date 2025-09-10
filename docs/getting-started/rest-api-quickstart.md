---
title: "REST API Quick Start Guide"
date: 2025-09-10
layout: page
categories: [Getting Started]
---

**Last Updated**: 2025-01-14 **Requirements**: Caxton server running on
localhost:8080 **Tools**: curl (or any HTTP client)

## Overview

This guide provides working examples for all implemented REST API endpoints in
Caxton. Copy and paste these commands to interact with your Caxton server
immediately.

## Prerequisites

1. Start Caxton server:

```bash
cargo run --release
# Server starts on http://localhost:8080
```

1. Verify server is running:

```bash
curl http://localhost:8080/api/v1/health
```

Expected response:

```json
{"status":"healthy"}
```

## Working Examples

### 1. Health Check

**Purpose**: Verify server is responsive and healthy

```bash
# Basic health check
curl http://localhost:8080/api/v1/health

# Pretty-printed JSON
curl http://localhost:8080/api/v1/health | jq '.'

# With verbose output
curl -v http://localhost:8080/api/v1/health

# In a monitoring script
if curl -f -s http://localhost:8080/api/v1/health > /dev/null; then
    echo "Server is healthy"
else
    echo "Server is down"
fi
```

**Expected Response**:

```json
{
  "status": "healthy"
}
```

### 2. List All Agents

**Purpose**: Retrieve list of deployed agents

```bash
# List all agents (initially empty)
curl http://localhost:8080/api/v1/agents

# Pretty-printed
curl http://localhost:8080/api/v1/agents | jq '.'

# Save to file
curl http://localhost:8080/api/v1/agents > agents.json

# Check if any agents exist
AGENT_COUNT=$(curl -s http://localhost:8080/api/v1/agents | jq '. | length')
echo "Number of agents: $AGENT_COUNT"
```

**Expected Response** (no agents deployed):

```json
[]
```

**Expected Response** (with agents):

```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "example-agent",
    "wasm_module": "...",
    "resource_limits": {
      "max_memory_bytes": 10485760,
      "max_cpu_millis": 1000000,
      "max_execution_time_ms": 5000
    }
  }
]
```

### 3. Deploy New Agent

**Purpose**: Deploy a WebAssembly agent with resource constraints

```bash
# Deploy with minimal configuration
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "example-agent",
    "wasm_module": "AGFzbQEAAAA=",
    "resource_limits": {
      "max_memory_bytes": 10485760,
      "max_cpu_millis": 1000000,
      "max_execution_time_ms": 5000
    }
  }'

# Deploy with pretty response
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "processor-agent",
    "wasm_module": "AGFzbQEAAAA=",
    "resource_limits": {
      "max_memory_bytes": 52428800,
      "max_cpu_millis": 5000000,
      "max_execution_time_ms": 10000
    }
  }' | jq '.'

# Deploy from file
cat > agent-config.json << EOF
{
  "name": "file-based-agent",
  "wasm_module": "AGFzbQEAAAA=",
  "resource_limits": {
    "max_memory_bytes": 10485760,
    "max_fuel": 1000000,
    "max_execution_time_ms": 5000
  }
}
EOF

curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d @agent-config.json

# Deploy with base64-encoded WASM from actual file
WASM_B64=$(base64 -w 0 < my-agent.wasm)
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"real-agent\",
    \"wasm_module\": \"$WASM_B64\",
    \"resource_limits\": {
      \"max_memory_bytes\": 10485760,
      \"max_cpu_millis\": 1000000,
      \"max_execution_time_ms\": 5000
    }
  }"

# Capture agent ID from response
AGENT_ID=$(curl -s -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "tracked-agent",
    "wasm_module": "AGFzbQEAAAA=",
    "resource_limits": {
      "max_memory_bytes": 10485760,
      "max_cpu_millis": 1000000,
      "max_execution_time_ms": 5000
    }
  }' | jq -r '.id')
echo "Created agent with ID: $AGENT_ID"
```

**Expected Success Response** (201 Created):

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "example-agent",
  "wasm_module": "AGFzbQEAAAA=",
  "resource_limits": {
    "max_memory_bytes": 10485760,
    "max_fuel": 1000000,
    "max_execution_time_ms": 5000
  }
}
```

### 4. Get Agent by ID

**Purpose**: Retrieve details of a specific agent

```bash
# Get specific agent (replace with actual ID)
AGENT_ID="550e8400-e29b-41d4-a716-446655440000"
curl http://localhost:8080/api/v1/agents/$AGENT_ID

# With error handling
AGENT_ID="550e8400-e29b-41d4-a716-446655440000"
RESPONSE=$(curl -s -w "\n%{http_code}" http://localhost:8080/api/v1/agents/$AGENT_ID)
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)

if [ "$HTTP_CODE" -eq 200 ]; then
    echo "Agent found:"
    echo "$BODY" | jq '.'
else
    echo "Error $HTTP_CODE:"
    echo "$BODY" | jq '.'
fi

# Get all agent details in a loop
for AGENT_ID in $(curl -s http://localhost:8080/api/v1/agents | jq -r '.[].id'); do
    echo "Agent $AGENT_ID:"
    curl -s http://localhost:8080/api/v1/agents/$AGENT_ID | jq '.name'
done
```

**Expected Success Response** (200 OK):

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "example-agent",
  "wasm_module": "AGFzbQEAAAA=",
  "resource_limits": {
    "max_memory_bytes": 10485760,
    "max_fuel": 1000000,
    "max_execution_time_ms": 5000
  }
}
```

**Expected Error Response** (404 Not Found):

```json
{
  "error": "Agent not found",
  "details": {
    "agent_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

## Error Handling Examples

### Validation Errors (400 Bad Request)

```bash
# Empty agent name
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "",
    "wasm_module": "AGFzbQEAAAA=",
    "resource_limits": {
      "max_memory_bytes": 10485760,
      "max_cpu_millis": 1000000,
      "max_execution_time_ms": 5000
    }
  }'
# Response: {"error":"Validation error","details":{"field":"name","reason":"cannot be empty"}}

# Zero memory limit
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-agent",
    "wasm_module": "AGFzbQEAAAA=",
    "resource_limits": {
      "max_memory_bytes": 0,
      "max_cpu_millis": 1000000,
      "max_execution_time_ms": 5000
    }
  }'
# Response: {"error":"Validation error","details":{"field":"max_memory_bytes","reason":"must be greater than 0"}}

# Malformed JSON
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{"name": "bad-json"'
# Response: {"error":"Invalid JSON","details":{"reason":"EOF while parsing an object"}}
```

### Not Found Errors (404)

```bash
# Non-existent agent
curl -i http://localhost:8080/api/v1/agents/00000000-0000-0000-0000-000000000000
# HTTP/1.1 404 Not Found
# {"error":"Agent not found","details":{"agent_id":"00000000-0000-0000-0000-000000000000"}}

# Invalid UUID format
curl -i http://localhost:8080/api/v1/agents/not-a-uuid
# HTTP/1.1 404 Not Found
# {"error":"Invalid agent ID format","details":{"agent_id":"not-a-uuid"}}
```

## Complete Workflow Example

```bash
#!/bin/bash
# Complete agent deployment and verification workflow

# 1. Check server health
echo "Checking server health..."
if ! curl -f -s http://localhost:8080/api/v1/health > /dev/null; then
    echo "Server is not running!"
    exit 1
fi
echo "Server is healthy"

# 2. List existing agents
echo -e "\nExisting agents:"
curl -s http://localhost:8080/api/v1/agents | jq '.'

# 3. Deploy new agent
echo -e "\nDeploying new agent..."
AGENT_JSON=$(curl -s -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "workflow-example-agent",
    "wasm_module": "AGFzbQEAAAA=",
    "resource_limits": {
      "max_memory_bytes": 10485760,
      "max_cpu_millis": 1000000,
      "max_execution_time_ms": 5000
    }
  }')

AGENT_ID=$(echo "$AGENT_JSON" | jq -r '.id')
echo "Created agent with ID: $AGENT_ID"

# 4. Verify agent exists
echo -e "\nVerifying agent deployment..."
curl -s http://localhost:8080/api/v1/agents/$AGENT_ID | jq '.'

# 5. List all agents to confirm
echo -e "\nAll agents after deployment:"
curl -s http://localhost:8080/api/v1/agents | jq '.'
```

## Resource Limit Guidelines

### Memory Limits

- **Minimum**: 1 MB (1048576 bytes)
- **Typical**: 10 MB (10485760 bytes)
- **Large**: 50 MB (52428800 bytes)
- **Maximum**: 100 MB (104857600 bytes)

### CPU Fuel

- **Minimum**: 100000 units
- **Typical**: 1000000 units
- **Compute-intensive**: 5000000 units
- **Maximum**: 10000000 units

### Execution Time

- **Fast**: 1000 ms
- **Normal**: 5000 ms
- **Long-running**: 10000 ms
- **Maximum**: 30000 ms

## Testing with HTTPie (Alternative to curl)

```bash
# Install HTTPie
pip install httpie

# Health check
http GET localhost:8080/api/v1/health

# List agents
http GET localhost:8080/api/v1/agents

# Deploy agent
http POST localhost:8080/api/v1/agents \
  name="httpie-agent" \
  wasm_module="AGFzbQEAAAA=" \
  resource_limits:='{"max_memory_bytes":10485760,"max_cpu_millis":1000000,"max_execution_time_ms":5000}'

# Get agent
http GET localhost:8080/api/v1/agents/550e8400-e29b-41d4-a716-446655440000
```

## Troubleshooting

### Connection Refused

```bash
curl: (7) Failed to connect to localhost port 8080: Connection refused
```

**Solution**: Ensure Caxton server is running with `cargo run --release`

### 404 Not Found on Valid Endpoints

```bash
{"error":"Not found","details":{}}
```

**Solution**: Check the URL path is exactly `/api/v1/...` (note the v1)

### Invalid JSON Errors

```bash
{"error":"Invalid JSON","details":{"reason":"..."}}
```

**Solution**: Validate JSON with `jq` before sending:

```bash
echo '{"your":"json"}' | jq '.' # Will error if invalid
```

### Resource Limit Validation Errors

```bash
{"error":"Validation error","details":{"field":"max_memory_bytes","reason":"must be greater than 0"}}
```

**Solution**: Ensure all resource limits are positive integers

## Next Steps

1. **Integrate with CLI**: Use these endpoints in Story 008 CLI implementation
2. **Add Authentication**: Implement auth proxy for production use
3. **Monitor Health**: Set up automated health checks
4. **Script Deployments**: Automate agent deployment workflows

## Related Documentation

- [API Implementation Status](../api/implementation-status.md) - What's
  implemented vs planned
- [API Reference](../developer-guide/api-reference.md) - Complete API
  specification
- [Operational Runbook](../operations/operational-runbook.md) - Production
  operations guide
