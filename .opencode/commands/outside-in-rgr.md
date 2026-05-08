---
description: Run a fine-grained outside-in RGR workflow with specialist agents.
agent: phoenix-commanded-implementer
---

Run the specialist outside-in RGR workflow for: $ARGUMENTS

Use the `outside-in-rgr-microcycle` skill and keep a visible RGR ledger. The primary implementer orchestrates; the RED/GREEN/review agents own their steps. Do not skip from RED to broad implementation.

Workflow:

1. Dispatch `rgr-test-author` to write or activate the next smallest failing test and capture RED.
2. Dispatch `rgr-test-reviewer` before any production edit.
3. If the reviewer vetoes, return to `rgr-test-author` with the mandatory notes.
4. Dispatch `rgr-diagnostic-implementer` to treat exactly one current diagnostic.
5. If the diagnostic is ambiguous, require a lower-level unit test and route it through test review.
6. When the failure changes, stop implementation and return control to the orchestrator.
7. When the focused test passes, dispatch `rgr-implementation-reviewer` for the production diff.
8. If the reviewer vetoes, return to `rgr-diagnostic-implementer` with the mandatory notes.
9. Continue one diagnostic at a time until all current cycle tests pass.
10. Run focused verification before handoff and state any skipped broader gate.

Do not commit unless the user explicitly requests a commit.
