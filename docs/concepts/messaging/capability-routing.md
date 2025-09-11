---
title: "Capability-Based Routing Concepts"
description: "Understanding how Caxton routes messages based on agent
  capabilities rather than direct addressing for flexible, scalable communication"
date: 2025-01-13
categories: [Concepts, Messaging, Routing, Architecture]
layout: concept
level: intermediate
---

## Why Capability-Based Routing Matters

Traditional agent systems use **direct addressing**—sending messages to specific
agent instances by name or ID. This creates **tight coupling** between agents
and makes systems brittle as they scale.

Capability-based routing uses **functional addressing**—sending messages to
**capabilities** rather than specific agents. Think of it like calling a phone
number for "pizza delivery" rather than calling a specific delivery person by
name. The system finds an available agent who can fulfill the request.

## Core Routing Concepts

### From Direct to Capability-Based Addressing

**Direct Addressing Problems**:

```yaml
# Brittle: Tightly coupled to specific agent
to: "sales_agent_instance_3"
performative: REQUEST
content: "Generate Q3 sales report"

# What happens when sales_agent_instance_3 is:
# - Overloaded with requests?
# - Offline for maintenance?
# - Replaced with a better agent?
# - Moved to different server?
```

**Capability-Based Solution**:

```yaml
# Flexible: Routes to any capable agent
capability: "sales-reporting"
performative: REQUEST
content: "Generate Q3 sales report"

# System automatically:
# - Finds agents with "sales-reporting" capability
# - Routes to best available agent
# - Handles load balancing and failover
# - Works with any number of capable agents
```

### Capability Declaration and Discovery

**How Agents Declare Capabilities**:

```yaml
# Configuration agent declares what it can do
---
name: SalesAnalyzer
capabilities:
  - data-analysis      # General analysis capability
  - sales-reporting    # Specific reporting capability
  - chart-generation   # Visualization capability
tools:
  - csv_parser
  - chart_generator
---
```

**Dynamic Capability Registry**:
The system maintains a **live registry** that tracks:

- Which agents provide which capabilities
- Agent availability and current load
- Performance characteristics and specializations
- Version compatibility and feature support

### Routing Strategy Selection

**Single Recipient Routing**:

- Route to **one best agent** for the capability
- Best for tasks requiring single processing
- Uses load balancing, performance history, specialization matching

**Broadcast Routing**:

- Route to **all agents** with the capability
- Best for information sharing and notifications
- Ensures all relevant agents receive important updates

**Load Balanced Routing**:

- Distribute requests **across multiple agents**
- Best for high-volume, parallel processing
- Strategies: round-robin, least-loaded, performance-based

## Practical Routing Scenarios

### Simple Analysis Request

**The Request**:

```yaml
# Business intelligence agent needs data analysis
performative: REQUEST
capability: "data-analysis"
conversation_id: "bi_analysis_001"
content: |
  Analyze customer churn patterns from Q3 data:
  - Identify customers at risk of churning
  - Find common characteristics among churned customers
  - Suggest retention strategies based on patterns
```

**What Happens Behind the Scenes**:

1. **Capability Lookup**: System finds all agents with "data-analysis" capability
2. **Agent Selection**: Evaluates available agents based on load, performance, specialization
3. **Context Preparation**: Gathers conversation history and relevant patterns
4. **Message Delivery**: Routes to selected agent with prepared context
5. **Response Handling**: Routes response back through conversation thread

### Multi-Agent Workflow

**Pipeline Processing Example**:

```yaml
# Stage 1: Extract data
capability: "data-extraction"
content: "Extract customer data from CRM for last 90 days"

# Stage 2: Process data (after Stage 1 completes)
capability: "data-cleaning"
content: "Clean and normalize extracted customer dataset"

# Stage 3: Analyze data (after Stage 2 completes)
capability: "predictive-analysis"
content: "Build churn prediction model from cleaned data"

# Stage 4: Generate report (after Stage 3 completes)
capability: "report-generation"
content: "Create executive report with model insights and recommendations"
```

**Pipeline Benefits**:

- **Specialization**: Each stage handled by agents optimized for specific tasks
- **Parallelization**: Multiple pipelines can run simultaneously
- **Fault Tolerance**: If one agent fails, another with same capability takes over
- **Scalability**: Add more agents for any capability to increase throughput

### Error Handling and Fallbacks

**No Available Agents**:

