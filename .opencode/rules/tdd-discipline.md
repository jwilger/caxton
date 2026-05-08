# TDD Discipline

Production Elixir/Phoenix/Commanded behavior requires observed failing ExUnit evidence first. Follow RED -> GREEN -> REFACTOR with the specialist RGR agents, record the failing output in the RGR ledger, and make the smallest production edit that changes the observed failure.

Default code-writing handoff: use `rgr-test-author` to write or activate RED, `rgr-test-reviewer` to approve RED before production edits, `rgr-diagnostic-implementer` to make each smallest GREEN production edit, and `rgr-implementation-reviewer` to approve GREEN before refactor or broader verification. The primary implementer orchestrates and keeps the ledger; it should not directly author behavior tests or production Elixir/Phoenix/Commanded code when a specialist agent can do that step.

Exemptions are narrow: docs-only work, pure moves or renames, generated lockfile churn, and non-behavioral chores. If a test is hard to write, extract a testable seam instead of skipping RED.
