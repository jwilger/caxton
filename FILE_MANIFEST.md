# Documentation File Manifest After Reorganization

## Overview

This manifest documents all documentation files after the audience-first
reorganization, providing a complete reference for navigation configuration and
cleanup planning.

## Website Directory (`/website/`) - Jekyll Site

### Jekyll Configuration and Assets

- `_config.yml` - Jekyll site configuration
- `Gemfile` - Ruby dependencies
- `_data/navigation.yml` - ✅ NEW: Comprehensive navigation structure
- `_data/release.yml` - Release information

### Jekyll Components

- `_includes/navigation.html` - Navigation component
- `_includes/footer.html` - Footer component
- `_layouts/default.html` - Default page layout
- `_layouts/home.html` - Homepage layout
- `_layouts/documentation.html` - Documentation page layout
- `_layouts/adr.html` - ADR page layout
- `_layouts/adr-index.html` - ADR index layout

### Jekyll Assets

- `assets/css/main.scss` - Main stylesheet
- `assets/css/style.scss` - Additional styles
- `assets/css/documentation.css` - Documentation-specific styles
- `assets/css/adr.css` - ADR-specific styles
- `assets/js/caxton.js` - Main JavaScript
- `assets/js/adr-carousel.js` - ADR carousel functionality
- `assets/js/search-integration.js` - Search functionality
- `assets/img/logo.svg` - Site logo

### Jekyll Collections

#### ADRs (`_adrs/`) - 18 files

- `0001-observability-first-architecture.md`
- `0002-webassembly-for-agent-isolation.md`
- `0003-fipa-messaging-protocol.md`
- `0004-minimal-core-philosophy.md`
- `0005-mcp-for-external-tools.md`
- `0006-application-server-architecture.md`
- `0007-management-api-design.md`
- `0008-agent-deployment-model.md`
- `0009-cli-tool-design.md`
- `0010-external-agent-routing-api.md`
- `0011-capability-registration-in-code.md`
- `0012-pragmatic-fipa-subset.md`
- `0013-state-management-architecture.md`
- `0014-coordination-first-architecture.md`
- `0015-distributed-protocol-architecture.md`
- `0028-configuration-driven-agent-architecture.md`
- `0029-fipa-acl-lightweight-messaging.md`
- `0030-embedded-memory-system.md`

### Website Documentation (`docs/`) - 8 files

- `index.md` - Documentation homepage
- `ARCHITECTURE.md` - High-level architecture overview
- `ROADMAP.md` - Development roadmap
- `developer-guide/api-reference.md`
- `developer-guide/building-agents.md`
- `developer-guide/message-protocols.md`
- `developer-guide/testing.md`
- `operations/deployment.md`
- `operations/monitoring.md`
- `operations/security.md`
- `developer-guide/wasm-integration.md`

### Website Pages

- `index.md` - Homepage
- `adr/index.md` - ADR index page

## Documentation Directory (`/docs/`) - Comprehensive Reference

### Getting Started (5 files)

- `getting-started/quickstart.md` - 5-minute introduction
- `getting-started/installation.md` - Installation guide
- `getting-started/configuration.md` - Configuration setup
- `getting-started/first-agent.md` - First agent tutorial
- `getting-started/rest-api-quickstart.md` - REST API tutorial

### Audience-Specific Documentation

#### Agent Developers (`audiences/agent-developers/`) - 4 files

- `index.md` - Agent developer overview
- `building-agents.md` - Agent development guide
- `api-reference.md` - Runtime APIs
- `security.md` - Security guidelines

#### Operators (`audiences/operators/`) - 2 files

- `installation.md` - Production installation
- `runbook.md` - Operational runbook

#### Experimenters (`audiences/experimenters/`) - 2 files

- `index.md` - Experimenter overview
- `quickstart.md` - Quick experiments guide

#### Contributors (`audiences/contributors/`) - 6 files

- `index.md` - Contributor overview
- `development-setup.md` - Development environment
- `architecture-overview.md` - System architecture
- `coding-standards.md` - Code quality standards
- `domain-modeling.md` - Domain modeling guide
- `testing-guide.md` - Testing practices

### Technical Documentation

#### API Reference (`api/`) - 8 files

- `implementation-status.md` - API implementation status
- `config-agents.md` - Config agent API
- `config-agent-patterns.md` - Configuration patterns
- `capability-registration.md` - Capability registration API
- `configuration-validation.md` - Config validation system
- `memory-integration.md` - Memory system integration
- `performance-specifications.md` - Performance requirements
- `mcp-integration.md` - MCP integration API

#### Architecture (`architecture/`) - 4 files

- `coordination-first-overview.md` - Core architectural philosophy
- `component-interaction-diagrams.md` - Component interactions
- `routing-algorithm-design.md` - Message routing design
- `configuration-and-deployment.md` - Config and deployment

#### Messaging (`messaging/`) - 5 files

- `fipa-acl-subset.md` - Supported FIPA message types
- `fipa-patterns.md` - FIPA interaction patterns
- `conversation-management.md` - Conversation handling
- `capability-routing.md` - Capability-based routing
- `config-agent-integration.md` - Config agent messaging

#### Memory System (`memory-system/`) - 5 files

- `overview.md` - Memory system architecture
- `embedded-backend.md` - Embedded vector database
- `model-management.md` - Embedding model management
- `usage-patterns.md` - Memory usage patterns
- `migration.md` - Memory data migration

