---
title: "Configuration Agent Best Practices"
date: 2025-09-10
layout: guide
categories: [Agent Developers, Configuration, Agents, Best Practices]
difficulty: intermediate
audience: agent-developers
---

## Context Management Excellence - **Advanced**

### Leverage Automatic Context Intelligence

Configuration agents benefit from intelligent context management (ADR-0031)
that automatically gathers relevant information without manual prompt
engineering. Design your agents to work with this system effectively.

**Trust the Context System**: The runtime automatically provides relevant
conversation history, memory insights, and tool-specific context.

```yaml
# ❌ Don't manually manage context in prompts
---
name: bad-example
llm:
  provider: openai
  model: gpt-4
---

# Data Analyst

You are a data analyst. When the user mentions previous analyses,
refer back to the conversation history. Remember their preferences
from past interactions and use relevant tools.

# ✅ Focus on expertise and behavior, let context system handle the rest
---
name: good-example
llm:
  provider: openai
  model: gpt-4
memory:
  enabled: true
  context_preparation:
    enabled: true
    relevance_threshold: 0.8
---

# Senior Data Analyst

You are a senior data analyst specializing in business intelligence and
predictive modeling.

Your approach:
1. Validate data quality before analysis
2. Apply appropriate statistical methods
3. Present insights with confidence intervals
4. Recommend actionable next steps
```

### Design for Context Efficiency - **Advanced**

**Specify Context Preferences**: Use agent configuration to hint at context
preferences without micromanaging the system.

```yaml
# Indicate context preferences to optimize performance
memory:
  context_preparation:
    enabled: true
    max_context_length: 6000      # Reasonable context window
    relevance_threshold: 0.7      # Good relevance balance
    conversation_focus: "recent"   # Prioritize recent exchanges
    memory_relevance: "high"      # Only highly relevant memories
```

## Configuration Design Patterns - **Intermediate**

### 1. Single Responsibility Principle

Each agent should have one clear, focused purpose:

```yaml
# ✅ Good: Focused responsibility
---
name: sales-report-analyzer
description: "Analyzes monthly sales reports and identifies trends"
capabilities:
  - data_analysis
tools:
  - csv_reader
  - statistical_analyzer
  - chart_generator
---

# ❌ Bad: Multiple responsibilities
---
name: everything-agent
description: "Handles sales, marketing, and HR tasks"
capabilities:
  - data_analysis
  - content_generation
  - email_management
  - user_support
---
```

### 2. Progressive Enhancement

Start simple, add complexity gradually:

```yaml
# Version 1.0: Basic functionality
---
name: data-analyzer
version: "1.0.0"
capabilities: ["data_analysis"]
tools: ["csv_reader"]
---

# Version 1.1: Add visualization
---
name: data-analyzer
version: "1.1.0"
capabilities: ["data_analysis", "visualization"]
tools: ["csv_reader", "chart_generator"]
---

# Version 2.0: Add automation
---
name: data-analyzer
version: "2.0.0"
capabilities: ["data_analysis", "visualization", "report_generation"]
tools: ["csv_reader", "chart_generator", "pdf_generator"]
schedule:
  cron: "0 9 * * MON"
---
```

### 3. Environment-Aware Configuration

Use different configurations for different environments:

```yaml
# development.yaml
---
name: data-analyzer-dev
llm:
  provider: openai
  model: gpt-3.5-turbo  # Cheaper for development
  temperature: 0.3
permissions:
  file_access: readwrite  # More permissive for testing
development:
  hot_reload: true
  debug_mode: true
---

# production.yaml
---
name: data-analyzer
llm:
  provider: openai
  model: gpt-4  # More capable for production
  temperature: 0.1
permissions:
  file_access: readonly  # More secure
monitoring:
  health_check_interval: "5m"
  performance_tracking: true
audit:
  log_operations: true
---
```

## Prompt Engineering Best Practices - **Intermediate**

### 1. Clear Role Definition

Define the agent's role and expertise clearly:

```markdown
# ✅ Good: Specific expertise
# Senior Financial Analyst

You are a senior financial analyst with 10+ years of experience in
corporate finance and data analysis. You specialize in:

- Financial statement analysis
- Cash flow forecasting
- Risk assessment
- Performance metrics calculation

# ❌ Bad: Vague role
# Helpful Assistant

You are a helpful assistant who can analyze data and answer questions.
```

### 2. Structured Instructions

Use clear, numbered instructions:

```markdown
# ✅ Good: Clear structure
## Analysis Process

1. **Data Validation**
   - Check for missing values
   - Validate data types
   - Identify outliers

2. **Statistical Analysis**
   - Calculate descriptive statistics
   - Perform trend analysis
   - Identify correlations

3. **Insight Generation**
   - Summarize key findings
   - Highlight significant trends
   - Provide actionable recommendations

# ❌ Bad: Unclear instructions
Analyze the data and provide insights. Look for trends and patterns.
Tell me what's important.
```

### 3. Input/Output Examples

Provide clear examples of expected interactions:

```markdown
## Example Interaction

**Input**: CSV file with columns: date, revenue, customers, region
**Process**:
1. Load and validate CSV structure
2. Calculate monthly growth rates
3. Analyze regional performance
4. Generate summary report

**Output**:
```

📊 Sales Analysis Report

Key Metrics:

- Total Revenue: $2.4M (+15% MoM)
- Customer Growth: +8% MoM
- Top Region: West Coast (35% of revenue)

Recommendations:

1. Investigate East Coast performance decline
2. Replicate West Coast success factors
3. Focus customer acquisition efforts

```text
```

## Tool Usage Best Practices - **Intermediate**

### 1. Minimal Tool Set

Only include tools you actually need:

```yaml
# ✅ Good: Minimal, focused tools
tools:
  - csv_reader      # Core functionality
  - chart_generator # Essential for output

# ❌ Bad: Kitchen sink approach
tools:
  - csv_reader
  - excel_reader
  - json_processor
  - xml_parser
  - pdf_generator
  - email_sender
  - slack_notifier
  - database_connector
```

### 2. Tool Configuration

Configure tools appropriately for your use case:

```yaml
tools:
  - name: csv_reader
    config:
      max_file_size: 10MB     # Appropriate limit
      encoding: utf-8         # Standard encoding
      delimiter: ","          # Standard delimiter

  - name: chart_generator
    config:
      default_format: png     # Good for web
      max_width: 800         # Reasonable size
      max_height: 600
      dpi: 150               # Good quality/size balance
```

### 3. Error Handling

Design for graceful tool failures:

```markdown
## Error Handling

If tools fail or data is unavailable:

1. **CSV Reader Fails**: Request user to check file format and size
2. **Chart Generation Fails**: Provide textual description of trends
3. **External API Down**: Use cached data if available, otherwise notify user

Always provide alternative analysis methods when primary tools are unavailable.
```

## Security Best Practices - **Intermediate**

### 1. Principle of Least Privilege

Grant minimal necessary permissions:

```yaml
# ✅ Good: Minimal permissions
permissions:
  file_access: readonly      # Only read data files
  network_access: none       # No external access needed
  memory_limit: 50MB        # Reasonable for task

# ❌ Bad: Excessive permissions
permissions:
  file_access: readwrite    # More than needed
  network_access: full      # Unnecessary risk
  memory_limit: 1GB        # Wasteful
```

### 2. Input Validation

Validate all inputs in your instructions:

```markdown
## Data Validation Requirements

Before processing any data:

1. **File Validation**
   - Must be valid CSV format
   - Maximum size: 10MB
   - Required columns: date, amount

2. **Content Validation**
   - Dates must be in YYYY-MM-DD format
   - Amounts must be numeric
   - No malicious code in text fields

3. **Business Rules**
   - Date range within last 5 years
   - Amounts within reasonable bounds
   - No duplicate transactions
```

### 3. Sensitive Data Handling

Be explicit about sensitive data:

```yaml
audit:
  log_operations: true
  log_sensitive_data: false    # Don't log PII

# In instructions:
```

```markdown
## Data Privacy

When handling sensitive data:

1. **Never log**: Customer names, emails, phone numbers
2. **Anonymize**: Use customer IDs instead of names in reports
3. **Aggregate**: Present only summary statistics, not individual records
4. **Secure**: All data processing happens in secure sandbox
```

## Performance Optimization - **Advanced**

### 1. LLM Configuration

Optimize LLM settings for your use case:

```yaml
# For deterministic analysis tasks
llm:
  provider: openai
  model: gpt-4
  temperature: 0.1        # Low for consistency
  max_tokens: 1500       # Sufficient for most analyses
  top_p: 0.9            # Good balance

# For creative/exploratory tasks
llm:
  provider: openai
  model: gpt-4
  temperature: 0.7       # Higher for creativity
  max_tokens: 2500      # More room for exploration
```

### 2. Memory Management

Configure memory for optimal performance:

```yaml
memory:
  enabled: true
  retention_period: "30d"     # Keep relevant context
  max_entries: 1000          # Reasonable limit

  context_preparation:
    enabled: true
    max_context_length: 6000  # Efficient context window
    relevance_threshold: 0.8  # High relevance only
```

### 3. Caching Strategies

Use caching for expensive operations:

```markdown
## Caching Strategy

For computationally expensive analyses:

1. **Cache Results**: Store analysis results for identical datasets
2. **Cache Intermediate**: Store preprocessed data for reuse
3. **Cache Models**: Reuse statistical models when appropriate
4. **Cache Visualizations**: Store generated charts for identical data

Always check cache validity before reusing results.
```

## Testing and Validation - **Intermediate**

### 1. Configuration Testing

Test your configuration before deployment:

```bash
# Validate configuration syntax
curl -X POST http://localhost:3000/api/validate \
  -d '{"definition": "'$(cat agent.md)'"}'

# Test with sample data
curl -X POST http://localhost:3000/api/agents/test-agent/test \
  -d '{"input": "sample data", "expected_output": "expected result"}'
```

