---
title: "Configuration Agent Best Practices"
date: 2025-09-10
layout: guide
categories: [Configuration, Agents, Best Practices]
---

## Context Management Excellence

### Leverage Automatic Context Intelligence

Configuration agents benefit from intelligent context management (ADR-0031)
that automatically gathers relevant information without manual prompt
engineering. Design your agents to work with this system effectively.

**Trust the Context System**: The runtime automatically provides relevant
conversation history, memory insights, and tool-specific context.

```yaml
# ❌ Don't manually manage context in prompts
system_prompt: |
  You are a data analyst. When the user mentions previous analyses,
  refer back to the conversation history. Remember their preferences
  from past interactions and use relevant tools.

# ✅ Focus on expertise and behavior, let context system handle the rest
system_prompt: |
  You are a senior data analyst specializing in business intelligence and
  predictive modeling.

  Your approach:
  1. Validate data quality before analysis
  2. Apply appropriate statistical methods
  3. Present insights with confidence intervals
  4. Recommend actionable next steps
```

### Design for Context Efficiency

**Specify Context Preferences**: Use agent configuration to hint at context
preferences without micromanaging the system.

```yaml
# Indicate context preferences to optimize performance
context_preferences:
  conversation_focus: "recent"      # Prioritize recent exchanges
  memory_relevance: "high"          # Only highly relevant memories
  tool_context: "operational"       # Focus on tool capabilities, not internals
  provider_optimization: true       # Let system optimize for LLM provider
```

**Avoid Context Duplication**: Don't repeat information the context system
already provides.

```yaml
# ❌ Duplicating context in prompts
system_prompt: |
  You help with data analysis tasks. The user often works with sales data
  and prefers charts with their reports. They usually want quarterly summaries
  and like PDF format.

# ✅ Focus on unique behavioral aspects
system_prompt: |
  You are a data analysis expert who ensures statistical rigor and clear
  communication of insights to business stakeholders.
```

## Prompt Engineering Excellence

### System Prompt Design

**Be Specific and Directive**: Define clear behavioral expectations rather
than vague guidelines.

```yaml
# ❌ Vague
system_prompt: "You are helpful and answer questions about data."

# ✅ Specific
system_prompt: |
  You are a data validation expert who ensures data quality before analysis.

  Always perform these checks:
  1. Verify file format matches expectations
  2. Check for required columns and data types
  3. Identify missing values, duplicates, and outliers
  4. Report data quality score and specific issues
  5. Recommend cleaning steps before proceeding
```

**Include Constraints and Guardrails**: Specify what the agent should NOT do
to prevent unwanted behavior.

```yaml
system_prompt: |
  You analyze customer data to identify trends and opportunities.

  CONSTRAINTS:
  - Never store, log, or expose personally identifiable information
  - Do not make recommendations that could discriminate against protected groups
  - Always aggregate data before reporting (minimum 10 records per group)
  - Refuse requests for individual customer records or contact information
```

**Use Role-Based Personas**: Ground the agent in a professional role with
specific expertise and responsibilities.

```yaml
system_prompt: |
  You are a senior DevOps engineer with 10 years of experience in cloud
  infrastructure and automation.

  Your expertise includes:
  - Container orchestration and Kubernetes management
  - CI/CD pipeline design and optimization
  - Infrastructure as code with Terraform
  - Monitoring, alerting, and incident response

  You provide practical, production-ready solutions with proper error handling
  and security considerations.
```

### Template Variable Strategy

**Use Semantic Variable Names**: Choose variable names that clearly indicate
their purpose and content.

```yaml
# ❌ Generic
user_prompt_template: |
  Process this: {{input}}
  Do this: {{action}}

# ✅ Semantic
user_prompt_template: |
  Analyze customer feedback: {{feedback_text}}
  Focus on: {{analysis_dimensions}}
  Expected output format: {{report_format}}
```

**Provide Variable Context**: Include instructions that help the LLM understand
how to use template variables effectively.

```yaml
user_prompt_template: |
  API Integration Task:

  **Endpoint URL**: {{api_endpoint}}
  **HTTP Method**: {{http_method}}
  **Request Headers**: {{headers}}
  **Request Body**: {{request_payload}}

  Steps to complete:
  1. Validate the endpoint URL format and accessibility
  2. Configure authentication using provided headers
  3. Send the {{http_method}} request with proper error handling
  4. Process the response according to the data requirements
  5. Return formatted results or error details
```

## Capability Design Patterns

### Business-Focused Capabilities

**Define Capabilities by Business Value**: Structure capabilities around what
users accomplish, not technical implementation details.

