---
title: "Conversation Management"
description: "Managing multi-turn conversations and message threading in
  Caxton's agent messaging system"
layout: documentation
categories: [Messaging, Conversation Threading, State Management]
date: 2025-09-10
---

## Overview

Caxton's messaging system maintains conversation context across multi-turn
message exchanges, enabling agents to build on previous interactions and
maintain coherent dialogue threads. This is particularly important for
configuration agents that handle complex, multi-step workflows and
collaborative problem-solving.

## Core Conversation Concepts

### Conversation Identity

Each conversation has a unique identifier that links related messages together:

```yaml
# Starting a new conversation
conversation_id: conv_data_analysis_2024_09_10_001
```

**Naming Conventions:**

- Use descriptive prefixes: `conv_`, `thread_`, `workflow_`
- Include date/time for uniqueness: `2024_09_10`
- Add sequence numbers for multiple conversations: `_001`
- Include topic identifiers: `data_analysis`, `customer_support`

### Message Correlation

Messages within a conversation use correlation fields to maintain threading:

```yaml
# Initial message
performative: REQUEST
conversation_id: conv_analysis_001
reply_with: req_sales_analysis_001
content: "Analyze Q3 sales data"

# Response message
performative: INFORM
conversation_id: conv_analysis_001
in_reply_to: req_sales_analysis_001
content: "Analysis complete: Sales increased 15%"

# Follow-up question
performative: QUERY
conversation_id: conv_analysis_001
reply_with: query_details_001
content: "What drove the 15% increase?"

# Follow-up response
performative: INFORM
conversation_id: conv_analysis_001
in_reply_to: query_details_001
content: "Primary drivers: new product launch (8%), market expansion (7%)"
```

## Conversation Lifecycle Management

### Starting Conversations

Conversations begin when an agent sends the first message with a new
`conversation_id`:

```yaml
# Configuration agent starting a conversation
name = "CustomerServiceBot"
capabilities = [
  "customer-support",
  "order-tracking"
]
tools = [
  "message_client",
  "order_database"
]

# First message in conversation
performative: QUERY
capability: order-management
conversation_id: conv_order_inquiry_2024_09_10_145623
reply_with: query_order_status_001
content: |
  Customer inquiry about order status:
  - Customer: jane.smith@email.com
  - Order ID: ORD-2024-091001
  - Inquiry: "Where is my order? It was supposed to arrive yesterday."

  Please provide:
  - Current shipping status
  - Expected delivery date
  - Any delays or issues

customer_context:
  email: "jane.smith@email.com"
  order_id: "ORD-2024-091001"
  inquiry_type: "order_status"
  expected_delivery: "2024-09-09"
```

### Maintaining Context

Agents can maintain conversation context across multiple exchanges:

```yaml
# Turn 1 - Initial analysis request
performative: REQUEST
capability: financial-analysis
conversation_id: conv_budget_review_001
reply_with: req_budget_analysis_001
content: |
  Please analyze our Q3 budget performance:
  - Compare actual vs planned expenses
  - Identify significant variances (>10%)
  - Recommend adjustments for Q4

# Turn 2 - Response with analysis
performative: INFORM
conversation_id: conv_budget_review_001
in_reply_to: req_budget_analysis_001
content: |
  Budget analysis complete:
  - Total variance: +$47,000 (12% over budget)
  - Major variances: Marketing (+$65k), IT infrastructure (-$18k)
  - Q4 recommendation: Reduce marketing spend by 15%

# Turn 3 - Follow-up question
performative: QUERY
conversation_id: conv_budget_review_001
reply_with: query_marketing_details_001
content: |
  Can you break down the $65k marketing overage?
  - Which campaigns exceeded budget?
  - Were they effective (ROI analysis)?
  - Which should be cut for Q4?

# Turn 4 - Detailed breakdown
performative: INFORM
conversation_id: conv_budget_review_001
in_reply_to: query_marketing_details_001
content: |
  Marketing overage breakdown:
  - Digital advertising: +$42k (ROI: 3.2x - KEEP)
  - Trade shows: +$18k (ROI: 0.8x - CUT)
  - Content marketing: +$5k (ROI: 2.1x - REDUCE)

  Q4 Recommendation: Cut trade shows (-$25k), reduce content (-$15k)

# Turn 5 - Implementation request
performative: REQUEST
conversation_id: conv_budget_review_001
reply_with: req_budget_update_001
content: |
  Please update our Q4 budget plan based on your analysis:
  - Remove trade show budget allocation
  - Reduce content marketing by 60%
  - Maintain digital advertising at current levels
```

