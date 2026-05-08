---
name: phoenix-commanded-engineering
description: Phoenix 1.8, Postgres, Commanded, Ecto, ExUnit, Dialyzer, and strict static-check conventions.
---

# Phoenix Commanded Engineering

## Tooling

Use the project-pinned Nix dev shell for project commands: `nix develop --command bash -lc '<command>'`. Keep `MIX_ENV` scoped to the command being run.

## Phoenix And Ecto

- Keep Phoenix contexts as public application APIs and keep web modules thin.
- Use Ecto changesets for data validation at persistence boundaries.
- Use migrations, constraints, foreign keys, unique indexes, partial indexes, and check constraints when Postgres must enforce integrity.
- Keep read models query-friendly and explicitly owned by the context or projector that maintains them.
- Prefer explicit transactions where consistency depends on multiple database writes.

## Commanded

- Model write-side invariants in commands, aggregates, and events.
- Author command validation and authorization before dispatch where possible.
- Treat events as durable contracts. Prefer additive changes; version event shape changes and provide upcasters or compatibility handling for replay.
- Make handlers, projectors, and process managers idempotent. Account for retries, duplicate events, out-of-order operational recovery, and replay.
- Keep side effects at handler/process boundaries and avoid mixing projection updates with aggregate decisions.

## Elixir Conventions

- Prefer small functions, pattern matching, tagged tuples, and guard clauses.
- Use exceptions for framework-conventional exceptional failures, not expected domain outcomes.
- Avoid speculative behaviours, macros, and generic service layers.
- Add typespecs for public context functions, commands, events, behaviours, and structs when they improve Dialyzer signal and caller clarity.
- Do not casually suppress Dialyzer warnings; adjust code or types first.

## Tests

- Use focused ExUnit commands first, including `mix test path/to/file_test.exs:line` when practical.
- Use `DataCase` for Ecto/Postgres behavior, `ConnCase` for request behavior, LiveView tests for interactive UI, aggregate/command tests for write-side invariants, and projection tests for read-model updates.
- Exercise database constraints and migrations when behavior depends on Postgres.
- Keep factories and fixtures domain-specific rather than broad object builders.

## Static Gates

- Run `mix format --check-formatted` before handoff.
- Compile test and prod with warnings as errors: `MIX_ENV=test mix compile --warnings-as-errors --force` and `MIX_ENV=prod mix compile --warnings-as-errors --force`.
- Run `mix test --warnings-as-errors` for broad test coverage.
- Treat `mix dialyzer --halt-exit-status` as a required full gate once configured.
- Run configured strict lint/security gates such as `mix credo --strict` and `mix sobelow`.
