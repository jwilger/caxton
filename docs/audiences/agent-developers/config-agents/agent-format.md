---
title: "Configuration Agent Format Specification"
date: 2025-09-10
layout: guide
categories: [Agent Developers, Configuration, Agents, Reference]
difficulty: beginner
audience: agent-developers
---

## File Structure - **Beginner**

Configuration agents are TOML configuration files (`.toml`) that
define agent behavior, capabilities, and documentation in a single file:

```text
agent-name.toml
├── Agent metadata
├── Capability declarations
├── Tool permissions
├── LLM configuration
├── System prompt (multiline string)
├── User prompt template (multiline string)
├── Optional parameters
└── Documentation (embedded markdown string)
    ├── Agent description
    ├── Usage instructions
    └── Example interactions
```

## TOML Schema Reference - **Beginner**

> **Note**: The following shows schema documentation mixing type annotations
> (e.g., `string`, `boolean`) with example values (e.g., `capability1`,
> `data_analysis`). See the examples section below for complete working TOML.

### Required Fields

```toml
name = "string"                    # Agent identifier (required)
version = "string"                 # Semantic version (required)
description = "string"             # Brief agent purpose (required)
capabilities = [                   # Capability declarations (required)
  "capability1",                   # List of capability names
  "capability2"
]
tools = [                          # Tool permission list (required)
  "tool1",                         # List of tool names
  "tool2"
]

[llm]                              # LLM configuration (required)
provider = "string"                # LLM provider
model = "string"                   # Model name

system_prompt = '''                # Agent system prompt (required)
System prompt content here...
'''

user_prompt_template = '''         # User prompt template (required)
User prompt template here...
'''

documentation = '''               # Agent documentation (optional)
# Agent Documentation
Markdown content here...
'''
```

### Complete Schema - **Intermediate**

```toml
# Core Identity (Required)
name = "string"                    # Agent name (kebab-case recommended)
version = "string"                # Semantic versioning (e.g., "1.0.0")
description = "string"            # Brief agent purpose (1-2 sentences)

# Capability System (Required)
capabilities = [                   # High-level capabilities
  "capability_name",              # List of capability names
  "data_analysis"                 # e.g., "data_analysis"
]

tools = [                          # Specific tools to access
  "tool_name",                    # List of tool names
  "csv_reader"                    # e.g., "csv_reader"
]

# LLM Configuration (Required)
[llm]
provider = "string"               # openai|anthropic|local
model = "string"                  # Model identifier
temperature = 0.7                 # 0.0-2.0 (optional, default: 0.7)
max_tokens = 2000                 # Max response length (optional)
top_p = 1.0                       # Nucleus sampling (optional)
frequency_penalty = 0.0           # Repetition penalty (optional)
presence_penalty = 0.0            # New topic encouragement (optional)

# Security & Permissions (Optional)
[permissions]
file_access = "readonly"          # readonly|readwrite|none
network_access = "restricted"     # restricted|full|none
memory_limit = "100MB"            # e.g., "100MB"
cpu_limit = "100ms"               # e.g., "100ms"

# Automation (Optional)
[schedule]
cron = "string"                   # Cron expression
timezone = "string"               # Timezone identifier
enabled = false                   # Schedule activation

# Memory Configuration (Optional)
[memory]
enabled = true                    # Enable persistent memory
scope = "agent"                   # Memory scope
retention_period = "30d"          # e.g., "30d"
max_entries = 1000                # Maximum stored entries

[memory.context_preparation]
enabled = true                    # Auto context preparation
max_context_length = 8000         # Max context tokens
relevance_threshold = 0.8         # Relevance threshold

# Parameters (Optional)
[parameters]
max_file_size = "10MB"            # File processing limits
supported_formats = ["csv", "json"] # Supported file formats

# Monitoring & Audit (Optional)
[audit]
log_operations = true             # Log all operations
log_sensitive_data = false        # Log sensitive data
retention_period = "1y"           # Log retention

[monitoring]
health_check_interval = "5m"      # e.g., "5m"
performance_tracking = true       # Performance tracking
error_reporting = true            # Error reporting

# Development Settings (Optional)
[development]
hot_reload = true                 # Enable hot reload
debug_mode = false                # Enable debug logging
test_mode = false                 # Test mode activation

# Agent Prompts (Required)
system_prompt = '''
You are an AI agent with specific capabilities.
Define your behavior and instructions here...
'''

user_prompt_template = '''
Handle the following request: {{request}}

Available context: {{context}}
Memory context: {{memory_context}}
'''

# Documentation (Optional)
documentation = '''
# Agent Name

Agent description and usage instructions in markdown format.

## Usage Examples

Provide examples of how to use this agent.
'''
```

