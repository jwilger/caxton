---
description:
  Orchestrate SPARC for the next unfinished story in @PLANNING.md (or the
  specified one) using strict Rust TDD + type-driven design.
argument-hint: [optional-instructions or explicit story]
model: claude-opus-4-1-20250805
allowed-tools: Task, Bash, BashOutput
---

<!-- cSpell:ignore nextest clippy proptest nutype thiserror wasmtime newtypes nocapture -->

# SPARC Orchestration (Rust)

ULTRATHINK

Execute the complete SPARC workflow directly, using specialized agents for each
phase.

## Context

- User input: **$ARGUMENTS**
- Planning file: PLANNING.md
- Git status: !`git status --porcelain`
- Current branch: !`git branch --show-current`

## Pre-Workflow Setup

**CRITICAL FIRST STEP**: Before delegating to any agents, the SPARC coordinator
MUST:

1. **Set Cargo Working Directory**: Call `mcp__cargo__set_working_directory`
   with the absolute path to the project root (where Cargo.toml exists)
2. **Start Bacon Continuous Testing**: Launch `bacon --headless` in background
   for real-time test monitoring
3. **Verify Setup**: Ensure the working directory is set correctly and bacon is
   running for all subsequent operations

This ensures all agents have proper access to cargo commands and continuous test
feedback without manual test execution.

### Bacon Integration Throughout Workflow

**MANDATORY**: All agents must monitor bacon output instead of running manual
test commands:

- **Use BashOutput tool** to check bacon status when tests are expected to
  change
- **Look for expected failures** during RED phase - bacon should show the
  failing test
- **Confirm test passes** during GREEN phase - bacon should show all tests
  passing
- **Verify no regressions** during REFACTOR phase - bacon should maintain green
  status
- **React immediately** to any unexpected compilation errors or test failures in
  bacon output

## SPARC Coordinator Role

**CRITICAL: The SPARC coordinator (you) is STRICTLY an orchestrator. You MUST
NOT:**

- Write or read any code directly
- Perform any research or web searches
- Create or modify any plans
- Run any commands or tests
- Make any implementation decisions
- Analyze code or requirements

**Your ONLY job is to:**

1. **Delegate ALL work** to specialized subagents using the Task tool
2. **Relay information** between subagents and present results to the user
3. **Route information requests** between agents as needed
4. **Track workflow state** and enforce correct SPARC phase ordering
5. **Interface with human** for approvals and decisions
6. **ENFORCE MEMORY STORAGE** - Verify each agent stored knowledge before
   proceeding

## SPARC Workflow

Execute each phase using specialized agents:

### A) STORY SELECTION

Use Task tool with `planner` agent:

1. Read PLANNING.md to identify available stories
2. If $ARGUMENTS specifies a story, select it; otherwise pick next unfinished
   (`- [ ]`)
3. Return chosen story text and ID for coordinator to present to user

### A.5) BRANCH SETUP

Use Task tool with `pr-manager` agent:

- Create feature branch: `story-{zero-padded-id}-{kebab-case-slug}`
- Switch to feature branch
- Record story/branch mapping in `.claude/branch.info`

### B) RESEARCH

Use Task tool with `researcher` agent:

- Research external dependencies (Rust tools: cargo MCP server, clippy,
  proptest, nutype)
- Gather authoritative sources and documentation
- Return "Research Brief" with assumptions, key facts, and open questions

### C) PLAN

Use Task tool with `planner` agent:

- Plan Kent Beck Red→Green→Refactor loop (one failing test)
- List new/updated domain types (nutype) and function signatures
- Specify pure functions vs shell boundaries and error railway
  (Result/thiserror)
- Define acceptance checks and rollback plan

### D) APPROVAL GATE

**Coordinator responsibilities:**

- Present plan from planner agent to user for approval
- Collect user approval/feedback
- If approved, delegate to `pr-manager` agent to write `.claude/plan.approved`
  with plan content
- Block further progress until approved

### E) IMPLEMENT (TDD Cycle)

**RED Phase** - Use Task tool with `red-implementer` agent:

- Write exactly ONE failing test that captures the next behavior
- Use `unimplemented!()` to force clear failure
- **Monitor bacon output** to verify test fails for the right reason (use
  BashOutput tool)
- Create `.claude/tdd.red` state file
- Store test patterns in MCP memory
- **COORDINATOR VALIDATION**: Verify response contains ONLY test code, no
  implementation
- **BACON VERIFICATION**: Confirm bacon shows the expected test failure

**GREEN Phase** - Use Task tool with `green-implementer` agent:

