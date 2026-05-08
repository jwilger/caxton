---
description: Reproduce a defect with a failing test before applying a bug fix.
agent: phoenix-commanded-implementer
---

Fix this bug through RGR: $ARGUMENTS

First reproduce the defect with a failing ExUnit test or a minimal existing test that currently fails. Record the failing command/output in the RGR ledger before editing production code. Then implement the minimum fix, run the focused test to GREEN, refactor if needed, and run relevant verification.
