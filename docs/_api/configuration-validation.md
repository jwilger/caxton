---
title: "Configuration Validation & Testing API"
date: 2025-09-10
layout: page
categories: [api, validation]
---

## Overview

The Configuration Validation & Testing API helps developers create valid
configuration agents by providing comprehensive validation, testing, and
development support. These endpoints enable rapid iteration and debugging
of agent configurations before deployment.

**Key Features**:

- **YAML validation**: Syntax and schema validation for frontmatter
- **Capability verification**: Check that declared capabilities are available
- **Tool availability**: Verify required MCP tools are accessible
- **Template management**: Pre-built configurations for common use cases
- **Configuration testing**: Simulate agent behavior without deployment
- **Hot-reload support**: Real-time configuration updates during development

## Core Validation Endpoints

### Validate Configuration

**POST** `/api/v1/validate/config`

Comprehensive validation of agent configuration without deployment.

#### Request Body

```json
{
  "content": "---\nname: DataAnalyzer\nversion: \"1.0.0\"\ncapabilities:\n  - data-analysis\n  - report-generation\ntools:\n  - http_client\n  - csv_parser\n  - chart_generator\nparameters:\n  max_file_size: \"10MB\"\n  supported_formats: [\"csv\", \"json\", \"xlsx\"]\nmemory_enabled: true\nmemory_scope: \"workspace\"\nsystem_prompt: |\n  You are a data analysis expert...\nuser_prompt_template: |\n  Analyze: {{request}}\n---\n\n# DataAnalyzer\n\nSpecializes in data analysis tasks.",
  "workspace": "development",
  "strict_mode": true,
  "check_tool_availability": true
}
```

#### Request Fields

- `content` (string, required): Complete markdown content with YAML frontmatter
- `workspace` (string, optional): Target workspace for validation (default: "default")
- `strict_mode` (boolean, optional): Enable strict validation rules (default: false)
- `check_tool_availability` (boolean, optional): Verify tool accessibility
  (default: true)

#### Response (200 OK)

```json
{
  "valid": true,
  "validation_id": "val_abc123def456",
  "parsed_config": {
    "name": "DataAnalyzer",
    "version": "1.0.0",
    "capabilities": ["data-analysis", "report-generation"],
    "tools": ["http_client", "csv_parser", "chart_generator"],
    "parameters": {
      "max_file_size": "10MB",
      "supported_formats": ["csv", "json", "xlsx"]
    },
    "memory_enabled": true,
    "memory_scope": "workspace",
    "system_prompt": "You are a data analysis expert...",
    "user_prompt_template": "Analyze: {{request}}"
  },
  "validation_results": {
    "yaml_syntax": "valid",
    "schema_compliance": "valid",
    "name_uniqueness": "valid",
    "capability_availability": "valid",
    "tool_availability": "valid",
    "prompt_structure": "valid",
    "memory_configuration": "valid"
  },
  "warnings": [
    {
      "type": "performance",
      "message": "Large system prompt may impact response time",
      "field": "system_prompt",
      "severity": "low"
    }
  ],
  "suggestions": [
    {
      "type": "optimization",
      "message": "Consider adding error handling in user_prompt_template",
      "field": "user_prompt_template"
    }
  ],
  "estimated_resources": {
    "memory_usage_mb": 85,
    "startup_time_seconds": 3.2,
    "tokens_per_request_estimate": 450
  }
}
```

#### Validation Errors (422 Unprocessable Entity)

```json
{
  "valid": false,
  "validation_id": "val_def789ghi012",
  "errors": [
    {
      "type": "schema_violation",
      "field": "capabilities",
      "message": "At least one capability must be declared",
      "location": "frontmatter.capabilities",
      "severity": "error"
    },
    {
      "type": "tool_unavailable",
      "field": "tools",
      "message": "Tool 'advanced_analytics' is not available in this workspace",
      "location": "frontmatter.tools[2]",
      "severity": "error",
      "available_alternatives": ["basic_analytics", "statistical_analysis"]
    },
    {
      "type": "yaml_syntax",
      "field": "frontmatter",
      "message": "Invalid YAML: mapping values are not allowed here",
      "location": "line 8, column 15",
      "severity": "error"
    }
  ],
  "warnings": [],
  "suggestions": [
    {
      "type": "fix_suggestion",
      "message": "Add at least one capability like 'data-processing'",
      "field": "capabilities"
    }
  ]
}
```

### Validate Capabilities

**POST** `/api/v1/validate/capabilities`

Verify that declared capabilities are valid and available.

#### Request Body

```json
{
  "capabilities": ["data-analysis", "report-generation", "custom-analytics"],
  "workspace": "development",
  "check_conflicts": true
}
```