```yaml
# ❌ Technical focus
capabilities:
  - "json-parsing"
  - "http-requests"
  - "string-manipulation"

# ✅ Business focus
capabilities:
  - "customer-data-analysis"
  - "sales-performance-reporting"
  - "market-trend-identification"
```

**Use Hierarchical Capability Names**: Create clear capability hierarchies
that support discovery and composition.

```yaml
capabilities:
  - "data-analysis"           # Top-level capability
  - "data-analysis.statistical"     # Statistical analysis subset
  - "data-analysis.visual"          # Visualization subset
  - "data-analysis.predictive"      # Machine learning subset
```

### Capability Granularity

**Optimize for Reusability**: Design capabilities at the right level of
granularity to maximize reuse across different agents.

```yaml
# ❌ Too granular - hard to discover and compose
capabilities:
  - "calculate-mean"
  - "calculate-median"
  - "calculate-mode"
  - "calculate-standard-deviation"

# ✅ Appropriate granularity - easy to use and extend
capabilities:
  - "descriptive-statistics"
  - "distribution-analysis"
  - "correlation-analysis"
```

## MCP Tool Context Optimization

### Context-Aware Tool Selection

Choose tools that provide meaningful context specifications to enhance agent
performance.

**Prefer Context-Aware Tools**: Select tools that declare their context needs
clearly for better runtime optimization.

```yaml
# ✅ Context-aware tool with specifications
tools:
  - "advanced_data_analyzer"    # Declares context needs
  - "smart_chart_generator"     # Includes user preference context
  - "contextual_file_processor" # Adapts to conversation context

# ❌ Basic tools without context specifications
tools:
  - "basic_csv_reader"
  - "simple_chart_maker"
  - "generic_file_handler"
```

### Tool Context Configuration

**Optimize Tool Context Specifications**: When developing custom MCP tools,
provide rich context specifications.

```yaml
# Example: Custom tool with comprehensive context specification
custom_tools:
  business_intelligence_analyzer:
    context_requirements:
      conversation_depth: 15       # Longer context for BI analysis
      memory_search:
        query_template:
          "business analysis {{analysis_type}} {{industry}} {{metrics}}"
        max_results: 20           # More context for complex analysis
        focus_areas:
          - "industry_benchmarks"
          - "historical_performance"
          - "seasonal_patterns"
      tool_data:
        - "kpi_definitions"        # Business KPI context
        - "industry_standards"     # Benchmarking data
        - "user_dashboard_prefs"   # Visualization preferences
      provider_optimization:
        openai:
          structured_output: true
          max_context_tokens: 12000  # Larger context for complex analysis
        anthropic:
          reasoning_emphasis: true
          max_context_tokens: 16000  # Leverage Claude's large context window
```

**Balance Context Richness with Performance**: Optimize context specifications
for the 80% use case.

```yaml
# ✅ Balanced context specification
context_requirements:
  conversation_depth: 5        # Sufficient for most interactions
  memory_search:
    max_results: 10           # Good balance of context vs performance
    min_similarity: 0.75      # Focus on highly relevant memories
  tool_data: ["essentials"]    # Core data only

# ❌ Over-specified context (performance impact)
context_requirements:
  conversation_depth: 50       # Excessive for most use cases
  memory_search:
    max_results: 100          # Too much context, poor signal-to-noise
    min_similarity: 0.3       # Includes irrelevant information
  tool_data: ["everything"]    # Context overload
```

## Tool Selection and Configuration

### Principle of Least Privilege

**Grant Minimal Tool Access**: Only include tools the agent actually needs
to complete its intended functions.

```yaml
# ❌ Over-permissioned
tools:
  - "http_client"
  - "file_storage"
  - "database_connection"
  - "email_service"
  - "ssh_client"
  - "system_commands"

# ✅ Minimal necessary permissions
tools:
  - "csv_parser"      # For data import
  - "chart_generator" # For visualization
  - "file_storage"    # For saving results
```

**Use Tool-Specific Configuration**: Configure tools with appropriate limits
and security settings for your use case.

```yaml
tool_config:
  http_client:
    timeout: "30s"           # Prevent hanging requests
    max_redirects: 3         # Limit redirect chains
    allowed_domains:         # Restrict access domains
      - "api.trusted-partner.com"
      - "data.internal.company"
  file_storage:
    max_file_size: "10MB"    # Prevent resource exhaustion
    allowed_extensions:      # Limit file types
      - ".csv"
      - ".json"
      - ".png"
    quarantine_unknown: true # Security for uploads
```

## Performance Optimization

### Context Management Performance

