#!/usr/bin/env python3
"""
Hook to verify PR operations don't violate safety rules.
Prevents marking PRs ready-for-review and other unsafe operations.
"""
import sys, json, re

def is_sparc_context(data):
    """Check if this is a SPARC-related operation"""
    user_prompt = data.get("user_prompt", "").lower()

    # SPARC command indicators
    sparc_indicators = ["/sparc", "sparc-orchestrator", "implementer", "planner",
                       "researcher", "type-architect", "test-hardener", "expert",
                       "pr-manager", ".claude/plan.approved", ".claude/tdd.red"]

    return any(indicator in user_prompt for indicator in sparc_indicators)

def main():
    data = json.load(sys.stdin)

    # Only enforce PR state checks for SPARC operations
    if not is_sparc_context(data):
        print(json.dumps({"block": False}))
        return

    # Get the tool being used and any relevant content
    tool_name = data.get("tool", "")
    user_prompt = data.get("user_prompt", "").lower()

    # Patterns that indicate unsafe PR operations
    unsafe_patterns = [
        "ready for review",
        "mark.*ready",
        "change.*status",
        "pr.*ready",
        "--ready",
        "ready-for-review",
        "remove.*draft",
        "publish.*pr"
    ]

    # Check for unsafe PR status changes
    if any(pattern in user_prompt for pattern in unsafe_patterns):
        print(json.dumps({
            "block": True,
            "message": "Blocked: Only humans can mark PRs as ready-for-review. Claude Code creates draft PRs only."
        }))
        return

    # Check for gh pr ready command specifically
    if "gh pr ready" in user_prompt:
        print(json.dumps({
            "block": True,
            "message": "Blocked: Cannot use 'gh pr ready'. PRs must remain in draft status until human review."
        }))
        return

    print(json.dumps({"block": False}))

if __name__ == "__main__":
    main()
