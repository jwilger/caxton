#!/bin/bash
# memory-cli-config.sh - Configuration for the memory CLI system

# Valid agents (from the existing private index)
VALID_AGENTS=(
    "researcher"
    "planner"
    "implementer"
    "type-architect"
    "test-hardener"
    "expert"
    "pr-manager"
)

# Valid scopes
VALID_SCOPES=("private" "shared")

# Valid categories
VALID_CATEGORIES=("decisions" "learnings" "context" "general")

# Valid priority levels
VALID_PRIORITIES=("low" "medium" "high")

# Base paths
MEMORY_BASE_PATH=".claude/memories"
TEMPLATE_PATH=".claude/templates/memory-template.json"

# File operation settings
MAX_RETRIES=3
RETRY_DELAY=1

# Search defaults
DEFAULT_SEARCH_LIMIT=10
MAX_SEARCH_LIMIT=100

# Security settings - restrict operations to memory directories only
ALLOWED_PATHS=(
    ".claude/memories/shared"
    ".claude/memories/private"
    ".claude/templates"
)

# Required tools
REQUIRED_TOOLS=("jq" "grep" "find" "sort" "date")

# Validation patterns
ID_PATTERN="^[0-9]{10}-[a-zA-Z0-9_-]+-[a-zA-Z0-9_-]+-[a-f0-9]{6}$"
AGENT_PATTERN="^[a-zA-Z0-9_-]+$"
SCOPE_PATTERN="^(private|shared)$"
CATEGORY_PATTERN="^(decisions|learnings|context|general)$"

# Function to check if agent is valid
is_valid_agent() {
    local agent="$1"
    for valid_agent in "${VALID_AGENTS[@]}"; do
        if [[ "$agent" == "$valid_agent" ]]; then
            return 0
        fi
    done
    return 1
}

# Function to check if scope is valid
is_valid_scope() {
    local scope="$1"
    for valid_scope in "${VALID_SCOPES[@]}"; do
        if [[ "$scope" == "$valid_scope" ]]; then
            return 0
        fi
    done
    return 1
}

# Function to check if category is valid
is_valid_category() {
    local category="$1"
    for valid_category in "${VALID_CATEGORIES[@]}"; do
        if [[ "$category" == "$valid_category" ]]; then
            return 0
        fi
    done
    return 1
}

# Function to check if priority is valid
is_valid_priority() {
    local priority="$1"
    for valid_priority in "${VALID_PRIORITIES[@]}"; do
        if [[ "$priority" == "$valid_priority" ]]; then
            return 0
        fi
    done
    return 1
}

# Function to validate path is allowed
is_allowed_path() {
    local path="$1"
    local resolved_path=$(realpath "$path" 2>/dev/null || echo "$path")

    for allowed in "${ALLOWED_PATHS[@]}"; do
        local allowed_resolved=$(realpath "$allowed" 2>/dev/null || echo "$allowed")
        if [[ "$resolved_path" == "$allowed_resolved"* ]]; then
            return 0
        fi
    done
    return 1
}

# Function to check required tools
check_required_tools() {
    local missing=()
    for tool in "${REQUIRED_TOOLS[@]}"; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            missing+=("$tool")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        echo "Error: Missing required tools: ${missing[*]}" >&2
        return 1
    fi
    return 0
}
