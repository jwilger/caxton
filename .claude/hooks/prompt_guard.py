#!/usr/bin/env python3
import sys, json

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

    # Always apply basic safety checks, but be more restrictive for SPARC
    user_prompt = data.get("user_prompt", "").lower()

    # Basic safety patterns (always active)
    basic_dangerous = ["BEGIN SSH KEY", "rm -rf", "DROP TABLE", "--no-verify", "sudo rm", "exec", "eval"]

    # SPARC-specific dangerous patterns (only for SPARC contexts)
    sparc_dangerous = ["cargo clean", "rm cargo.lock", "unsafe {", "--release --force"]

    # Check basic patterns
    if any(s in user_prompt for s in basic_dangerous):
        print(json.dumps({"block": True, "message": "Blocked potentially dangerous instruction. Rephrase with intent."}))
        return

    # Check SPARC-specific patterns only in SPARC context
    if is_sparc_context(data) and any(s in user_prompt for s in sparc_dangerous):
        print(json.dumps({"block": True, "message": "Blocked potentially dangerous Rust operation during SPARC workflow."}))
        return

    print(json.dumps({"block": False}))

if __name__ == "__main__":
    main()