- Implement minimal code to make the failing test pass
- Use simplest possible solution (fake it 'til you make it)
- **Monitor bacon output** to verify test passes and no existing tests break
  (use BashOutput tool)
- Create `.claude/tdd.green` state file
- Store minimal implementation patterns in MCP memory
- **COORDINATOR VALIDATION**: Verify response contains ONLY implementation code,
  no tests
- **BACON VERIFICATION**: Confirm bacon shows all tests passing

**REFACTOR Phase** - Use Task tool with `refactor-implementer` agent:

- Remove duplication and improve code structure
- Extract pure functions (functional core / imperative shell)
- **Monitor bacon output** to ensure all tests stay green throughout refactoring
  (use BashOutput tool)
- Use cargo MCP server for `cargo_clippy` and `cargo_fmt_check`
- Store refactoring patterns in MCP memory
- **COORDINATOR VALIDATION**: Verify response contains ONLY implementation
  improvements, no test changes
- **BACON VERIFICATION**: Confirm bacon shows no test regressions during
  refactoring
- **COMMIT**: Create descriptive commit with Claude Code attribution

**TYPE PASS**: Use Task tool with `type-architect` to replace primitives with
nutype domain types

### F) TEST-HARDENING

Use Task tool with `test-hardener` agent:

- For each test added/changed, propose type/API changes that make failures
  impossible at compile time
- If safe, implement type changes with small diffs
- Update call sites and **monitor bacon output** to verify no test regressions
  (use BashOutput tool)
- Use cargo MCP server for `cargo_clippy` only - bacon handles continuous test
  monitoring

### G) EXPERT CHECK (Optional)

Use Task tool with `expert` agent:

- Request brief review on type-state soundness and error pipeline
- Get validation of implementation approach

### H) PR CREATION

Use Task tool with `pr-manager` agent:

- Create draft PR with comprehensive description
- Link story acceptance criteria and implementation summary
- Update `.claude/branch.info` with PR number
- Never mark PR ready-for-review (human only)

### I) PR REVIEW LOOP (if feedback exists)

Use Task tool with `pr-manager` agent:

- Monitor for PR comments via GitHub MCP server tools
- Respond with Claude Code attribution
- Address requested changes using TDD discipline
- Create follow-up commits as needed

### J) COMPLETION

Use Task tool with `pr-manager` agent:

- **MANDATORY PLANNING.md UPDATE**: Update PLANNING.md to mark the story as
  completed with `[x]` checkbox and completion status
- Commit the PLANNING.md update as part of the story completion (must be
  included in the same PR)
- Remove `.claude/plan.approved` file
- Generate summary of files changed and commits made
- Ensure PR remains in draft status for human review and merge

## Completion Summary

The coordinator presents the final summary to the user.

### Memory Storage Verification (UUID-Based Protocol)

**MANDATORY FINAL STEP**: Verify all agents used correct UUID-based memory
protocol per `.claude/MEMORY_VERIFICATION_CHECKLIST.md`:

1. **UUID Generation**: Confirm each storage operation started with
   `mcp__uuid__generateUuid`
2. **Qdrant Format**: Check all memories include `[UUID: xxx]` tag at END
3. **Graph Nodes**: Verify sparc-memory entities use UUID as `name` field
4. **Relations**: Confirm all relations link UUID to UUID, not descriptive names
5. **Search Pattern**: Validate agents extracted UUIDs and used `open_nodes`,
   not semantic search

**REFERENCE**: See `.claude/MEMORY_VERIFICATION_CHECKLIST.md` for detailed
validation criteria and examples

**ENFORCEMENT**: Agents violating UUID protocol must redo their memory
operations correctly 6. Verify expert analysis was preserved 7. Ensure PR
workflow patterns were captured

**If any agent failed to store knowledge, request immediate remediation before
marking story complete.**

## TDD Role Validation Protocol (CRITICAL)

**COORDINATOR MUST VALIDATE** every TDD agent response before acceptance:

### Red-Implementer Validation

- ✅ **Required**: Response begins with "I am red-implementer. I write ONLY
  tests. I do NOT write implementation code."
- ✅ **Required**: Contains test code blocks with
  `// Test that verifies [specific behavior]`
- ✅ **Required**: Ends with "Test written and failing. Ready for
  green-implementer."
- ✅ **Required**: Includes "**ROLE COMPLIANCE**: I have verified this response
  contains only test code and no implementation code."
- ❌ **Forbidden**: ANY implementation code blocks
- ❌ **Forbidden**: Modifications to existing implementation code

### Green-Implementer Validation

- ✅ **Required**: Response begins with "I am green-implementer. I write ONLY
  implementation code. I do NOT write tests."
- ✅ **Required**: Contains implementation code blocks with
  `// Minimal implementation to pass test`
- ✅ **Required**: Ends with "Test now passes. Ready for refactor-implementer or
  next red cycle."
- ✅ **Required**: Includes "**ROLE COMPLIANCE**: I have verified this response
  contains only implementation code and no test code."
- ❌ **Forbidden**: ANY test code blocks
- ❌ **Forbidden**: Modifications to existing test code

### Refactor-Implementer Validation

- ✅ **Required**: Response begins with "I am refactor-implementer. I improve
  ONLY implementation code. I do NOT modify tests."
- ✅ **Required**: Contains implementation improvements only
- ✅ **Required**: Ends with "Code improved. All tests still green. Ready for
  next red cycle."
- ✅ **Required**: Includes "**ROLE COMPLIANCE**: I have verified this response
  contains only implementation improvements and no test modifications."
- ❌ **Forbidden**: ANY test modifications
- ❌ **Forbidden**: Changes to test behavior or expectations

### Violation Response Protocol

**If validation fails:**

1. **Immediately re-delegate** to the same agent with role reminder
2. **Include validation error** in the re-delegation prompt
3. **Do NOT proceed** with workflow until validation passes
4. **Escalate to human** if violations persist after 2 attempts

## Information Request Routing Protocol

### Information Request Processing

During SPARC workflow execution, agents may include "Information Requests"
sections in their responses. The coordinator MUST handle these by:

1. **Parse Information Requests**: Look for sections labeled "## Information
   Requests" or "### Information Requests" in agent responses
2. **Route to Target Agent**: Use Task tool to delegate each request to the
   appropriate agent
3. **Track Requests**: Maintain request tracking to prevent infinite loops and
   cycles
4. **Relay Responses**: Collect responses and provide them back to the
   requesting agent

### Request Format Recognition

Agents will format information requests as:

```markdown
## Information Requests

### Request 1: [Brief Description]

- **Target Agent**: [agent-name]
- **Request**: [specific information needed]
- **Context**: [why this information is needed]

### Request 2: [Brief Description]

- **Target Agent**: [agent-name]
- **Request**: [specific information needed]
- **Context**: [why this information is needed]
```

### Routing Logic

**Agent Specializations:**

- `researcher` → External docs, APIs, dependencies, best practices
- `planner` → Architecture decisions, implementation strategies, TDD plans
- `red-implementer` → Test writing, behavior capture, failure specification
- `green-implementer` → Minimal implementations, test satisfaction, simple
  solutions
- `refactor-implementer` → Code improvement, FCIS architecture, duplication
  removal
- `type-architect` → Domain types, type safety, API design
- `test-hardener` → Test coverage, invariants, property-based tests
- `expert` → Code review, soundness validation, optimization
- `pr-manager` → Git operations, branch status, PR management

### Response Integration Workflow

1. **Collect Information Request**:

   ```text
   Agent X provides response with Information Requests section
   ```

2. **Route Each Request**:

   ```text
   For each request in the section:
   - Use Task tool with target agent
   - Provide request context and specific question
   - Collect response from target agent
   ```

3. **Relay Response Back**:

   ```text
   Continue with Agent X, providing:
   - Original request context
   - Information responses from target agents
   - Any additional context needed
   ```

### Loop Prevention

**Request Tracking Rules:**

- Maintain request chain: `Agent A → Agent B → Agent C`
- Prevent cycles: Never allow `Agent A → Agent B → Agent A`
- Maximum depth: Limit request chains to 3 levels deep
- Timeout handling: If request chain exceeds reasonable time, escalate to human

### Common Information Exchange Patterns

### Pattern 1: Implementer → Researcher

```text
implementer needs external API documentation
→ coordinator routes to researcher
→ researcher provides API docs and examples
→ coordinator relays back to implementer
→ implementer continues with implementation
```

### Pattern 2: Type-Architect → Expert

```text
type-architect proposes complex type design
→ coordinator routes soundness question to expert
→ expert validates/suggests improvements
→ coordinator relays feedback to type-architect
→ type-architect refines design
```

### Pattern 3: Test-Hardener → Refactor-Implementer

```text
test-hardener needs existing code patterns
→ coordinator routes to refactor-implementer for code analysis
→ refactor-implementer provides pattern analysis and structure insights
→ coordinator relays to test-hardener
→ test-hardener proposes type improvements
```

### Request Context Preservation

**Context to Maintain:**

- Original SPARC phase being executed
- Story context and requirements
- Previous agent responses and decisions
- Request chain history to prevent loops
- Timeout tracking for each request

**Context to Relay:**

- Full original request context
- Previous related information exchanges
- Current SPARC workflow state
- Any constraints or requirements from story

### Enhanced Orchestrator Examples

### Example 1: Information Request During Implementation

```markdown
implementer response includes:

## Information Requests

### Request 1: WASM Runtime API Documentation

- **Target Agent**: researcher
- **Request**: Find official documentation for wasmtime crate's async execution
  APIs
- **Context**: Need to implement async WASM module execution for agent runtime

Coordinator action:

1. Use Task tool with researcher agent
2. Provide context: "implementer needs wasmtime async execution docs for
   story-051"
3. Collect researcher response with documentation links and examples
4. Continue with implementer, providing researcher's findings
```

### Example 2: Cross-Phase Information Exchange

```markdown
type-architect response includes:

## Information Requests

### Request 1: Validation of Resource Limit Types

- **Target Agent**: expert
- **Request**: Review proposed CpuFuel and MemoryBytes newtypes for soundness
- **Context**: Ensuring resource limits are mathematically sound and prevent
  overflow

Coordinator action:

1. Use Task tool with expert agent
2. Provide type-architect's proposed designs
3. Collect expert's soundness validation and suggestions
4. Continue with type-architect, providing expert feedback
```

### Request Timeout and Escalation

**Timeout Rules:**

- Single request timeout: 2 minutes
- Request chain timeout: 5 minutes total
- If timeout exceeded, escalate to human with context

**Escalation Format:**

```markdown
## Information Request Timeout

**Chain**: implementer → researcher → expert **Duration**: 5 minutes 30 seconds
**Last Request**: [description] **Status**: Awaiting response from expert agent

**Action Needed**: Human intervention to resolve or continue workflow
```

## Information Request Handling Rules

**Coordinator Information Processing Rules:**

- **NEVER analyze** the content of information requests - only route them
- **NEVER research** or answer requests directly - always delegate to
  appropriate agent
- **NEVER modify** or interpret request content - relay exactly as provided
- **ALWAYS track** request chains to prevent infinite loops
- **ALWAYS preserve** full context when routing between agents
- **ESCALATE** to human if request chains become complex or timeout

**Request Processing Steps:**

1. **Detect**: Scan agent response for "Information Requests" sections
2. **Parse**: Extract target agent, request, and context for each request
3. **Route**: Use Task tool with target agent, providing full context
4. **Collect**: Gather response from target agent without analysis
5. **Relay**: Continue with requesting agent, providing target agent's response
6. **Track**: Log request chain to prevent cycles

**Multi-Agent Coordination Principles:**

- Coordinator is pure information router, never content creator
- Each agent maintains their specialized domain expertise
- Information flows through coordinator but is never modified
- Request chains enable collaborative problem-solving between agents
- Human escalation ensures complex coordination doesn't block progress

## Critical Rules

- Follow Kent Beck TDD discipline strictly: Red→Green→Refactor
- **Use bacon for continuous testing** instead of manual test commands - monitor
  BashOutput for test feedback
- Treat clippy warnings as errors (`-- -D warnings`)
- **NEVER** add clippy allow attributes without explicit team approval
- All new domain types must use nutype with sanitize/validate
- Maintain functional core / imperative shell boundaries
- Use Result/thiserror for error handling railway
- All commits include Claude Code attribution
- **MANDATORY MEMORY STORAGE**: Every agent MUST store knowledge after
  significant actions

## Memory Storage Enforcement

**CRITICAL**: After each agent completes a phase, you MUST verify they stored
knowledge:

- **Researcher**: Must store research findings, sources, and patterns
- **Planner**: Must store planning decisions, strategies, and rationale
- **Red-Implementer**: Must store test patterns, behavior capture techniques,
  and failure strategies
- **Green-Implementer**: Must store minimal implementation patterns, solution
  strategies, and test satisfaction approaches
- **Refactor-Implementer**: Must store refactoring patterns, FCIS improvements,
  and code quality enhancements
- **Type-Architect**: Must store domain type designs, validation patterns, and
  decisions
- **Test-Hardener**: Must store test improvements and type strengthening
  patterns
- **Expert**: Must store architectural insights, quality patterns, and reviews
- **PR-Manager**: Must store workflow patterns, PR strategies, and outcomes

If an agent fails to store knowledge, **immediately request they do so before
continuing**.
