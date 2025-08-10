#!/usr/bin/env python3
import sys, json, os

def is_sparc_context(data):
    """Check if this is a SPARC-related operation"""
    user_prompt = data.get("user_prompt", "").lower()
    tool_context = data.get("tool", "")

    # SPARC command indicators
    sparc_indicators = ["/sparc", "sparc-orchestrator", "implementer", "planner",
                       "researcher", "type-architect", "test-hardener", "expert",
                       "pr-manager", ".claude/plan.approved", ".claude/tdd.red"]

    return any(indicator in user_prompt for indicator in sparc_indicators)

def main():
    data = json.load(sys.stdin)

    # Only enforce plan mode for SPARC operations
    if not is_sparc_context(data):
        print(json.dumps({"block": False}))
        return

    project_dir = os.getenv("CLAUDE_PROJECT_DIR", ".")
    plan_ok = os.path.exists(os.path.join(project_dir, ".claude", "plan.approved"))

    if not plan_ok:
        print(json.dumps({
            "block": True,
            "message": "No approved plan found. Run /sparc:plan, review, then create .claude/plan.approved"
        }))
    else:
        print(json.dumps({"block": False}))

if __name__ == "__main__":
    main()
