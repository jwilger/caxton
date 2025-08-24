# Information Request Protocol

This document defines the standardized format for information requests and
responses between SPARC workflow agents.

## InfoRequest Format

Use this format when requesting information from another agent:

```markdown
## InfoRequest

**Requesting Agent**: {agent_name} **Target Agent**: {target_agent_name}
**Request Type**: {request_type} **Priority**: {low|medium|high}

### Context Needed

{Brief description of what information is needed and why}

### Specific Questions

1. {Detailed question 1}
2. {Detailed question 2}
3. {Additional questions...}

### Expected Deliverables

- [ ] {Expected output 1}
- [ ] {Expected output 2}
- [ ] {Additional deliverables...}

### Scope Boundaries

- **In Scope**: {What should be included}
- **Out of Scope**: {What should be excluded}

### Deadline/Urgency

{When this information is needed and why}
```

## InfoResponse Format

Use this format when responding to information requests:

```markdown
## InfoResponse

**Responding Agent**: {agent_name} **Original Request**: {reference to request}
**Response Status**: {complete|partial|unable_to_fulfill}

### Findings

{Main findings and information discovered}

#### {Category 1}

{Detailed findings for this category}

#### {Category 2}

{Additional findings as needed}

### Sources/References

- {Source 1}: {Description/relevance}
- {Source 2}: {Description/relevance}
- {Additional sources...}

### Unable to Answer

{List any questions that could not be answered and reasons why}

### Recommendations/Next Steps

{Actionable recommendations based on findings}

### Quality Indicators

- **Confidence Level**: {high|medium|low}
- **Completeness**: {complete|partial|limited}
- **Currency**: {how recent the information is}
```

## Request Taxonomy

### Agent-Specific Request Types

#### external_docs

**Target**: researcher agent **Purpose**: Research external documentation, APIs,
standards, or best practices **Typical Questions**:

- What are the current best practices for {technology/approach}?
- What external libraries/tools are available for {use case}?
- What are the known limitations or gotchas for {approach}?

#### implementation_context

**Target**: implementer agent **Purpose**: Understand codebase context,
patterns, and implementation constraints **Typical Questions**:

- What existing patterns should I follow for {functionality}?
- What are the current testing conventions for {component type}?
- What dependencies/modules are already available for {use case}?

#### failure_patterns

**Target**: implementer agent **Purpose**: Learn from previous implementation
attempts or common failure modes **Typical Questions**:

- What are common failure patterns for {implementation approach}?
- What edge cases should be considered for {functionality}?
- What defensive programming practices are used in this codebase?

#### type_requirements

**Target**: type-architect agent **Purpose**: Define domain types and
type-safety requirements **Typical Questions**:

- What domain types need to be created for {feature}?
- How should {concept} be represented in the type system?
- What validation rules should apply to {data type}?

#### test_scenarios

**Target**: test-hardener agent **Purpose**: Identify comprehensive testing
approaches and edge cases **Typical Questions**:

- What test scenarios should cover {functionality}?
- What property-based tests would be valuable for {domain type}?
- What integration test fixtures are needed for {feature}?

#### cross_cutting_analysis

**Target**: expert agent **Purpose**: High-level architecture and cross-cutting
concern analysis **Typical Questions**:

- How does {proposed change} affect the overall architecture?
- What are the security implications of {approach}?
- What performance considerations apply to {implementation}?

## Request/Response Lifecycle

### 1. Request Initiation

- Requesting agent creates InfoRequest using standard format
- Request is logged/tracked for completion
- Target agent is notified of pending request

### 2. Response Processing

- Target agent acknowledges request receipt
- Agent performs research/analysis within their domain
- Response is formatted using InfoResponse template

### 3. Information Integration

- Requesting agent receives and processes response
- Information is integrated into agent's work products
- Follow-up requests initiated if needed

### 4. Quality Assurance

- Both agents verify information accuracy and completeness
- Any gaps or uncertainties are clearly documented
- Sources are properly attributed

## Scope Boundaries and Access Policies

### Information Access by Agent Type

#### researcher

- **Allowed**: External web searches, documentation, API references, standards
- **Restricted**: Internal codebase analysis, implementation details
- **Special Access**: Web search tools, documentation databases

#### planner