#### Config Agents (`config-agents/`) - 5 files

- `overview.md` - Config agents introduction
- `agent-format.md` - Agent configuration format
- `best-practices.md` - Configuration best practices
- `examples.md` - Configuration examples
- `llm-providers.md` - LLM provider configuration

#### Operations (`operations/`) - 5 files

- `agent-lifecycle-management.md` - Agent lifecycle
- `performance-tuning.md` - Performance optimization
- `state-recovery-patterns.md` - State recovery procedures
- `devops-security-guide.md` - DevOps security practices
- `error-handling-guide.md` - Error handling procedures

#### Developer Guide (`developer-guide/`) - 3 files

- `api-reference.md` - Complete API documentation
- `building-agents.md` - Agent development guide
- `security-guide.md` - Security best practices

### Architecture Decision Records (`adr/`) - 31 files

#### Foundation ADRs

- `0001-observability-first-architecture.md`
- `0002-webassembly-for-agent-isolation.md`
- `0003-fipa-messaging-protocol.md`

#### System Design ADRs

- `0004-minimal-core-philosophy.md`
- `0006-application-server-architecture.md`
- `0014-coordination-first-architecture.md`
- `0015-distributed-protocol-architecture.md`
- `0027-single-codebase-architecture.md`

#### Integration ADRs

- `0005-mcp-for-external-tools.md`
- `0030-embedded-memory-system.md`
- `0031-context-management-architecture.md`

#### API Design ADRs

- `0007-management-api-design.md`
- `0009-cli-tool-design.md`
- `0010-external-agent-routing-api.md`
- `0026-simplified-management-api-protocol.md`

#### Messaging ADRs

- `0012-pragmatic-fipa-subset.md`
- `0029-fipa-acl-lightweight-messaging.md`

#### Agent Design ADRs

- `0008-agent-deployment-model.md`
- `0011-capability-registration-in-code.md`
- `0028-configuration-driven-agent-architecture.md`

#### State Management ADRs

- `0013-state-management-architecture.md`

#### Implementation ADRs

- `0016-security-architecture.md`
- `0018-domain-types-nutype.md`
- `0019-primitives-at-boundaries.md`
- `0020-parse-dont-validate.md`
- `0021-atomic-primitives-exception.md`

### Specialized Topics

#### Performance (`benchmarks/`) - 1 file

- `performance-benchmarking-guide.md`

#### Security (`security/`) - 1 file

- `security-audit-checklist.md`

#### Patterns (`patterns/`) - 1 file

- `agent-communication-patterns.md`

#### Research (`research/`) - 1 file

- `lightweight-state-alternatives.md`

#### Learning (`learning/`) - 1 file

- `fipa-knowledge-base.md`

#### Specifications (`specifications/`) - 1 file

- `message-router-specification.md`

#### Technology (`technology/`) - 1 file

- `webassembly-ecosystem-guide.md`

#### MCP Integration (`mcp/`) - 1 file

- `state-tool-specification.md`

#### Monitoring (`monitoring/`) - 1 file

- `metrics-integration-guide.md`

#### Development (`development/`) - 1 file

- `testing-strategy.md`

#### Developer Resources (`developer/`) - 1 file

- `developer-experience-guide.md`

#### User Guides (`user-guide/`) - 1 file

- `clustering.md`

### Legacy/Root-Level Files (NEEDS REVIEW)

- `README.md` - Documentation index
- `TESTING.md` - Testing overview
- `type_system_improvements_report.md` - Type system analysis
- `performance_and_safety_metrics.md` - Performance metrics
- `domain-types.md` - Domain type documentation
- `wasm-runtime-architecture.md` - WASM runtime design

### Duplicate Content Directories (NEEDS CONSOLIDATION)

#### `/contributors/` (2 files) - Potential duplicates

- `development-guide.md` - Development workflow
- `testing.md` - Testing guide

## Orphaned Files Requiring Action

### Legacy Static Site Files (REMOVE)

```text
/docs/site/adr-template.html
/docs/site/css/style.css
/docs/site/img/logo.svg
/docs/site/js/adr-carousel.js
/docs/site/js/caxton.js
/docs/site/.nojekyll
```

### System Files (REMOVE IF FOUND)

- `.DS_Store` files
- `Thumbs.db` files
- `*.tmp` files
- `*.bak` files

## Summary Statistics

### By Location

- **Website (`/website/`)**: 39 files (Jekyll site + 8 docs + 18 ADRs + assets)
- **Documentation (`/docs/`)**: 120+ files across 18 directories
- **Total Documentation**: 150+ markdown files
- **Total Project**: 200+ files including assets and configuration

### By Content Type

- **ADRs**: 31 architecture decisions (18 in website, 31 in docs - some duplicated)
- **Getting Started**: 5 essential guides
- **Audience Docs**: 14 audience-specific guides
- **Technical Specs**: 8 API references + 20 technical guides
- **Operational Guides**: 5 production guides
- **Specialized Topics**: 8 specialized guides

### Navigation Coverage

- ✅ All major sections covered in navigation.yml
- ✅ Hierarchical structure defined
- ✅ Audience-first organization maintained
- ✅ ADR categorization complete
- ⚠️ Some orphaned files need integration

This manifest provides the foundation for completing the cleanup plan and
ensuring all documentation is properly organized and accessible.
