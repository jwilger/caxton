#!/usr/bin/env python3
"""
Hook to verify branch safety before commits/edits.
Blocks operations on main branch during stories and on closed PR branches.
"""
import sys, json, os, subprocess

def run_cmd(cmd):
    """Run command and return output, or None on error"""
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=10)
        return result.stdout.strip() if result.returncode == 0 else None
    except:
        return None

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

    # Only enforce branch checks for SPARC operations
    if not is_sparc_context(data):
        print(json.dumps({"block": False}))
        return

    project_dir = os.getenv("CLAUDE_PROJECT_DIR", ".")

    # Get current branch
    current_branch = run_cmd("git branch --show-current")
    if not current_branch:
        print(json.dumps({"block": False}))  # Not in git repo
        return

    # Check if we're in a story workflow
    branch_info_path = os.path.join(project_dir, ".claude", "branch.info")
    plan_approved = os.path.exists(os.path.join(project_dir, ".claude", "plan.approved"))

    if plan_approved and current_branch == "main":
        print(json.dumps({
            "block": True,
            "message": f"Cannot commit to main branch during story development. Switch to feature branch first."
        }))
        return

    # If we have branch info, verify PR status
    if os.path.exists(branch_info_path):
        try:
            with open(branch_info_path) as f:
                branch_info = json.load(f)

            pr_number = branch_info.get("pr_number")
            if pr_number:
                # Check PR status
                pr_status = run_cmd(f'gh pr view {pr_number} --json state --jq .state')
                if pr_status in ["MERGED", "CLOSED"]:
                    print(json.dumps({
                        "block": True,
                        "message": f"Cannot commit to branch {current_branch}. PR #{pr_number} is {pr_status.lower()}."
                    }))
                    return
        except (json.JSONDecodeError, FileNotFoundError, KeyError):
            pass  # Ignore errors, allow operation

    print(json.dumps({"block": False}))

if __name__ == "__main__":
    main()
