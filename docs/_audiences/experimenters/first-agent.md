---
title: "Create Your First Config Agent"
date: 2025-09-10
layout: page
audience: experimenters
navigation_order: 2
categories: [Experimenters, Tutorials]
---

> **🚧 Implementation Status**
>
> This tutorial represents the intended configuration agent development
> experience from ADR-28. The YAML schema and agent deployment workflow
> described here serves as acceptance criteria for implementation.
>
> **Target**: 5-minute configuration-driven agent creation
> **Status**: Agent format specification and runtime being developed

## Build intelligent agents in 5 minutes using configuration files

This guide teaches you to create **configuration agents** - the primary way to
build agents in Caxton. No compilation, no complex toolchains, just markdown
files with YAML frontmatter.

Perfect for researchers and experimenters who want to focus on agent behavior
and multi-agent interactions rather than infrastructure complexity.

## Configuration Agent Fundamentals

Every configuration agent consists of:

1. **YAML frontmatter** - Agent metadata, capabilities, and configuration
2. **Markdown content** - Documentation and usage examples
3. **System prompts** - Instructions defining agent behavior
4. **Capability declarations** - What the agent can do
5. **Tool integrations** - External services the agent can use

## Agent Lifecycle

Configuration agents follow a simple lifecycle:

- **Creation**: Write markdown file with YAML configuration
- **Deployment**: `caxton agent deploy agent.md` validates and loads
- **Execution**: Agent processes capability-based messages via LLM
  orchestration
- **Learning**: Agent stores successful patterns in embedded memory
- **Updates**: Edit config file and redeploy for instant changes

## Your First Agent: Task Manager

Let's create a task management agent that helps users organize and track work.
This example demonstrates core agent concepts while building something
practically useful.

### 1. Create the Agent File

Create `task-manager.md`:

```yaml
---
name: TaskManager
version: "1.0.0"
description: "Intelligent task management and productivity assistant"
capabilities:
  - task-management
  - productivity-coaching
  - time-tracking
tools:
  - calendar_integration
  - notification_service
  - file_storage
memory:
  enabled: true
  scope: workspace
  retention: "30d"
parameters:
  max_tasks_per_user: 100
  default_priority: "medium"
  time_zone: "UTC"
system_prompt: |
  You are TaskManager, an intelligent productivity assistant. Your role is to
  help users organize, prioritize, and complete their work effectively.

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

  Personality: Encouraging, organized, and practical. Help users feel
  accomplished.

user_prompt_template: |
  Task Request: {{request}}

  User Context: {{user_context}}
  Current Tasks: {{current_tasks}}
  Relevant Memory: {{memory_context}}
  Deadline Information: {{deadlines}}

  Please help with this task management request.
---

## TaskManager Agent

An intelligent task management assistant that helps you stay organized and
productive.

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
```

### 2. Deploy Your Agent

```bash
# Deploy the task manager
caxton agent deploy task-manager.md

# Verify it's running
caxton agent list
```

Expected output:

```text
NAME         TYPE     STATUS    CAPABILITIES
TaskManager  config   running   task-management, productivity-coaching
```

### 3. Interact with Your Agent

Send capability-based messages:

```bash
# Add a task
caxton message send \
  --capability "task-management" \
  --performative request \
  --content '{
    "request": "Add a task to prepare slides for Monday presentation",
    "user_context": "Working on quarterly review",
    "deadline": "2025-09-15T09:00:00Z"
  }'

# Get productivity advice
caxton message send \
  --capability "productivity-coaching" \
  --performative query \
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

```yaml
---
name: CustomerSupport
version: "2.0.0"
capabilities:
  - customer-inquiry
  - order-tracking
  - technical-support
  - escalation-management
tools:
  - crm_system
  - knowledge_base
  - email_service
  - ticket_system
memory:
  enabled: true
  scope: global  # Share knowledge across all support interactions
system_prompt: |
  You are a customer support specialist with access to multiple systems.
  Route inquiries appropriately and escalate when needed.

  For each capability:
  - customer-inquiry: Handle general questions and provide information
  - order-tracking: Look up order status and shipping information
  - technical-support: Troubleshoot product issues and provide solutions
  - escalation-management: Route complex issues to human agents

  Always check memory for similar issues and their resolutions.
---
```

### Workflow Orchestration Agent

Create an agent that coordinates other agents:

```yaml
---
name: ProjectOrchestrator
version: "1.0.0"
capabilities:
  - project-coordination
  - workflow-management
memory:
  enabled: true
  scope: workspace
system_prompt: |
  You coordinate complex projects by delegating tasks to other agents.

  When you receive project requests:
  1. Break down the project into subtasks
  2. Send subtasks to appropriate capabilities
  3. Monitor progress and coordinate between agents
  4. Aggregate results into final deliverable

  Use capability-based messaging to delegate work:
  - Send data analysis tasks to "data-analysis" capability
  - Send document creation to "document-generation" capability
  - Send notifications via "notification-service" capability