```yaml
# System response when capability not available
performative: FAILURE
content: |
  No agents available for capability: "advanced-machine-learning"

  Available related capabilities:
  - data-analysis (3 agents available)
  - statistical-modeling (1 agent available)
  - basic-ml-analysis (2 agents available)

  Suggestions:
  - Use "data-analysis" with statistical methods
  - Break request into smaller parts for "basic-ml-analysis"
  - Wait for "advanced-machine-learning" agent to become available
```

## Context-Aware Routing

### Intelligent Agent Selection

Beyond simple capability matching, Caxton uses **context-aware routing** to
make smarter agent selection decisions:

**Context Factors in Routing**:

- **Conversation Continuity**: Prefer agents already involved in conversation thread
- **Domain Specialization**: Route to agents with relevant domain expertise
- **Context Window Size**: Match complex requests with agents that can handle
  extensive context
- **Performance History**: Consider agent success rates with similar requests

**Enhanced Agent Registry**:

```yaml
# Agents register with context capabilities
agent: "FinancialExpert"
capabilities:
  - financial-analysis
  - budget-planning
context_metadata:
  specializations: ["budget_variance", "cost_optimization"]
  context_window: "large"           # Can handle extensive context
  conversation_continuity: "excellent"  # Maintains thread context well
  domain_expertise: ["financial_planning", "variance_analysis"]
```

### Context-Enhanced Routing Example

**Complex Financial Analysis Request**:

```yaml
performative: REQUEST
capability: "financial-analysis"
conversation_id: "budget_review_001"
content: |
  Continue our budget variance analysis from yesterday. Focus on the
  operational cost overruns we discussed, particularly in manufacturing
  and logistics. Use the seasonal adjustment methodology we agreed on.

# Context Router Analysis:
# - High context requirements (references previous conversation)
# - Domain-specific terminology (manufacturing, logistics, seasonal adjustments)
# - Conversation continuity important (continues previous discussion)

# Routing Decision:
# - Selects agent with "financial-analysis" + manufacturing specialization
# - Prefers agent already in conversation thread
# - Ensures selected agent can handle complex context requirements
```

## Routing Optimization Patterns

### Performance-Based Selection

**Load Balancing Intelligence**:

- **Current Load**: Route to agents with available capacity
- **Response Time History**: Prefer agents with consistent fast responses
- **Success Rate**: Prioritize agents with high success rates for similar requests
- **Specialization Match**: Balance performance with domain expertise

**Dynamic Performance Tracking**:

```text
Agent Performance Metrics:
├── Current message queue length
├── Average response time (last 100 messages)
├── Success rate by message type
├── Context preparation efficiency
└── User satisfaction scores
```

### Conversation Continuity Optimization

**Same-Agent Preference**:
When beneficial, keep conversations with the same agent to:

- **Maintain Context**: Agent already has conversation background
- **Reduce Context Preparation**: Skip loading conversation history
- **Improve Coherence**: Consistent response style and approach
- **Enable Learning**: Agent can build on previous interactions

**Cross-Agent Handoff**:
When switching agents is better:

- **Load Balancing**: Original agent overloaded
- **Specialization Match**: New request needs different expertise
- **Performance Optimization**: Another agent performs better for specific task type
- **Availability**: Original agent offline or unavailable

## Advanced Routing Concepts

### Capability Versioning and Evolution

**Version-Aware Routing**:

```yaml
# Agent supports multiple capability versions
capabilities:
  - data-analysis:v1.0    # Legacy compatibility
  - data-analysis:v2.0    # Current version with new features
  - advanced-analytics:v1.0  # New specialized capability

# Request with version preferences
capability: "data-analysis:v2.0"
fallback_versions: ["v2.0", "v1.0"]  # Try v2.0 first, fall back to v1.0
```

**Capability Evolution Strategy**:

- **Backward Compatibility**: Support older versions during transitions
- **Gradual Migration**: Move to new versions as agents are updated
- **Feature Detection**: Route based on specific feature requirements
- **Deprecation Management**: Clear timeline for phasing out old versions

### Conditional and Rule-Based Routing

**Content-Based Routing Rules**:

```yaml
# Route based on message characteristics
routing_rules:
  - condition: "data_size > 100MB"
    strategy: "high_capacity_agents"
  - condition: "priority == urgent"
    strategy: "fastest_response_agents"
  - condition: "content.contains('financial')"
    capability: "financial-analysis"
  - default: "standard_load_balanced"
```