## Field Descriptions - **Beginner**

### Core Identity Fields

**name** (required):

- Unique identifier for the agent
- Must be valid as filename and URL path
- Recommended: kebab-case (e.g., "data-analyzer")
- Must match pattern: `^[a-z][a-z0-9-]*[a-z0-9]$`

**version** (required):

- Semantic versioning following semver.org
- Format: `MAJOR.MINOR.PATCH` (e.g., "1.2.3")
- Used for deployment strategies and compatibility

**description** (required):

- Brief, clear explanation of agent purpose
- 1-2 sentences maximum
- Used in agent listings and documentation

### Capability System - **Intermediate**

**capabilities** (required):

- Array of high-level capabilities the agent provides
- Used for message routing and discovery
- Examples: `["data_analysis", "report_generation"]`

Available capabilities:

```toml
# Data Processing
capabilities = [
  "data_analysis",           # Analyze structured data
  "data_transformation",    # Transform data formats
  "data_validation"         # Validate data quality
]

# Content & Communication
capabilities = [
  "content_generation",     # Generate text content
  "language_translation",   # Translate between languages
  "summarization"           # Summarize long content
]

# Integration & Automation
capabilities = [
  "api_integration",        # Connect to external APIs
  "workflow_orchestration", # Coordinate multi-step processes
  "notification_delivery"   # Send notifications
]

# Specialized
capabilities = [
  "image_processing",       # Process and analyze images
  "code_generation",        # Generate code
  "monitoring"              # Monitor systems and metrics
]
```

**tools** (required):

- Array of specific tools the agent can use
- Must be pre-registered in the system
- Each tool has specific permissions and capabilities

Available tools:

```toml
# File Processing
tools = [
  "csv_reader",            # Read CSV files
  "excel_reader",          # Read Excel files
  "json_processor",        # Process JSON data
  "xml_parser"             # Parse XML documents
]

# Data Analysis
tools = [
  "statistical_analyzer",  # Calculate statistics
  "chart_generator",       # Create charts/graphs
  "data_validator"         # Validate data quality
]

# External Integration
tools = [
  "http_client",           # Make HTTP requests
  "database_connector",    # Connect to databases
  "email_sender",          # Send emails
  "slack_notifier"         # Send Slack messages
]

# Content Processing
tools = [
  "pdf_generator",         # Generate PDF documents
  "image_processor",       # Process images
  "text_extractor"         # Extract text from documents
]
```

### LLM Configuration - **Intermediate**

**llm.provider** (required):

- Specifies which LLM provider to use
- Options: `openai`, `anthropic`, `local`

**llm.model** (required):

- Specific model to use from the provider
- OpenAI: `gpt-4`, `gpt-4-turbo`, `gpt-3.5-turbo`
- Anthropic: `claude-3-opus`, `claude-3-sonnet`, `claude-3-haiku`
- Local: depends on local setup

**llm.temperature** (optional, default: 0.7):

- Controls response randomness
- Range: 0.0 (deterministic) to 2.0 (very random)
- Recommended: 0.1 for analysis, 0.7 for creative tasks

**llm.max_tokens** (optional):

- Maximum length of response
- Helps control costs and response size
- Consider context window limits

### System and User Prompts - **Intermediate**

**system_prompt** (required):

- Defines the agent's behavior and capabilities
- Uses TOML triple-quoted strings for multiline content
- Should include clear instructions and context

**user_prompt_template** (required):

- Template for formatting user requests
- Supports variable substitution with `{{variable}}` syntax
- Common variables: `{{request}}`, `{{context}}`, `{{memory_context}}`

**documentation** (optional):

- Embedded markdown documentation
- Describes agent usage and examples
- Displayed in agent listings and help

## Example Configurations - **Beginner**

### Simple Data Analyzer

