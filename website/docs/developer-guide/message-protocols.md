---
title: FIPA Message Protocols
layout: documentation
description: Comprehensive guide to FIPA message protocols used in Caxton for agent communication.
---

# Message Protocols for Service Communication

This guide explains how to send messages between services in Caxton using FIPA-based protocols. Think of it as a standardized way for your services to talk to each other - like having a common language for requests, responses, and negotiations.

## What Are Message Protocols?

Message protocols define how services communicate with each other. Instead of just sending raw data, these protocols include information about:

- **What** you're sending (the data)
- **Why** you're sending it (request, notification, proposal, etc.)
- **What response** you expect (if any)
- **How long** to wait for a response

FIPA (Foundation for Intelligent Physical Agents) provides proven patterns that have been used in distributed systems for over 20 years. Caxton implements a pragmatic subset focused on practical service communication needs.

### Why Use Structured Message Types?

- **Clear Intent**: Both sender and receiver know exactly what's expected
- **Error Handling**: Standardized ways to handle failures and rejections
- **Conversations**: Group related messages together for complex workflows
- **Timeouts**: Built-in support for request deadlines

## FIPA Message Structure

Every FIPA message in Caxton follows this standardized structure:

```json
{
  "performative": "request",
  "sender": "agent_123",
  "receiver": "agent_456",
  "content": {
    "action": "process_data",
    "parameters": {"data_type": "json", "size": 1024}
  },
  "conversation_id": "conv_789",
  "reply_with": "msg_001",
  "in_reply_to": null,
  "ontology": "caxton-v1",
  "language": "json",
  "protocol": "fipa-request",
  "reply_by": "2024-01-15T10:35:00Z"
}
```

### Required Fields

- **performative**: The message type (see Performatives section)
- **sender**: Unique identifier of the sending agent
- **receiver**: Unique identifier of the receiving agent
- **content**: The actual message payload

### Optional Fields

- **conversation_id**: Groups related messages into conversations
- **reply_with**: Unique ID for this message, used for correlation
- **in_reply_to**: References the message this is responding to
- **ontology**: Semantic framework for interpreting content
- **language**: Content encoding format (json, xml, etc.)
- **protocol**: Interaction protocol being used
- **reply_by**: Deadline for response (ISO 8601 format)

## Common Message Types

### When to Use Each Message Type

| Message Type | Use When | Expected Response |
|-------------|----------|------------------|
| `INFORM` | Sharing information, status updates | None (fire-and-forget) |
| `REQUEST` | Asking another service to do something | `AGREE`/`REFUSE`, then `INFORM`/`FAILURE` |
| `QUERY_IF` | Asking "is this true?" | `INFORM` with true/false |
| `QUERY_REF` | Asking "what is the value of X?" | `INFORM` with the data |
| `CFP` | Starting a bidding process | `PROPOSE` or `REFUSE` |
| `PROPOSE` | Making an offer or bid | `ACCEPT_PROPOSAL`/`REJECT_PROPOSAL` |

### Detailed Message Types

### Information Exchange

#### INFORM
Shares information without expecting a response.

```rust
let message = FipaMessage::inform()
    .sender("weather_agent")
    .receiver("display_agent")
    .content(json!({
        "temperature": 22,
        "humidity": 65,
        "conditions": "partly_cloudy"
    }))
    .build();
```

**Use Cases:**
- Status updates
- Event notifications
- Data broadcasting

#### QUERY_IF
Asks whether a given proposition is true.

```rust
let message = FipaMessage::query_if()
    .sender("scheduler")
    .receiver("resource_manager")
    .content(json!({
        "query": "available_memory > 1024MB"
    }))
    .reply_with("query_001")
    .build();
```

**Expected Response:** INFORM with true/false

#### QUERY_REF
Requests the value of a reference or variable.

```rust
let message = FipaMessage::query_ref()
    .sender("client")
    .receiver("database_agent")
    .content(json!({
        "query": "SELECT * FROM users WHERE active = true"
    }))
    .reply_with("db_query_001")
    .build();
```

**Expected Response:** INFORM with requested data

### Action Requests

#### REQUEST
Asks another agent to perform an action.

```rust
let message = FipaMessage::request()
    .sender("user_interface")
    .receiver("file_processor")
    .content(json!({
        "action": "process_file",
        "file_path": "/data/input.csv",
        "format": "csv",
        "validation": true
    }))
    .reply_with("process_req_001")
    .reply_by("2024-01-15T10:35:00Z")
    .build();
```

