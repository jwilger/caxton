---
title: "ADR-0038: Template-Based Agent Creation"
date: 2025-01-17
status: accepted
layout: adr
categories: [Architecture, Developer Experience, Templates]
---

## Status

Accepted

## Context

Caxton aims to provide a "5-10 minute from zero to first agent" developer
experience. Currently, creating a new agent requires understanding TOML
configuration syntax, capability definitions, prompt engineering best
practices, and tool integration patterns. This learning curve creates friction
for new users and slows adoption.

### Core Challenge

The agent creation challenge involves multiple dimensions:

- **Configuration Complexity**: Agent TOML files require specific syntax and
  structure knowledge
- **Best Practices Discovery**: Users must learn prompt patterns, tool usage,
  and error handling
- **Initial Friction**: Creating the first agent from scratch is intimidating
  and error-prone
- **Consistency Issues**: Without templates, agents vary widely in quality
  and structure
- **Documentation Gap**: Static documentation cannot provide the interactive
  guidance users need

### Requirements

The solution must deliver:

- **5-10 Minute Experience**: From installation to working agent in under 10
  minutes
- **Zero External Dependencies**: Templates work offline without internet
  connectivity
- **Compile-Time Validation**: Templates validated during build to prevent
  runtime errors
- **Educational Value**: Templates teach best practices through working
  examples
- **Production Ready**: Generated agents should be immediately useful, not
  just demos

### Alternative Approaches Considered

**External Template Repository**: Rejected due to dependency on network
access and version synchronization complexity.

**Code Generation Framework**: Rejected due to complexity overhead and
maintenance burden of a full templating engine.

**Interactive Web UI**: Rejected as it violates the CLI-first philosophy and
adds deployment complexity.

**Documentation-Only Approach**: Rejected as copy-paste from docs is
error-prone and provides poor user experience.

## Decision

We will **embed 10 high-quality agent templates directly in the Caxton
binary** and provide a simple `caxton init` command for instant agent
creation.

### Implementation Design

**Template Storage**: Templates embedded in binary using Rust's
`include_str!` macro, ensuring zero runtime dependencies and compile-time
validation.

**Variable Substitution**: Simple string replacement for `{{agent_name}}`,
`{{agent_id}}`, and `{{created_at}}` variables using standard `str::replace`.

**Template Selection**: Core templates covering common use cases:

1. Data Analyzer - CSV/JSON processing with pandas-like operations
2. Code Reviewer - Repository analysis and code quality checks
3. Task Automator - Shell commands and file operations
4. API Client - HTTP requests and webhook handlers
5. Content Generator - Document creation and formatting
6. Data Validator - Schema validation and data quality
7. System Monitor - Resource tracking and alerting
8. Test Runner - Automated testing orchestration
9. Documentation Builder - Markdown generation and updates
10. Pipeline Coordinator - Multi-agent workflow orchestration

**Folder Structure**: Each template creates:

```text
my-agent/
├── agent.toml          # Complete configuration
├── README.md           # Usage instructions
├── examples/           # Example invocations
│   ├── basic.sh
│   └── advanced.sh
└── prompts/            # Optional prompt library
    └── templates.md
```

### CLI Interface

```bash
# List available templates
caxton template list

# Create agent from template
caxton init my-agent --template data-analyzer

# Create with custom ID
caxton init my-agent --template api-client --id custom-id-123

# Create in specific directory
caxton init /path/to/my-agent --template task-automator
```

## Consequences

### Positive Consequences

- **Instant Productivity**: Users have working agents in minutes, not hours
- **Best Practices Propagation**: Templates encode proven patterns and
  approaches
- **Reduced Support Burden**: Fewer questions about basic agent structure
- **Consistent Quality**: All agents start from validated, tested templates
- **Offline Capability**: Works in air-gapped environments without
  internet

### Negative Consequences

- **Binary Size Increase**: ~20-50KB for all templates (negligible compared
  to embedded models)
- **Template Maintenance**: Templates must evolve with configuration schema
  changes
- **Limited Customization**: v1.0 provides fixed templates without user
  extensibility
- **Version Lock-In**: Templates tied to binary version, requiring updates
  for changes

### Success Metrics

- **Time to First Agent**: <5 minutes for 90% of new users
- **Template Usage**: >80% of new agents created from templates
- **Error Reduction**: 50% fewer configuration errors in first week
- **User Satisfaction**: Positive feedback on onboarding experience

## Implementation Approach

Development proceeds in phases:

1. **Template Creation**: Develop and test 10 core templates with real use
   cases
2. **Binary Embedding**: Implement include_str! macro integration
3. **CLI Command**: Add init subcommand with template selection
4. **Validation Suite**: Compile-time and runtime template validation
5. **Documentation**: Template gallery with use case descriptions

## Alignment with Existing ADRs

This decision reinforces:

- **ADR-0028 (Config Agents)**: Templates showcase configuration-driven
  approach
- **ADR-0009 (CLI Design)**: Extends CLI with developer-friendly commands
- **ADR-0032 (TOML Configuration)**: Templates demonstrate TOML best
  practices
- **ADR-0004 (Minimal Core)**: Templates embedded in binary maintain zero
  dependencies

## Industry Precedent

The template approach follows successful patterns:

- **Vite**: Create-vite provides instant project scaffolding with embedded
  templates
- **Create React App**: Popularized the template-first onboarding
  experience
- **Cargo Generate**: Rust ecosystem's template-based project creation
- **Vue CLI**: Template selection for different application types
- **Yeoman**: Pioneered generator-based scaffolding (though we avoid its
  complexity)

These tools demonstrate that templates dramatically improve developer
onboarding and adoption.

## Future Considerations

- **Custom Templates**: v2.0 could support user-defined templates in
  ~/.caxton/templates
- **Template Marketplace**: Community-contributed templates with curation
- **Interactive Mode**: TUI-based template customization during creation
- **Template Updating**: Mechanism to refresh templates without binary updates
- **Composition**: Combining multiple templates for complex agents

## Security Considerations

Templates must be reviewed for:

- **No Privileged Operations**: Templates use only safe, sandboxed
  capabilities
- **Input Validation**: Example code demonstrates proper validation
- **Error Handling**: Templates include comprehensive error management
- **Resource Limits**: Appropriate memory and CPU constraints configured

## References

- [Vite's Template Strategy](https://vitejs.dev/guide/#scaffolding-your-first-vite-project)
- [Create React App Templates](https://create-react-app.dev/docs/custom-templates/)
- [Cargo Generate](https://github.com/cargo-generate/cargo-generate)
- ADR-0028: Configuration-Driven Agent Architecture
- ADR-0032: TOML Agent Configuration