### 2. Behavioral Testing

Test agent behavior systematically:

```yaml
# test-cases.yaml
test_cases:
  - name: "Basic CSV Analysis"
    input: "sample-sales-data.csv"
    expected_capabilities: ["data_analysis"]
    expected_tools: ["csv_reader", "statistical_analyzer"]
    success_criteria:
      - "Report includes total revenue"
      - "Identifies growth trends"
      - "Provides recommendations"

  - name: "Error Handling"
    input: "malformed-data.csv"
    expected_behavior: "graceful_error_handling"
    success_criteria:
      - "Identifies data format issues"
      - "Provides helpful error message"
      - "Suggests corrective actions"
```

### 3. Performance Testing

Monitor agent performance:

```bash
# Test response time
time curl -X POST http://localhost:3000/api/agents/my-agent/messages \
  -d '{"content": "test message"}'

# Monitor resource usage
curl http://localhost:3000/api/agents/my-agent/metrics | jq .memory_usage
```

## Deployment Strategies - **Advanced**

### 1. Blue-Green Deployment

Deploy new versions without downtime:

```bash
# Deploy new version
curl -X POST http://localhost:3000/api/agents/my-agent/deploy \
  -d '{
    "strategy": "blue-green",
    "definition": "'$(cat agent-v2.md)'",
    "validation_period": "5m"
  }'

# Monitor new version
curl http://localhost:3000/api/agents/my-agent-blue/health

# Promote if successful
curl -X POST http://localhost:3000/api/agents/my-agent/promote \
  -d '{"version": "blue"}'
```

### 2. Canary Deployment

Gradually shift traffic to new version:

```bash
# Start with 10% traffic
curl -X POST http://localhost:3000/api/agents/my-agent/deploy \
  -d '{
    "strategy": "canary",
    "traffic_percentage": 10,
    "definition": "'$(cat agent-v2.md)'"
  }'

# Increase traffic if metrics look good
curl -X PUT http://localhost:3000/api/agents/my-agent/canary \
  -d '{"traffic_percentage": 50}'
```

### 3. Rollback Strategies

Always have a rollback plan:

```bash
# Quick rollback to previous version
curl -X POST http://localhost:3000/api/agents/my-agent/rollback \
  -d '{"version": "previous"}'

# Rollback to specific version
curl -X POST http://localhost:3000/api/agents/my-agent/rollback \
  -d '{"version": "1.2.3"}'
```

## Monitoring and Maintenance - **Intermediate**

### 1. Health Monitoring

Set up comprehensive monitoring:

```yaml
monitoring:
  health_check_interval: "5m"
  performance_tracking: true
  error_reporting: true

  alerts:
    response_time_threshold: "2s"
    error_rate_threshold: "5%"
    memory_usage_threshold: "80%"
```

### 2. Log Analysis

Monitor agent logs for issues:

```bash
# Check for errors
curl http://localhost:3000/api/agents/my-agent/logs | grep ERROR

# Monitor performance
curl http://localhost:3000/api/agents/my-agent/metrics | jq .response_times

# Analyze usage patterns
curl http://localhost:3000/api/agents/my-agent/analytics
```

### 3. Continuous Improvement

Iterate based on usage data:

```markdown
## Improvement Cycle

1. **Monitor**: Track performance and usage metrics
2. **Analyze**: Identify bottlenecks and improvement opportunities
3. **Test**: Validate changes in development environment
4. **Deploy**: Use safe deployment strategies
5. **Validate**: Confirm improvements in production
6. **Document**: Update best practices based on learnings
```

## Common Antipatterns - **Intermediate**

### ❌ Avoid These Mistakes

### 1. Overly Complex Prompts

```markdown
# Don't micromanage every detail
You are a data analyst. First, you need to check if the file exists,
then validate the format, then read line by line, then check each
column, then calculate statistics, then generate a chart, then...
```

### 2. Tool Overload

```yaml
# Don't include tools you don't need
tools: [csv_reader, excel_reader, json_processor, xml_parser,
        pdf_generator, email_sender, slack_notifier, twitter_api,
        database_connector, file_uploader, image_processor, ...]
```

### 3. Hardcoded Values

```yaml
# Don't hardcode environment-specific values
tools:
  - name: database_connector
    config:
      host: "prod-db-01.company.com"  # Should be environment variable
      username: "admin"               # Should be secure
```

### 4. Poor Error Handling

```markdown
# Don't ignore error scenarios
Analyze the CSV file and generate a report.
# What if file is corrupted? Network is down? Wrong format?
```

## Related Documentation

- [Agent Format](agent-format.md) - **Beginner**
- [Configuration Examples](examples.md) - **Beginner**
- [LLM Provider Configuration](llm-providers.md) - **Beginner**
- [Overview](overview.md) - **Beginner**
- [Building Agents Guide](../building-agents.md) - **Beginner**
- [Security Guide](../security.md) - **Intermediate**