```toml
name = "simple-data-analyzer"
version = "1.0.0"
description = "Analyzes CSV data and provides basic insights"

capabilities = ["data_analysis"]
tools = ["csv_reader", "statistical_analyzer"]

[llm]
provider = "openai"
model = "gpt-4"
temperature = 0.1

system_prompt = '''
You are a data analyst who helps users understand their CSV data.
You can load CSV files and perform statistical analysis to provide insights.

When you receive requests:
1. Load CSV data using csv_reader
2. Analyze using statistical_analyzer
3. Provide clear summary of findings
4. Highlight key trends and anomalies
'''

user_prompt_template = '''
Analyze the following CSV data request: {{request}}

Data source: {{data_source}}
Requirements: {{requirements}}
'''

documentation = '''
# Simple Data Analyzer

This agent analyzes CSV data and provides basic statistical insights.

## Usage Examples

- "Analyze sales data in quarterly-report.csv"
- "What are the key trends in this dataset?"
- "Provide summary statistics for all numeric columns"
'''
```

### Advanced API Integrator - **Intermediate**

```toml
name = "advanced-api-integrator"
version = "2.1.0"
description = "Integrates with external APIs and processes responses"

capabilities = [
  "api_integration",
  "data_transformation",
  "notification_delivery"
]

tools = ["http_client", "json_processor", "email_sender"]

[llm]
provider = "openai"
model = "gpt-4-turbo"
temperature = 0.2
max_tokens = 2000

[permissions]
network_access = "restricted"
file_access = "readonly"
memory_limit = "50MB"

[schedule]
cron = "0 */6 * * *"  # Every 6 hours
timezone = "UTC"
enabled = true

[memory]
enabled = true
scope = "agent"
retention_period = "7d"
max_entries = 1000

[audit]
log_operations = true
log_sensitive_data = false

system_prompt = '''
You integrate with external APIs and process responses intelligently.

Core responsibilities:
1. Make HTTP requests to specified APIs using http_client
2. Process JSON responses with json_processor
3. Transform data as needed for downstream consumers
4. Send notifications on important events via email_sender
5. Cache results efficiently using memory system

Security requirements:
- Only access pre-approved APIs from the allowlist
- Never log sensitive authentication data
- Validate all responses before processing
- Report suspicious activity immediately
'''

user_prompt_template = '''
API Integration Request: {{request}}

Target API: {{api_endpoint}}
Expected Response Format: {{response_format}}
Processing Requirements: {{requirements}}
Notification Settings: {{notifications}}

Previous Integration Results: {{memory_context}}
'''

documentation = '''
# Advanced API Integrator

This agent integrates with external APIs and intelligently processes responses.

## Capabilities

- HTTP API integration with rate limiting
- JSON response processing and transformation
- Intelligent caching with memory system
- Email notifications for important events
- Scheduled execution every 6 hours

## Security Features

- Restricted network access to approved APIs only
- No sensitive data logging
- Comprehensive operation audit trail
- Automatic validation of all API responses

## Usage Examples

- "Fetch latest data from the sales API and send summary email"
- "Monitor the status endpoint and notify if any services are down"
- "Transform the customer data format and cache results"
'''
```

### Scheduled Report Generator - **Advanced**

```toml
name = "scheduled-report-generator"
version = "1.5.0"
description = "Generates automated reports on schedule"

capabilities = [
  "data_analysis",
  "report_generation",
  "content_generation"
]

tools = [
  "csv_reader",
  "excel_reader",
  "chart_generator",
  "pdf_generator",
  "email_sender"
]

[llm]
provider = "anthropic"
model = "claude-3-sonnet"
temperature = 0.3
max_tokens = 4000

[permissions]
file_access = "readwrite"
network_access = "restricted"
memory_limit = "200MB"
cpu_limit = "5m"

[schedule]
cron = "0 8 * * MON"  # Every Monday at 8 AM
timezone = "America/New_York"
enabled = true

[memory]
enabled = true
scope = "agent"
retention_period = "90d"
max_entries = 5000

[memory.context_preparation]
enabled = true
max_context_length = 8000
relevance_threshold = 0.8

[parameters]
max_file_size = "50MB"
supported_formats = ["csv", "xlsx", "json"]
report_template = "executive-summary"

[monitoring]
health_check_interval = "15m"
performance_tracking = true
error_reporting = true

[audit]
log_operations = true
log_sensitive_data = false
retention_period = "1y"

system_prompt = '''
You are an automated report generator that creates comprehensive reports on
schedule.

Core workflow:
1. **Data Collection**: Gather data from configured sources using csv_reader
   and excel_reader
2. **Analysis**: Perform statistical analysis and identify trends
3. **Visualization**: Create charts and graphs with chart_generator
4. **Report Generation**: Generate formatted PDF reports with pdf_generator
5. **Distribution**: Email reports to stakeholders using email_sender

Report quality standards:
- All data must be validated before analysis
- Charts must be clearly labeled and accessible
- Reports must follow consistent formatting
- Distribution list must be verified before sending
- Include executive summary, detailed analysis, and appendix sections
'''

user_prompt_template = '''
Generate scheduled report for: {{report_period}}

Data Sources: {{data_sources}}
Recipients: {{email_recipients}}
Report Type: {{report_type}}
Special Requirements: {{requirements}}

Previous Report Context: {{memory_context}}
Historical Trends: {{trend_data}}
'''

documentation = '''
# Scheduled Report Generator

This agent generates comprehensive automated reports on a weekly schedule.

## Features

- **Automated Data Collection**: Reads from CSV and Excel sources
- **Advanced Analytics**: Statistical analysis with trend identification
- **Rich Visualizations**: Professional charts and graphs
- **PDF Report Generation**: Formatted reports with consistent branding
- **Automated Distribution**: Email delivery to stakeholder lists
- **Memory Integration**: Learns from previous reports for better insights

## Report Structure

### Executive Summary
- Key metrics and trends
- Notable changes from previous period
- Actionable recommendations

### Detailed Analysis
- Comprehensive data breakdown
- Trend analysis with visualizations
- Comparative analysis with historical data

### Appendix
- Raw data summaries
- Methodology notes
- Data quality assessments

## Schedule

- **Frequency**: Every Monday at 8:00 AM EST
- **Data Collection**: Automatic from configured sources
- **Delivery**: Email to predefined stakeholder list
- **Retention**: Reports stored for 90 days with full audit trail

## Usage Examples

- Weekly sales performance reports
- Monthly operational metrics summaries
- Quarterly trend analysis reports
- Custom ad-hoc report generation
'''
```

