# Security

Do not read or commit secrets. Treat `.env*`, keys, tokens, credentials, production data, and incident material as out of scope unless the user explicitly authorizes a specific read.

Changes touching Phoenix authentication, session handling, CSRF, LiveView boundaries, command authorization, event payload PII, Postgres data access, background jobs, webhooks, secrets, dependencies, or deployment must be reviewed for security impact. Check `docs/THREAT-MODEL.md` and relevant security tests when those files exist or when the change introduces a documented threat boundary.
