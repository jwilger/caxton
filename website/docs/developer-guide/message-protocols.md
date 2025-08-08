---
title: FIPA Message Protocols
layout: documentation
description: Comprehensive guide to FIPA message protocols used in Caxton for agent communication.
---

# FIPA Message Protocols

FIPA (Foundation for Intelligent Physical Agents) message protocols provide standardized communication patterns for multi-agent systems. This guide covers how Caxton implements and uses FIPA protocols for secure, semantic agent communication.

## Overview

FIPA ACL (Agent Communication Language) defines a set of message types called **performatives** that express the intent behind agent communications. Each performative has well-defined semantics that enable agents to understand not just what data is being sent, but why it's being sent and what response is expected.

### Core Benefits

- **Semantic Clarity**: Messages express intent, not just data
- **Interoperability**: Standard protocol enables integration with other FIPA systems
- **Proven Patterns**: 20+ years of research and real-world deployment
- **Rich Interaction Models**: Support for negotiations, auctions, and complex workflows

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

## Performatives

FIPA defines several performatives, each with specific semantics and usage patterns.

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

### Contract Net Protocol

Distributed task allocation through bidding:

```
Initiator → Participants: CFP (Call for Proposals)
Participants → Initiator: PROPOSE|REFUSE
Initiator → Winner: ACCEPT_PROPOSAL
Initiator → Others: REJECT_PROPOSAL
Winner → Initiator: INFORM (result) | FAILURE
```

**Implementation Example:**

```rust
// Task coordinator (initiator)
async fn run_contract_net_auction(
    coordinator: &FipaClient,
    task: TaskDescription
) -> Result<TaskResult> {
    let conversation_id = Uuid::new_v4().to_string();
    
    // Send CFP to all potential bidders
    let cfp = FipaMessage::cfp()
        .receiver("broadcast://compute_agents")
        .content(serde_json::to_value(&task)?)
        .conversation_id(&conversation_id)
        .reply_with("cfp_001")
        .reply_by(Utc::now() + chrono::Duration::minutes(2))
        .build();
    
    coordinator.send(cfp).await?;
    
    // Collect proposals
    let mut proposals = Vec::new();
    let deadline = Utc::now() + chrono::Duration::minutes(2);
    
    while Utc::now() < deadline {
        if let Ok(msg) = coordinator.receive_timeout(
            Duration::from_secs(30)
        ).await {
            if msg.conversation_id.as_ref() == Some(&conversation_id) {
                match msg.performative {
                    Performative::Propose => {
                        proposals.push((msg.sender.clone(), msg.content.clone()));
                    }
                    Performative::Refuse => {
                        println!("Agent {} refused to bid", msg.sender);
                    }
                    _ => {}
                }
            }
        }
    }
    
    // Select best proposal
    let winner = select_best_proposal(&proposals)?;
    
    // Send acceptance to winner
    let accept = FipaMessage::accept_proposal()
        .receiver(&winner.0)
        .conversation_id(&conversation_id)
        .build();
    coordinator.send(accept).await?;
    
    // Send rejections to others
    for (agent_id, _) in proposals.iter() {
        if agent_id != &winner.0 {
            let reject = FipaMessage::reject_proposal()
                .receiver(agent_id)
                .conversation_id(&conversation_id)
                .build();
            coordinator.send(reject).await?;
        }
    }
    
    // Wait for result
    let result_msg = coordinator.wait_for_message(|msg| {
        msg.conversation_id.as_ref() == Some(&conversation_id) &&
        msg.sender == winner.0 &&
        matches!(msg.performative, Performative::Inform | Performative::Failure)
    }).await?;
    
    match result_msg.performative {
        Performative::Inform => Ok(serde_json::from_value(result_msg.content)?),
        Performative::Failure => Err(result_msg.content["error"].as_str()
            .unwrap_or("Task failed").into()),
        _ => unreachable!()
    }
}

// Compute agent (participant)
async fn handle_cfp(agent: &FipaClient, cfp_message: FipaMessage) {
    let task: TaskDescription = match serde_json::from_value(cfp_message.content) {
        Ok(task) => task,
        Err(_) => {
            let refuse = FipaMessage::refuse()
                .receiver(&cfp_message.sender)
                .conversation_id(&cfp_message.conversation_id.unwrap())
                .content(json!({"reason": "invalid_task_format"}))
                .build();
            agent.send(refuse).await.unwrap();
            return;
        }
    };
    
    // Evaluate capability and generate proposal
    match evaluate_task_capability(&task).await {
        Some(proposal_details) => {
            let propose = FipaMessage::propose()
                .receiver(&cfp_message.sender)
                .conversation_id(&cfp_message.conversation_id.unwrap())
                .content(serde_json::to_value(proposal_details).unwrap())
                .build();
            agent.send(propose).await.unwrap();
        }
        None => {
            let refuse = FipaMessage::refuse()
                .receiver(&cfp_message.sender)
                .conversation_id(&cfp_message.conversation_id.unwrap())
                .content(json!({"reason": "insufficient_capability"}))
                .build();
            agent.send(refuse).await.unwrap();
        }
    }
}
```

