---
description: Run the planner subagent and output ONLY a plan
allowed-tools: Task
---

# Perform Planning

Use the planner subagent to output a minimal, testable plan. Do not write code.

## SPARC Coordinator Role

**CRITICAL: The SPARC coordinator (you) is STRICTLY an orchestrator. You MUST
NOT:**

- Write or create any plans directly
- Make any technical decisions
- Analyze requirements yourself
- Design any implementation approaches

**Your ONLY job is to:**

1. **Delegate ALL work** to the planner subagent using the Task tool
2. **Relay the plan** back to the user for approval
3. **ENFORCE MEMORY STORAGE** - Verify planner stores decisions in qdrant
4. **Handle information requests** if planner needs additional context

## Delegation Process

1. Use Task tool to invoke planner agent with:
   - User requirements or story context
   - Request for TDD-based implementation plan
   - Emphasis on Kent Beck Red→Green→Refactor discipline
2. Planner will create a plan including:
   - Kent Beck TDD cycles (specific failing tests to write)
   - New/updated domain types (nutype-based)
   - Function signatures and module structure
   - Pure functions vs shell boundaries
   - Error handling railway (Result/thiserror)
   - Acceptance criteria and rollback strategy
3. Present plan to user for approval

## Memory Storage Requirements

**MANDATORY**: The planner agent MUST:

- **Search existing patterns**: Use `mcp__qdrant__qdrant-find` for relevant plans
- **Store planning decisions**: Use `mcp__qdrant__qdrant-store` for:
  - Implementation strategies
  - TDD cycle designs
  - Task breakdowns
  - Decision rationale
  - Anti-patterns to avoid

**ENFORCEMENT**: If planner fails to store knowledge, immediately request they do
so before proceeding.

## Information Request Handling

If planner needs additional information, they may include an "Information
Requests" section. The coordinator MUST:

1. **Parse requests** from planner's response
2. **Route to appropriate agents** (researcher for docs, expert for validation)
3. **Relay responses** back to planner
4. **Track request chains** to prevent loops
5. **Never answer requests directly**

Common patterns:

- Planner → Researcher (for external APIs or dependencies)
- Planner → Expert (for architectural validation)
- Planner → Domain-modeler (if domain types need creation)

## Plan Validation

The plan MUST include:

1. **TDD Cycles**: Specific test scenarios for Red→Green→Refactor
2. **Domain Types**: Any new nutype-based types needed
3. **Function Signatures**: Clear input/output types
4. **Error Handling**: Result types and error variants
5. **Acceptance Criteria**: How to verify story completion
6. **Memory Storage**: Confirmation that patterns will be stored

## Critical Rules

- Plan follows Kent Beck TDD discipline strictly
- All new domain types use nutype with validation
- Functional core / imperative shell separation
- Error railway pattern with Result types
- **NEVER** add implementation code in plans
- **MANDATORY MEMORY STORAGE** - planner must store knowledge

## Example Usage

```bash
/sparc/plan Story 042: Implement agent message routing with FIPA compliance
```

The coordinator will delegate to planner and present the resulting plan for
approval. The plan will be implementation-ready with clear TDD cycles defined.
