---
description: Create draft PR for the completed story using pr-manager agent
argument-hint: [optional story context]
allowed-tools: Bash(gh:*), Bash(git:*), Read, Write
---

# Create Pull Request

Use the pr-manager subagent to create a draft pull request for the completed story.

## Process

1. Verify story implementation is complete
2. Check current branch is a feature branch (not main)
3. Ensure all tests pass and code is formatted
4. Create draft PR with:
   - Title: `[Story {id}] {story-title}`
   - Comprehensive description with story context
   - Link to acceptance criteria
   - Summary of changes made

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
*This PR was created by Claude Code as part of the SPARC workflow. All commits follow strict TDD discipline with Red→Green→Refactor cycles.*
```

## Safety Checks
- Verify working on feature branch
- Confirm no pending changes
- Check GitHub authentication
- Ensure PR created in draft status only

Never mark PRs as ready-for-review - only humans should do that.
