# Toolchain

Run project commands inside the Nix dev shell by default: `nix develop --command bash -lc '<command>'`. This includes `mix`, `lefthook`, `tea`, `forgejo-mcp`, `node`, `bun`, test runners, formatters, linters, and build tools.

Use the ambient shell only for bootstrap/debug commands that intentionally inspect the environment outside the flake, such as checking `nix`, debugging shell startup, inspecting OpenCode itself, or intentionally testing outside the project toolchain.

After changing `flake.nix`, immediately run commands through `nix develop --command bash -lc '<command>'` in the current session instead of requiring an OpenCode restart. If quoting or shell semantics fail inside `nix develop`, adjust the invocation rather than falling back to the ambient shell.

Do not install global toolchains, global Mix archives, or global Node packages unless the user explicitly asks for that machine-level change. Keep caches project-local under `.dependencies/` when configured, and set `MIX_ENV` per command instead of globally.
