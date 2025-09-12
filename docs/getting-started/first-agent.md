---
title: "Create Your First Agent"
date: 2025-09-10
layout: page
categories: [Getting Started]
---

> **🚧 Implementation Status**
>
> This tutorial represents the intended agent development
> experience from ADR-32. The TOML configuration and agent deployment workflow
> described here serves as acceptance criteria for implementation.
>
> **Target**: 5-minute configuration-driven agent creation
> **Status**: Agent format specification and runtime being developed

## Build intelligent agents in 5 minutes using configuration files

This guide teaches you to create **agents** - the configuration-driven way
to build agents in Caxton. No compilation, no complex toolchains, just
TOML configuration files.

## Agent Fundamentals

Every agent consists of:

1. **TOML configuration** - Agent metadata, capabilities, and configuration
2. **Documentation section** - Usage examples and feature descriptions
3. **System prompts** - Instructions defining agent behavior
4. **Capability declarations** - What the agent can do
5. **Tool integrations** - External services the agent can use

## Agent Lifecycle

Agents follow a simple lifecycle:

- **Creation**: Write TOML configuration file
- **Deployment**: `caxton agent deploy agent.toml` validates and loads
- **Execution**: Agent processes messages via LLM orchestration
- **Learning**: Agent stores successful patterns in embedded memory
- **Updates**: Edit config file and redeploy for instant changes

## Your First Agent: Task Manager

Let's create a task management agent that helps users organize and track work.

### 1. Create the Agent File

Create `task-manager.toml`:

```toml
name = "TaskManager"
version = "1.0.0"
description = "Intelligent task management and productivity assistant"
capabilities = ["task-management", "productivity-coaching", "time-tracking"]
tools = ["calendar_integration", "notification_service", "file_storage"]

[memory]
enabled = true
scope = "workspace"
retention = "30d"

[parameters]
max_tasks_per_user = 100
default_priority = "medium"
time_zone = "UTC"

system_prompt = '''
You are TaskManager, an intelligent productivity assistant. Your role is to help
users organize, prioritize, and complete their work effectively.

Core responsibilities:
1. Create, update, and track tasks with proper metadata
2. Suggest priorities based on deadlines and importance
3. Provide productivity coaching and time management advice
4. Learn user preferences and adapt recommendations
5. Integrate with calendars and notification systems

When processing task requests:
- Always check memory for user preferences and past patterns
- Suggest realistic timelines based on task complexity
- Offer productivity tips relevant to the task type
- Store successful task completion patterns for future reference

Personality: Encouraging, organized, and practical. Help users feel accomplished.
'''

user_prompt_template = '''
Task Request: {{request}}

User Context: {{user_context}}
Current Tasks: {{current_tasks}}
Relevant Memory: {{memory_context}}
Deadline Information: {{deadlines}}

Please help with this task management request.
'''

documentation = '''
# TaskManager Agent

An intelligent task management assistant that helps you stay organized and productive.

## Features

- **Smart Task Creation**: Automatically categorize and prioritize tasks
- **Deadline Tracking**: Monitor due dates and send proactive reminders
- **Productivity Coaching**: Suggest optimal work patterns and time management
- **Learning**: Adapts to your working style and preferences over time
- **Integration**: Works with calendars, notifications, and file storage

## Usage Examples

### Basic Task Management
- "Add a task to review the Q3 budget by Friday"
- "Show me all high-priority tasks this week"
- "Mark the presentation task as completed"

### Productivity Coaching
- "Help me prioritize my tasks for tomorrow"
- "I'm feeling overwhelmed, can you help organize my workload?"
- "What's the best time to schedule deep work based on my patterns?"

### Smart Suggestions
- "Suggest a realistic timeline for this project"
- "Break down this large task into manageable steps"
- "What tasks are similar to ones I've completed successfully before?"

## Memory-Powered Learning

The TaskManager learns from your interactions:
- **Work patterns**: When you're most productive
- **Task preferences**: How you like to organize work
- **Success factors**: What helps you complete tasks effectively
- **Time estimates**: How long different types of tasks actually take

This learning makes the agent more helpful over time.
'''
```

### 2. Deploy Your Agent

```bash
# Deploy the task manager
caxton agent deploy task-manager.toml

# Verify it's running
caxton agent list
```

Expected output:

```text
NAME         TYPE     STATUS    CAPABILITIES
TaskManager  config   running   task-management, productivity-coaching
```

### 3. Interact with Your Agent

Send messages to your agent:

```bash
# Add a task
caxton message send \
  --agent "TaskManager" \
  --content '{
    "request": "Add a task to prepare slides for Monday presentation",
    "user_context": "Working on quarterly review",
    "deadline": "2025-09-15T09:00:00Z"
  }'

# Get productivity advice
caxton message send \
  --agent "TaskManager" \
  --content '{
    "request": "Help me organize my work for this week",
    "current_tasks": ["presentation", "budget review", "team meeting prep"],
    "context": "Feeling a bit overwhelmed"
  }'
```

