---
description: Select the next unblocked Forgejo issue from dependency state before starting work.
agent: phoenix-commanded-implementer
---

Select the next available Forgejo issue for: $ARGUMENTS

This command is the required entry point for vague requests such as "next issue", "continue the roadmap", "pick up the next task", or any issue-targeted work where the issue number is not explicitly supplied.

Workflow:

1. Confirm the working tree is clean with `git status --short --branch`. If it is not clean, stop and report the existing changes before switching branches.
2. Derive the Forgejo owner/repo from the `origin` remote or the current `tea` repository context. Use Forgejo/`tea`, not GitHub tooling.
3. Fetch all open issues across pages; do not rely on the default first page of `tea issues list`.
4. For each open issue, query `tea api repos/<owner>/<repo>/issues/<index>/dependencies`.
5. Treat an issue as available only when all returned dependencies are closed.
6. Cross-check each candidate's body-level `Depends on:` line against its Forgejo dependency list. If the body graph and API graph disagree, stop and report the mismatch before implementation.
7. Report the full available issue set. Prefer the lowest-numbered non-roadmap implementation issue unless the user explicitly asked for roadmap-management work; otherwise choose the lowest-numbered available issue.
8. Create and switch to a dedicated branch before edits, using `issue-<index>-<short-slug>`.
9. Proceed with the issue-specific workflow: docs/config-only issues make the scoped change and run available non-Mix checks; behavior changes load the RGR skills and follow RED-GREEN-REFACTOR with specialist agents.

Reference audit snippet for dependency-based availability:

```sh
nix develop --command bash -lc 'python - <<'"'"'PY'"'"'
import json, subprocess

OWNER = "Slipstream"
REPO = "caxton"

def api(path):
    result = subprocess.run(["tea", "api", path], text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if result.returncode:
        raise SystemExit(result.stderr)
    return json.loads(result.stdout or "null")

issues = []
seen = set()
page = 1
while True:
    batch = api(f"repos/{OWNER}/{REPO}/issues?state=open&type=issues&limit=50&page={page}")
    if not batch:
        break
    for issue in batch:
        if issue["number"] not in seen:
            seen.add(issue["number"])
            issues.append(issue)
    page += 1

available = []
for issue in sorted(issues, key=lambda item: item["number"]):
    deps = api(f"repos/{OWNER}/{REPO}/issues/{issue['number']}/dependencies")
    open_deps = [dep for dep in deps if dep.get("state") == "open"]
    if not open_deps:
        available.append(issue)

for issue in available:
    print(f"#{issue['number']} {issue['title']}")
PY'
```
