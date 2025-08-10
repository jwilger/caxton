---
description: Check current branch, PR status, and story progress using pr-manager agent
allowed-tools: Bash(gh:*), Bash(git:*), Read
---

# Check SPARC Status

Use the pr-manager subagent to check the current status of branch, PR, and story progress.

## Information Displayed

### Branch Status
- Current branch name
- Relation to main branch (ahead/behind)
- Uncommitted changes
- Story association (from `.claude/branch.info`)

### PR Status (if exists)
- PR number and title
- Current state (draft/ready/merged/closed)
- Review status and requested reviewers
- Recent comments and feedback
- CI/check status

### Story Progress
- Current story ID and title
- SPARC phase (research/plan/implement/expert/pr/review)
- TDD state (red/green/refactor)
- Plan approval status

### GitHub Authentication
- Authentication status with GitHub
- Repository access verification

## Example Output Format

```
📋 SPARC Status Report

🌿 Branch: story-001-wasm-runtime-foundation
   └─ 3 commits ahead of main
   └─ No uncommitted changes
   └─ Story: 001 - WASM Runtime Foundation

🔄 Pull Request: #42 [Story 001] WASM Runtime Foundation
   └─ Status: Draft
   └─ Reviews: Pending (no reviewers assigned)
   └─ Checks: ✅ All passing
   └─ Last updated: 2 hours ago

⚙️  SPARC Phase: Implementation Complete
   └─ Plan: Approved (.claude/plan.approved exists)
   └─ TDD: Green (.claude/tdd.green exists)
   └─ Ready for PR creation

🔐 GitHub: ✅ Authenticated as username
   └─ Repository: jwilger/caxton (write access)
```

Use this command to get oriented before starting work or checking progress.
