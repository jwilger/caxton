#!/usr/bin/env python3
"""
Hook to prevent git commit --no-verify commands.
This enforces the CLAUDE.md rule: NEVER bypass pre-commit hooks.
"""
import sys
import json
import re

def main():
    try:
        # Read JSON data from stdin
        data = json.load(sys.stdin)
    except:
        # If we can't parse JSON, allow the operation
        print(json.dumps({"block": False}))
        return

    # Check if this is a Bash tool call
    tool_name = data.get("tool", {}).get("name")
    if tool_name != "Bash":
        # Not a bash command, allow it
        print(json.dumps({"block": False}))
        return

    # Get the bash command from the tool parameters
    tool_params = data.get("tool", {}).get("parameters", {})
    bash_command = tool_params.get("command", "")

    if not bash_command:
        # No command specified, allow it
        print(json.dumps({"block": False}))
        return

    # Check for git commit with --no-verify flag
    if is_git_commit_no_verify(bash_command):
        error_message = """🚫 BLOCKED: git commit --no-verify is forbidden!

CLAUDE.md Rule: NEVER bypass pre-commit hooks with --no-verify

If you need to bypass hooks in a genuine emergency:
1. Create GitHub issue explaining why
2. Get team approval in issue comments
3. Notify team and run command manually from terminal

Otherwise, fix the underlying issues causing pre-commit failures."""

        print(json.dumps({
            "block": True,
            "message": error_message
        }))
        return

    # Allow all other commands
    print(json.dumps({"block": False}))

def is_git_commit_no_verify(command):
    """Check if command contains git commit with --no-verify flag."""

    # Normalize whitespace and convert to lowercase for checking
    normalized = ' '.join(command.split()).lower()

    # Check for git commit with --no-verify anywhere in the command
    # This handles various combinations like:
    # - git commit --no-verify
    # - git commit -m "msg" --no-verify
    # - git commit --no-verify -m "msg"
    # - nested commands with git commit --no-verify

    git_commit_pattern = r'\bgit\s+commit\b'
    no_verify_pattern = r'--no-verify\b'

    # Must have both "git commit" and "--no-verify"
    has_git_commit = re.search(git_commit_pattern, normalized)
    has_no_verify = re.search(no_verify_pattern, normalized)

    return has_git_commit and has_no_verify

if __name__ == "__main__":
    main()