#### Request Fields

- `capabilities` (array, required): List of capability names to validate
- `workspace` (string, optional): Workspace context for validation
- `check_conflicts` (boolean, optional): Check for capability conflicts
  (default: true)

#### Response (200 OK)

```json
{
  "valid": true,
  "capabilities": [
    {
      "name": "data-analysis",
      "status": "valid",
      "existing_providers": 2,
      "competition_level": "medium"
    },
    {
      "name": "report-generation",
      "status": "valid",
      "existing_providers": 1,
      "competition_level": "low"
    },
    {
      "name": "custom-analytics",
      "status": "warning",
      "message": "Non-standard capability name - consider using 'data-analysis'",
      "existing_providers": 0,
      "competition_level": "none"
    }
  ],
  "conflicts": [],
  "recommendations": [
    {
      "type": "naming",
      "message": "Consider renaming 'custom-analytics' to 'advanced-analytics' for better discoverability"
    }
  ]
}
```

### Validate YAML Syntax

**POST** `/api/v1/validate/yaml`

Validate only YAML frontmatter syntax and structure.

#### Request Body

```json
{
  "yaml_content": "name: DataAnalyzer\nversion: \"1.0.0\"\ncapabilities:\n  - data-analysis\ntools:\n  - http_client\nmemory_enabled: true"
}
```

#### Response (200 OK)

```json
{
  "valid": true,
  "parsed_yaml": {
    "name": "DataAnalyzer",
    "version": "1.0.0",
    "capabilities": ["data-analysis"],
    "tools": ["http_client"],
    "memory_enabled": true
  },
  "syntax_checks": {
    "yaml_structure": "valid",
    "indentation": "valid",
    "quotes_balanced": "valid",
    "required_fields": "valid"
  }
}
```

## Configuration Testing

### Test Configuration

**POST** `/api/v1/test/config`

Test agent configuration by simulating behavior without full deployment.

#### Request Body

```json
{
  "content": "---\nname: TestAgent\n...",
  "test_scenarios": [
    {
      "name": "basic_data_request",
      "input": "Analyze the sales data from Q3",
      "expected_capabilities": ["data-analysis"],
      "expected_tools": ["http_client", "csv_parser"]
    },
    {
      "name": "chart_generation",
      "input": "Create a bar chart showing monthly trends",
      "expected_capabilities": ["report-generation"],
      "expected_tools": ["chart_generator"]
    }
  ],
  "workspace": "testing"
}
```

#### Request Fields

- `content` (string, required): Agent configuration to test
- `test_scenarios` (array, required): List of test scenarios to run
- `workspace` (string, optional): Testing workspace
- `timeout_seconds` (integer, optional): Test timeout (default: 30)

#### Response (200 OK)

```json
{
  "test_id": "test_mno456pqr789",
  "overall_result": "passed",
  "configuration_valid": true,
  "scenarios": [
    {
      "name": "basic_data_request",
      "status": "passed",
      "execution_time_ms": 245,
      "capabilities_triggered": ["data-analysis"],
      "tools_used": ["http_client", "csv_parser"],
      "response_preview": "I'll analyze the Q3 sales data. Let me fetch the data...",
      "prompt_tokens": 156,
      "response_tokens": 23
    },
    {
      "name": "chart_generation",
      "status": "passed",
      "execution_time_ms": 312,
      "capabilities_triggered": ["report-generation"],
      "tools_used": ["chart_generator"],
      "response_preview": "I'll create a bar chart showing monthly trends...",
      "prompt_tokens": 189,
      "response_tokens": 31
    }
  ],
  "performance_metrics": {
    "average_response_time_ms": 278,
    "total_tokens_used": 399,
    "memory_usage_peak_mb": 67
  },
  "issues_found": []
}
```

#### Test Failures (422 Unprocessable Entity)

```json
{
  "test_id": "test_stu901vwx234",
  "overall_result": "failed",
  "configuration_valid": false,
  "scenarios": [
    {
      "name": "basic_data_request",
      "status": "failed",
      "error": "Tool 'csv_parser' not available in testing workspace",
      "execution_time_ms": 50
    }
  ],
  "issues_found": [
    {
      "type": "tool_unavailable",
      "severity": "error",
      "message": "Required tool 'csv_parser' not found"
    }
  ]
}
```

### Test Prompt Templates

**POST** `/api/v1/test/prompts`

Test prompt template rendering with sample data.

#### Request Body

