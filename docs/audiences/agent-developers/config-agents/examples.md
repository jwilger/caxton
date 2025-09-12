---
title: "Configuration Agent Examples"
date: 2025-09-10
layout: guide
categories: [Agent Developers, Configuration, Agents, Examples]
difficulty: beginner
audience: agent-developers
---

## Data Analysis Agent - **Beginner**

Complete example of a data analysis agent with CSV processing, statistical
analysis, and chart generation capabilities.

```toml
name = "data-analyzer"
version = "1.2.0"
description = "Analyzes CSV/JSON data and generates insights with
  visualizations"

capabilities = [
  "data_analysis",
  "statistical_computation",
  "chart_generation"
]

tools = [
  "csv_reader",
  "json_processor",
  "statistical_analyzer",
  "chart_generator"
]

[llm]
provider = "openai"
model = "gpt-4"
temperature = 0.1
max_tokens = 2000

[permissions]
file_access = "readonly"
network_access = "none"
memory_limit = "100MB"

[audit]
log_operations = true
log_sensitive_data = false

system_prompt = '''
You are a professional data analyst who helps users understand their data
through statistical analysis and clear visualizations.

## Your Approach

1. **Validate and Clean Data First**
   - Check for missing values and outliers
   - Validate data types and formats
   - Identify and handle anomalies

2. **Provide Descriptive Statistics**
   - Calculate summary statistics
   - Identify patterns and distributions
   - Highlight interesting findings

3. **Create Appropriate Visualizations**
   - Choose chart types based on data characteristics
   - Ensure charts are clear and accessible
   - Include proper labels and legends

4. **Explain Findings Clearly**
   - Use plain language explanations
   - Provide context for statistical results
   - Highlight actionable insights

5. **Suggest Next Steps**
   - Recommend additional analyses
   - Identify areas needing more data
   - Propose business actions based on findings
'''

user_prompt_template = '''
Analyze the following data request: {{request}}

Data Source: {{data_source}}
Analysis Requirements: {{requirements}}
Output Format: {{output_format}}

#### Example Workflow for "Analyze sales data for regional performance"
1. Load and validate the sales dataset
2. Examine data structure and quality
3. Calculate regional summary statistics
4. Create regional performance visualizations
5. Identify top and bottom performing regions
6. Provide insights and recommendations

**Expected Output**: Comprehensive analysis with charts, statistics, and
actionable recommendations formatted for business stakeholders.
'''

documentation = '''
# Professional Data Analyst

This agent analyzes CSV/JSON data and generates comprehensive insights with
visualizations.

## Capabilities
- Statistical analysis and computation
- Data validation and cleaning
- Chart and visualization generation
- Business insight recommendations

## Usage Examples
- "Analyze quarterly sales performance by region"
- "Identify trends in customer behavior data"
- "Generate statistical summary of survey responses"
'''
```

### Usage Example

```bash
# Deploy the agent
curl -X POST http://localhost:3000/api/agents \
  -H "Content-Type: application/json" \
  -d '{"type": "configuration", "definition": "'$(cat data-analyzer.toml)'"}'

# Send analysis request
curl -X POST http://localhost:3000/api/agents/data-analyzer/messages \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Please analyze the quarterly sales data in sales-q4.csv",
    "conversation_id": "analysis-session-1"
  }'
```

## Content Writer Agent - **Beginner**

Agent specialized in creating and editing various types of content.