## Advanced Features

### Conversation Management

Conversations group related messages and maintain context:

```rust
pub struct ConversationManager {
    conversations: HashMap<String, ConversationContext>,
}

impl ConversationManager {
    pub fn start_conversation(&mut self, protocol: &str) -> String {
        let conversation_id = Uuid::new_v4().to_string();
        let context = ConversationContext {
            id: conversation_id.clone(),
            protocol: protocol.to_string(),
            state: ConversationState::Started,
            participants: HashSet::new(),
            messages: Vec::new(),
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };
        
        self.conversations.insert(conversation_id.clone(), context);
        conversation_id
    }
    
    pub fn add_message(&mut self, message: &FipaMessage) {
        if let Some(conv_id) = &message.conversation_id {
            if let Some(context) = self.conversations.get_mut(conv_id) {
                context.participants.insert(message.sender.clone());
                context.participants.insert(message.receiver.clone());
                context.messages.push(message.clone());
                context.last_activity = Utc::now();
                
                // Update conversation state based on message
                self.update_conversation_state(context, message);
            }
        }
    }
    
    fn update_conversation_state(
        &self, 
        context: &mut ConversationContext, 
        message: &FipaMessage
    ) {
        match (&context.protocol[..], &message.performative) {
            ("fipa-request", Performative::Request) => {
                context.state = ConversationState::RequestSent;
            }
            ("fipa-request", Performative::Agree) => {
                context.state = ConversationState::Agreed;
            }
            ("fipa-request", Performative::Inform) => {
                context.state = ConversationState::Completed;
            }
            ("contract-net", Performative::Cfp) => {
                context.state = ConversationState::ProposalPhase;
            }
            ("contract-net", Performative::AcceptProposal) => {
                context.state = ConversationState::ExecutionPhase;
            }
            _ => {}
        }
    }
}
```

### Message Validation

Implement comprehensive validation for incoming messages:

```rust
pub struct FipaValidator;

impl FipaValidator {
    pub fn validate_message(message: &FipaMessage) -> Result<(), ValidationError> {
        // Basic field validation
        Self::validate_required_fields(message)?;
        Self::validate_performative_semantics(message)?;
        Self::validate_conversation_consistency(message)?;
        Self::validate_content_format(message)?;
        
        Ok(())
    }
    
    fn validate_required_fields(message: &FipaMessage) -> Result<(), ValidationError> {
        if message.sender.is_empty() {
            return Err(ValidationError::MissingField("sender"));
        }
        if message.receiver.is_empty() {
            return Err(ValidationError::MissingField("receiver"));
        }
        // Additional validations...
        Ok(())
    }
    
    fn validate_performative_semantics(
        message: &FipaMessage
    ) -> Result<(), ValidationError> {
        match message.performative {
            Performative::Request => {
                // REQUEST must have reply_with for correlation
                if message.reply_with.is_none() {
                    return Err(ValidationError::SemanticError(
                        "REQUEST performative requires reply_with field"
                    ));
                }
            }
            Performative::Inform => {
                // INFORM responding to a request must have in_reply_to
                if message.in_reply_to.is_none() && 
                   message.content.get("unsolicited").is_none() {
                    return Err(ValidationError::SemanticError(
                        "INFORM must specify in_reply_to or mark as unsolicited"
                    ));
                }
            }
            Performative::Propose => {
                // PROPOSE should be part of a conversation
                if message.conversation_id.is_none() {
                    return Err(ValidationError::SemanticError(
                        "PROPOSE should be part of a conversation"
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

### Error Handling

Robust error handling with FIPA semantics:

```rust
#[derive(Debug, Clone)]
pub enum FipaError {
    MessageDeliveryFailed(String),
    InvalidPerformative(String),
    ConversationTimeout(String),
    ProtocolViolation(String),
    AgentNotFound(String),
    ValidationError(String),
}

impl FipaError {
    pub fn to_fipa_message(&self, original_message: &FipaMessage) -> FipaMessage {
        match self {
            FipaError::MessageDeliveryFailed(reason) => {
                FipaMessage::failure()
                    .receiver(&original_message.sender)
                    .content(json!({
                        "error": "message_delivery_failed",
                        "reason": reason
                    }))
                    .in_reply_to(&original_message.reply_with)
                    .build()
            }
            FipaError::AgentNotFound(agent_id) => {
                FipaMessage::not_understood()
                    .receiver(&original_message.sender)
                    .content(json!({
                        "error": "agent_not_found",
                        "agent_id": agent_id
                    }))
                    .in_reply_to(&original_message.reply_with)
                    .build()
            }
            _ => {
                FipaMessage::failure()
                    .receiver(&original_message.sender)
                    .content(json!({
                        "error": "processing_error",
                        "details": self.to_string()
                    }))
                    .in_reply_to(&original_message.reply_with)
                    .build()
            }
        }
    }
}
```

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