### Conversation State Tracking

The system tracks conversation state automatically:

```yaml
# System-maintained conversation state
conversation_state:
  id: "conv_budget_review_001"
  created_at: "2024-09-10T14:30:00Z"
  last_activity: "2024-09-10T15:45:00Z"
  participants: ["CustomerServiceBot", "FinanceAnalyzer", "BudgetManager"]
  message_count: 5
  status: "active"
  topic: "Q3 budget analysis and Q4 planning"

  # Message thread tracking
  message_chain:
    - id: "req_budget_analysis_001"
      timestamp: "2024-09-10T14:30:00Z"
      sender: "CustomerServiceBot"
      performative: "REQUEST"
    - id: "inform_budget_results_001"
      timestamp: "2024-09-10T14:45:00Z"
      sender: "FinanceAnalyzer"
      performative: "INFORM"
      in_reply_to: "req_budget_analysis_001"
    - id: "query_marketing_details_001"
      timestamp: "2024-09-10T15:15:00Z"
      sender: "CustomerServiceBot"
      performative: "QUERY"
    # ... continues
```

## Advanced Conversation Patterns

### Multi-Party Conversations

Multiple agents can participate in the same conversation thread:

```yaml
# Agent A starts conversation
performative: REQUEST
capability: incident-response
conversation_id: conv_security_incident_001
reply_with: req_investigate_001
content: |
  Security alert detected:
  - Suspicious login attempts from IP 192.168.1.100
  - Multiple failed authentication attempts
  - Potential brute force attack

  Need immediate investigation and response plan.

# Agent B (Security Analyzer) responds
performative: INFORM
conversation_id: conv_security_incident_001
in_reply_to: req_investigate_001
content: |
  Initial analysis complete:
  - IP confirmed as external threat actor
  - 247 failed login attempts in 10 minutes
  - Targeting admin accounts
  - Recommend immediate IP blocking

# Agent C (Network Manager) joins conversation
performative: INFORM
conversation_id: conv_security_incident_001
content: |
  IP blocking implemented:
  - Added 192.168.1.100 to firewall blacklist
  - Blocked at network perimeter
  - Monitoring for additional IPs from same subnet

# Agent A coordinates next steps
performative: REQUEST
capability: incident-documentation
conversation_id: conv_security_incident_001
reply_with: req_document_incident_001
content: |
  Please document this security incident:
  - Timeline of events
  - Actions taken by each team
  - Current threat status
  - Lessons learned and improvements needed
```

### Branching Conversations

Conversations can branch into multiple sub-threads:

```yaml
# Main conversation
performative: REQUEST
capability: project-planning
conversation_id: conv_project_launch_001
reply_with: req_project_plan_001
content: "Plan the launch of our new mobile app"

# Branch 1: Technical planning
performative: REQUEST
capability: technical-architecture
conversation_id: conv_project_launch_001_tech
parent_conversation: conv_project_launch_001
reply_with: req_tech_plan_001
content: "Design technical architecture for mobile app backend"

# Branch 2: Marketing planning
performative: REQUEST
capability: marketing-strategy
conversation_id: conv_project_launch_001_marketing
parent_conversation: conv_project_launch_001
reply_with: req_marketing_plan_001
content: "Create marketing campaign for mobile app launch"

# Branch 3: Legal review
performative: REQUEST
capability: legal-compliance
conversation_id: conv_project_launch_001_legal
parent_conversation: conv_project_launch_001
reply_with: req_legal_review_001
content: "Review app privacy policy and terms of service"
```

### Conversation Merging

Related conversations can be merged when they converge:

```yaml
# Merge technical and marketing conversations
performative: INFORM
capability: project-coordination
conversation_id: conv_project_launch_001
merge_conversations:
  - conv_project_launch_001_tech
  - conv_project_launch_001_marketing
content: |
  Merging technical and marketing plans:
  - Technical architecture supports marketing requirements
  - Analytics tracking aligned with campaign needs
  - Launch timeline coordinated across teams

consolidated_plan:
  technical_readiness: "2024-09-20"
  marketing_campaign_start: "2024-09-18"
  coordinated_launch_date: "2024-09-25"
```