**Expected Responses:**
- AGREE (will perform the action)
- REFUSE (cannot/will not perform)
- NOT_UNDERSTOOD (cannot parse request)

### Negotiation

#### PROPOSE
Offers to perform an action or provide information, often with conditions.

```rust
let message = FipaMessage::propose()
    .sender("compute_agent")
    .receiver("task_coordinator")
    .content(json!({
        "proposal_id": "prop_001",
        "action": "run_analysis",
        "conditions": {
            "estimated_time": "5 minutes",
            "cost": 10,
            "confidence": 0.95,
            "requirements": ["GPU", "8GB_RAM"]
        }
    }))
    .conversation_id("task_negotiation_001")
    .build();
```

**Expected Responses:**
- ACCEPT_PROPOSAL
- REJECT_PROPOSAL
- COUNTER_PROPOSE

#### CFP (Call for Proposals)
Initiates a bidding process for a task or service.

```rust
let message = FipaMessage::cfp()
    .sender("task_manager")
    .receiver("all_compute_agents") // Broadcast
    .content(json!({
        "task_id": "analysis_task_001",
        "description": "Analyze 1TB dataset for patterns",
        "deadline": "2024-01-15T18:00:00Z",
        "requirements": {
            "memory": "16GB+",
            "storage": "100GB+",
            "capabilities": ["machine_learning", "data_processing"]
        },
        "evaluation_criteria": ["cost", "time", "accuracy"]
    }))
    .conversation_id("auction_001")
    .reply_with("cfp_001")
    .reply_by("2024-01-15T12:00:00Z")
    .build();
```

**Expected Response:** PROPOSE (or REFUSE if cannot bid)

### Response Management

#### AGREE/REFUSE
Responds to requests indicating willingness to perform actions.

```rust
// Positive response
let agree = FipaMessage::agree()
    .sender("processor")
    .receiver("client")
    .content(json!({
        "accepted_action": "data_processing",
        "estimated_completion": "2024-01-15T10:40:00Z",
        "tracking_id": "proc_001"
    }))
    .in_reply_to("process_req_001")
    .build();

// Negative response
let refuse = FipaMessage::refuse()
    .sender("processor")
    .receiver("client")
    .content(json!({
        "reason": "insufficient_memory",
        "details": "Required 8GB, available 4GB",
        "alternative": "retry_after_5_minutes"
    }))
    .in_reply_to("process_req_001")
    .build();
```

#### ACCEPT_PROPOSAL/REJECT_PROPOSAL
Responds to proposals in negotiation scenarios.

```rust
let accept = FipaMessage::accept_proposal()
    .sender("task_coordinator")
    .receiver("compute_agent")
    .content(json!({
        "proposal_id": "prop_001",
        "contract_id": "contract_001",
        "start_immediately": true
    }))
    .conversation_id("task_negotiation_001")
    .build();
```

## Interaction Protocols

FIPA defines several standard interaction protocols that combine multiple performatives into coordinated patterns.

### Request Protocol

Simple request-response interaction:

```
Client → Server: REQUEST
Server → Client: AGREE|REFUSE
[If AGREE]
Server → Client: INFORM (result) | FAILURE
```

**Implementation Example:**

```rust
// Client side
async fn request_data_processing(client: &FipaClient) -> Result<ProcessingResult> {
    let request = FipaMessage::request()
        .receiver("data_processor")
        .content(json!({"file": "data.csv", "operation": "analyze"}))
        .reply_with("req_001")
        .build();

    client.send(request).await?;

    // Wait for AGREE/REFUSE
    let response = client.wait_for_reply("req_001").await?;
    match response.performative {
        Performative::Agree => {
            // Wait for result
            let result = client.wait_for_message(|msg| {
                msg.performative == Performative::Inform &&
                msg.in_reply_to.as_ref() == Some(&"req_001")
            }).await?;
            Ok(serde_json::from_value(result.content)?)
        }
        Performative::Refuse => {
            let reason = response.content["reason"].as_str().unwrap_or("unknown");
            Err(format!("Request refused: {}", reason).into())
        }
        _ => Err("Unexpected response".into())
    }
}

// Server side
async fn handle_request(server: &FipaServer, message: FipaMessage) {
    match validate_request(&message.content) {
        Ok(_) => {
            // Send agreement
            let agree = FipaMessage::agree()
                .receiver(&message.sender)
                .in_reply_to(&message.reply_with.unwrap())
                .build();
            server.send(agree).await.unwrap();

            // Process and send result
            match process_data(&message.content).await {
                Ok(result) => {
                    let inform = FipaMessage::inform()
                        .receiver(&message.sender)
                        .content(result)
                        .in_reply_to(&message.reply_with.unwrap())
                        .build();
                    server.send(inform).await.unwrap();
                }
                Err(e) => {
                    let failure = FipaMessage::failure()
                        .receiver(&message.sender)
                        .content(json!({"error": e.to_string()}))
                        .in_reply_to(&message.reply_with.unwrap())
                        .build();
                    server.send(failure).await.unwrap();
                }
            }
        }
        Err(e) => {
            let refuse = FipaMessage::refuse()
                .receiver(&message.sender)
                .content(json!({"reason": e.to_string()}))
                .in_reply_to(&message.reply_with.unwrap())
                .build();
            server.send(refuse).await.unwrap();
        }
    }
}
```