**Geographic and Resource-Based Routing**:

- **Data Locality**: Route to agents near data sources
- **Regulatory Compliance**: Route to agents in compliant regions
- **Resource Requirements**: Match compute/memory needs with agent capabilities
- **Cost Optimization**: Balance performance with infrastructure costs

## Best Practices

### Capability Design Guidelines

**Naming Conventions**:

- **Use kebab-case**: `data-analysis`, `report-generation`
- **Be specific**: `financial-analysis` vs generic `analysis`
- **Include domain**: `medical-imaging`, `legal-document-review`
- **Avoid overlap**: Clear boundaries between capabilities

**Capability Granularity**:

- **Too Broad**: `general-assistant` (hard to route effectively)
- **Too Narrow**: `csv-file-parser-for-sales-data` (limits reusability)
- **Just Right**: `data-analysis`, `document-processing`, `image-classification`

### Message Design for Routing

**Clear Capability Requirements**:

```yaml
# Good: Specific capability with clear requirements
capability: "data-analysis"
content: |
  Analyze customer satisfaction survey data (CSV format, 10K responses).
  Required: Statistical analysis, sentiment analysis, trend identification.
  Output: Executive summary with key findings and recommendations.
```

**Context Hints for Better Routing**:

```yaml
# Help router make intelligent decisions
capability: "financial-analysis"
context_hints:
  specialization_needed: "budget_variance"
  complexity: "high"
  context_requirements: "extensive"
  performance_priority: "accuracy"  # vs "speed"
```

### Monitoring and Observability

**Routing Health Metrics**:

- **Capability Coverage**: Are all needed capabilities available?
- **Load Distribution**: Are requests evenly distributed across agents?
- **Response Time**: How quickly are routing decisions made?
- **Success Rate**: How often does routing result in successful task completion?
- **Context Efficiency**: How well does context-aware routing work?

**Routing Decision Debugging**:

```yaml
# Debug why specific routing decision was made
performative: QUERY
capability: "system:routing-debug"
content: "Explain routing decision for message req_analysis_001"

# System response shows routing logic
routing_decision:
  requested_capability: "data-analysis"
  available_agents: 3
  selected_agent: "DataExpert-2"
  selection_reasons:
    - "Lowest current load (2/10 messages)"
    - "High success rate for similar requests (94%)"
    - "Specialization match: customer_analytics"
    - "Available context window: large"
  alternative_agents:
    - name: "GeneralAnalyzer-1"
      reason_not_selected: "Higher load (7/10 messages)"
    - name: "DataExpert-1"
      reason_not_selected: "Currently processing large dataset"
```

## Learning Path

### For Developers

1. **Capability Design**: How to design effective, reusable capabilities
2. **Message Patterns**: Structuring messages for effective routing
3. **Context Integration**: Using context hints to improve routing decisions
4. **Debugging Tools**: Understanding routing decisions and troubleshooting issues

### For Operations Teams

1. **Registry Management**: Monitoring capability availability and health
2. **Performance Tuning**: Optimizing routing strategies for workload patterns
3. **Load Balancing**: Ensuring effective distribution across agents
4. **Capacity Planning**: Scaling agents based on capability demand

### For Product Teams

1. **Capability Modeling**: Organizing agent functions as discoverable capabilities
2. **User Experience**: How routing affects response quality and consistency
3. **Workflow Design**: Planning multi-agent interactions and handoffs
4. **Feature Planning**: Capability evolution and version management

### For Stakeholders

1. **Scalability Benefits**: How capability routing enables system growth
2. **Fault Tolerance**: How routing provides resilience and reliability
3. **Resource Efficiency**: How intelligent routing optimizes infrastructure usage
4. **Integration Possibilities**: How capability-based design enables extensibility

## Related Concepts

- **[Messaging Overview](/docs/concepts/messaging/overview.md)**:
  Foundational messaging architecture and FIPA-ACL integration
- **[FIPA-ACL Subset](/docs/concepts/messaging/fipa-acl-subset.md)**:
  Standardized communication protocols enabling capability routing
- **[Conversation Management](/docs/concepts/messaging/conversation-management.md)**:
  Multi-turn dialogues and context preservation with capability routing
- **[Architecture Concepts](/docs/concepts/architecture/)**:
  System design supporting flexible, scalable agent communication
- **[Memory System](/docs/concepts/memory-system/)**:
  How routing integrates with knowledge storage and retrieval
