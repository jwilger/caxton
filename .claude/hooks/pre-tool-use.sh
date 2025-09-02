#!/usr/bin/env bash
# Pre-tool-use hook for Caxton project
# Enforces strict agent permissions

TOOL_NAME="$1"
AGENT_TYPE="$2"
COMMAND="$3"

# Log for debugging (optional)
echo "[HOOK] Tool: $TOOL_NAME, Agent: $AGENT_TYPE, Command: ${COMMAND:0:50}..." >> "$HOME/.config/claude-code/hook.log"

# If it's not a Bash tool call, allow it
if [[ "$TOOL_NAME" != "Bash" ]]; then
    exit 0
fi

# Special case: pr-manager can use 'gh' commands
if [[ "$AGENT_TYPE" == "pr-manager" && "$COMMAND" =~ ^gh[[:space:]] ]]; then
    echo "INFO: pr-manager authorized for gh command"
    exit 0
fi

# Check if we're in an agent context (vs main Claude Code)
if [[ -n "$AGENT_TYPE" && "$AGENT_TYPE" != "general-purpose" ]]; then
    echo "ERROR: Agent '$AGENT_TYPE' is not authorized to use Bash tool"
    echo "Agents must use BashOutput to monitor bacon results"
    exit 1
fi

# For main context (SPARC coordinator), only allow bacon commands
if [[ ! "$COMMAND" =~ ^bacon ]]; then
    echo "ERROR: Only 'bacon' commands are permitted"
    echo "Attempted command: $COMMAND"
    echo "Use MCP servers for other functionality"
    exit 1
fi

exit 0
