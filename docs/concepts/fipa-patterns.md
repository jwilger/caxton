---
title: "Lightweight Agent Messaging Patterns"
description:
  "Practical patterns for using agent messaging performatives in Caxton's
  configuration agent system"
layout: documentation
categories: [Messaging, Agent Communication, Communication Patterns]
date: 2025-09-10
---

## Overview

Caxton implements a lightweight capability-based messaging system optimized for
configuration-driven agents. This approach provides standardized communication
patterns while remaining accessible to agents defined through YAML
configuration and natural language prompts.

## Core Performatives

### REQUEST Pattern

Used when an agent needs another agent to perform an action.

**Structure:**

```yaml
performative: REQUEST
capability: target-capability
conversation_id: unique-thread-id
reply_with: unique-request-id
content: "Natural language description of what needs to be done"
parameters:
  key1: value1
  key2: value2
```

**Example - Data Processing Request:**

```yaml
performative: REQUEST
capability: data-processing
conversation_id: conv_batch_001
reply_with: req_process_001
content: |
  Please process the customer order data for September 2024:
  - Validate all order entries for completeness
  - Calculate monthly sales totals by product category
  - Generate exception report for orders missing required fields

parameters:
  file_location: "/data/orders_2024_09.csv"
  output_format: "json"
  validation_level: "strict"
  deadline: "2024-09-10T17:00:00Z"
```

**Response Pattern:**

```yaml
# Success response
performative: INFORM
conversation_id: conv_batch_001
in_reply_to: req_process_001
content: |
  Data processing completed successfully:
  - Processed 15,847 order records
  - Found 23 validation errors (see attached report)
  - Monthly totals calculated for 12 product categories

results:
  total_orders: 15847
  validation_errors: 23
  processing_time_ms: 3420
  output_location: "/results/processed_orders_2024_09.json"

# Failure response
performative: FAILURE
conversation_id: conv_batch_001
in_reply_to: req_process_001
content: |
  Data processing failed: Unable to access input file

error:
  type: "file_access_error"
  details: "Permission denied: /data/orders_2024_09.csv"
  recovery_suggestions:
    - "Check file permissions"
    - "Verify file path is correct"
    - "Ensure data volume is mounted"
```

### INFORM Pattern

Used to share information or report results without expecting a response.

**Structure:**

```yaml
performative: INFORM
capability: target-capability # Optional for broadcasts
conversation_id: thread-id # Optional for new information
content: "Information to share"
```

**Example - Status Update:**

```yaml
performative: INFORM
conversation_id: conv_monitor_001
content: |
  System performance report for 2024-09-10:
  - Average response time: 234ms
  - Success rate: 99.7%
  - Peak concurrent users: 1,247
  - Disk usage: 67% of capacity

metrics:
  response_time_avg_ms: 234
  success_rate: 0.997
  peak_users: 1247
  disk_usage_percent: 67
  report_timestamp: "2024-09-10T14:30:00Z"
```

**Example - Unsolicited Information:**

```yaml
performative: INFORM
capability: system-monitoring
content: |
  Alert: High memory usage detected on server-prod-03
  - Current usage: 94% of 32GB
  - Top processes consuming memory:
    * java_app (8.2GB)
    * elasticsearch (6.8GB)
    * mysql (4.1GB)
  - Recommended action: Investigate memory leaks

alert_level: "warning"
affected_systems: ["server-prod-03"]
recommended_actions:
  - "Review application memory usage"
  - "Check for memory leaks"
  - "Consider scaling if persistent"
```

### QUERY Pattern

Used to request information without asking for an action to be performed.

**Structure:**

```yaml
performative: QUERY
capability: target-capability
conversation_id: thread-id
reply_with: query-id
content: "What information is being requested"
query_params:
  key1: value1
```

**Example - Information Request:**