### 4. Watch the Agent Learn

```bash
# Follow agent activity
caxton logs TaskManager --follow
```

You'll see the agent:

- Searching memory for relevant patterns
- Storing new task management insights
- Adapting recommendations based on past interactions

```text
[TaskManager] Searching memory: "presentation task patterns"
[TaskManager] Found 3 similar tasks from memory
[TaskManager] Applying learned timing: "presentation prep usually takes 3-4 hours"
[TaskManager] Storing new pattern: "quarterly_presentation_workflow"
```

## Advanced Configuration Patterns

### Multi-Capability Agent

Create an agent that handles multiple related capabilities:

```toml
name = "CustomerSupport"
version = "2.0.0"
capabilities = ["customer-inquiry", "order-tracking", "technical-support", "escalation-management"]
tools = ["crm_system", "knowledge_base", "email_service", "ticket_system"]

[memory]
enabled = true
scope = "global"  # Share knowledge across all support interactions

system_prompt = '''
You are a customer support specialist with access to multiple systems.
Route inquiries appropriately and escalate when needed.

For each capability:
- customer-inquiry: Handle general questions and provide information
- order-tracking: Look up order status and shipping information
- technical-support: Troubleshoot product issues and provide solutions
- escalation-management: Route complex issues to human agents

Always check memory for similar issues and their resolutions.
'''
```

### Workflow Orchestration Agent

Create an agent that coordinates other agents:

```toml
name = "ProjectOrchestrator"
version = "1.0.0"
capabilities = ["project-coordination", "workflow-management"]

[memory]
enabled = true
scope = "workspace"

system_prompt = '''
You coordinate complex projects by delegating tasks to other agents.

When you receive project requests:
1. Break down the project into subtasks
2. Send subtasks to appropriate capabilities
3. Monitor progress and coordinate between agents
4. Aggregate results into final deliverable

Use direct messaging to delegate work:
- Send data analysis tasks to DataAnalyzer agents
- Send document creation to DocumentGenerator agents
- Send notifications via NotificationService agents
'''
```

### Memory-Intensive Learning Agent

Create an agent optimized for learning and knowledge building:

```toml
name = "KnowledgeAssistant"
version = "1.0.0"
capabilities = ["knowledge-search", "learning-assistance", "information-synthesis"]
tools = ["web_search", "document_parser", "citation_manager"]

[memory]
enabled = true
scope = "global"
semantic_search = true
relationship_tracking = true

[parameters]
max_search_results = 10
citation_style = "APA"

system_prompt = '''
You are a knowledge assistant that helps users learn and research topics.

Your unique strength is building and connecting knowledge over time:
1. Search your memory for related concepts and prior research
2. Identify knowledge gaps and suggest research directions
3. Synthesize information from multiple sources
4. Store new insights and their relationships to existing knowledge
5. Build semantic maps of interconnected concepts

Always explain how new information connects to what you've learned before.
'''
```

## Configuration Schema Reference

### Required Fields

```toml
name = "string"              # Unique agent identifier
version = "string"          # Semantic version
capabilities = ["string"]   # What the agent can do
system_prompt = "string"    # Core behavior instructions
```

### Optional Configuration

```toml
description = "string"                # Human-readable description
tools = ["string"]                   # External services to use

[memory]
enabled = true                        # Enable persistent memory
scope = "agent"                       # "agent"|"workspace"|"global" - Memory sharing level
retention = "string"                  # How long to keep memories
semantic_search = true                # Enable vector search
relationship_tracking = true          # Track entity relationships

[parameters]                          # Custom agent parameters
key = "value"

user_prompt_template = "string"       # Template for user interactions

[conversation]
max_turns = 20                        # Conversation length limit
timeout = "string"                    # Response timeout

[security]
restricted_tools = ["string"]         # Limit tool access
max_memory_usage = "string"           # Memory usage limit
```

### Capability Naming

Use descriptive, hyphenated capability names:

- `data-analysis`, `report-generation`
- `customer-support`, `order-tracking`
- `code-review`, `documentation-writing`
- `project-management`, `task-scheduling`

This enables clear agent identification and routing.

### Memory Configuration

Choose memory scope based on use case:

- **agent**: Private to this agent instance
- **workspace**: Shared within a project or team
- **global**: Shared across all agents (use carefully)

### Tool Integration

List external tools your agent needs:

```toml
tools = [
  "http_client",          # Web requests
  "database_connection",  # Database access
  "email_service",       # Email sending
  "file_storage",        # File operations
  "calendar_integration", # Calendar access
  "notification_service" # Push notifications
]
```

## Testing Your Agent

### Unit Testing Agent Responses