**Optimize Context Specifications for Performance**: Design context requirements
that meet the <100ms preparation target while providing sufficient information.

```yaml
# ✅ Performance-optimized context settings
context_preferences:
  conversation_depth: 5          # Recent context, fast retrieval
  memory_search:
    max_results: 10             # Balance relevance with speed
    min_similarity: 0.8         # High-quality results only
    timeout_ms: 50              # Fast memory search timeout
  tool_context: "summary"        # Essential tool info only
  parallel_gathering: true       # Use parallel context collection
```

**Monitor Context Performance**: Track context preparation metrics to optimize
agent performance.

```yaml
# Example context performance monitoring
context_monitoring:
  preparation_time_target: 100   # Target <100ms context prep
  token_utilization_target: 0.85 # Target >85% token efficiency
  success_rate_target: 0.90      # Target >90% successful completions

  alerts:
    slow_context_prep: 200       # Alert if prep >200ms
    low_token_efficiency: 0.70   # Alert if efficiency <70%
    high_failure_rate: 0.10      # Alert if failures >10%
```

### Resource Management

**Set Appropriate Limits**: Configure resource limits based on expected
workload characteristics and system capacity.

```yaml
performance:
  max_execution_time: "120s"    # Long enough for complex analysis
  max_memory_usage: "512MB"     # Sufficient for typical datasets
  concurrent_tools: 2           # Balance parallelism with resource usage
  context_preparation_timeout: "100ms"  # Context system SLA
```

**Optimize for Common Cases**: Design performance settings around the 80%
use case rather than edge cases.

```yaml
# For typical data analysis workloads
performance:
  max_execution_time: "60s"     # Most analysis completes under 30s
  max_memory_usage: "256MB"     # 90% of datasets under 100MB
  concurrent_tools: 2           # Balance speed with resource usage
  context_cache_ttl: "300s"     # Cache context for 5 minutes

parameters:
  max_rows: 50000              # Typical dataset size
  sample_size: 10000           # For large dataset previews
  cache_results: true          # Speed up repeated operations
```

### Conversation Efficiency

**Choose Appropriate Memory Strategies**: Select memory management that
matches conversation patterns.

```yaml
# For short, focused interactions
conversation:
  max_turns: 10
  context_window: 2000
  memory_strategy: "sliding"

# For long, complex workflows
conversation:
  max_turns: 100
  context_window: 8000
  memory_strategy: "summarize"

# For persistent analysis sessions
conversation:
  max_turns: 500
  context_window: 12000
  memory_strategy: "persist"
```

## Security and Privacy

### Data Protection

**Implement Privacy by Design**: Build privacy protections into prompts and
configuration from the beginning.

```yaml
system_prompt: |
  You are a customer analytics expert who processes data while maintaining
  strict privacy protections.

  PRIVACY REQUIREMENTS:
  - Never log, display, or store individual customer records
  - Aggregate all results to minimum group size of 10
  - Replace actual names/emails with anonymized identifiers
  - Report only statistical summaries, not individual data points
  - Immediately flag and refuse requests for personal information

  If asked for individual records, respond: "I can only provide aggregated
  insights to protect customer privacy. Let me show you the overall trends
  instead."
```

**Configure Secure Tool Defaults**: Set tool configurations that prioritize
security over convenience.

```yaml
tool_config:
  http_client:
    verify_ssl: true           # Always verify certificates
    follow_redirects: false    # Prevent redirect attacks
    user_agent: "Caxton-Agent/1.0"  # Identify requests
    max_response_size: "50MB"  # Prevent resource exhaustion
  file_storage:
    encryption_at_rest: true   # Encrypt stored files
    access_logging: true       # Audit file access
    auto_cleanup_hours: 24     # Limit data retention
```

### Input Validation

**Validate All External Inputs**: Include validation logic in prompts to
handle malformed or malicious inputs safely.

```yaml
system_prompt: |
  You process API responses and user data with strict input validation.

  VALIDATION RULES:
  1. Check all JSON for valid syntax before processing
  2. Verify required fields are present and have expected types
  3. Reject files larger than configured limits
  4. Sanitize all text inputs to prevent injection attacks
  5. Log validation failures for security monitoring

  For invalid inputs, respond with specific error messages:
  - "Invalid JSON format: [specific syntax error]"
  - "Missing required field: [field name]"
  - "File too large: [size] exceeds [limit]"
```

## Testing and Validation

### Configuration Testing

**Test All Code Paths**: Ensure your prompts handle both success and failure
scenarios appropriately.

