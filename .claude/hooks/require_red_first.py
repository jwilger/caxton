#!/usr/bin/env python3
# Block the FIRST implementation edit unless we can detect a failing test indicator.
# Simple heuristic: expect a sentinel file ".claude/tdd.red" to exist during RED phase.
import os, sys, json

def is_sparc_context(data):
    """Check if this is a SPARC-related operation"""
    user_prompt = data.get("user_prompt", "").lower()

    # SPARC command indicators
    sparc_indicators = ["/sparc", "sparc-orchestrator", "implementer", "planner",
                       "researcher", "type-architect", "test-hardener", "expert",
                       "pr-manager", ".claude/plan.approved", ".claude/tdd.red"]

    return any(indicator in user_prompt for indicator in sparc_indicators)

def main():
    try:
        data = json.load(sys.stdin)
    except:
        data = {}

    # Only enforce TDD for SPARC operations
    if not is_sparc_context(data):
        sys.exit(0)

    root = os.getenv("CLAUDE_PROJECT_DIR", ".")
    red = os.path.exists(os.path.join(root, ".claude", "tdd.red"))
    green = os.path.exists(os.path.join(root, ".claude", "tdd.green"))

    # Only block if we haven't seen RED yet and plan is approved.
    if not red and not green:
        sys.stderr.write("TDD: start with one failing test (create .claude/tdd.red). The implementer should do this automatically on the first RED step.\n")
        sys.exit(2)
    sys.exit(0)

if __name__ == "__main__":
    main()