```yaml
performative: QUERY
capability: inventory-management
conversation_id: conv_stock_001
reply_with: query_stock_001
content: |
  What is the current stock level for the following products?
  - Product SKU: LAPTOP_001
  - Product SKU: MOUSE_002
  - Product SKU: MONITOR_003

  Please include:
  - Current quantity on hand
  - Reserved quantity for pending orders
  - Available quantity for new orders
  - Next expected restock date

query_params:
  product_skus: ["LAPTOP_001", "MOUSE_002", "MONITOR_003"]
  include_reserved: true
  include_restock_dates: true
```

**Response Pattern:**

```yaml
performative: INFORM
conversation_id: conv_stock_001
in_reply_to: query_stock_001
content: |
  Current inventory levels as of 2024-09-10 15:30:00:

  LAPTOP_001:
  - On hand: 45 units
  - Reserved: 12 units
  - Available: 33 units
  - Next restock: 2024-09-15

  MOUSE_002:
  - On hand: 156 units
  - Reserved: 23 units
  - Available: 133 units
  - Next restock: 2024-09-20

  MONITOR_003:
  - On hand: 8 units
  - Reserved: 15 units
  - Available: -7 units (BACKORDER)
  - Next restock: 2024-09-12 (URGENT)

inventory_data:
  - sku: "LAPTOP_001"
    on_hand: 45
    reserved: 12
    available: 33
    next_restock: "2024-09-15"
  - sku: "MOUSE_002"
    on_hand: 156
    reserved: 23
    available: 133
    next_restock: "2024-09-20"
  - sku: "MONITOR_003"
    on_hand: 8
    reserved: 15
    available: -7
    status: "backorder"
    next_restock: "2024-09-12"
```

## Negotiation Patterns

### PROPOSE-ACCEPT/REJECT Pattern

Used for simple negotiation without complex bidding.

**Example - Resource Allocation Proposal:**

Step 1 - Proposal:

```yaml
performative: PROPOSE
capability: resource-allocation
conversation_id: conv_resource_001
reply_with: prop_cpu_001
content: |
  I propose allocating additional CPU resources for the data processing job:
  - Current allocation: 4 CPU cores
  - Proposed allocation: 8 CPU cores
  - Estimated completion time improvement: 60%
  - Additional cost: $12.50 per hour
  - Duration needed: 3 hours

proposal_details:
  resource_type: "cpu_cores"
  current_allocation: 4
  proposed_allocation: 8
  cost_per_hour: 12.50
  duration_hours: 3
  performance_improvement: 0.60
```

Step 2a - Accept Proposal:

```yaml
performative: ACCEPT_PROPOSAL
conversation_id: conv_resource_001
in_reply_to: prop_cpu_001
content: |
  Proposal accepted. Proceeding with CPU scaling:
  - Scaling from 4 to 8 cores approved
  - Budget allocation: $37.50 total approved
  - Start time: Immediately
  - Expected completion: 2024-09-10 18:30:00

approval_details:
  approved_allocation: 8
  approved_budget: 37.50
  start_time: "2024-09-10T15:30:00Z"
  expected_completion: "2024-09-10T18:30:00Z"
```

Step 2b - Reject Proposal:

```yaml
performative: REJECT_PROPOSAL
conversation_id: conv_resource_001
in_reply_to: prop_cpu_001
content: |
  Proposal rejected due to budget constraints:
  - Current budget remaining: $25.00
  - Proposed cost: $37.50
  - Alternative suggestion: Scale to 6 cores for $18.75

rejection_reason: "budget_exceeded"
alternative_proposal:
  resource_allocation: 6
  cost_per_hour: 6.25
  duration_hours: 3
  total_cost: 18.75
```

## Error Handling Patterns

### FAILURE Pattern

Used when an agent cannot fulfill a request.

**Structure:**

```yaml
performative: FAILURE
conversation_id: thread-id
in_reply_to: original-message-id
content: "Description of what failed and why"
error:
  type: error-type
  details: "Detailed error information"
  recovery_suggestions: ["suggestion1", "suggestion2"]
```

**Example - Processing Failure:**