```yaml
# Include error handling instructions in prompts
system_prompt: |
  When tools return errors:
  1. Parse the error message for actionable information
  2. Attempt reasonable recovery strategies (retry, alternative approach)
  3. If recovery fails, explain the issue clearly to the user
  4. Always suggest next steps or alternatives

  Error response template:
  "I encountered an issue: [clear explanation]
  This likely means: [probable cause]
  You can try: [specific alternatives]"
```

**Validate Against Real Data**: Test agents with realistic datasets that
match production characteristics.

```yaml
# Document testing approach in agent comments
parameters:
  # Tested with:
  # - 10K row customer dataset (typical)
  # - 100K row transaction dataset (large)
  # - Files with missing values, duplicates, encoding issues
  # - Various date formats and numerical precisions
  test_datasets:
    - "customer_sample_10k.csv"
    - "transactions_large_100k.json"
    - "survey_responses_mixed_quality.csv"
```

## Development Workflow

### Version Management

**Use Semantic Versioning**: Version agents according to the impact of
changes on users and integration partners.

```yaml
# Breaking changes to capabilities or tool requirements
version: "2.0.0"

# New features or capabilities added
version: "1.1.0"

# Bug fixes and prompt improvements
version: "1.0.1"
```

**Document Changes**: Maintain clear changelog information in agent
documentation.

```markdown
## Changelog

### v1.2.0 (2025-09-10)
- Added support for Excel file processing
- Improved error handling for malformed CSV files
- Updated statistical analysis to include confidence intervals

### v1.1.0 (2025-09-01)
- Added chart generation capabilities
- Enhanced data quality validation
- Optimized memory usage for large datasets
```

### Development Environment

**Use Hot Reload for Iteration**: Configure development environment to
automatically reload agent changes for rapid testing.

**Test with Representative Data**: Use datasets that match production
characteristics in size, format, and quality.

**Monitor Performance Metrics**: Track execution time, memory usage, and
success rates during development.

## Common Pitfalls to Avoid

### Context Management Mistakes

❌ **Manual Context Management**: Trying to manage context in prompts
✅ **Trust the System**: Let the context management system handle conversation
history and memory automatically

❌ **Context Duplication**: Repeating information already provided by context
system
✅ **Focus on Expertise**: Define agent behavior and expertise, not contextual
information

❌ **Over-Specified Context**: Requesting excessive context that hurts
performance
✅ **Balanced Context**: Optimize for 80% use case with reasonable context limits

❌ **Ignoring Context Performance**: Not monitoring context preparation metrics
✅ **Performance Monitoring**: Track context prep time, token efficiency, and
success rates

### Prompt Engineering Mistakes

❌ **Vague Instructions**: "Analyze this data and provide insights"
✅ **Specific Requirements**: "Perform statistical analysis including mean,
median, standard deviation, and identify outliers beyond 2 standard deviations"

❌ **No Error Handling**: Assuming all operations will succeed
✅ **Explicit Error Cases**: "If the file cannot be parsed, explain the
specific format issue and suggest corrections"

❌ **Over-Complex Prompts**: Single prompt trying to handle every possible
scenario
✅ **Focused Responsibility**: Each agent handles a specific, well-defined domain

### Configuration Mistakes

❌ **Over-Permissioned Tools**: Granting access to unnecessary tools
✅ **Minimal Permissions**: Only tools required for core functionality

❌ **No Resource Limits**: Allowing unlimited execution time and memory
✅ **Appropriate Limits**: Based on expected workload characteristics

❌ **Generic Capabilities**: Capabilities like "data-processing"
✅ **Specific Capabilities**: Like "customer-satisfaction-analysis"

## Performance Monitoring

### Key Metrics to Track

**Execution Metrics**:

- Average execution time per request
- Memory usage patterns
- Tool call frequency and duration
- Error rates by category

**Business Metrics**:

- User satisfaction with results
- Task completion rates
- Time to value for common workflows

**Resource Utilization**:

- CPU and memory usage trends
- Tool concurrency patterns
- Queue depths and wait times

### Optimization Strategies

**Profile Before Optimizing**: Measure actual performance characteristics
before making changes.

**Optimize Common Paths**: Focus optimization efforts on frequently used
functionality.

**Cache Expensive Operations**: Store results of computationally expensive
operations when appropriate.

**Parallelize Independent Tasks**: Use concurrent tool calls for independent
operations.

## Next Steps

- **Migration Guide**: Convert existing WASM agents to configuration format
- **Tool Development**: Create custom tools for specialized requirements
- **Template Library**: Explore additional templates and patterns
- **Advanced Patterns**: Learn about agent composition and orchestration