---
```

### Memory-Intensive Learning Agent

Create an agent optimized for learning and knowledge building:

```yaml
---
name: KnowledgeAssistant
version: "1.0.0"
capabilities:
  - knowledge-search
  - learning-assistance
  - information-synthesis
tools:
  - web_search
  - document_parser
  - citation_manager
memory:
  enabled: true
  scope: global
  semantic_search: true
  relationship_tracking: true
parameters:
  max_search_results: 10
  citation_style: "APA"
system_prompt: |
  You are a knowledge assistant that helps users learn and research topics.

  Your unique strength is building and connecting knowledge over time:
  1. Search your memory for related concepts and prior research
  2. Identify knowledge gaps and suggest research directions
  3. Synthesize information from multiple sources
  4. Store new insights and their relationships to existing knowledge
  5. Build semantic maps of interconnected concepts

  Always explain how new information connects to what you've learned before.
---
```

## Configuration Schema Reference

### Required Fields

```yaml
name: string              # Unique agent identifier
version: string          # Semantic version
capabilities: [string]   # What the agent can do
system_prompt: string    # Core behavior instructions
```

### Optional Configuration

```yaml
description: string                # Human-readable description
tools: [string]                   # External services to use
memory:
  enabled: boolean                # Enable persistent memory
  scope: "agent"|"workspace"|"global"  # Memory sharing level
  retention: string             # How long to keep memories
  semantic_search: boolean      # Enable vector search
  relationship_tracking: boolean # Track entity relationships
parameters:                       # Custom agent parameters
  key: value
user_prompt_template: string     # Template for user interactions
conversation:
  max_turns: integer            # Conversation length limit
  timeout: string              # Response timeout
security:
  restricted_tools: [string]    # Limit tool access
  max_memory_usage: string     # Memory usage limit
```

### Capability Naming

Use descriptive, hyphenated capability names:

- `data-analysis`, `report-generation`
- `customer-support`, `order-tracking`
- `code-review`, `documentation-writing`
- `project-management`, `task-scheduling`

This enables precise capability-based routing.

### Memory Configuration

Choose memory scope based on use case:

- **agent**: Private to this agent instance
- **workspace**: Shared within a project or team
- **global**: Shared across all agents (use carefully)

### Tool Integration

List external tools your agent needs:

```yaml
tools:
  - http_client          # Web requests
  - database_connection  # Database access
  - email_service       # Email sending
  - file_storage        # File operations
  - calendar_integration # Calendar access
  - notification_service # Push notifications
```

## Experimentation Techniques

### Pattern Discovery Experiments

Enable memory and run similar tasks to discover patterns:

```bash
# Run multiple related analyses
caxton message send --capability "data-analysis" --content '{"dataset": "jan-sales.csv"}'
caxton message send --capability "data-analysis" --content '{"dataset": "feb-sales.csv"}'
caxton message send --capability "data-analysis" --content '{"dataset": "mar-sales.csv"}'

# Check what patterns the agent discovered
caxton memory inspect DataAnalyzer --patterns
```

### Emergent Behavior Studies

Create agents with overlapping capabilities and observe self-organization:

```yaml
# Agent 1: Broad data analysis
capabilities: ["data-analysis", "visualization"]

# Agent 2: Specialized in finance
capabilities: ["data-analysis", "financial-modeling"]

# Agent 3: Report specialist
capabilities: ["visualization", "report-generation"]
```

### Knowledge Building Experiments

Use global memory scope to create knowledge-accumulating agents:

```yaml
memory:
  enabled: true
  scope: global
  semantic_search: true
  relationship_tracking: true
```

### A/B Testing Agent Variants

Compare different system prompts or capabilities:

```bash
# Deploy variant for testing
caxton agent deploy task-manager-variant.md \
  --strategy a-b-test \
  --traffic-split 20

# Monitor performance differences
caxton agent compare TaskManager TaskManager-variant
```

## Testing Your Agent

### Unit Testing Agent Responses

```bash
# Test basic functionality
caxton agent test task-manager.md \
  --scenario "basic_task_creation" \
  --input '{
    "request": "Add task: Review contract by Thursday",
    "user_context": "Legal team member"
  }'

# Test memory integration
caxton agent test task-manager.md \
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

### Memory Performance Analysis

```bash
# Check memory system performance
caxton memory stats TaskManager

# View agent learning patterns
caxton memory inspect TaskManager --relationships

# Analyze knowledge growth over time
caxton memory analyze TaskManager --growth-patterns
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

# Check capability routing
caxton capability test task-management
```

## Advanced Experimentation

### Multi-Agent Societies

Create societies of agents with different specializations:

