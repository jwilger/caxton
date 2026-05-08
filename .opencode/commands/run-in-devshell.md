---
description: Run a project command through the Nix dev shell.
agent: phoenix-commanded-implementer
---

Run this project command through the canonical Nix dev shell: $ARGUMENTS

Use:

```sh
nix develop --command bash -lc '$ARGUMENTS'
```

If quoting or shell semantics fail, adjust the invocation while preserving the `nix develop --command bash -lc` wrapper. Use the ambient shell only if the command intentionally inspects the environment outside the flake.