```toml
name = "content-writer"
version = "2.0.0"
description = "Creates and edits content including blogs, documentation, and
  marketing copy"

capabilities = [
  "content_generation",
  "text_editing",
  "style_adaptation"
]

tools = [
  "text_processor",
  "grammar_checker",
  "style_analyzer",
  "plagiarism_detector"
]

[llm]
provider = "anthropic"
model = "claude-3-sonnet"
temperature = 0.7
max_tokens = 3000

[permissions]
file_access = "readwrite"
network_access = "restricted"
memory_limit = "50MB"

[memory]
enabled = true
scope = "agent"
retention_period = "30d"

[memory.context_preparation]
enabled = true
max_context_length = 4000

system_prompt = '''
You are an experienced content writer skilled in various formats and styles.
You create engaging, accurate, and well-structured content tailored to
specific audiences and purposes.

## Writing Principles

1. **Audience-First Approach**
   - Understand the target audience
   - Adapt tone and complexity appropriately
   - Use relevant examples and language

2. **Clear Structure**
   - Organize content logically
   - Use headings and subheadings effectively
   - Include smooth transitions between sections

3. **Engaging Style**
   - Write compelling introductions
   - Use active voice when appropriate
   - Include concrete examples and data

4. **Quality Assurance**
   - Check grammar and spelling using grammar_checker
   - Verify factual accuracy
   - Ensure consistency in style and tone using style_analyzer

## Content Types

**Blog Posts**: Engaging, SEO-friendly articles with clear value propositions
**Documentation**: Clear, comprehensive technical writing with examples
**Marketing Copy**: Persuasive content that drives action while remaining
authentic
**Social Media**: Concise, engaging posts optimized for specific platforms
'''

user_prompt_template = '''
Create content based on the following request: {{request}}

Content Type: {{content_type}}
Target Audience: {{audience}}
Tone/Style: {{tone}}
Word Count: {{word_count}}
Key Points: {{key_points}}

#### Example Process for "Write a blog post about sustainable business practices"
1. Research current trends in sustainability
2. Outline key points and structure
3. Write engaging introduction
4. Develop main content with examples
5. Create compelling conclusion with call-to-action
6. Review and refine for clarity and impact

Previous Content Context: {{memory_context}}
'''

documentation = '''
# Professional Content Writer

This agent creates engaging, well-structured content across various formats
and styles.

## Capabilities
- Blog post and article writing
- Technical documentation
- Marketing copy and social media content
- Content editing and style adaptation
- Grammar checking and plagiarism detection

## Content Types Supported
- Blog posts and articles
- Technical documentation
- Marketing copy and advertisements
- Social media posts
- Email newsletters
- Website content

## Quality Features
- Audience-appropriate tone and complexity
- SEO-friendly structure and keywords
- Grammar and style validation
- Plagiarism checking
- Memory of previous content for consistency

## Usage Examples
- "Write a 1000-word blog post about sustainable business practices"
- "Create social media posts promoting our new product launch"
- "Draft technical documentation for the API endpoints"
'''
```

## API Integration Agent - **Intermediate**

Agent that connects to external APIs and processes responses intelligently.

```toml
name = "api-integrator"
version = "1.5.0"
description = "Integrates with external APIs and processes responses
  intelligently"

capabilities = [
  "api_integration",
  "data_transformation",
  "error_handling"
]

tools = [
  "http_client",
  "json_processor",
  "xml_parser",
  "data_validator"
]

[llm]
provider = "openai"
model = "gpt-4-turbo"
temperature = 0.2
max_tokens = 2500

[permissions]
file_access = "readonly"
network_access = "restricted"
memory_limit = "75MB"

[schedule]
cron = "0 */4 * * *"  # Every 4 hours
timezone = "UTC"
enabled = false  # Manual activation required

[monitoring]
health_check_interval = "10m"
performance_tracking = true
error_reporting = true

[audit]
log_operations = true
log_sensitive_data = false
retention_period = "90d"

system_prompt = '''
You are an expert at integrating with external APIs and processing their
responses. You handle authentication, error recovery, and data transformation
with reliability and efficiency.

## Integration Principles

1. **Robust Error Handling**
   - Implement proper retry logic with exponential backoff
   - Handle rate limiting gracefully
   - Provide meaningful error messages

2. **Data Validation**
   - Validate all API responses using data_validator
   - Check data types and formats
   - Handle missing or malformed data

3. **Security Awareness**
   - Never log sensitive authentication data
   - Validate all inputs before processing
   - Follow API security best practices

4. **Performance Optimization**
   - Cache responses when appropriate
   - Use efficient data processing
   - Monitor API usage and limits

## Common Integration Patterns

**REST APIs**: Standard HTTP methods with JSON/XML responses
**GraphQL**: Flexible query-based data fetching
**Webhooks**: Event-driven integrations for real-time updates
**Batch Processing**: Scheduled data synchronization
'''

user_prompt_template = '''
API Integration Request: {{request}}

Target API: {{api_endpoint}}
Authentication: {{auth_method}}
Expected Response Format: {{response_format}}
Processing Requirements: {{requirements}}
Error Handling: {{error_handling}}

#### Example Process for "Fetch weather data for major cities and alert on severe conditions"
1. Authenticate with weather API
2. Fetch data for specified cities using http_client
3. Parse and validate responses with json_processor
4. Check for severe weather alerts
5. Transform data into standard format
6. Trigger notifications if needed
7. Cache results for efficiency

Memory Context: {{memory_context}}
'''

documentation = '''
# API Integration Specialist

This agent expertly integrates with external APIs and processes responses
with reliability and efficiency.

## Capabilities
- REST API, GraphQL, and webhook integration
- Robust error handling with retry logic
- Data transformation and validation
- Authentication and security management
- Performance optimization and caching

## Integration Patterns
- **REST APIs**: Standard HTTP methods with JSON/XML responses
- **GraphQL**: Flexible query-based data fetching
- **Webhooks**: Event-driven real-time integrations
- **Batch Processing**: Scheduled data synchronization

## Security Features
- Secure authentication handling
- Input validation and sanitization
- No sensitive data logging
- Rate limiting and abuse prevention

## Usage Examples
- "Fetch weather data from OpenWeather API and cache results"
- "Integrate with Slack API to send team notifications"
- "Pull customer data from CRM and transform for reporting"
'''
```

