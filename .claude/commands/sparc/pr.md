---
description:
  Create draft PR for the completed story by delegating to pr-manager agent
argument-hint: [optional story context]
allowed-tools: Task
---

# Create Pull Request

Delegate to the pr-manager subagent to create a draft pull request for the
completed story.

## Delegation Process

1. Use Task tool to invoke pr-manager agent
2. Pass story context and any additional requirements
3. Let pr-manager handle all git and GitHub operations:
   - Verify story implementation is complete
   - Check current branch is a feature branch (not main)
   - Ensure all tests pass and code is formatted
   - Create draft PR with proper title and description

## PR Description Template

The PR manager should create descriptions following this format:

```markdown
## Story Context

{Story description and acceptance criteria}

## Implementation Summary

- {List of key changes}
- {Files modified}
- {New domain types introduced}

## Testing

- {Tests added/modified}
- {How to verify the changes}

## Notes

{Any additional context for reviewers}

---

_This PR was created by Claude Code as part of the SPARC workflow. All commits
follow strict TDD discipline with Red→Green→Refactor cycles._
```

## Delegation Notes

This command acts as a pure orchestrator:

- NEVER perform git or GitHub operations directly
- NEVER read code files or run commands
- ONLY use Task tool to delegate to pr-manager agent
- Pass along any story context provided by user

The pr-manager agent will handle all safety checks and PR creation operations.
