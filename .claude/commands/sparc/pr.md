---
description:
  Create draft PR for the completed story by delegating to pr-manager agent
argument-hint: [optional story context]
allowed-tools: Task
---

# Create Pull Request

Delegate to the pr-manager subagent to create a draft pull request for the
completed story.

## SPARC Coordinator Role

**CRITICAL: The SPARC coordinator (you) is STRICTLY an orchestrator. You MUST
NOT:**

- Perform any git or GitHub operations directly
- Write PR descriptions yourself
- Read code files or run commands
- Make decisions about PR content

**Your ONLY job is to:**

1. **Delegate ALL work** to the pr-manager subagent using the Task tool
2. **Relay PR creation results** back to the user
3. **ENFORCE MEMORY STORAGE** - Verify pr-manager stores workflow patterns
4. **Handle information requests** if pr-manager needs context

## Delegation Process

1. Use Task tool to invoke pr-manager agent
2. Pass story context and any additional requirements
3. Let pr-manager handle all git and GitHub operations:
   - Verify story implementation is complete
   - Check current branch is a feature branch (not main)
   - Ensure all tests pass and code is formatted
   - **Update PLANNING.md** to mark story as completed (MANDATORY)
   - Create draft PR with proper title and description

## PR Description Template

**Keep descriptions concise. Focus on what changed and why.**

```markdown
{One-line story summary}

**Changes:**
- {Brief list of main changes, 3-5 bullets max}

**Testing:** {One line about test coverage}

{Optional: Any critical context for reviewers, 1-2 lines max}
```

**NO self-congratulatory language. NO lengthy explanations. Facts only.**

## Memory Storage Requirements

**MANDATORY**: The pr-manager agent MUST:

- **Search existing patterns**: Use `mcp__qdrant__qdrant-find` for PR workflows
- **Store workflow patterns**: Use `mcp__qdrant__qdrant-store` for:
  - PR strategies and templates
  - Successful PR descriptions
  - Branch management patterns
  - GitHub API interactions
  - Story completion workflows

**ENFORCEMENT**: If pr-manager fails to store knowledge, immediately request they
do so before proceeding.

## PLANNING.md Update Requirement

**CRITICAL**: The pr-manager MUST update PLANNING.md as part of story completion:

1. **Locate story in PLANNING.md** by story ID
2. **Mark checkbox as completed**: Change `- [ ]` to `- [x]`
3. **Add completion status**: e.g., "✅ (COMPLETED - All acceptance criteria met)"
4. **Commit update in same PR** - PLANNING.md update must be included
5. **Verify story ID matches** current branch before marking complete

Example update:

```diff
- [ ] Story 052: Dependency Vulnerability Resolution
+ [x] Story 052: Dependency Vulnerability Resolution ✅ (COMPLETED)
```

## Information Request Handling

If pr-manager needs additional information, they may include an "Information
Requests" section. The coordinator MUST:

1. **Parse requests** from pr-manager's response
2. **Route to appropriate agents** (implementer for code, planner for context)
3. **Relay responses** back to pr-manager
4. **Track request chains** to prevent loops
5. **Never provide information directly**

## Critical Rules

- PR created in **draft status only** - humans control ready-for-review
- **NEVER modify PR back to draft** once human marks ready-for-review
- All PRs include Claude Code attribution in commits
- **MANDATORY PLANNING.md UPDATE** when completing story
- **MANDATORY MEMORY STORAGE** - pr-manager must store patterns
- Treat clippy warnings as errors
- **NEVER** add clippy allow attributes

## Delegation Notes

This command acts as a pure orchestrator:

- NEVER perform git or GitHub operations directly
- NEVER read code files or run commands
- ONLY use Task tool to delegate to pr-manager agent
- Pass along any story context provided by user
- Verify memory storage and PLANNING.md update

The pr-manager agent will handle all safety checks and PR creation operations.