### Bidding System (Contract Net Protocol)

The Contract Net Protocol is a bidding system where one service asks others to bid on a task:

1. **Coordinator** sends `CFP` (Call for Proposals) to multiple services
2. **Services** respond with `PROPOSE` (their bid) or `REFUSE` (can't do it)
3. **Coordinator** picks the best bid and sends `ACCEPT_PROPOSAL` to winner, `REJECT_PROPOSAL` to others
4. **Winner** does the work and sends `INFORM` (result) or `FAILURE`

**Simple Example:**
```rust
// Ask for bids on a data processing task
let cfp = FipaMessage::cfp()
    .receiver("broadcast://data_processors")
    .content(json!({
        "task": "process_large_dataset",
        "size_gb": 100,
        "deadline": "2024-01-15T18:00:00Z"
    }))
    .conversation_id("auction_001")
    .reply_by("2024-01-15T12:00:00Z")
    .build();

// Services respond with their bids
let proposal = FipaMessage::propose()
    .content(json!({
        "cost": 50,
        "estimated_time": "2 hours",
        "confidence": 0.95
    }))
    .build();
```

## Pragmatic FIPA Implementation

Caxton implements a pragmatic subset of FIPA protocols, focusing on the most commonly needed message patterns while maintaining compatibility with the broader FIPA ecosystem. Our approach prioritizes:

- **Simplicity**: Use only the message types you actually need
- **Performance**: Efficient serialization and routing
- **Reliability**: Built-in timeouts and error handling
- **Debuggability**: Clear message tracing and logging

This pragmatic approach follows our architectural decision (see [ADR-0012]({{ '/adr/0012-pragmatic-fipa-subset/' | relative_url }})) to adopt proven FIPA patterns without the complexity of full academic FIPA implementations.

## Advanced Features

*For complex implementations that require detailed conversation management, message validation, and error handling patterns, see the [Advanced FIPA Implementation Guide](/docs/advanced/fipa-advanced).*

## Quick Start Examples

### Send a Simple Request

```rust
// Ask a service to process data
let request = FipaMessage::request()
    .receiver("data_processor")
    .content(json!({
        "action": "analyze",
        "file": "data.csv"
    }))
    .reply_with("req_001")
    .reply_by("2024-01-15T10:35:00Z")
    .build();

// Send and wait for response
let response = client.send_and_wait(request).await?;
match response.performative {
    Performative::Agree => {
        // Service accepted, wait for result
        let result = client.wait_for_inform("req_001").await?;
        println!("Result: {}", result.content);
    }
    Performative::Refuse => {
        let reason = response.content["reason"].as_str().unwrap();
        eprintln!("Request refused: {}", reason);
    }
    _ => {}
}
```

### Send Information Updates

```rust
// Notify other services of status change
let status_update = FipaMessage::inform()
    .receiver("monitoring_service")
    .content(json!({
        "service": "data_processor",
        "status": "ready",
        "capacity": 80
    }))
    .build();

client.send(status_update).await?;
```

### Ask a Simple Question

```rust
// Check if a service is available
let health_check = FipaMessage::query_if()
    .receiver("target_service")
    .content(json!({
        "query": "status == 'healthy'"
    }))
    .reply_with("health_001")
    .build();

let response = client.send_and_wait(health_check).await?;
let is_healthy = response.content["result"].as_bool().unwrap_or(false);
```

## Getting Started

These examples should cover most common service communication needs. For advanced patterns like complex negotiations, distributed coordination, or custom protocol implementations, consult the [Advanced FIPA Guide](/docs/advanced/fipa-advanced) or our API documentation.

## Best Practices

### Message Design

1. **Use Appropriate Performatives**: Choose the performative that best matches your intent
2. **Include Context**: Use conversation_id for related messages
3. **Set Timeouts**: Include reply_by for time-sensitive requests
4. **Handle Errors**: Always handle REFUSE, FAILURE, and NOT_UNDERSTOOD responses

### Performance Optimization

1. **Batch Messages**: Group related communications when possible
2. **Use Efficient Serialization**: Consider binary formats for high-throughput scenarios
3. **Implement Caching**: Cache frequently accessed ontologies and schemas
4. **Monitor Conversations**: Clean up expired conversations to prevent memory leaks

### Security Considerations

1. **Validate All Inputs**: Never trust message content without validation
2. **Implement Authentication**: Verify sender identity for sensitive operations
3. **Rate Limiting**: Prevent message flooding attacks
4. **Audit Logging**: Log all messages for security analysis

## Troubleshooting

### Common Issues

#### Messages Not Being Delivered
```bash
# Check agent connectivity
curl -X GET http://localhost:8080/api/v1/agents/agent_123/status

# Verify message format
curl -X POST http://localhost:8080/api/v1/messages/validate \
  -H "Content-Type: application/json" \
  -d '{"performative": "request", ...}'
```

#### Conversation Timeouts
```rust
// Set reasonable timeouts
let message = FipaMessage::request()
    .reply_by(Utc::now() + chrono::Duration::minutes(5))
    .build();

// Implement retry logic
async fn send_with_retry(
    client: &FipaClient,
    message: FipaMessage
) -> Result<FipaMessage> {
    for attempt in 1..=3 {
        match client.send_and_wait(message.clone(), Duration::from_secs(30)).await {
            Ok(response) => return Ok(response),
            Err(e) if attempt < 3 => {
                tokio::time::sleep(Duration::from_secs(2_u64.pow(attempt))).await;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

#### Protocol Violations
```rust
// Implement protocol state machines
pub enum RequestProtocolState {
    Initial,
    RequestSent,
    Agreed,
    Completed,
    Failed,
}

impl RequestProtocolState {
    pub fn is_valid_transition(&self, performative: &Performative) -> bool {
        match (self, performative) {
            (RequestProtocolState::Initial, Performative::Request) => true,
            (RequestProtocolState::RequestSent, Performative::Agree) => true,
            (RequestProtocolState::RequestSent, Performative::Refuse) => true,
            (RequestProtocolState::Agreed, Performative::Inform) => true,
            (RequestProtocolState::Agreed, Performative::Failure) => true,
            _ => false,
        }
    }
}
```

### Debugging Tools

#### Message Tracing
```rust
// Enable detailed message logging
pub struct MessageTracer {
    logs: Vec<MessageLogEntry>,
}

impl MessageTracer {
    pub fn trace_message(&mut self, message: &FipaMessage, direction: Direction) {
        let entry = MessageLogEntry {
            timestamp: Utc::now(),
            direction,
            message: message.clone(),
            conversation_id: message.conversation_id.clone(),
            protocol: message.protocol.clone(),
        };

        self.logs.push(entry);

        // Log to structured logger
        info!(
            message.sender = %message.sender,
            message.receiver = %message.receiver,
            message.performative = ?message.performative,
            message.conversation_id = ?message.conversation_id,
            direction = ?direction,
            "FIPA message traced"
        );
    }
}
```

#### Conversation Analysis
```rust
// Analyze conversation patterns
pub fn analyze_conversation(
    conversation_id: &str,
    messages: &[FipaMessage]
) -> ConversationAnalysis {
    let mut analysis = ConversationAnalysis::new(conversation_id);

    for message in messages {
        analysis.add_message(message);
    }

    // Detect patterns
    analysis.detect_protocol_violations();
    analysis.calculate_latency_metrics();
    analysis.identify_performance_bottlenecks();

    analysis
}
```

This comprehensive guide provides the foundation for implementing robust FIPA-based agent communication in Caxton. The protocol's semantic richness enables sophisticated agent interactions while maintaining interoperability with existing multi-agent systems.
