---
description: Implement behavior through an explicit RED-GREEN-REFACTOR cycle.
agent: phoenix-commanded-implementer
---

Use the `outside-in-tdd`, `outside-in-rgr-microcycle`, and `rgr-plan-structure` skills for: $ARGUMENTS

This command is a compatibility entry point for the specialist-agent RGR workflow. Do not perform code-writing steps directly when the RED/GREEN/review agents can own the step.

Workflow:

1. Identify the smallest failing test for the requested behavior.
2. Dispatch `rgr-test-author` to write or activate that test and run the focused command.
3. Dispatch `rgr-test-reviewer` to approve RED before any production edit.
4. Record RED with the RGR ledger tool, including command and real output.
5. Dispatch `rgr-diagnostic-implementer` to make exactly one minimum production edit for the current diagnostic.
6. Run the focused test and record GREEN when it passes.
7. Dispatch `rgr-implementation-reviewer` to approve the GREEN diff.
8. Refactor only with tests green and reviewer-approved, then record REFACTOR.
9. Run the strongest relevant verification gate feasible before handoff.
