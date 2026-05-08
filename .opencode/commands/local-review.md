---
description: Review committed branch changes with architecture, test coverage, and security reviewers.
---

Review all committed changes on the current branch versus its base branch for: $ARGUMENTS

Dispatch read-only review subagents:

1. `architecture-reviewer` for Phoenix contexts, Commanded boundaries, Ecto/Postgres design, docs, env parsing, errors, and observability.
2. `test-coverage-reviewer` for RGR evidence and ExUnit/Phoenix/Commanded coverage.
3. `security-reviewer` for threat model, Phoenix auth/session/CSRF, command authorization, event data, secrets, and dependency risks.

Return findings first, ordered by severity, with file and line references where possible.
