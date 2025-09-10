---
title: "Quick Start Guide"
date: 2025-09-10
layout: page
categories: [Getting Started]
---

Get a multi-agent system running in under 3 minutes.

## Prerequisites

- Caxton installed and running ([Installation Guide](installation.md))
- Basic command-line familiarity

## 1. Start the Caxton Server

```bash
# Start the server (single instance mode)
caxton server start

# Or start with cluster coordination enabled
caxton server start --cluster --seeds node1:7946,node2:7946

# Verify it's running
caxton server status
```

You should see:

```text
✓ Server running at http://localhost:8080
✓ Dashboard available at http://localhost:8080/dashboard
✓ Metrics available at http://localhost:9090/metrics
✓ Cluster coordination: enabled (3 nodes)
```

> **Note**: Caxton uses a coordination-first architecture with no external
> dependencies. Each instance maintains its own local state while coordinating
> with other instances via the SWIM protocol. See
> [ADR-0014](../adr/0014-coordination-first-architecture.md) for details.

## 2. Deploy Example Agents

We'll deploy two simple agents that communicate with each other:

```bash
# Download example agents
curl -O https://caxton.io/examples/ping.wasm
curl -O https://caxton.io/examples/pong.wasm

# Deploy the agents
caxton deploy ping.wasm --name ping-agent
caxton deploy pong.wasm --name pong-agent
```

## 3. Watch Them Communicate

```bash
# Follow the logs to see the agents talking
caxton logs --agents ping-agent,pong-agent --follow
```

You'll see output like:

```text
[ping-agent] Sending ping to pong-agent
[pong-agent] Received ping, sending pong back
[ping-agent] Received pong, sending ping to pong-agent
[pong-agent] Received ping, sending pong back
...
```

Press `Ctrl+C` to stop following logs.

## 4. Explore Agent Status

```bash
# List all deployed agents
caxton agent list

# Get details about a specific agent
caxton agent info ping-agent

# Check agent metrics
caxton agent metrics ping-agent
```

## 5. Send a Message to an Agent

You can manually send messages to agents:

```bash
# Send a custom message
caxton message send \
  --to ping-agent \
  --performative inform \
  --content '{"text": "Hello from CLI!"}'
```

## 6. Deploy Your Own Agent

### JavaScript Agent Example

Create `hello.js`:

```javascript
// Simple JavaScript agent
export function handle_message(message) {
    console.log(`Received: ${JSON.stringify(message)}`);

    // Reply to the sender
    return {
        performative: 'inform',
        receiver: message.sender,
        content: {
            text: `Hello! You said: ${message.content.text}`
        }
    };
}
```

Compile to WebAssembly:

```bash
# Using Javy (JavaScript to WASM compiler)
javy compile hello.js -o hello.wasm

# Deploy your agent
caxton deploy hello.wasm --name hello-agent
```

### Python Agent Example

Create `hello.py`:

```python
import json

def handle_message(message_json):
    """Handle incoming FIPA messages"""
    message = json.loads(message_json)
    print(f"Received: {message}")

    # Create reply
    reply = {
        "performative": "inform",
        "receiver": message["sender"],
        "content": {
            "text": f"Hello! You said: {message['content']['text']}"
        }
    }

    return json.dumps(reply)
```

Compile to WebAssembly:

```bash
# Using Wasmtime-py
wasmtime-py compile hello.py -o hello.wasm

# Deploy your agent
caxton deploy hello.wasm --name hello-agent
```

## 7. Create a Multi-Agent Workflow

Deploy a coordinated task using Contract Net Protocol:

```bash
# Deploy a manager agent
caxton deploy manager.wasm --name task-manager

# Deploy worker agents
caxton deploy worker.wasm --name worker-1
caxton deploy worker.wasm --name worker-2
caxton deploy worker.wasm --name worker-3

# Initiate a task that will be distributed
caxton task create \
  --manager task-manager \
  --workers worker-1,worker-2,worker-3 \
  --task "Process dataset" \
  --protocol contract-net
```

Watch the negotiation:

```bash
caxton logs --agents task-manager,worker-1,worker-2,worker-3 --follow
```

## 8. Monitor with the Dashboard

Open your browser to http://localhost:8080/dashboard to see:

- Real-time agent status
- Message flow visualization
- Performance metrics
- Resource usage
- Conversation tracking

## 9. Clean Up

When you're done experimenting:

```bash
# Stop specific agents
caxton agent stop ping-agent
caxton agent stop pong-agent

# Remove agents
caxton agent remove ping-agent
caxton agent remove pong-agent

# Or stop all agents
caxton agent stop --all

# Stop the server
caxton server stop
```

## What's Next?

Now that you have Caxton running:

- **[Build Your First Agent](first-agent.md)** - Create custom agents in your
  preferred language
- **[Agent Communication Patterns](../developer-guide/message-protocols.md)** -
  Learn FIPA protocols
- **[Production Deployment](../operations/production-deployment.md)** - Scale to
  production
- **[API Reference](../developer-guide/api-reference.md)** - Full API
  documentation

## Common Patterns

### Request-Reply Pattern

```bash
# Agent that processes requests
caxton deploy processor.wasm --name processor

# Send request and wait for reply
caxton message send \
  --to processor \
  --performative request \
  --content '{"action": "calculate", "data": [1,2,3]}' \
  --wait-reply
```

### Pub-Sub Pattern

```bash
# Deploy publisher
caxton deploy publisher.wasm --name news-publisher

# Deploy subscribers
caxton deploy subscriber.wasm --name subscriber-1
caxton deploy subscriber.wasm --name subscriber-2

# Subscribe agents to topics
caxton subscribe subscriber-1 --topic news
caxton subscribe subscriber-2 --topic news
```

### Task Distribution

```bash
# Use Contract Net Protocol for task allocation
caxton task distribute \
  --task "analyze-logs" \
  --data @logfile.json \
  --protocol contract-net \
  --timeout 30s
```

## Tips

1. **Use `--help`** with any command for detailed options
2. **Enable debug logging** with `RUST_LOG=debug caxton ...`
3. **Check metrics** at http://localhost:9090/metrics for Prometheus
4. **View traces** in Jaeger UI if configured
5. **Use `caxton doctor`** to diagnose issues
