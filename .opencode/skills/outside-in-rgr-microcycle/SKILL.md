---
name: outside-in-rgr-microcycle
description: Fine-grained outside-in RED-GREEN-REFACTOR microcycles with specialist agents and single-diagnostic implementation.
---

# Outside-In RGR Microcycle

Use this skill when `/outside-in-rgr` is orchestrating one behavior through specialist-agent handoffs.

## Ledger

Keep a visible ledger for each active cycle:

```text
Goal:
Current test:
Focused command:
Observed failure:
Expected failure reason:
Reviewer decision:
Diagnostic under treatment:
Allowed immediate change:
Result:
Next control owner:
```

## RED

RED is valid when a focused command was run and produced an observed failure that is expected for the requested behavior. Compile errors count as RED when the test intentionally pressures a missing module, context API, command, event, schema, or boundary.

Fix test misuse before production edits. Do not treat accidental misuse of existing code as implementation pressure.

## Test Review

Send every new or activated RED test to `rgr-test-reviewer` before production edits. A reviewer veto blocks implementation until the test author addresses the mandatory notes and records a new RED.

## Single Diagnostic

The implementer may treat exactly one current diagnostic at a time. The allowed production edit is the smallest concrete change that removes or changes that diagnostic. Do not predict later errors, prebuild adjacent behavior, refactor opportunistically, or batch fixes.

## Ambiguous Failure Escape Hatch

If the current diagnostic does not identify one concrete code change, write or request a lower-level unit test that exposes the next decision point. That lower-level test must go through RED and test review before production edits.

## GREEN

GREEN means the focused command for the current test passes after the smallest demanded implementation change. When the observed failure changes but the test still fails, stop the implementer turn and return control to the orchestrator with the new diagnostic.

## Implementation Review

After GREEN, send the production diff to `rgr-implementation-reviewer`. A reviewer veto blocks refactor, broader verification, and handoff until the implementer addresses the mandatory notes with focused tests still green.

## REFACTOR

Refactor only after GREEN and implementation review approval. Refactors must preserve behavior and keep the focused command green. Avoid abstractions not demanded by the current behavior.

## Control Transfer

Return control to the orchestrator whenever a test is authored, a reviewer approves or vetoes, a diagnostic changes, a focused test passes, the same failure remains after an attempted edit, or a blocked state needs a decision.

## Stop Conditions

Stop the active microcycle when all current cycle tests pass, reviewer vetoes are resolved, focused verification passes, and the ledger identifies the next handoff. Do not commit unless the user explicitly requests it.

## Blocked States

Report a blocked state when no focused command can be run, the failure output is unavailable, the diagnostic is ambiguous and no lower-level test seam is apparent, or required changes would touch unrelated user work.

## Verification

Run the narrow focused test first. Before handoff, run the strongest relevant gate feasible for the files changed and state any skipped gate with the reason.
