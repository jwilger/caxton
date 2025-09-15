---
title: "Agent Messaging Protocol Concepts"
description:
  "Understanding Caxton's lightweight agent messaging implementation for
  standardized agent communication with context-aware routing"
date: 2025-01-13
categories: [Concepts, Messaging, Communication]
layout: concept
level: intermediate
---

## Why Agent Messaging Protocols Matter

Agent messaging protocols provide a **standardized vocabulary** for agents
to communicate
effectively. Think of it as establishing **common protocols** for agent
conversations—similar to how HTTP provides standards for web communication.

Without standard communication protocols, agents would need custom integration
for each interaction. Agent messaging protocols enable **interoperability**
and **semantic
clarity** in multi-agent systems.

## Core Communication Concepts

### Performatives as Communication Intent

**Performatives** are the **speech acts** that define what an agent intends
to accomplish with a message. They transform natural language into structured
communication with clear expectations.

**Request Pattern**:

```text
Human Communication: "Could you please analyze this data?"
Agent Equivalent: REQUEST(capability: data-analysis, content:
"analyze this data")
```

**Information Sharing Pattern**:

```text
Human Communication: "Here are the results from my analysis"
Agent Equivalent: INFORM(content: "analysis results: [data]")
```

**Question Pattern**:

```text
Human Communication: "What is the current status of the project?"
Agent Equivalent: QUERY(content: "project status request")
```

### Caxton's Practical Agent Messaging Subset

Rather than implementing complex messaging specifications, Caxton focuses on
**essential communication patterns** that
serve 90% of agent interaction needs:

**Core Performatives (Production Ready)**:

- **REQUEST**: "Please do this action"
- **INFORM**: "Here is information/results"
- **QUERY**: "What information can you provide?"
- **PROPOSE**: "I suggest this approach"
- **ACCEPT_PROPOSAL**: "I agree with your suggestion"
- **REJECT_PROPOSAL**: "I don't agree with your suggestion"
- **FAILURE**: "I couldn't complete the requested action"
- **NOT_UNDERSTOOD**: "I don't understand your message"

**Deferred Performatives (Future Versions)**:

- **CFP** (Call for Proposals): Complex bidding scenarios
- **SUBSCRIBE**: Event notification systems
- **CANCEL**: Message cancellation protocols
- **CONFIRM/DISCONFIRM**: Confirmation workflows

This subset approach enables **immediate productivity** while maintaining a
**clear evolution path** for more complex scenarios.

## Context-Aware Communication

### Messages as Context Sources

In Caxton's architecture, agent messages serve a **dual purpose**:

1. **Communication Medium**: Enable structured agent interaction
2. **Context Source**: Provide conversation history for intelligent responses

This integration means every message contributes to building **conversational
intelligence** that improves agent responses over time.

### Conversation Threading

**Thread Continuity** enables complex, multi-turn interactions:

```yaml
# Initial request establishes conversation thread
performative: REQUEST
conversation_id: "project_analysis_001"
reply_with: "req_001"
content: "Analyze Q3 sales performance by region"

# Follow-up maintains thread context
performative: QUERY
conversation_id: "project_analysis_001"  # Same thread
in_reply_to: "inform_results_001"        # References previous response
reply_with: "query_002"
content: "What factors drove the 15% increase in the West region?"
```

**Thread Benefits**:

- **Context Accumulation**: Each message builds on previous conversation
- **Coherent Interaction**: Agents understand conversation flow
- **Reduced Redundancy**: No need to repeat previously provided information
- **Intelligent Routing**: Continue with same agent when beneficial

### Capability-Based Routing

Instead of sending messages to **specific agents**, Caxton routes messages to
**capabilities**. This enables flexible, scalable agent architectures:

**Traditional Direct Routing**:

```yaml
# Tightly coupled to specific agent
to: "sales_agent_instance_3"
performative: REQUEST
content: "Generate sales report"
```

**Capability-Based Routing**:

```yaml
# Flexible routing to any capable agent
capability: "sales-reporting"
performative: REQUEST
content: "Generate sales report"
```

**Routing Benefits**:

- **Load Distribution**: Requests distributed across capable agents
- **Fault Tolerance**: System continues if specific agents are unavailable
- **Scalability**: Add more agents without changing message routing
- **Specialization**: Route to agents optimized for specific tasks

## Message Structure and Flow

### Enhanced Message Format

Caxton extends basic agent messaging with **context integration** and
**performance
optimization**:

