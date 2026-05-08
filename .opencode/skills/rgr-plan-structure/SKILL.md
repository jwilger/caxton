---
name: rgr-plan-structure
description: Write implementation plans as test-addressed red-green-refactor cycles rather than component waterfalls.
---

# RGR Plan Structure

Use this skill before writing plans, todo lists, PR checklists, or session outlines for behavior work.

## Good Plans

RGR-shaped plans name the failing test that justifies each production edit. They keep at most one active cycle in progress and avoid speculative downstream component tasks.

## Bad Plans

Waterfall plans list components in construction order: models, events, handlers, persistence, UI, then tests. If a task cannot name the failing test it addresses, it is speculative.

## Template

Cycle 1:

1. RED: add or activate `<test name>` for `<observable behavior>` and run `<command>`.
2. GREEN: make the minimum production edit to pass that failure.
3. REFACTOR: improve only after `<command>` is green.

Repeat only after the previous cycle is green.
