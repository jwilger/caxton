---
name: review-taxonomy
description: Classify review feedback and choose proportionate guardrails without inflating default context cost.
---

# Review Taxonomy

Use this skill when processing PR feedback or deciding whether to add a new rule, skill, command, or plugin behavior.

## Categories

- Missing test pressure.
- Component-waterfall planning.
- Forgejo/GitHub workflow confusion.
- Toolchain bypass.
- Scope hygiene failure.
- Security or threat-model blind spot.
- Documentation or operator-facing drift.
- Knowing guardrail bypass.

## Classification

`guardrail-gap` means the failure is systemic and the added control is worth its ongoing context and maintenance cost. `one-off` means fix the immediate issue without adding always-loaded guidance.

Knowing bypass of an existing guardrail is prima facie `guardrail-gap` because the existing control failed.

## Cost Gate

Prefer the smallest durable control: plugin enforcement for hard safety rules, a short always-loaded rule for frequent guardrails, a skill for longer procedures, a slash command for repeatable workflows, and no new guardrail for isolated mistakes.