## Conversation Cleanup and Maintenance

### Timeout Management

Conversations automatically timeout to prevent memory leaks:

```yaml
# System configuration for conversation timeouts
conversation_timeouts:
  inactive_timeout: "24h" # No messages for 24 hours
  max_lifetime: "7d" # Maximum conversation duration
  cleanup_interval: "1h" # How often to check for expired conversations

  # Custom timeouts by capability
  capability_timeouts:
    urgent-support: "2h" # Customer support conversations
    data-analysis: "12h" # Data processing workflows
    project-planning: "7d" # Long-term planning discussions
```

Agents can extend conversation timeouts when needed:

```yaml
performative: REQUEST
capability: system:conversation-management
conversation_id: conv_complex_analysis_001
reply_with: req_extend_timeout_001
content: |
  Please extend timeout for this conversation:
  - Current analysis requires additional data gathering
  - Estimated completion time: 6 hours
  - Reason: Large dataset processing and validation

extend_timeout:
  new_timeout: "8h"
  reason: "complex_data_analysis"
  estimated_completion: "2024-09-10T23:00:00Z"
```

### Conversation Archival

Completed conversations can be archived for future reference:

```yaml
performative: REQUEST
capability: system:conversation-archive
conversation_id: conv_project_retrospective_001
reply_with: req_archive_001
content: |
  Archive this project retrospective conversation:
  - Contains valuable lessons learned
  - Should be searchable for future projects
  - Include all participant contributions
  - Tag with project phase and outcome

archive_metadata:
  project_name: "Mobile App Launch"
  project_phase: "retrospective"
  outcome: "successful"
  key_lessons:
    - "technical architecture"
    - "marketing coordination"
    - "timeline management"
  retention_period: "2y"
```

### Conversation Analytics

The system provides analytics on conversation patterns:

```yaml
# Query conversation analytics
performative: QUERY
capability: system:analytics
conversation_id: conv_analytics_query_001
reply_with: query_conversation_stats_001
content: |
  Provide conversation analytics for the last 30 days:
  - Average conversation length (messages)
  - Most common capability requests
  - Conversation success rates
  - Agent participation patterns

# Response with analytics
performative: INFORM
conversation_id: conv_analytics_query_001
in_reply_to: query_conversation_stats_001
content: |
  Conversation analytics (30 days):
  - Total conversations: 1,247
  - Average length: 4.3 messages
  - Success rate: 87.2%
  - Most requested capabilities:
    1. data-analysis (342 requests)
    2. customer-support (189 requests)
    3. report-generation (156 requests)

analytics_data:
  period: "30d"
  total_conversations: 1247
  avg_messages_per_conversation: 4.3
  success_rate: 0.872
  top_capabilities:
    - name: "data-analysis"
      request_count: 342
      success_rate: 0.91
    - name: "customer-support"
      request_count: 189
      success_rate: 0.84
    - name: "report-generation"
      request_count: 156
      success_rate: 0.93
```

## Configuration Agent Integration

### Automatic Context Preservation

Configuration agents automatically maintain conversation context:

```yaml
# Agent configuration with conversation awareness
name = "DataAnalysisAssistant"
capabilities = [
  "data-analysis",
  "statistical-modeling"
]
tools = [
  "data_processor",
  "chart_generator"
]

[conversation_settings]
maintain_context = true
context_window = 10        # Remember last 10 messages
auto_summarize = true      # Summarize long conversations
timeout = "4h"            # Custom timeout for data work
```

The runtime automatically includes relevant context in agent prompts:

```yaml
# System-generated context for configuration agent
conversation_context:
  id: "conv_sales_analysis_001"
  participant_count: 2
  message_count: 6
  duration: "45 minutes"

  # Previous messages summary
  summary: |
    Conversation started with request to analyze Q3 sales data.
    Analysis revealed 15% growth with primary drivers being new product
    launch (8%) and market expansion (7%). Follow-up questions explored
    regional performance and seasonal trends.

  # Recent messages (raw)
  recent_messages:
    - performative: "QUERY"
      content: "What seasonal patterns do you see in the data?"
      timestamp: "2024-09-10T15:30:00Z"
    - performative: "INFORM"
      content: "Strong Q4 seasonality with 35% higher sales in December"
      timestamp: "2024-09-10T15:25:00Z"
```