```yaml
# Data Scientist Agent
capabilities: ["data-analysis", "statistical-modeling"]

# Business Analyst Agent
capabilities: ["business-analysis", "requirement-gathering"]

# Project Manager Agent
capabilities: ["project-coordination", "resource-planning"]

# Report Writer Agent
capabilities: ["document-generation", "presentation-creation"]
```

### Emergent Communication Protocols

Let agents develop their own communication patterns:

```yaml
system_prompt: |
  You can communicate with other agents by sending capability-based messages.
  Develop efficient communication patterns and learn from successful
  collaborations.
```

### Knowledge Graph Experiments

Use relationship tracking to build knowledge graphs:

```yaml
memory:
  enabled: true
  relationship_tracking: true
  semantic_search: true

# Then observe how agents build interconnected knowledge
caxton memory visualize --agent KnowledgeBot --format graph
```

### Adaptation Experiments

Create agents that adapt their behavior based on success metrics:

```yaml
system_prompt: |
  Monitor your success rate and adapt your approach based on feedback.
  Store successful strategies in memory and avoid patterns that fail.
```

## Best Practices for Experimenters

### 1. Start Simple, Iterate Quickly

Begin with single-capability agents:

```yaml
# Good: Simple, focused agent
capabilities: ["document-analysis"]

# Avoid: Complex, multi-purpose agent initially
capabilities: ["analysis", "generation", "management", "coordination"]
```

### 2. Enable Memory for Learning

Always enable memory for experimental agents:

```yaml
memory:
  enabled: true
  scope: workspace        # Share knowledge appropriately
  semantic_search: true   # Find related past experiences
  relationship_tracking: true  # Build knowledge graphs
```

### 3. Use Descriptive System Prompts

Be specific about experimental goals:

```yaml
system_prompt: |
  You are experimenting with adaptive task prioritization strategies.

  Try different approaches:
  1. Deadline-based prioritization
  2. Importance-weighted scheduling
  3. User preference learning

  Store successful strategies and their effectiveness metrics.
```

### 4. Monitor Learning Patterns

Track how agents learn and adapt:

```bash
# Monitor memory growth
caxton memory stats --agent ExperimentAgent --live

# Analyze learning patterns
caxton memory analyze ExperimentAgent --learning-curves

# Export data for analysis
caxton memory export ExperimentAgent --format csv
```

### 5. Document Your Experiments

Use the markdown content to document experiments:

```markdown
## Experiment: Adaptive Priority Learning

### Hypothesis
Agents can learn user-specific prioritization preferences through interaction
patterns.

### Methodology
1. Deploy agent with priority learning capability
2. Track task completion patterns over 2 weeks
3. Measure adaptation accuracy using success metrics

### Results
- Initial accuracy: 67%
- Final accuracy: 89%
- Learning stabilized after ~50 interactions
```

## Troubleshooting Common Issues

### Agent Won't Deploy

```bash
# Check YAML syntax
caxton agent validate task-manager.md --strict

# Verify capability names
caxton capability list --available

# Check tool availability
caxton tools list --status
```

### Messages Not Routing

```bash
# Debug capability routing
caxton capability debug task-management

# Check agent registration
caxton agent status TaskManager --capabilities

# Verify message format
caxton message validate --performative request --content '{...}'
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

## Research Integration

### Data Collection

Export agent behavior data for research:

```bash
# Export conversation data
caxton conversations export --agent TaskManager --format json

# Export memory evolution
caxton memory export --agent TaskManager --include-history

# Export performance metrics
caxton metrics export --agent TaskManager --time-range 30d
```

### Integration with Research Tools

```python
# Python research integration
from caxton import CaxtonClient
import pandas as pd

client = CaxtonClient('http://localhost:8080')

# Collect agent performance data
metrics = client.agents.get_metrics('TaskManager')
df = pd.DataFrame(metrics)

# Analyze learning curves
learning_data = client.memory.get_learning_curves('TaskManager')
```

## Next Steps for Experimenters

You now understand configuration agents! Continue experimenting:

- **[Quickstart](quickstart.md)** - Multi-agent workflow examples
- **[Configuration Reference](../operators/configuration.md)** - Complete
  YAML schema
- **[Agent Patterns](../../developer-guide/agent-patterns.md)** - Advanced
  composition patterns
- **[Memory System Guide](../../memory-system/overview.md)** - Deep dive into
  agent learning
- **[API Integration](../agent-developers/api-quickstart.md)** - REST API
  usage patterns

### Experimental Ideas

1. **Swarm Intelligence**: Create agents that solve problems collectively
2. **Language Evolution**: Let agents develop domain-specific communication
3. **Adaptive Workflows**: Build agents that optimize their own processes
4. **Knowledge Networks**: Create interconnected knowledge-building agents
5. **Emergent Specialization**: Let agents discover their own niches

**Ready to experiment?** Start with simple single-capability agents, then
explore how they interact and evolve in multi-agent environments!
