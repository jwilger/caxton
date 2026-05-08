---
name: forgejo-feedback-protocol
description: Process Forgejo PR feedback with reflection, guardrail-gap classification, and inline thread replies.
---

# Forgejo Feedback Protocol

Use this skill for every actionable PR review comment.

## Process

1. Fetch review comments with `tea` or the Forgejo REST API.
2. For each item, write a short reflection: why was the correct thing not done first?
3. Classify as `guardrail-gap` or `one-off` using `review-taxonomy`.
4. Remediate the code or guardrail according to the classification.
5. Reply on each inline thread before posting any top-level summary.

## Inline Reply Rule

Forgejo threads replies by review, path, and diff position. For an inline reply, post to the existing review comments endpoint with:

```json
{
  "body": "<reply>",
  "path": "<comment.path>",
  "new_position": <comment.position>,
  "old_position": 0
}
```

Do not use prose line numbers from the comment body as `new_position`.

## Top-Level Comments

Top-level PR comments are allowed only after all actionable inline threads have a per-thread response.