```json
{
  "system_prompt": "You are a data analysis expert who helps users understand their data.",
  "user_prompt_template": "Analyze the following data request: {{request}}\n\nAvailable data: {{context}}\nUser requirements: {{requirements}}",
  "test_data": [
    {
      "variables": {
        "request": "Show me Q3 sales trends",
        "context": "CSV file with 1,250 rows of sales data",
        "requirements": "Focus on regional performance"
      },
      "expected_keywords": ["Q3", "sales", "trends", "regional"]
    }
  ]
}
```

#### Response (200 OK)

```json
{
  "template_valid": true,
  "test_results": [
    {
      "test_index": 0,
      "rendered_prompt": "Analyze the following data request: Show me Q3 sales trends\n\nAvailable data: CSV file with 1,250 rows of sales data\nUser requirements: Focus on regional performance",
      "template_variables_found": ["request", "context", "requirements"],
      "missing_variables": [],
      "extra_variables": [],
      "keyword_matches": ["Q3", "sales", "trends", "regional"],
      "prompt_length": 156,
      "estimated_tokens": 39
    }
  ],
  "template_analysis": {
    "variable_count": 3,
    "conditional_logic": false,
    "loops": false,
    "complexity_score": "simple"
  }
}
```

## Template Management

### List Configuration Templates

**GET** `/api/v1/templates`

Retrieve available configuration agent templates.

#### Query Parameters

- `category` (string, optional): Filter by template category
- `capability` (string, optional): Filter by required capability
- `complexity` (string, optional): Filter by complexity level (`simple`,
  `intermediate`, `advanced`)
- `language` (string, optional): Filter by primary language/domain

#### Response (200 OK)

```json
{
  "templates": [
    {
      "id": "data-analyzer-basic",
      "name": "Basic Data Analyzer",
      "description": "Analyzes CSV, JSON files with basic visualizations",
      "category": "data-processing",
      "complexity": "simple",
      "capabilities": ["data-analysis"],
      "required_tools": ["http_client", "csv_parser"],
      "optional_tools": ["chart_generator"],
      "use_cases": [
        "Sales data analysis",
        "Survey result processing",
        "Basic reporting"
      ],
      "parameters": [
        {
          "name": "AGENT_NAME",
          "description": "Name for your analyzer agent",
          "required": true,
          "example": "SalesAnalyzer"
        }
      ],
      "estimated_setup_time": "5 minutes"
    },
    {
      "id": "web-researcher-advanced",
      "name": "Advanced Web Researcher",
      "description": "Comprehensive web research with source validation",
      "category": "research",
      "complexity": "advanced",
      "capabilities": ["web-search", "content-analysis", "fact-checking"],
      "required_tools": ["web_search", "html_parser", "pdf_generator", "fact_checker"],
      "memory_enabled": true,
      "use_cases": [
        "Academic research",
        "Market intelligence",
        "Competitive analysis"
      ],
      "estimated_setup_time": "15 minutes"
    }
  ],
  "total": 2,
  "categories": ["data-processing", "research", "content-generation", "automation"],
  "complexity_levels": ["simple", "intermediate", "advanced"]
}
```

### Get Template Details

**GET** `/api/v1/templates/{template_id}`

Retrieve detailed template information including full configuration content.

#### Path Parameters

- `template_id` (string, required): Template identifier

#### Response (200 OK)