```yaml
# Core Agent Messaging Fields
performative: REQUEST
capability: "data-analysis"
conversation_id: "analysis_session_001"
reply_with: "req_analysis_001"
content: |
  Analyze customer churn patterns in our subscription data.
  Focus on customers who cancelled in Q3 and identify
  common characteristics that might predict future churn.

# Context Enhancement Fields
context_hints:
  conversation_depth: 3 # Include last 3 messages for context
  memory_relevance: high # Search memory for related patterns
  tool_context_required: true # Include relevant tool configurations

# Performance Fields
priority: normal # Message processing priority
timeout_ms: 30000 # Maximum processing time
expected_response_size: medium # Help with resource allocation
```

### Message Flow Architecture

**Complete Message Processing Flow**:

1. **Message Creation**: Agent creates message with performative and content
2. **Context Analysis**: System analyzes context requirements
3. **Capability Routing**: Route to appropriate agent based on capabilities
4. **Context Preparation**: Gather conversation history, memory patterns,
   tool data
5. **Agent Processing**: Target agent receives message + prepared context
6. **Response Generation**: Agent creates contextually-aware response
7. **Response Routing**: Reply sent back through conversation thread
8. **Context Storage**: Successful interactions stored for future reference

## Communication Patterns

### Request-Response Pattern

**Basic Information Request**:

```yaml
# Request
performative: REQUEST
capability: "system-status"
conversation_id: "health_check_001"
reply_with: "req_status_001"
content: "Provide current system health status"

# Response
performative: INFORM
conversation_id: "health_check_001"
in_reply_to: "req_status_001"
content: |
  System Status Report:
  - CPU Usage: 45%
  - Memory Usage: 62%
  - Active Agents: 12
  - Message Queue: 3 pending
  - Overall Status: Healthy
```

### Multi-Turn Problem Solving

**Complex Analysis with Context Building**:

```yaml
# Turn 1: Initial request
performative: REQUEST
capability: "financial-analysis"
conversation_id: "budget_review_001"
reply_with: "req_budget_001"
content: "Analyze Q3 budget variance and identify concerning trends"

# Turn 2: Follow-up based on results (agent maintains context)
performative: QUERY
conversation_id: "budget_review_001"
in_reply_to: "inform_variance_001"
reply_with: "query_detail_001"
content: |
  You mentioned marketing spend was 23% over budget.
  What specific marketing activities drove this variance?

# Turn 3: Request for recommendations (builds on entire conversation)
performative: REQUEST
conversation_id: "budget_review_001"
in_reply_to: "inform_marketing_001"
reply_with: "req_recommendations_001"
content: |
  Based on the marketing overspend analysis, please provide
  recommendations for Q4 budget adjustments to prevent
  similar variances.
```

### Negotiation and Collaboration

**Proposal-Based Interaction**:

```yaml
# Agent A proposes approach
performative: PROPOSE
capability: "project-planning"
conversation_id: "deployment_strategy_001"
reply_with: "proposal_001"
content: |
  For the system deployment, I propose a blue-green strategy:
  - Deploy to parallel environment
  - Run validation tests for 24 hours
  - Switch traffic if tests pass
  - Keep old environment as rollback option

# Agent B considers and accepts
performative: ACCEPT_PROPOSAL
conversation_id: "deployment_strategy_001"
in_reply_to: "proposal_001"
content: |
  Blue-green deployment proposal accepted.
  This approach provides good risk mitigation.
  Please proceed with environment setup.
```

## Error Handling and Recovery

### Graceful Failure Communication

**Failure with Recovery Guidance**:

```yaml
performative: FAILURE
conversation_id: "data_analysis_001"
in_reply_to: "req_analysis_001"
content: |
  Analysis failed: Unable to access data source.

  Error Details:
  - Connection timeout to database server
  - Retried 3 times over 30 seconds
  - Network connectivity appears normal

  Suggested Actions:
  - Check database server status
  - Verify connection credentials
  - Try analysis with cached data instead
  - Contact system administrator if issue persists

recovery_options:
  - alternative_capability: "cached-data-analysis"
  - fallback_approach: "use_sample_data"
  - retry_conditions: "database_restored"
```

### Comprehension Issues

**Clear Communication of Understanding Problems**:

```yaml
performative: NOT_UNDERSTOOD
conversation_id: "complex_request_001"
in_reply_to: "req_complex_001"
content: |
  I don't understand the request fully. Specific issues:

  - Ambiguous time period: "recent data" - what date range?
  - Unknown abbreviation: "CLTV analysis" - customer lifetime value?
  - Missing context: which product lines should be included?
  - Unclear output: what format do you need for results?

  Could you please clarify these points so I can help effectively?

clarification_needed:
  - time_period: "specify date range"
  - abbreviations: "define CLTV and other acronyms"
  - scope: "list specific product lines"
  - output_format: "specify preferred format (report, charts, data)"
```

## Performance and Optimization

### Context-Aware Performance

**Performance Integration with Context Management**:

- **Context Preparation**: <100ms target for gathering relevant background
- **Message Routing**: <5ms additional overhead for capability-based routing
- **Token Efficiency**: >85% utilization of available context window
- **Response Quality**: Context-enhanced responses vs. isolated responses

**Performance Monitoring**:

```yaml
# Message performance metrics
message_metrics:
  context_preparation_ms: 75 # Within 100ms target
  routing_decision_ms: 3 # Within 5ms target
  total_processing_ms: 1250 # End-to-end timing
  context_utilization: 0.87 # 87% of context window used
  response_quality_score: 0.92 # User satisfaction metric
```

### Optimized Message Routing

**Smart Routing Decisions**:

- **Context Continuity**: Keep conversations with same agent when beneficial
- **Load Balancing**: Distribute messages across available agents
- **Specialization Matching**: Route to agents with relevant expertise
- **Performance Optimization**: Consider agent response times and current load

## Integration with Agent Types

### Configuration Agent Integration

**Agent Messages for Configuration Agents**:
Configuration agents (YAML + markdown) receive agent messages transformed into
natural language prompts with structured context:

```yaml
# Agent REQUEST becomes contextualized prompt
agent_prompt: |
  You received a REQUEST for data-analysis capability:

  User Request: "Analyze Q3 sales trends and project Q4 performance"

  Conversation Context:
  - This continues our discussion from yesterday about retail performance
  - Previous analysis showed 15% growth in online channels
  - User prefers visual charts with executive summaries

  Your Tools:
  - CSV data parser
  - Chart generation
  - Statistical analysis

  Please provide comprehensive analysis addressing the request.
```

### WASM Agent Integration

**Agent Messages for WASM Agents**:
WASM agents receive structured agent messages directly and can generate
structured responses:

```rust
// WASM agent handles agent message directly
pub fn handle_message(message: AgentMessage) -> Result<AgentMessage, AgentError> {
    match message.performative {
        Performative::Request => {
            // Process request with conversation context
            let context = extract_context(&message.conversation_id)?;
            let result = perform_analysis(&message.content, context)?;

            Ok(AgentMessage {
                performative: Performative::Inform,
                conversation_id: message.conversation_id,
                in_reply_to: Some(message.reply_with),
                content: format_results(result),
                ..default_response()
            })
        }
        // Handle other performatives...
    }
}
```

## Learning Path

### For Developers

1. **Messaging Basics**: Understanding performatives and message structure
2. **Context Integration**: How messages contribute to conversational intelligence
3. **Capability Routing**: Designing for flexible, scalable agent interaction
4. **Error Handling**: Building resilient communication patterns

### For Product Teams

1. **Communication Patterns**: How agent messages enable agent collaboration
2. **User Experience**: How standardized communication improves consistency
3. **Capability Modeling**: Organizing agent functions as discoverable capabilities
4. **Workflow Design**: Planning multi-agent interactions and handoffs

### For Operations Teams

1. **Message Monitoring**: Tracking communication patterns and performance
2. **Routing Health**: Ensuring capability-based routing works effectively
3. **Context Performance**: Monitoring context preparation and utilization
4. **Error Analysis**: Understanding communication failures and recovery

### For Stakeholders

1. **Agent Collaboration**: How standardized messaging enables team agent work
2. **Scalability Benefits**: How capability routing supports growth
3. **Integration Possibilities**: How agent messaging enables third-party agent integration
4. **Quality Assurance**: How structured communication improves reliability

## Related Concepts

- **[Messaging Overview](/docs/concepts/messaging/overview.md)**:
  Foundational messaging architecture and capability routing
- **[Conversation Management](/docs/concepts/messaging/conversation-management.md)**:
  Multi-turn dialogue and context preservation
- **[Capability Routing](/docs/concepts/messaging/capability-routing.md)**:
  Advanced routing strategies and agent selection
- **[Architecture Concepts](/docs/concepts/architecture/)**:
  System design enabling effective agent communication
- **[Memory System](/docs/concepts/memory-system/)**:
  How conversation history integrates with knowledge storage
