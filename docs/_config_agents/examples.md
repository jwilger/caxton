---
title: "Configuration Agent Examples"
date: 2025-09-10
layout: guide
categories: [Configuration, Agents, Examples]
---

## Data Analysis Agent

Complete example of a data analysis agent with CSV processing, statistical
analysis, and chart generation capabilities.

```toml
name = "data-analyzer"
version = "1.2.0"
description = "Analyzes CSV/JSON data and generates insights with visualizations"
capabilities = [
  "data-analysis",
  "statistical-computation",
  "chart-generation"
]
tools = [
  "csv_parser",
  "json_processor",
  "statistics_engine",
  "chart_generator",
  "file_storage"
]

system_prompt = '''
You are a professional data analyst who helps users understand their data
through statistical analysis and clear visualizations.

Your approach:
1. Validate and clean the data first
2. Provide descriptive statistics and identify patterns
3. Create appropriate visualizations based on data types
4. Explain findings in plain language
5. Suggest actionable insights and next steps

Data handling rules:
- Always check for missing values and outliers
- Respect data privacy and never store sensitive information
- Use appropriate statistical tests for the data distribution
- Create visualizations that enhance understanding, not just decoration
'''

user_prompt_template = '''
Analyze this data request:

**Data Source**: {{data_source}}
**User Request**: {{request}}
**Analysis Requirements**: {{requirements}}

Please provide:
1. Data summary (rows, columns, data types)
2. Data quality assessment (missing values, outliers)
3. Statistical analysis based on the request
4. Appropriate visualizations
5. Key insights and recommendations
'''

[tool_config.csv_parser]
max_file_size = "10MB"
encoding = "utf-8"
delimiter = ","

[tool_config.statistics_engine]
confidence_level = 0.95
precision = 4

[tool_config.chart_generator]
default_width = 800
default_height = 600
formats = ["png", "svg"]

[conversation]
max_turns = 20
context_window = 8000
memory_strategy = "summarize"

[performance]
max_execution_time = "120s"
max_memory_usage = "512MB"
concurrent_tools = 2

[parameters]
supported_formats = ["csv", "json", "xlsx"]
max_rows = 100000
chart_types = ["bar", "line", "scatter", "histogram", "box"]

author = "Caxton Team"
license = "MIT"
tags = ["data", "analytics", "visualization", "statistics"]
created = "2025-09-10T00:00:00Z"
updated = "2025-09-10T00:00:00Z"

documentation = '''
# Data Analyzer Agent

This agent specializes in data analysis tasks, providing statistical insights
and visualizations for CSV, JSON, and Excel datasets.

## Capabilities

- **Data Import**: Parse CSV, JSON, and Excel files up to 10MB
- **Statistical Analysis**: Descriptive statistics, correlation, hypothesis
  testing
- **Visualization**: Bar charts, line plots, scatter plots, histograms, box
  plots
- **Data Quality**: Missing value detection, outlier identification
- **Insights**: Plain-language explanations of findings and recommendations

## Usage Examples

**Basic Analysis**:
```text
Please analyze this sales data: sales_2024.csv
I want to understand monthly trends and identify top-performing products.
```

**Detailed Statistical Analysis**:

```text
Analyze customer_survey.json for satisfaction patterns.
Requirements: correlation analysis, confidence intervals, regional breakdowns.
```

**Quick Data Summary**:

```text
Give me a quick overview of employee_data.xlsx - what's the structure and
any obvious patterns?
```

## Output Format

The agent provides structured analysis with:

1. **Data Summary**: Row/column counts, data types, basic statistics
2. **Quality Assessment**: Missing values, duplicates, outliers
3. **Analysis Results**: Statistical tests, correlations, distributions
4. **Visualizations**: Charts saved to file storage with descriptions
5. **Insights**: Key findings in business-friendly language
6. **Recommendations**: Suggested next steps or follow-up analyses
'''

```toml

## API Integration Agent

Example of an agent that integrates with REST APIs, handles authentication,
and processes responses.

