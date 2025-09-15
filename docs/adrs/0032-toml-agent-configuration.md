---
title: "ADR-0032: TOML Agent Configuration Format"
date: 2025-01-21
status: accepted
layout: adr
categories: [Configuration, Agent Architecture]
---

## Status

Accepted

## Context

The current configuration-driven agent architecture (ADR-0028) uses markdown
files with YAML frontmatter to define agent behavior, capabilities, and
configuration. While this approach has proven successful, it introduces several
complexity and maintenance challenges.

### Current State Problems

**Parsing Complexity**: Mixing YAML frontmatter with markdown content requires
complex parsing logic that must handle both formats correctly and maintain
proper boundaries between configuration and documentation sections.

**Error-Prone Format**: The YAML frontmatter approach is susceptible to
formatting errors, particularly with multiline strings and indentation issues
that can break agent parsing.

**Limited Tooling Support**: Most editors and IDEs don't provide optimal syntax
highlighting or validation for the YAML frontmatter + markdown combination,
leading to development friction.

**Validation Challenges**: Validating the configuration portion separately from
the documentation requires careful parsing to extract just the frontmatter,
making schema validation more complex.

**Template String Limitations**: YAML's handling of multiline strings for
system prompts and user prompt templates can be awkward, requiring careful
attention to indentation and escaping.

### Current Format Example

```yaml
---
name: DataAnalyzer
version: "1.0.0"
capabilities: ["data-analysis", "report-generation"]
tools: ["http_client", "csv_parser"]
system_prompt: |
  You are a data analysis expert...

  When you receive requests:
  1. Check your memory
  2. Use appropriate tools
user_prompt_template: |
  Analyze: {{request}}
  Context: {{context}}
---
# DataAnalyzer Agent

Documentation content here...
```

### Technical Requirements

Agent configuration needs to support:

- Simple scalar values (name, version, capabilities list)
- Complex nested structures (memory settings, parameters)
- Long multiline strings (system prompts, user prompt templates)
- Clear documentation and examples
- Schema validation and tooling support
- Template variable substitution patterns

## Decision

We will **replace markdown files with YAML frontmatter with pure TOML
configuration files** for defining configuration-driven agents.

### New TOML Format

Agents will be defined as `.toml` files with embedded markdown content:

```toml
name = "DataAnalyzer"
version = "1.0.0"
capabilities = ["data-analysis", "report-generation"]
tools = ["http_client", "csv_parser", "chart_generator"]

[memory]
enabled = true
scope = "agent"

[parameters]
max_file_size = "10MB"
supported_formats = ["csv", "json", "xlsx"]

system_prompt = '''
You are a data analysis expert who helps users understand their data.
You can fetch data from URLs, parse various formats, and create visualizations.

When you receive data analysis requests:
1. Check your memory for similar past analyses
2. Use appropriate tools to fetch and parse data
3. Generate clear insights and recommendations
4. Store successful patterns in memory for future use
'''

user_prompt_template = '''
Analyze the following data request: {{request}}

Available context from memory: {{memory_context}}
Data source: {{data_source}}
Requirements: {{requirements}}
'''

documentation = '''
# DataAnalyzer Agent

This agent specializes in data analysis tasks and can:
- Fetch data from HTTP endpoints
- Parse CSV, JSON, and Excel files
- Generate charts and visualizations
- Provide statistical summaries

## Usage Examples

Ask me to:
- "Analyze the sales data at https://example.com/sales.csv"
- "Create a chart showing monthly trends"
- "Summarize the key metrics in this dataset"
'''
```

### Key Design Principles

**Clean Separation**: Configuration data and documentation are clearly separated
within the same file, eliminating parsing ambiguity.

