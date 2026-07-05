---
description: Select the next open GitHub issue before starting work.
agent: phoenix-commanded-implementer
---

Select the next available GitHub issue for: $ARGUMENTS

This command is the required entry point for vague requests such as "next issue", "continue the roadmap", "pick up the next task", or any issue-targeted work where the issue number is not explicitly supplied.

Workflow:

1. Confirm the working tree is clean with `git status --short --branch`. If it is not clean, stop and report the existing changes before switching branches.
2. Derive the GitHub owner/repo from the `origin` remote or use `jwilger/caxton`.
3. Fetch all open issues with `gh issue list --repo jwilger/caxton --state open --limit 100 --json number,title,body,labels`.
4. Cross-check any body-level `Depends on:` references against the referenced issue state with `gh issue view`.
5. Report the full available issue set. Prefer the lowest-numbered non-roadmap implementation issue unless the user explicitly asked for roadmap-management work; otherwise choose the lowest-numbered available issue.
6. Create and switch to a dedicated branch before edits, using `issue-<index>-<short-slug>`.
7. Proceed with the issue-specific workflow: docs/config-only issues make the scoped change and run available non-Mix checks; behavior changes load the RGR skills and follow RED-GREEN-REFACTOR with specialist agents.

Reference audit snippet:

```sh
gh issue list --repo jwilger/caxton --state open --limit 100 --json number,title,labels \
  --jq '.[] | select([.labels[].name] | index("roadmap") | not) | "#\(.number) \(.title)"'
```