```yaml
performative: FAILURE
conversation_id: conv_analysis_001
in_reply_to: req_analyze_001
content: |
  Data analysis failed due to data format issues:
  - Expected CSV format with headers
  - Received binary file (Excel format)
  - Cannot process Excel files with current tools

error:
  type: "unsupported_format"
  details: "File appears to be Excel (.xlsx) format, but agent only supports
    CSV"
  file_info:
    detected_format: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    expected_format: "text/csv"
    file_size: 2458623
  recovery_suggestions:
    - "Convert Excel file to CSV format"
    - "Use data-conversion capability to transform file"
    - "Request file in CSV format from data source"
```

### NOT_UNDERSTOOD Pattern

Used when an agent cannot interpret a message.

**Example:**

```yaml
performative: NOT_UNDERSTOOD
conversation_id: conv_unclear_001
in_reply_to: req_unclear_001
content: |
  I cannot understand this request due to ambiguous parameters:
  - "Process the data" is too vague
  - No data source specified
  - Processing method not clear

  Please clarify:
  - What specific data should be processed?
  - What type of processing is needed?
  - What format should the output be?

clarification_needed:
  - "data_source"
  - "processing_type"
  - "output_format"
examples:
  - "Analyze sales data from /data/sales.csv for trends"
  - "Validate customer records in database table 'customers'"
  - "Aggregate log files from /logs/ directory by hour"
```

## Communication Patterns for Configuration Agents

### Multi-Turn Conversations

Configuration agents can maintain context across multiple message exchanges:

```yaml
# Turn 1 - Initial request
performative: REQUEST
capability: financial-analysis
conversation_id: conv_finance_001
reply_with: req_analysis_001
content: |
  Please analyze our Q3 financial performance:
  - Revenue trends by month
  - Expense category breakdown
  - Profit margin analysis

# Turn 2 - Follow-up question
performative: QUERY
conversation_id: conv_finance_001
reply_with: query_detail_001
content: |
  Based on your analysis, which expense categories show the most volatility?
  Can you provide month-by-month breakdown for the top 3 volatile categories?

# Turn 3 - Additional request
performative: REQUEST
conversation_id: conv_finance_001
reply_with: req_forecast_001
content: |
  Using the volatility patterns you identified, can you create a forecast
  model for Q4 expenses in those categories?
```

### Information Broadcasting

Agents can broadcast information to all agents with specific capabilities:

```yaml
performative: INFORM
capability: system-monitoring # Broadcast to all monitoring agents
content: |
  Scheduled maintenance notification:
  - Date: 2024-09-15
  - Time: 02:00-04:00 UTC
  - Affected systems: Database cluster
  - Expected downtime: 90 minutes
  - Impact: Read-only mode during maintenance

broadcast: true
maintenance_event:
  scheduled_date: "2024-09-15"
  start_time: "02:00 UTC"
  end_time: "04:00 UTC"
  affected_systems: ["db-primary", "db-replica-1", "db-replica-2"]
  impact_level: "read_only"
```

### Capability Discovery

Agents can discover what capabilities are available:

```yaml
performative: QUERY
capability: system:registry
conversation_id: conv_discovery_001
reply_with: query_capabilities_001
content: |
  What capabilities are currently available for data processing tasks?
  Please include information about:
  - Supported data formats
  - Performance characteristics
  - Current availability status

query_params:
  capability_category: "data-processing"
  include_metadata: true
  include_availability: true
```

Response:

```yaml
performative: INFORM
conversation_id: conv_discovery_001
in_reply_to: query_capabilities_001
content: |
  Available data processing capabilities:

  1. data-cleaning (3 agents available)
     - Formats: CSV, JSON, XML
     - Avg response: 2.3 seconds
     - Max file size: 100MB

  2. statistical-analysis (2 agents available)
     - Formats: CSV, JSON
     - Avg response: 8.7 seconds
     - Specialties: Time series, regression

  3. data-visualization (1 agent available)
     - Formats: CSV, JSON
     - Avg response: 12.4 seconds
     - Output: PNG, SVG, PDF charts

available_capabilities:
  - name: "data-cleaning"
    agent_count: 3
    supported_formats: ["csv", "json", "xml"]
    avg_response_time_ms: 2300
    max_file_size_mb: 100
  - name: "statistical-analysis"
    agent_count: 2
    supported_formats: ["csv", "json"]
    avg_response_time_ms: 8700
    specialties: ["time_series", "regression"]
  - name: "data-visualization"
    agent_count: 1
    supported_formats: ["csv", "json"]
    avg_response_time_ms: 12400
    output_formats: ["png", "svg", "pdf"]
```

