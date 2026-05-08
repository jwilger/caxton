# Forgejo

This repo uses Forgejo at `git.johnwilger.com`, not GitHub. Use `tea` for issues and pull requests. Do not introduce `gh` workflows.

Before selecting or starting "the next issue", query Forgejo dependency state rather than relying on list order, milestone order, or issue numbers. An issue is available only when it is open and every issue returned by `GET /repos/{owner}/{repo}/issues/{index}/dependencies` is closed. If multiple issues are available, prefer the lowest-numbered non-roadmap implementation issue unless the user explicitly asks to finish roadmap-management work first. Report the available issue set and chosen issue before creating the branch.

If body-level `Depends on:` references and Forgejo dependency relationships disagree, stop implementation work and resolve or report the dependency mismatch first. Roadmap bodies should link dependency keys to their Forgejo issues so humans and agents can cross-check the graph.

Inline review feedback must be answered on the inline thread first. For Forgejo API replies, copy the original comment `position` into the reply payload as `new_position` and set `old_position` to `0`.