**Native Multiline Support**: TOML's triple-quoted strings (`'''`) provide
excellent support for system prompts and templates without indentation issues.

**Schema-Friendly**: Pure TOML structure enables straightforward schema
validation and IDE support without mixed-format complexity.

**Rust Ecosystem Standard**: TOML is the standard configuration format across
the Rust ecosystem, providing mature tooling and community familiarity.

**Template Integration**: Template variable patterns (`{{variable}}`) work
naturally within TOML string values.

## Alternatives Considered

### Alternative 1: Keep Current YAML Frontmatter

**Rejected** because it maintains the existing complexity of mixed formats and
parsing challenges that motivated this decision.

### Alternative 2: Pure JSON Configuration

**Rejected** because JSON lacks native multiline string support, making system
prompts and templates difficult to read and maintain. JSON also lacks comment
support for configuration documentation.

### Alternative 3: Pure YAML Files

**Rejected** because while YAML supports multiline strings, TOML provides
better tooling support in the Rust ecosystem and cleaner syntax for the nested
configuration structures needed by agents.

### Alternative 4: Separate Config + Markdown Files

**Rejected** because maintaining agent definitions across multiple files
increases complexity for users and deployment systems. The single-file approach
maintains ease of sharing and version control benefits.

## Consequences

### Positive

**Simplified Parsing**: Single-format parsing eliminates frontmatter extraction
complexity and reduces parser error surface area.

**Better Developer Experience**: Superior syntax highlighting, validation, and
IDE support for TOML files compared to mixed YAML frontmatter approach.

**Robust Multiline Strings**: TOML's triple-quoted strings provide clean,
readable format for system prompts and user prompt templates without
indentation issues.

**Schema Validation**: Direct TOML schema validation without complex
frontmatter extraction, enabling better development tooling and error reporting.

**Rust Ecosystem Alignment**: Follows Rust community standards and leverages
mature TOML parsing libraries like `serde` and `toml`.

**Template Clarity**: Template variable substitution patterns (`{{variable}}`)
are clearly visible within TOML string values.

**Single File Convenience**: Maintains the benefits of single-file agent
definitions for easy sharing and version control.

### Negative

**Migration Overhead**: Existing agents in markdown/YAML frontmatter format
will need conversion to the new TOML format.

**Documentation Location Change**: Documentation moves from markdown body to
TOML `documentation` field, changing how users view and edit agent docs.

**Format Learning Curve**: Users familiar with YAML frontmatter will need to
learn TOML syntax, though TOML is generally simpler than YAML.

**File Extension Change**: Agent files change from `.md` to `.toml`, requiring
updates to tooling, documentation, and user workflows.

### Risk Mitigation

**Automated Migration Tools**: Provide conversion utilities to automatically
migrate existing YAML frontmatter agents to TOML format with validation.

**Backward Compatibility Period**: Support both formats during a transition
period to allow gradual migration without breaking existing agents.

**Documentation and Examples**: Comprehensive TOML format documentation with
examples covering all common agent patterns and use cases.

**Template Library Updates**: Convert all existing agent templates and examples
to the new TOML format with side-by-side migration guides.

## Implementation Approach

The implementation will focus on four core areas:

1. **TOML Schema Definition**: Define comprehensive schema for agent
   configuration with validation rules for required fields and value constraints

2. **Parser Updates**: Replace YAML frontmatter parsing with TOML parsing using
   `serde` and `toml` crates for robust deserialization

3. **Migration Tooling**: Command-line utilities to convert existing agents
   from markdown/YAML to TOML format with validation and error reporting

4. **Development Experience**: Update IDE support, syntax highlighting, and
   validation tools for the new TOML agent configuration format

## Alignment with Existing ADRs

- **ADR-0028 (Configuration-Driven Agent Architecture)**: Maintains the
  configuration-driven approach while improving the configuration format
- **ADR-0031 (Context Management Architecture)**: TOML format better supports
  the context requirement specifications needed by the context management
  system
- **ADR-0018 (Domain Types with nutype)**: TOML parsing integrates cleanly
  with domain type validation using `serde` derive macros
- **ADR-0020 (Parse, Don't Validate)**: Pure TOML format enables better
  parse-time validation of agent configurations

## Related Decisions

- ADR-0028: Configuration-Driven Agent Architecture (defines the agents that
  will use TOML format)
- ADR-0031: Context Management Architecture (benefits from cleaner context
  requirement specifications)
- ADR-0018: Domain Types with nutype (TOML parsing integrates with domain type
  validation)

## References

- [TOML Specification](https://toml.io/en/) - Official TOML format
  specification
- [serde TOML](https://docs.rs/toml/) - Rust TOML parsing and serialization
  library
- Configuration format analysis from technical architecture research
- Developer experience patterns from Rust ecosystem configuration tools

---

**Implementation Status**: This ADR documents an architectural decision being
made. The TOML agent configuration format will replace the current
markdown/YAML frontmatter approach, with migration tooling provided to convert
existing agents to the new format.
