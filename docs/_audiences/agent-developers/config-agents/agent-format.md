---
title: "Configuration Agent Format Specification"
date: 2025-09-10
layout: guide
categories: [Agent Developers, Configuration, Agents, Reference]
difficulty: beginner
audience: agent-developers
---

## File Structure - **Beginner**

Configuration agents are markdown files (`.md`) with YAML frontmatter that
define agent behavior, capabilities, and documentation in a single file:

```text
agent-name.md
├── YAML frontmatter (--- delimited)
│   ├── Agent metadata
│   ├── Capability declarations
│   ├── Tool permissions
│   ├── LLM configuration
│   └── Optional parameters
└── Markdown documentation
    ├── Agent description
    ├── Usage instructions
    └── Example interactions
```

## YAML Schema Reference - **Beginner**

> **Note**: The following shows schema documentation mixing type annotations
> (e.g., `string`, `boolean`) with example values (e.g., `capability1`,
> `data_analysis`). See the examples section below for complete working YAML.

### Required Fields

```yaml
---
name: string                    # Agent identifier (required)
version: string                 # Semantic version (required)
description: string             # Brief agent purpose (required)
capabilities:                   # Capability declarations (required)
  - capability1                 # List of capability names
  - capability2
tools:                         # Tool permission list (required)
  - tool1                       # List of tool names
  - tool2
llm:                           # LLM configuration (required)
  provider: string             # LLM provider
  model: string               # Model name
---
```

### Complete Schema - **Intermediate**

```yaml
---
# Core Identity (Required)
name: string                    # Agent name (kebab-case recommended)
version: string                # Semantic versioning (e.g., "1.0.0")
description: string            # Brief agent purpose (1-2 sentences)

# Capability System (Required)
capabilities:                   # High-level capabilities
  - capability_name             # List of capability names
  - data_analysis               # e.g., "data_analysis"

tools:                         # Specific tools to access
  - tool_name                   # List of tool names
  - csv_reader                  # e.g., "csv_reader"

# LLM Configuration (Required)
llm:
  provider: string             # openai|anthropic|local
  model: string               # Model identifier
  temperature: float          # 0.0-2.0 (optional, default: 0.7)
  max_tokens: integer         # Max response length (optional)
  top_p: float               # Nucleus sampling (optional)
  frequency_penalty: float   # Repetition penalty (optional)
  presence_penalty: float    # New topic encouragement (optional)

# Security & Permissions (Optional)
permissions:
  file_access: string         # readonly|readwrite|none
  network_access: string     # restricted|full|none
  memory_limit: string       # e.g., "100MB"
  cpu_limit: string          # e.g., "100ms"

# Automation (Optional)
schedule:
  cron: string               # Cron expression
  timezone: string           # Timezone identifier
  enabled: boolean           # Schedule activation

# Memory Configuration (Optional)
memory:
  enabled: boolean           # Enable persistent memory
  retention_period: string   # e.g., "30d"
  max_entries: integer       # Maximum stored entries
  context_preparation:
    enabled: boolean         # Auto context preparation
    max_context_length: integer
    relevance_threshold: float

# Monitoring & Audit (Optional)
audit:
  log_operations: boolean    # Log all operations
  log_sensitive_data: boolean # Log sensitive data
  retention_period: string   # Log retention

monitoring:
  health_check_interval: string # e.g., "5m"
  performance_tracking: boolean
  error_reporting: boolean

# Development Settings (Optional)
development:
  hot_reload: boolean        # Enable hot reload
  debug_mode: boolean        # Enable debug logging
  test_mode: boolean         # Test mode activation
---
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

```yaml
# Data Processing
- data_analysis           # Analyze structured data
- data_transformation    # Transform data formats
- data_validation        # Validate data quality

# Content & Communication
- content_generation     # Generate text content
- language_translation   # Translate between languages
- summarization         # Summarize long content

# Integration & Automation
- api_integration       # Connect to external APIs
- workflow_orchestration # Coordinate multi-step processes
- notification_delivery  # Send notifications

# Specialized
- image_processing      # Process and analyze images
- code_generation       # Generate code
- monitoring           # Monitor systems and metrics
```

**tools** (required):

- Array of specific tools the agent can use
- Must be pre-registered in the system
- Each tool has specific permissions and capabilities

Available tools:

```yaml
# File Processing
- csv_reader            # Read CSV files
- excel_reader          # Read Excel files
- json_processor        # Process JSON data
- xml_parser           # Parse XML documents

# Data Analysis
- statistical_analyzer  # Calculate statistics
- chart_generator      # Create charts/graphs
- data_validator       # Validate data quality

# External Integration
- http_client          # Make HTTP requests
- database_connector   # Connect to databases
- email_sender         # Send emails
- slack_notifier       # Send Slack messages

# Content Processing
- pdf_generator        # Generate PDF documents
- image_processor      # Process images
- text_extractor       # Extract text from documents
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