### Context-Aware Response Generation

Configuration agents can reference conversation history naturally:

```yaml
# Agent prompt includes conversation context
system_prompt: |
  You are analyzing sales data. Based on our conversation history, you've
  already identified:
  - 15% Q3 growth (new products 8%, expansion 7%)
  - Strong December seasonality (35% higher sales)

  Current question: "What seasonal patterns do you see in the data?"

  Provide insights that build on previous analysis.

# Agent response references context appropriately
performative: INFORM
conversation_id: conv_sales_analysis_001
in_reply_to: query_seasonal_001
content: |
  Building on our previous analysis of Q3 growth drivers, I see several
  seasonal patterns:

  1. December spike (35% higher, as mentioned earlier)
  2. Back-to-school boost in August/September (aligns with your new product
     launch impact)
  3. Summer dip in June/July (20% below average)
  4. Post-holiday decline in January/February

  The new product launch timing was strategic - it captured both the
  back-to-school season AND built momentum into the holiday season.
```

## Best Practices for Conversation Management

### Conversation Design

1. **Use descriptive conversation IDs**: Include topic, date, and sequence
2. **Plan for multi-turn interactions**: Design requests that may need follow-up
3. **Include context hints**: Help agents understand the broader conversation
   purpose
4. **Set appropriate expectations**: Indicate if this is a quick question or
   complex analysis

### Message Threading

1. **Always use reply correlation**: Link responses to specific requests
2. **Include conversation history**: Reference previous points when relevant
3. **Use clear language**: Natural language helps configuration agents
   understand context
4. **Break down complex requests**: Split large requests into smaller, threaded
   messages

### Performance Optimization

1. **Set reasonable timeouts**: Balance responsiveness with processing needs
2. **Clean up conversations**: Archive or delete completed conversations
3. **Monitor conversation metrics**: Track length, success rates, and participant
   patterns
4. **Optimize context window**: Balance context richness with processing speed

### Error Handling

1. **Handle conversation timeouts gracefully**: Provide clear error messages
2. **Support conversation recovery**: Allow resuming interrupted conversations
3. **Validate conversation state**: Check that referenced messages exist
4. **Provide conversation debugging**: Tools to inspect conversation state and
   history

## Monitoring and Debugging

### Conversation Inspection

Administrators can inspect active conversations:

```yaml
performative: QUERY
capability: system:conversation-debug
conversation_id: conv_debug_query_001
reply_with: query_inspect_001
content: |
  Inspect conversation conv_sales_analysis_001:
  - Show full message thread
  - Include participant details
  - Show current state and timeouts
  - Identify any issues or bottlenecks

# System response with conversation details
performative: INFORM
conversation_id: conv_debug_query_001
in_reply_to: query_inspect_001
content: |
  Conversation inspection results:

  ID: conv_sales_analysis_001
  Status: Active
  Duration: 1h 15m
  Messages: 8
  Participants: DataAnalyst, SalesManager

  Message Thread:
  1. REQUEST: Analyze Q3 sales data (14:30)
  2. INFORM: Analysis complete - 15% growth (14:45)
  3. QUERY: What drove the growth? (15:00)
  4. INFORM: New products 8%, expansion 7% (15:05)
  5. QUERY: Regional breakdown? (15:20)
  6. INFORM: West +22%, East +11%, Central +8% (15:30)
  7. QUERY: Seasonal patterns? (15:40)
  8. [PENDING] - DataAnalyst processing (1m elapsed)

  No issues detected. Normal processing flow.
```

### Performance Monitoring

Track conversation performance metrics:

```yaml
# Conversation performance metrics
conversation_metrics:
  active_conversations: 45
  avg_response_time_ms: 1250
  messages_per_minute: 28
  timeout_rate: 0.03

  # By capability
  capability_performance:
    - name: "data-analysis"
      active_conversations: 12
      avg_response_time_ms: 2150
      success_rate: 0.94
    - name: "customer-support"
      active_conversations: 18
      avg_response_time_ms: 850
      success_rate: 0.91
```

Conversation management is a critical component of Caxton's messaging system,
enabling sophisticated multi-turn interactions between configuration agents
while maintaining performance and reliability. Proper conversation design and
management ensures that agents can collaborate effectively on complex tasks
while keeping resource usage efficient.