```json
{
  "id": "data-analyzer-basic",
  "name": "Basic Data Analyzer",
  "description": "Analyzes CSV, JSON files with basic visualizations",
  "category": "data-processing",
  "complexity": "simple",
  "version": "1.2.0",
  "created_at": "2025-08-15T10:00:00Z",
  "updated_at": "2025-09-01T14:30:00Z",
  "content": "---\nname: {{AGENT_NAME}}\nversion: \"1.0.0\"\ncapabilities:\n  - data-analysis\ntools:\n  - http_client\n  - csv_parser\n  - {{OPTIONAL_CHART_TOOL:chart_generator}}\nparameters:\n  max_file_size: \"{{MAX_FILE_SIZE:10MB}}\"\n  supported_formats: [\"csv\", \"json\", \"xlsx\"]\nmemory_enabled: {{MEMORY_ENABLED:false}}\nsystem_prompt: |\n  You are a data analysis expert specializing in {{SPECIALIZATION:general analysis}}.\n  You help users understand their data through clear insights and visualizations.\n---\n\n# {{AGENT_NAME}}\n\nI specialize in analyzing structured data and providing actionable insights.\n\n## What I Can Do\n\n- Parse CSV, JSON, and Excel files\n- Generate summary statistics\n- Create basic visualizations\n- Identify trends and patterns\n\n## Usage Examples\n\nAsk me to:\n- \"Analyze the sales data at https://example.com/data.csv\"\n- \"Show me the top 10 customers by revenue\"\n- \"Create a chart of monthly trends\"",
  "parameters": [
    {
      "name": "AGENT_NAME",
      "description": "Name for your data analysis agent",
      "type": "string",
      "required": true,
      "example": "SalesAnalyzer",
      "validation": "^[A-Za-z][A-Za-z0-9_]{2,63}$"
    },
    {
      "name": "MAX_FILE_SIZE",
      "description": "Maximum file size to process",
      "type": "string",
      "required": false,
      "default": "10MB",
      "options": ["1MB", "10MB", "50MB", "100MB"]
    },
    {
      "name": "MEMORY_ENABLED",
      "description": "Enable memory for learning from past analyses",
      "type": "boolean",
      "required": false,
      "default": false
    },
    {
      "name": "SPECIALIZATION",
      "description": "Domain specialization for the analyzer",
      "type": "string",
      "required": false,
      "default": "general analysis",
      "options": ["sales analysis", "financial reporting", "survey analysis", "general analysis"]
    },
    {
      "name": "OPTIONAL_CHART_TOOL",
      "description": "Chart generation tool (leave empty if not needed)",
      "type": "string",
      "required": false,
      "default": "chart_generator",
      "conditional": true
    }
  ],
  "dependencies": {
    "required_tools": ["http_client", "csv_parser"],
    "optional_tools": ["chart_generator", "excel_parser"],
    "mcp_servers": ["file-processing-server", "chart-generation-server"]
  },
  "examples": [
    {
      "name": "Sales Analyzer",
      "description": "Configured for sales data analysis",
      "parameter_values": {
        "AGENT_NAME": "SalesAnalyzer",
        "MAX_FILE_SIZE": "50MB",
        "MEMORY_ENABLED": true,
        "SPECIALIZATION": "sales analysis"
      }
    }
  ]
}
```

### Generate Configuration from Template

**POST** `/api/v1/templates/{template_id}/generate`

Generate agent configuration from template with parameter substitution.

#### Path Parameters

- `template_id` (string, required): Template identifier

#### Request Body

```json
{
  "parameters": {
    "AGENT_NAME": "CustomerAnalyzer",
    "MAX_FILE_SIZE": "25MB",
    "MEMORY_ENABLED": true,
    "SPECIALIZATION": "customer behavior analysis"
  },
  "workspace": "customer-insights",
  "validate": true
}
```

#### Request Fields

- `parameters` (object, required): Parameter values for template substitution
- `workspace` (string, optional): Target workspace context
- `validate` (boolean, optional): Validate generated configuration (default: true)

#### Response (201 Created)

```json
{
  "generation_id": "gen_yza345bcd678",
  "template_id": "data-analyzer-basic",
  "generated_content": "---\nname: CustomerAnalyzer\nversion: \"1.0.0\"\ncapabilities:\n  - data-analysis\ntools:\n  - http_client\n  - csv_parser\n  - chart_generator\nparameters:\n  max_file_size: \"25MB\"\n  supported_formats: [\"csv\", \"json\", \"xlsx\"]\nmemory_enabled: true\nsystem_prompt: |\n  You are a data analysis expert specializing in customer behavior analysis.\n  You help users understand their data through clear insights and visualizations.\n---\n\n# CustomerAnalyzer\n\nI specialize in analyzing structured data and providing actionable insights...",
  "validation_result": {
    "valid": true,
    "warnings": [],
    "estimated_resources": {
      "memory_usage_mb": 92,
      "startup_time_seconds": 3.8
    }
  },
  "parameter_substitutions": 4,
  "ready_to_deploy": true
}
```

## Development Tools

### Hot-Reload Configuration

**PUT** `/api/v1/dev/config-agents/{id}/hot-reload`

Update a running configuration agent with new configuration (development only).

#### Path Parameters

- `id` (string, required): Configuration agent identifier

#### Request Body

```json
{
  "content": "---\nname: DataAnalyzer\nversion: \"1.1.0\"\n...",
  "validate_first": true,
  "backup_current": true
}
```

#### Response (200 OK)

```json
{
  "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
  "hot_reload_id": "reload_efg678hij901",
  "previous_version": "1.0.0",
  "new_version": "1.1.0",
  "changes_detected": [
    "version",
    "system_prompt",
    "parameters.max_file_size"
  ],
  "reload_time_ms": 1250,
  "backup_created": "backup_klm234nop567",
  "status": "success"
}
```

### Compare Configurations

**POST** `/api/v1/dev/config/compare`

Compare two configuration versions to identify differences.

#### Request Body