## Example Configurations - **Beginner**

### Simple Data Analyzer

```yaml
---
name: simple-data-analyzer
version: "1.0.0"
description: "Analyzes CSV data and provides basic insights"

capabilities:
  - data_analysis

tools:
  - csv_reader
  - statistical_analyzer

llm:
  provider: openai
  model: gpt-4
  temperature: 0.1
---

# Simple Data Analyzer

You are a data analyst. Analyze CSV files and provide insights.

## Instructions

1. Load CSV data using csv_reader
2. Analyze using statistical_analyzer
3. Provide clear summary of findings
```

### Advanced API Integrator - **Intermediate**

```yaml
---
name: advanced-api-integrator
version: "2.1.0"
description: "Integrates with external APIs and processes responses"

capabilities:
  - api_integration
  - data_transformation
  - notification_delivery

tools:
  - http_client
  - json_processor
  - email_sender

llm:
  provider: openai
  model: gpt-4-turbo
  temperature: 0.2
  max_tokens: 2000

permissions:
  network_access: restricted
  file_access: readonly
  memory_limit: 50MB

schedule:
  cron: "0 */6 * * *"  # Every 6 hours
  timezone: "UTC"
  enabled: true

memory:
  enabled: true
  retention_period: "7d"
  max_entries: 1000

audit:
  log_operations: true
  log_sensitive_data: false
---

# Advanced API Integrator

You integrate with external APIs and process responses intelligently.

## Instructions

1. Make HTTP requests to specified APIs
2. Process JSON responses
3. Transform data as needed
4. Send notifications on important events
5. Cache results for efficiency

## Security Notes

- Only access pre-approved APIs
- Never log sensitive authentication data
- Validate all responses before processing
```

### Scheduled Report Generator - **Advanced**

```yaml
---
name: scheduled-report-generator
version: "1.5.0"
description: "Generates automated reports on schedule"

capabilities:
  - data_analysis
  - report_generation
  - content_generation

tools:
  - csv_reader
  - excel_reader
  - chart_generator
  - pdf_generator
  - email_sender

llm:
  provider: anthropic
  model: claude-3-sonnet
  temperature: 0.3
  max_tokens: 4000

permissions:
  file_access: readwrite
  network_access: restricted
  memory_limit: 200MB
  cpu_limit: 5m

schedule:
  cron: "0 8 * * MON"  # Every Monday at 8 AM
  timezone: "America/New_York"
  enabled: true

memory:
  enabled: true
  retention_period: "90d"
  max_entries: 5000
  context_preparation:
    enabled: true
    max_context_length: 8000
    relevance_threshold: 0.8

monitoring:
  health_check_interval: "15m"
  performance_tracking: true
  error_reporting: true

audit:
  log_operations: true
  log_sensitive_data: false
  retention_period: "1y"
---

# Scheduled Report Generator

You generate comprehensive reports automatically on schedule.

## Instructions

1. **Data Collection**: Gather data from configured sources
2. **Analysis**: Perform statistical analysis and trend identification
3. **Visualization**: Create charts and graphs
4. **Report Generation**: Generate formatted PDF reports
5. **Distribution**: Email reports to stakeholders

## Report Structure

### Executive Summary
- Key metrics and trends
- Notable changes from previous period
- Actionable recommendations

### Detailed Analysis
- Comprehensive data breakdown
- Trend analysis with visualizations
- Comparative analysis

### Appendix
- Raw data summaries
- Methodology notes
- Data quality assessments

## Quality Standards

- All data must be validated before analysis
- Charts must be clearly labeled and accessible
- Reports must be generated in consistent format
- Distribution list must be verified before sending
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
  -d '{"definition": "'$(cat agent.md)'"}'
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
cat > my-agent.md << 'EOF'
---
name: test-agent
version: "1.0.0"
description: "Test agent for development"
capabilities: ["data_analysis"]
tools: ["csv_reader"]
llm:
  provider: openai
  model: gpt-4
---
# Test Agent
Simple test agent for development.
EOF

# Validate
curl -X POST http://localhost:3000/api/validate \
  -d '{"definition": "'$(cat my-agent.md)'"}'

# Deploy
curl -X POST http://localhost:3000/api/agents \
  -d '{"type": "configuration", "definition": "'$(cat my-agent.md)'"}'
```

### Production Deployment

```bash
# Use environment-specific configuration
envsubst < agent-template.md > agent-prod.md

# Validate with strict checks
curl -X POST http://localhost:3000/api/validate \
  -d '{"definition": "'$(cat agent-prod.md)'", "strict": true}'

# Deploy with monitoring
curl -X POST http://localhost:3000/api/agents \
  -d '{
    "type": "configuration",
    "definition": "'$(cat agent-prod.md)'",
    "monitoring": {
      "enabled": true,
      "health_checks": true,
      "performance_tracking": true
    }
  }'
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