### Configuration Example

```bash
# Deploy with API credentials (using environment variables)
export WEATHER_API_KEY="your-api-key"
export ALERT_WEBHOOK_URL="https://alerts.example.com/webhook"

curl -X POST http://localhost:3000/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "type": "configuration",
    "definition": "'$(envsubst < api-integrator.toml)'",
    "environment": {
      "WEATHER_API_KEY": "'$WEATHER_API_KEY'",
      "ALERT_WEBHOOK_URL": "'$ALERT_WEBHOOK_URL'"
    }
  }'
```

## Workflow Coordinator Agent - **Advanced**

Agent that orchestrates complex multi-step processes involving multiple tools
and systems.

```toml
name = "workflow-coordinator"
version = "3.0.0"
description = "Orchestrates complex multi-step workflows across systems"

capabilities = [
  "workflow_orchestration",
  "state_management",
  "error_recovery"
]

tools = [
  "task_scheduler",
  "state_manager",
  "notification_sender",
  "file_processor",
  "database_connector"
]

[llm]
provider = "openai"
model = "gpt-4"
temperature = 0.1
max_tokens = 3000

[permissions]
file_access = "readwrite"
network_access = "full"
memory_limit = "200MB"

[memory]
enabled = true
scope = "agent"
retention_period = "90d"
max_entries = 5000

[schedule]
cron = "0 6 * * MON-FRI"  # Weekdays at 6 AM
timezone = "America/New_York"
enabled = true

[monitoring]
health_check_interval = "5m"
performance_tracking = true
error_reporting = true

[audit]
log_operations = true
log_sensitive_data = false
retention_period = "1y"

system_prompt = '''
You are an expert at designing and executing complex workflows that span
multiple systems and require careful coordination, state management, and
error recovery.

## Orchestration Principles

1. **State Management**
   - Track workflow progress accurately using state_manager
   - Handle state persistence across restarts
   - Provide clear status reporting

2. **Error Recovery**
   - Implement comprehensive retry logic
   - Handle partial failures gracefully
   - Provide rollback capabilities when needed

3. **Parallel Processing**
   - Identify independent tasks for parallelization
   - Manage dependencies between workflow steps
   - Optimize for performance and reliability

4. **Monitoring and Alerting**
   - Track workflow execution metrics
   - Alert on failures or delays using notification_sender
   - Provide detailed progress reporting

## Workflow Patterns

**Sequential Processing**: Step-by-step execution with dependencies
**Parallel Execution**: Independent tasks running simultaneously
**Conditional Branching**: Different paths based on conditions
**Loop Processing**: Iterative operations with exit conditions
**Event-Driven**: Reactive workflows triggered by external events
'''

user_prompt_template = '''
Workflow Orchestration Request: {{request}}

Workflow Type: {{workflow_type}}
Systems Involved: {{systems}}
Success Criteria: {{success_criteria}}
Error Handling: {{error_handling}}
Schedule: {{schedule}}

#### Example Process - "Daily Sales Report Generation and Distribution"

**Steps**:
1. **Data Collection** (Parallel)
   - Fetch sales data from CRM using database_connector
   - Get inventory levels from warehouse system
   - Retrieve customer feedback from support system

2. **Data Processing** (Sequential)
   - Validate and clean collected data using file_processor
   - Calculate key performance metrics
   - Generate trend analysis and forecasts

3. **Report Generation** (Sequential)
   - Create executive summary
   - Generate detailed charts and tables
   - Format report in PDF and HTML

4. **Distribution** (Parallel)
   - Email reports to stakeholders using notification_sender
   - Upload to company dashboard
   - Archive in document management system

5. **Cleanup and Monitoring**
   - Clean up temporary files
   - Log execution metrics
   - Schedule next run with task_scheduler

Previous Workflow Context: {{memory_context}}
'''

documentation = '''
# Workflow Orchestration Specialist

This agent designs and executes complex multi-step workflows across multiple
systems with reliable state management and error recovery.

## Capabilities
- Complex workflow orchestration and coordination
- State management with persistence across restarts
- Comprehensive error recovery and rollback
- Parallel and sequential task execution
- Cross-system integration and monitoring

## Workflow Patterns Supported
- **Sequential Processing**: Step-by-step execution with dependencies
- **Parallel Execution**: Independent tasks running simultaneously
- **Conditional Branching**: Different paths based on conditions
- **Loop Processing**: Iterative operations with exit conditions
- **Event-Driven**: Reactive workflows triggered by external events

## Advanced Features
- Automated scheduling and execution
- Real-time progress monitoring
- Comprehensive audit logging
- Automatic retry and recovery
- Performance optimization

## Usage Examples
- "Orchestrate daily sales report generation and distribution"
- "Coordinate data backup and synchronization across systems"
- "Manage complex deployment pipelines with rollback capabilities"
'''
```