```json
{
  "original_content": "---\nname: DataAnalyzer\nversion: \"1.0.0\"\n...",
  "updated_content": "---\nname: DataAnalyzer\nversion: \"1.1.0\"\n...",
  "comparison_type": "semantic"
}
```

#### Response (200 OK)

```json
{
  "comparison_id": "comp_qrs789tuv012",
  "differences_found": 3,
  "changes": [
    {
      "type": "version_update",
      "field": "version",
      "old_value": "1.0.0",
      "new_value": "1.1.0",
      "impact": "low"
    },
    {
      "type": "content_change",
      "field": "system_prompt",
      "change_type": "text_modified",
      "impact": "medium",
      "description": "System prompt expanded with additional instructions"
    },
    {
      "type": "parameter_change",
      "field": "parameters.max_file_size",
      "old_value": "10MB",
      "new_value": "50MB",
      "impact": "high",
      "description": "Significant increase in resource usage"
    }
  ],
  "compatibility": {
    "backward_compatible": true,
    "requires_restart": false,
    "breaking_changes": []
  },
  "recommendations": [
    {
      "type": "testing",
      "message": "Test with larger files due to max_file_size increase"
    }
  ]
}
```

## Error Handling

### Validation Error Response Format

```json
{
  "error": "Configuration validation failed",
  "code": "VALIDATION_ERROR",
  "validation_id": "val_error_abc123",
  "details": {
    "error_count": 2,
    "warning_count": 1,
    "errors": [
      {
        "type": "schema_violation",
        "field": "capabilities",
        "message": "At least one capability must be declared",
        "location": "frontmatter.capabilities",
        "severity": "error"
      }
    ],
    "warnings": [
      {
        "type": "performance",
        "message": "Large system prompt may impact response time",
        "field": "system_prompt",
        "severity": "low"
      }
    ]
  },
  "timestamp": "2025-09-10T18:00:00Z",
  "request_id": "req_val_def456"
}
```

### Common Error Codes

- `VALIDATION_ERROR` - Configuration validation failed
- `YAML_SYNTAX_ERROR` - Invalid YAML syntax in frontmatter
- `TEMPLATE_NOT_FOUND` - Requested template does not exist
- `PARAMETER_MISSING` - Required template parameter not provided
- `TOOL_UNAVAILABLE` - Required tool not accessible
- `CAPABILITY_INVALID` - Invalid capability declaration
- `HOT_RELOAD_FAILED` - Hot-reload operation failed
- `TEST_EXECUTION_ERROR` - Configuration test failed to execute

## Performance Considerations

- **Validation speed**: Basic validation completes in 50-200ms
- **Full testing**: Complete configuration tests take 1-5 seconds
- **Template generation**: Parameter substitution completes in <100ms
- **Hot-reload**: Configuration updates apply in 1-3 seconds
- **Concurrent operations**: System supports 50+ concurrent validations

## Integration Examples

### Pre-deployment Validation

```javascript
// Validate configuration before deployment
const validation = await fetch('/api/v1/validate/config', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    content: configContent,
    workspace: 'production',
    strict_mode: true,
    check_tool_availability: true
  })
});

const result = await validation.json();
if (!result.valid) {
  console.error('Configuration validation failed:', result.errors);
  return;
}

// Deploy only if validation passes
await deployAgent(configContent);
```

### Template-Based Agent Creation

```javascript
// Generate configuration from template
const generated = await fetch('/api/v1/templates/data-analyzer-basic/generate', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    parameters: {
      AGENT_NAME: 'SalesAnalyzer',
      SPECIALIZATION: 'sales analysis',
      MEMORY_ENABLED: true
    },
    workspace: 'sales-team',
    validate: true
  })
});

const config = await generated.json();
if (config.ready_to_deploy) {
  await deployConfigAgent(config.generated_content);
}
```

### Development Workflow

```javascript
// Hot-reload during development
const hotReload = await fetch(`/api/v1/dev/config-agents/${agentId}/hot-reload`, {
  method: 'PUT',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    content: updatedConfig,
    validate_first: true,
    backup_current: true
  })
});

const reloadResult = await hotReload.json();
console.log(`Reloaded in
  ${reloadResult.reload_time_ms}ms`);
```

## Related Documentation

- [Configuration Agent API](config-agents.md) - Deploy and manage configuration agents
- [Capability Registration API](capability-registration.md) - Validate capability
  declarations
- [Memory System API](memory-integration.md) - Test memory-enabled configurations
- [Agent Messaging API](fipa-messaging.md) - Test agent communication capabilities
- [ADR-0028](../adr/0028-configuration-driven-agent-architecture.md) -
  Configuration agent architecture
