---
description: Run focused or full Phoenix/Elixir verification.
agent: phoenix-commanded-implementer
---

Verify the current work: $ARGUMENTS

Prefer focused checks first, then broader gates as needed. Run project commands through `nix develop --command bash -lc '<command>'`.

```sh
nix develop --command bash -lc 'mix test path/to/file_test.exs:line'
nix develop --command bash -lc 'mix format --check-formatted'
nix develop --command bash -lc 'MIX_ENV=test mix compile --warnings-as-errors --force'
nix develop --command bash -lc 'MIX_ENV=prod mix compile --warnings-as-errors --force'
nix develop --command bash -lc 'mix test --warnings-as-errors'
nix develop --command bash -lc 'mix dialyzer --halt-exit-status'
nix develop --command bash -lc 'mix credo --strict'
nix develop --command bash -lc 'mix sobelow'
nix develop --command bash -lc 'lefthook run pre-commit'
```

Run Dialyzer as a required full gate once configured. Run Credo, Sobelow, asset, database, and release checks when configured or when the change affects those surfaces. State any skipped gate and why.