## Validation Rules - **Intermediate**

### Required Field Validation

1. **name**: Must be present, non-empty, valid format
2. **version**: Must follow semantic versioning
3. **description**: Must be present and meaningful
4. **capabilities**: Must be non-empty array
5. **tools**: Must be non-empty array
6. **llm**: Must have provider and model

### Format Validation

```bash
# Validate agent configuration
curl -X POST http://localhost:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{"definition": "'$(cat agent.toml)'"}'
```

Example validation response:

```json
{
  "valid": false,
  "errors": [
    {
      "field": "name",
      "message": "Agent name must be kebab-case",
      "value": "MyAgent"
    }
  ],
  "warnings": [
    {
      "field": "description",
      "message": "Description should be more detailed",
      "suggestion": "Include specific use cases"
    }
  ]
}
```

## Deployment Examples - **Intermediate**

### Local Development

```bash
# Create agent file
cat > my-agent.toml << 'EOF'
name = "test-agent"
version = "1.0.0"
description = "Test agent for development"
capabilities = ["data_analysis"]
tools = ["csv_reader"]

[llm]
provider = "openai"
model = "gpt-4"

system_prompt = '''
You are a test agent for development purposes.
Provide helpful responses for testing scenarios.
'''

user_prompt_template = '''
Test request: {{request}}
Context: {{context}}
'''

documentation = '''
# Test Agent
Simple test agent for development and testing purposes.
'''
EOF

# Validate
curl -X POST http://localhost:3000/api/validate \
  -d '{"definition": "'$(cat my-agent.toml)'"}''

# Deploy
curl -X POST http://localhost:3000/api/agents \
  -d '{"type": "configuration", "definition": "'$(cat my-agent.toml)'"}''
```

### Production Deployment

```bash
# Use environment-specific configuration
envsubst < agent-template.toml > agent-prod.toml

# Validate with strict checks
curl -X POST http://localhost:3000/api/validate \
  -d '{"definition": "'$(cat agent-prod.toml)'", "strict": true}'

# Deploy with monitoring
curl -X POST http://localhost:3000/api/agents \
  -d '{
    "type": "configuration",
    "definition": "'$(cat agent-prod.toml)'",
    "monitoring": {
      "enabled": true,
      "health_checks": true,
      "performance_tracking": true
    }
  }''
```

## Related Documentation

- [Overview](overview.md) - **Beginner**
- [Best Practices](best-practices.md) - **Intermediate**
- [Configuration Examples](examples.md) - **Beginner**
- [LLM Provider Configuration](llm-providers.md) - **Beginner**
- [Building Agents Guide](../building-agents.md) - **Beginner**
- [Security Guide](../security.md) - **Intermediate**

## Schema Validation

The complete JSON schema for validation is available at:

- Development: `http://localhost:3000/api/schema/agent-config`
- Documentation: See [API Reference](../api-reference.md) for details