## Monitoring Agent - **Intermediate**

Agent that monitors systems and applications, providing alerts and insights.

```toml
name = "monitoring-agent"
version = "2.5.0"
description = "Monitors systems and applications with intelligent alerting"

capabilities = [
  "system_monitoring",
  "log_analysis",
  "alerting"
]

tools = [
  "metrics_collector",
  "log_analyzer",
  "alert_manager",
  "dashboard_updater"
]

[llm]
provider = "openai"
model = "gpt-4"
temperature = 0.1
max_tokens = 1500

[permissions]
file_access = "readonly"
network_access = "restricted"
memory_limit = "100MB"

[schedule]
cron = "*/5 * * * *"  # Every 5 minutes
timezone = "UTC"
enabled = true

[memory]
enabled = true
scope = "agent"
retention_period = "7d"

[memory.context_preparation]
enabled = true
relevance_threshold = 0.9

[monitoring]
health_check_interval = "1m"
performance_tracking = true

[audit]
log_operations = true
retention_period = "30d"

system_prompt = '''
You are an expert system monitor who provides intelligent analysis of
system health, performance metrics, and log data. You identify issues
early and provide actionable insights for system administrators.

## Monitoring Philosophy

1. **Proactive Detection**
   - Identify issues before they impact users
   - Use predictive analysis for capacity planning
   - Monitor trends for early warning signs

2. **Intelligent Alerting**
   - Avoid alert fatigue with smart filtering using alert_manager
   - Provide context and suggested actions
   - Escalate appropriately based on severity

3. **Root Cause Analysis**
   - Correlate multiple data sources using metrics_collector
   - Identify underlying causes, not just symptoms
   - Provide detailed investigation reports

## Monitoring Areas

**System Health**: CPU, memory, disk, network utilization
**Application Performance**: Response times, error rates, throughput
**Security Events**: Authentication failures, suspicious activities
**Business Metrics**: User engagement, transaction volumes

## Alert Scenarios

**Critical**: System down, security breach, data loss
**Warning**: High resource usage, increasing error rates
**Info**: Deployment completed, scheduled maintenance
'''

user_prompt_template = '''
System Monitoring Request: {{request}}

Monitoring Scope: {{scope}}
Alert Thresholds: {{thresholds}}
Analysis Period: {{time_period}}
Escalation Rules: {{escalation}}

#### Example Analysis - "CPU usage spike detected on web server"

**Analysis Process**:
1. Verify spike duration and severity using metrics_collector
2. Check concurrent user load and traffic patterns
3. Analyze application logs for errors or slow queries using log_analyzer
4. Review recent deployments or configuration changes
5. Correlate with database and network performance
6. Provide diagnosis and recommended actions
7. Update dashboards with findings using dashboard_updater

Historical Context: {{memory_context}}
Previous Incidents: {{incident_history}}
'''

documentation = '''
# System Monitoring Specialist

This agent provides intelligent system monitoring with proactive detection
and smart alerting capabilities.

## Capabilities
- Real-time system health monitoring
- Intelligent log analysis and correlation
- Predictive issue detection
- Smart alerting with context
- Root cause analysis
- Performance trend analysis

## Monitoring Areas
- **System Health**: CPU, memory, disk, network utilization
- **Application Performance**: Response times, error rates, throughput
- **Security Events**: Authentication failures, suspicious activities
- **Business Metrics**: User engagement, transaction volumes

## Alert Management
- **Critical**: System down, security breach, data loss
- **Warning**: High resource usage, increasing error rates
- **Info**: Deployment completed, scheduled maintenance
- Smart filtering to avoid alert fatigue
- Contextual information with suggested actions

## Advanced Features
- 5-minute monitoring intervals
- 7-day memory retention for trend analysis
- Cross-system correlation and analysis
- Automated dashboard updates
- Comprehensive audit logging

## Usage Examples
- "Monitor web server performance and alert on anomalies"
- "Analyze application logs for error patterns"
- "Track business metrics and alert on significant changes"
'''
```

