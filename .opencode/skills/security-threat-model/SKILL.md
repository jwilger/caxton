---
name: security-threat-model
description: Phoenix auth/session/CSRF, command authorization, event data sensitivity, Postgres, dependency, secret, and threat-model review triggers.
---

# Security Threat Model

Use this skill when changes touch Phoenix auth, sessions, CSRF, LiveView, command authorization, event payload PII, Postgres access, webhooks, background jobs, secrets, dependencies, or deployment.

## Sources

Read `docs/THREAT-MODEL.md` and the relevant ADRs before changing security-sensitive behavior when those files exist. If the change introduces a new threat boundary and no threat model exists yet, call that out explicitly.

## Review Questions

- Are Phoenix sessions, cookies, CSRF protection, LiveView boundaries, and plugs configured safely?
- Is authorization checked before Commanded command dispatch and before sensitive reads?
- Do event payloads avoid unnecessary PII, secrets, tokens, and irreversible sensitive data?
- Are webhook payloads authenticated before processing?
- Are secrets kept out of prompts, logs, commits, and review comments?
- Do migrations, queries, and constraints protect tenant/user data and preserve integrity?
- Are dependencies and generated assets trusted, pinned, and necessary?

## Coupling

If a documented threat changes, update the matching security test when needed. Deployment, telemetry, or alerting changes may require updates to operations docs and monitoring contracts.