```toml
name = "api-integrator"
version = "1.0.0"
description = "Integrates with REST APIs for data fetching and processing"
capabilities = [
  "api-integration",
  "data-transformation",
  "authentication-handling"
]
tools = [
  "http_client",
  "json_processor",
  "auth_manager",
  "data_transformer"
]

system_prompt = '''
You are an API integration specialist who helps users fetch and process data
from REST APIs safely and efficiently.

Your responsibilities:
1. Understand API requirements and authentication needs
2. Make secure, authenticated requests with proper error handling
3. Transform API responses into usable formats
4. Handle rate limiting and retry logic appropriately
5. Validate data integrity and report any issues

Security principles:
- Never log or expose authentication credentials
- Always validate SSL certificates
- Respect rate limits and API terms of service
- Handle errors gracefully with informative messages
'''

user_prompt_template = '''
API Integration Request:

**Endpoint**: {{endpoint}}
**Authentication**: {{auth_type}}
**Request Type**: {{method}}
**Data Processing**: {{processing_requirements}}

User Instructions: {{request}}

I'll handle the API call, authentication, and data processing according to
your requirements.
'''

[tool_config.http_client]
timeout = "30s"
max_retries = 3
retry_delay = "1s"

[tool_config.auth_manager]
supported_types = ["bearer", "basic", "api_key"]

[parameters]
supported_methods = ["GET", "POST", "PUT", "DELETE"]
max_response_size = "50MB"

[parameters.default_headers]
User-Agent = "Caxton-API-Agent/1.0.0"
Accept = "application/json"

documentation = '''
# API Integrator Agent

Handles REST API integrations with authentication, error handling, and data
transformation capabilities.

## Usage Examples

**Fetch JSON Data**:
```text
Get customer data from https://api.example.com/customers
Authentication: Bearer token from environment variable API_TOKEN
Transform the response to CSV format
```

**POST Request with Data**:

```text
Submit this order data to https://api.store.com/orders
Use API key authentication with key "store_api_key"
Data: {"customer_id": 12345, "items": [{"sku": "ABC123", "quantity": 2}]}
```

'''

```toml

## Content Generator Agent

Example of an agent that generates various types of content with templates
and style guidelines.

```toml
name = "content-generator"
version = "1.1.0"
description = "Generates documentation, reports, and marketing content"
capabilities = [
  "content-generation",
  "template-processing",
  "document-formatting"
]
tools = [
  "text_processor",
  "template_engine",
  "markdown_formatter",
  "pdf_generator"
]

system_prompt = '''
You are a professional content writer who creates clear, engaging, and
well-structured content for various business needs.

Your writing principles:
1. Clarity: Use simple, direct language appropriate for the audience
2. Structure: Organize content with clear headings and logical flow
3. Accuracy: Verify facts and maintain consistency throughout
4. Engagement: Write in an active voice with compelling openings
5. Purpose: Every piece of content should have a clear objective

Content types you excel at:
- Technical documentation and user guides
- Business reports and executive summaries
- Marketing copy and product descriptions
- Email templates and communications
- Process documentation and procedures
'''

user_prompt_template = '''
Content Generation Request:

**Content Type**: {{content_type}}
**Audience**: {{target_audience}}
**Tone**: {{tone_style}}
**Length**: {{target_length}}
**Key Points**: {{key_information}}

Additional Requirements: {{request}}

I'll create well-structured content that meets your specifications and
engages your target audience effectively.
'''

[parameters]
content_types = [
  "technical_documentation",
  "business_report",
  "marketing_copy",
  "user_guide",
  "email_template"
]
tone_options = [
  "professional",
  "friendly",
  "technical",
  "conversational",
  "formal"
]
max_length = 10000

documentation = '''
# Content Generator Agent

Creates professional content for documentation, reports, marketing materials,
and communications with consistent style and structure.

## Usage Examples

**Technical Documentation**:
```text
Create a user guide for our new API integration feature.
Audience: Software developers
Tone: Technical but approachable
Length: 2000-3000 words
Include code examples and troubleshooting section.
```

**Business Report**:

```text
Generate an executive summary of Q3 sales performance.
Audience: C-level executives
Tone: Professional and concise
Key points: 15% revenue growth, expanded customer base, operational challenges
Length: 500-750 words
```

'''

```toml

## Workflow Coordinator Agent

Example of an agent that orchestrates multi-step business processes by
coordinating with other agents.

```toml
name = "workflow-coordinator"
version = "1.0.0"
description = "Orchestrates complex multi-agent business workflows"
capabilities = [
  "workflow-orchestration",
  "process-automation",
  "error-handling"
]
requires = [
  "data-analysis",
  "api-integration",
  "content-generation",
  "notification-service"
]
tools = [
  "workflow_engine",
  "message_router",
  "state_manager",
  "error_handler"
]

system_prompt = '''
You are a workflow orchestration expert who manages complex business
processes by coordinating multiple agents and systems.

Your orchestration approach:
1. Break complex processes into discrete, manageable steps
2. Identify required capabilities and route requests to appropriate agents
3. Manage state and context across process steps
4. Handle errors gracefully with retry logic and fallbacks
5. Provide clear progress updates and final summaries

Process management principles:
- Always validate inputs before starting workflows
- Maintain clear audit trails of all process steps
- Implement proper error boundaries and recovery mechanisms
- Optimize for parallel execution where possible
- Provide meaningful status updates to users
'''

user_prompt_template = '''
Workflow Orchestration Request:

**Process Name**: {{workflow_name}}
**Input Data**: {{input_data}}
**Required Steps**: {{process_steps}}
**Success Criteria**: {{success_conditions}}
**Error Handling**: {{error_strategy}}

User Request: {{request}}

I'll orchestrate this workflow by coordinating the necessary agents and
managing the process from start to completion.
'''

[conversation]
max_turns = 100
context_window = 12000
memory_strategy = "persist"

[performance]
max_execution_time = "600s"
max_memory_usage = "1GB"
concurrent_tools = 5

[parameters]
max_parallel_steps = 10
retry_attempts = 3
timeout_per_step = "60s"

documentation = '''
# Workflow Coordinator Agent

Orchestrates complex business processes by coordinating multiple agents and
managing workflow state and error handling.

## Usage Examples

**Data Processing Pipeline**:
```text
Create a workflow to process customer feedback data:

1. Fetch data from API (last 30 days)
2. Analyze sentiment and themes
3. Generate executive summary report
4. Send notifications to relevant stakeholders
Error handling: Retry failed steps up to 3 times, send alerts on final failure
```

**Content Publishing Workflow**:

```text
Orchestrate blog post publication process:

1. Generate article content based on topic brief
2. Review and optimize for SEO
3. Create social media promotional content
4. Schedule publication across platforms
5. Set up performance monitoring
```

'''

```toml

## Monitoring Agent

Example of an agent that monitors system health and processes alerts.

```toml
name = "system-monitor"
version = "1.0.0"
description = "Monitors system health and processes alerts"
capabilities = [
  "health-monitoring",
  "alert-processing",
  "metric-analysis"
]
tools = [
  "metrics_collector",
  "log_analyzer",
  "alert_manager",
  "notification_service"
]

system_prompt = '''
You are a system reliability expert who monitors infrastructure health and
responds to operational issues.

Your monitoring responsibilities:
1. Continuously analyze system metrics and logs
2. Detect anomalies and performance degradation
3. Correlate events across multiple systems
4. Generate actionable alerts with appropriate severity
5. Recommend remediation steps for common issues

Alert handling principles:
- Minimize false positives through intelligent thresholds
- Provide clear context and impact assessment
- Include specific remediation steps when possible
- Route alerts to appropriate teams based on severity
- Track resolution times and improvement opportunities
'''

user_prompt_template = '''
System Monitoring Request:

**Monitoring Scope**: {{systems}}
**Alert Conditions**: {{alert_rules}}
**Notification Targets**: {{recipients}}
**Escalation Policy**: {{escalation_rules}}

Specific Request: {{request}}

I'll monitor the specified systems and handle alerts according to your
configuration and escalation policies.
'''

[conversation]
max_turns = 10
context_window = 4000
memory_strategy = "sliding"

[performance]
max_execution_time = "30s"
max_memory_usage = "256MB"
concurrent_tools = 3

[parameters]
check_interval = "60s"
alert_cooldown = "300s"
severity_levels = ["info", "warning", "error", "critical"]

documentation = '''
# System Monitor Agent

Monitors system health, analyzes metrics, and processes alerts with intelligent
routing and escalation capabilities.

## Usage Examples

**Server Health Monitoring**:
```text
Monitor web server cluster health:

- CPU usage > 80% for 5 minutes: WARNING
- Memory usage > 90% for 2 minutes: ERROR
- Response time > 5 seconds: WARNING
- Any 5xx errors: ERROR
Send alerts to ops-team@company.com with 15-minute escalation to on-call
```

**Application Performance Monitoring**:

```text
Monitor API performance and user experience:

- Track response times, error rates, and throughput
- Alert on performance degradation > 20% from baseline
- Generate daily performance summary reports
- Correlate issues with deployment events
```

'''

```toml

## Template Selection Guide

Choose the appropriate template based on your use case:

| Use Case | Template | Key Features |
|----------|----------|--------------|
| Data Processing | Data Analyzer | CSV/JSON parsing, statistics, charts |
| API Integration | API Integrator | REST calls, auth, data transformation |
| Content Creation | Content Generator | Docs, reports, marketing copy |
| Process Automation | Workflow Coordinator | Multi-agent orchestration |
| System Operations | System Monitor | Health checks, alerts, metrics |

## Customization Tips

**Prompt Engineering**: Modify `system_prompt` and `user_prompt_template` to
match your specific domain and requirements.

**Tool Selection**: Only include tools your agent actually needs to minimize
attack surface and improve performance.

**Performance Tuning**: Adjust `max_execution_time` and `max_memory_usage`
based on your workload characteristics.

**Capability Design**: Design capabilities around business functions rather
than technical features for better reusability.

## Next Steps

- **Best Practices**: Learn development guidelines and optimization techniques
- **Tool Integration**: Understand available tools and their configurations
- **Template Library**: Explore additional templates for specialized use cases
- **Migration Guide**: Convert existing WASM agents to configuration format