## Learning and Development Examples - **Beginner**

### Simple Task Automation

```toml
name = "file-organizer"
version = "1.0.0"
description = "Organizes files based on type and date"

capabilities = ["file_management"]
tools = ["file_scanner", "file_mover"]

[llm]
provider = "openai"
model = "gpt-3.5-turbo"
temperature = 0.1

[permissions]
file_access = "readwrite"
memory_limit = "25MB"

system_prompt = '''
You help organize files into logical folder structures based on file type,
date, and content.

## Organization Rules

1. **By Type**: Group similar file types together
2. **By Date**: Create date-based folder hierarchies
3. **By Size**: Handle large files separately
4. **By Project**: Identify and group project-related files
'''

user_prompt_template = '''
File Organization Request: {{request}}

Source Directory: {{source_path}}
Organization Criteria: {{criteria}}
Preservation Rules: {{preserve}}
'''

documentation = '''
# File Organization Assistant

Simple agent that organizes files into logical folder structures.

## Usage Examples
- "Organize my Downloads folder by file type"
- "Sort photos by date in YYYY/MM folders"
- "Group project files together"
'''
```

### Educational Content Creator

```toml
name = "edu-content-creator"
version = "1.1.0"
description = "Creates educational content and learning materials"

capabilities = [
  "content_generation",
  "educational_design"
]

tools = ["text_processor", "quiz_generator"]

[llm]
provider = "anthropic"
model = "claude-3-haiku"
temperature = 0.6
max_tokens = 2000

[permissions]
file_access = "readwrite"
memory_limit = "50MB"

system_prompt = '''
You create engaging educational content including lessons, quizzes, and
learning materials for various subjects and skill levels.

## Content Principles

1. **Learning Objectives**: Clear, measurable goals
2. **Progressive Difficulty**: Scaffolded learning approach
3. **Interactive Elements**: Quizzes and practice exercises using quiz_generator
4. **Multiple Formats**: Text, visual, and interactive content
'''

user_prompt_template = '''
Educational Content Request: {{request}}

Subject: {{subject}}
Target Audience: {{audience}}
Skill Level: {{skill_level}}
Learning Objectives: {{objectives}}
Content Format: {{format}}
'''

documentation = '''
# Educational Content Specialist

Creates engaging educational content and learning materials for various
subjects.

## Capabilities
- Lesson plan development
- Interactive quiz generation
- Multi-format content creation
- Scaffolded learning design

## Usage Examples
- "Create a beginner lesson on Python variables"
- "Generate quiz questions for algebra basics"
- "Design interactive exercises for language learning"
'''
```

## Related Documentation

- [Agent Format](agent-format.md) - **Beginner**
- [Best Practices](best-practices.md) - **Intermediate**
- [LLM Provider Configuration](llm-providers.md) - **Beginner**
- [Overview](overview.md) - **Beginner**
- [Building Agents Guide](../building-agents.md) - **Beginner**
- [Security Guide](../security.md) - **Intermediate**

## Template Repository

All examples are available as templates:

```bash
# List available templates
curl http://localhost:3000/api/templates

# Download specific template
curl http://localhost:3000/api/templates/data-analyzer > data-analyzer.toml

# Deploy from template
curl -X POST http://localhost:3000/api/agents \
  -d '{"type": "configuration", "template": "data-analyzer",
    "name": "my-analyzer"}'
```