## Advanced Patterns

### Conditional Processing

Agents can include conditional logic in their messages:

```yaml
performative: REQUEST
capability: data-processing
conversation_id: conv_conditional_001
reply_with: req_conditional_001
content: |
  Process the sales data with the following conditions:
  - If total sales > $100k, generate executive summary
  - If any product sales declined > 20%, flag for review
  - If data quality score < 0.8, run data cleaning first

conditional_logic:
  - condition: "total_sales > 100000"
    action: "generate_executive_summary"
  - condition: "any(product.sales_decline > 0.20)"
    action: "flag_for_review"
  - condition: "data_quality_score < 0.8"
    action: "run_data_cleaning_first"
```

### Workflow Coordination

Agents can coordinate complex workflows:

```yaml
performative: REQUEST
capability: workflow-orchestration
conversation_id: conv_pipeline_001
reply_with: req_orchestrate_001
content: |
  Execute the following data processing pipeline:

  1. Extract data from CRM (capability: data-extraction)
  2. Clean and validate data (capability: data-cleaning)
  3. Perform analysis (capability: statistical-analysis)
  4. Generate reports (capability: report-generation)
  5. Send notifications (capability: notification-service)

workflow_definition:
  name: "monthly_sales_analysis"
  steps:
    - capability: "data-extraction"
      parameters:
        source: "crm_database"
        date_range: "last_30_days"
    - capability: "data-cleaning"
      depends_on: ["data-extraction"]
      parameters:
        validation_rules: "strict"
    - capability: "statistical-analysis"
      depends_on: ["data-cleaning"]
      parameters:
        analysis_type: "trend_analysis"
    - capability: "report-generation"
      depends_on: ["statistical-analysis"]
      parameters:
        template: "executive_summary"
    - capability: "notification-service"
      depends_on: ["report-generation"]
      parameters:
        recipients: ["sales_team", "executives"]
```

## Best Practices for Agent Messaging

### Message Structure

1. **Use descriptive conversation_ids**: `conv_customer_analysis_2024_09_10`
   instead of `conv_001`
2. **Include context in content**: Natural language descriptions help
   configuration agents understand intent
3. **Provide structured parameters**: Use YAML structures for complex data
   alongside natural language
4. **Include error recovery suggestions**: Help receiving agents handle failures
   gracefully

### Conversation Management

1. **Link related messages**: Use `in_reply_to` to maintain conversation threads
2. **Set appropriate timeouts**: Include deadlines for time-sensitive requests
3. **Handle partial failures**: Design messages that can succeed partially
4. **Clean up conversations**: Set conversation timeouts to prevent memory leaks

### Performance Optimization

1. **Batch related requests**: Combine multiple related operations in single
   messages
2. **Use appropriate routing strategies**: Single recipient for critical tasks,
   load balancing for bulk work
3. **Include size estimates**: Help agents prepare appropriate resources
4. **Provide progress updates**: For long-running tasks, send periodic INFORM
   messages

### Security Considerations

1. **Validate input parameters**: Configuration agents should validate all
   message parameters
2. **Sanitize content**: Clean user-provided content before processing
3. **Check capabilities**: Verify agents have necessary permissions before
   sending requests
4. **Log security events**: Track security-relevant message patterns

The lightweight agent messaging system provides a robust foundation for
agent communication while remaining accessible to configuration-driven agents.
By following these patterns, agents can collaborate effectively while
maintaining loose coupling and operational flexibility.
