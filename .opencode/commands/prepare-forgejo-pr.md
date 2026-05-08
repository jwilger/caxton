---
description: Prepare a scoped Forgejo PR with explicit-path staging and conventional commit checks.
agent: phoenix-commanded-implementer
---

Prepare a Forgejo PR for: $ARGUMENTS

Workflow:

1. Audit scope with `git status` and diffs.
2. Stage only explicit paths; do not use `git add .`, `git add -A`, `git add -u`, or `git commit -a`.
3. Check commit titles follow conventional commits; release notes are generated from conventional commits.
4. Verify relevant gates through `nix develop --command bash -lc '<command>'`.
5. Derive the Forgejo repo from `git remote -v` when available, or require an explicit repo argument if no remote is configured.
6. Use `tea pr create --repo <owner>/<repo> --head <branch> --base main --title "..." --description "..."`.

Do not use `gh` for this repo.