- **Allowed**: High-level architecture, requirements analysis, design patterns
- **Restricted**: Low-level implementation details, specific test scenarios
- **Dependencies**: Requires researcher findings for external context

#### implementer

- **Allowed**: Code reading/writing, testing, build tools, file system
- **Restricted**: External research, architectural decisions
- **Dependencies**: Requires approved plans, type requirements

#### type-architect

- **Allowed**: Type system design, domain modeling, validation rules
- **Restricted**: Implementation details, test specifics, external research
- **Dependencies**: Requires domain understanding from other agents

#### test-hardener

- **Allowed**: Test design, property testing, fixture creation
- **Restricted**: Production code implementation, architectural changes
- **Dependencies**: Requires implementation context from implementer

#### expert

- **Allowed**: Code analysis, architecture review, best practice guidance
- **Restricted**: Active implementation, external research
- **Special Role**: Cross-cutting analysis and quality assessment

#### pr-manager

- **Allowed**: Git operations, GitHub API, branch management
- **Restricted**: Code implementation, research, testing
- **Special Role**: Process orchestration and external communication

### Information Flow Policies

1. **Single Source of Truth**: Each agent type is the authoritative source for
   their domain
2. **Explicit Handoffs**: Information must be explicitly requested and confirmed
   received
3. **Traceable Decisions**: All significant information exchanges must be
   documented
4. **Quality Gates**: Information quality must be assessed before integration
5. **Scope Enforcement**: Agents must stay within their defined access
   boundaries

### Emergency Protocols

In cases where normal information flow is blocked:

1. **Escalation Path**: Route through expert agent for cross-cutting analysis
2. **Bypass Conditions**: Only when normal flow would cause project failure
3. **Documentation Requirements**: All bypasses must be logged with
   justification
4. **Recovery Actions**: Return to normal flow as soon as possible

## Usage Examples

### Example 1: Researcher → Planner Information Flow

**InfoRequest**:

```markdown
## InfoRequest

**Requesting Agent**: planner **Target Agent**: researcher **Request Type**:
external_docs **Priority**: high

### Context Needed

Planning implementation of WebAssembly agent sandboxing and need current best
practices for security boundaries and resource limits.

### Specific Questions

1. What are the current industry standards for WASM sandboxing?
2. What are typical CPU and memory limits used in production WASM environments?
3. What security vulnerabilities should we be aware of in WASM runtimes?

### Expected Deliverables

- [ ] Summary of WASM sandboxing best practices
- [ ] Recommended resource limit ranges
- [ ] Security checklist for WASM runtime setup

### Scope Boundaries

- **In Scope**: WASM security, resource management, production deployment
  patterns
- **Out of Scope**: Specific implementation details, testing strategies
```

### Example 2: Type-Architect → Implementer Information Flow

**InfoRequest**:

```markdown
## InfoRequest

**Requesting Agent**: type-architect **Target Agent**: implementer **Request
Type**: implementation_context **Priority**: medium

### Context Needed

Designing domain types for agent resource limits and need to understand current
patterns and constraints in the codebase.

### Specific Questions

1. What validation patterns are currently used for resource limits?
2. How are errors currently handled for constraint violations?
3. What nutype patterns are established in the domain types module?

### Expected Deliverables

- [ ] Current validation patterns summary
- [ ] Error handling conventions
- [ ] Nutype usage examples from codebase

### Scope Boundaries

- **In Scope**: Domain type patterns, validation approaches, error handling
- **Out of Scope**: Business logic implementation, test scenarios
```

## Template Customization

Agents may customize these templates for specific use cases while maintaining
the core structure:

1. **Required Fields**: Always include requesting agent, target agent, request
   type
2. **Standard Sections**: Keep Context, Questions, Deliverables, Scope sections
3. **Agent-Specific Additions**: Add relevant fields for specific agent domains
4. **Consistent Format**: Maintain markdown formatting and structure for tool
   parsing

## Integration with SPARC Workflow

This information request protocol integrates with the SPARC phases:

- **Research Phase**: Heavy use of external_docs requests to researcher
- **Planning Phase**: Integration of research findings and type_requirements
  requests
- **Implementation Phase**: implementation_context and failure_patterns requests
- **Review Phase**: cross_cutting_analysis requests to expert for quality
  assessment

The protocol ensures information flows efficiently between agents while
maintaining clear boundaries and accountability for information quality.