```bash
# Test basic functionality
caxton agent test task-manager.toml \
  --scenario "basic_task_creation" \
  --input '{
    "request": "Add task: Review contract by Thursday",
    "user_context": "Legal team member"
  }'

# Test memory integration
caxton agent test task-manager.toml \
  --scenario "memory_recall" \
  --input '{
    "request": "What tasks have I completed this week?",
    "user_context": "Weekly review"
  }' \
  --with-memory
```

### Load Testing

```bash
# Test under concurrent load
caxton load-test \
  --agent TaskManager \
  --capability task-management \
  --concurrent-requests 10 \
  --duration 60s
```

### Memory Performance

```bash
# Check memory system performance
caxton memory stats TaskManager

# View agent learning patterns
caxton memory inspect TaskManager --relationships
```

## Deployment Strategies

### Blue-Green Deployment

Update agents without downtime:

```bash
# Deploy new version alongside current
caxton agent deploy task-manager-v2.toml --strategy blue-green

# Test new version
caxton agent test TaskManager-v2

# Switch traffic to new version
caxton agent promote TaskManager-v2
```

### A/B Testing

Compare agent versions:

```bash
# Deploy variant for testing
caxton agent deploy task-manager-variant.toml \
  --strategy a-b-test \
  --traffic-split 20

# Monitor performance differences
caxton agent compare TaskManager TaskManager-variant
```

## Debugging and Monitoring

### View Agent Activity

```bash
# Real-time logs
caxton logs TaskManager --follow --level debug

# Conversation history
caxton conversations list --agent TaskManager

# Memory operations
caxton memory logs TaskManager --operations
```

### Performance Metrics

```bash
# Response time statistics
caxton metrics TaskManager --capability task-management

# Memory usage trends
caxton metrics TaskManager --memory --time-range 1d

# Success/failure rates
caxton metrics TaskManager --error-rates --groupby capability
```

### Health Checks

```bash
# Verify agent health
caxton agent health TaskManager

# Run diagnostic tests
caxton agent diagnose TaskManager --comprehensive

# Check agent routing
caxton agent test TaskManager
```

## Best Practices

### 1. Design for Capabilities

Think in terms of **what** your agent can do, not just **who** it is:

```toml
# Good: Specific, actionable capabilities
capabilities = [
  "document-analysis",
  "compliance-checking",
  "risk-assessment"
]

# Avoid: Generic or overlapping capabilities
capabilities = [
  "general-assistant",
  "helpful-agent"
]
```

### 2. Write Clear System Prompts

Be specific about behavior and responsibilities:

```toml
system_prompt = '''
You are a financial analyst specializing in risk assessment.

When analyzing documents:
1. Identify potential financial risks and compliance issues
2. Quantify risk levels using standard metrics
3. Suggest mitigation strategies based on industry best practices
4. Reference relevant regulations and standards

Always provide confidence levels for your assessments.
'''
```

### 3. Use Memory Strategically

Enable memory for agents that benefit from learning:

```toml
[memory]
enabled = true
scope = "workspace"        # Share knowledge within team
retention = "90d"         # Keep relevant timeframe
semantic_search = true    # Find related past experiences
```

### 4. Design for Composability

Create agents that work well with others:

```toml
system_prompt = '''
When your analysis is complete, send results to agents with
"report-generation" capability for document creation.

Use direct messaging to coordinate with other agents.
'''
```

### 5. Test Thoroughly

Validate agent behavior before deployment:

```bash
# Test core functionality
caxton agent validate task-manager.toml

# Test agent routing
caxton agent validate TaskManager

# Test memory integration
caxton memory validate TaskManager
```

## Troubleshooting Common Issues

### Agent Won't Deploy

```bash
# Check TOML syntax
caxton agent validate task-manager.toml --strict

# Verify capability names
caxton capability list --available

# Check tool availability
caxton tools list --status
```

### Messages Not Routing

```bash
# Debug agent routing
caxton agent debug TaskManager

# Check agent registration
caxton agent status TaskManager --capabilities

# Verify message format
caxton message validate --content '{...}'
```

### Memory Not Working

```bash
# Check memory backend status
caxton memory status

# Verify memory permissions
caxton memory permissions TaskManager

# Test memory operations
caxton memory test TaskManager --basic-operations
```

### Performance Issues

```bash
# Profile agent execution
caxton profile TaskManager --capability task-management

# Check resource usage
caxton resources TaskManager --live

# Analyze memory patterns
caxton memory analyze TaskManager --performance
```

## Next Steps

You now understand agents! Continue learning:

- **[Configuration Reference](configuration.md)** - Complete TOML schema
- **[Agent Patterns](../developer-guide/agent-patterns.md)** - Advanced
  composition patterns
- **[Memory System Guide](../developer-guide/memory-system.md)** - Deep dive into
  agent learning
- **[API Integration](rest-api-quickstart.md)** - REST API usage patterns

**Ready to build?** Start with simple single-capability agents, then compose
them into powerful multi-agent workflows!
