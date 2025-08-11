#!/bin/bash
# memory-cli-lib.sh - Function library for memory CLI operations

# Source configuration
SCRIPT_DIR="$(dirname "${BASH_SOURCE[0]}")"
source "$SCRIPT_DIR/memory-cli-config.sh"

# Logging functions
log_error() {
    echo "ERROR: $*" >&2
}

log_info() {
    echo "INFO: $*" >&2
}

log_debug() {
    if [[ "${DEBUG:-}" == "1" ]]; then
        echo "DEBUG: $*" >&2
    fi
}

# Generate unique memory ID
generate_memory_id() {
    local agent="$1"
    local category="$2"
    local timestamp=$(date +%s)
    local random_hex=$(openssl rand -hex 3 2>/dev/null || printf "%06x" $((RANDOM * 65536 + RANDOM)))
    echo "${timestamp}-${agent}-${category}-${random_hex}"
}

# Generate JSON memory structure
generate_memory_json() {
    local agent="$1"
    local scope="$2"
    local category="$3"
    local title="$4"
    local content="$5"
    local tags="$6"
    local story_context="${7:-}"
    local priority="${8:-low}"

    local memory_id=$(generate_memory_id "$agent" "$category")
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    # Convert comma-separated tags to JSON array
    local tags_json="[]"
    if [[ -n "$tags" ]]; then
        # Split tags by comma and create JSON array
        IFS=',' read -ra tag_array <<< "$tags"
        local tag_list=""
        for tag in "${tag_array[@]}"; do
            # Trim whitespace
            tag=$(echo "$tag" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
            if [[ -n "$tag" ]]; then
                tag_list="$tag_list\"$tag\","
            fi
        done
        if [[ -n "$tag_list" ]]; then
            tag_list="${tag_list%,}"  # Remove trailing comma
            tags_json="[$tag_list]"
        fi
    fi

    # Generate search keywords from title and content
    local keywords=$(echo "$title $content" | tr '[:upper:]' '[:lower:]' | \
        grep -oE '\b[a-z0-9_-]{3,}\b' | sort -u | \
        jq -R . | jq -s .)

    cat <<EOF
{
  "id": "$memory_id",
  "agent": "$agent",
  "scope": "$scope",
  "category": "$category",
  "title": "$title",
  "content": "$content",
  "tags": $tags_json,
  "search_keywords": $keywords,
  "metadata": {
    "created_at": "$timestamp",
    "updated_at": "$timestamp",
    "version": "1.0",
    "priority": "$priority",
    "story_context": "$story_context",
    "expires_after": null,
    "related_memories": []
  }
}
EOF
}

# Validate JSON structure
validate_memory_json() {
    local json_content="$1"

    # Check if valid JSON
    if ! echo "$json_content" | jq empty >/dev/null 2>&1; then
        log_error "Invalid JSON structure"
        return 1
    fi

    # Check required fields
    local required_fields=("id" "agent" "scope" "category" "title" "content")
    for field in "${required_fields[@]}"; do
        if ! echo "$json_content" | jq -e ".$field" >/dev/null 2>&1; then
            log_error "Missing required field: $field"
            return 1
        fi
    done

    # Validate field values
    local agent=$(echo "$json_content" | jq -r '.agent')
    local scope=$(echo "$json_content" | jq -r '.scope')
    local category=$(echo "$json_content" | jq -r '.category')

    if ! is_valid_agent "$agent"; then
        log_error "Invalid agent: $agent"
        return 1
    fi

    if ! is_valid_scope "$scope"; then
        log_error "Invalid scope: $scope"
        return 1
    fi

    if ! is_valid_category "$category"; then
        log_error "Invalid category: $category"
        return 1
    fi

    return 0
}

# Atomic file write with retry
atomic_write() {
    local target_file="$1"
    local content="$2"
    local retries=0

    # Validate path
    if ! is_allowed_path "$(dirname "$target_file")"; then
        log_error "Path not allowed: $target_file"
        return 1
    fi

    while [[ $retries -lt $MAX_RETRIES ]]; do
        local temp_file="${target_file}.tmp.$$"

        if echo "$content" > "$temp_file" && mv "$temp_file" "$target_file"; then
            return 0
        fi

        rm -f "$temp_file" 2>/dev/null
        retries=$((retries + 1))
        log_debug "Retry $retries for $target_file"
        sleep $RETRY_DELAY
    done

    log_error "Failed to write $target_file after $MAX_RETRIES attempts"
    return 1
}

# Search memories with multiple criteria
search_memories() {
    local query="$1"
    local scope_filter="$2"
    local agent_filter="$3"
    local tag_filter="$4"
    local limit="${5:-$DEFAULT_SEARCH_LIMIT}"

    # Build search paths
    local search_paths=()
    case "$scope_filter" in
        "private")
            if [[ -n "$agent_filter" ]]; then
                search_paths=("$MEMORY_BASE_PATH/private/$agent_filter")
            else
                search_paths=("$MEMORY_BASE_PATH/private")
            fi
            ;;
        "shared")
            search_paths=("$MEMORY_BASE_PATH/shared")
            ;;
        "all"|*)
            if [[ -n "$agent_filter" ]]; then
                search_paths=("$MEMORY_BASE_PATH/private/$agent_filter" "$MEMORY_BASE_PATH/shared")
            else
                search_paths=("$MEMORY_BASE_PATH/shared" "$MEMORY_BASE_PATH/private")
            fi
            ;;
    esac

    # Find matching files
    local results=()
    local temp_results=$(mktemp)

    for search_path in "${search_paths[@]}"; do
        if [[ ! -d "$search_path" ]]; then
            continue
        fi

        # Search by query in title/content
        if [[ -n "$query" ]]; then
            find "$search_path" -name "*.json" -not -name "index.json" -exec grep -l -i "$query" {} \; >> "$temp_results" 2>/dev/null
        else
            find "$search_path" -name "*.json" -not -name "index.json" >> "$temp_results" 2>/dev/null
        fi
    done

    # Filter by agent if specified
    if [[ -n "$agent_filter" && "$scope_filter" != "private" ]]; then
        local agent_filtered=$(mktemp)
        while IFS= read -r file; do
            if jq -e ".agent == \"$agent_filter\"" "$file" >/dev/null 2>&1; then
                echo "$file" >> "$agent_filtered"
            fi
        done < "$temp_results"
        mv "$agent_filtered" "$temp_results"
    fi

    # Filter by tags if specified
    if [[ -n "$tag_filter" ]]; then
        local tag_filtered=$(mktemp)
        IFS=',' read -ra tag_array <<< "$tag_filter"
        while IFS= read -r file; do
            local match=true
            for tag in "${tag_array[@]}"; do
                tag=$(echo "$tag" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
                if ! jq -e ".tags | contains([\"$tag\"])" "$file" >/dev/null 2>&1; then
                    match=false
                    break
                fi
            done
            if [[ "$match" == "true" ]]; then
                echo "$file" >> "$tag_filtered"
            fi
        done < "$temp_results"
        mv "$tag_filtered" "$temp_results"
    fi

    # Sort by modification time (newest first) and apply limit
    sort -u "$temp_results" | xargs ls -t 2>/dev/null | head -n "$limit"

    rm -f "$temp_results"
}

# Update index files after memory operations
update_indices() {
    local memory_file="$1"
    local operation="$2"  # add, update, delete

    if [[ ! -f "$memory_file" ]]; then
        log_error "Memory file not found: $memory_file"
        return 1
    fi

    local memory_json=$(cat "$memory_file")
    local agent=$(echo "$memory_json" | jq -r '.agent')
    local scope=$(echo "$memory_json" | jq -r '.scope')
    local category=$(echo "$memory_json" | jq -r '.category')
    local memory_id=$(echo "$memory_json" | jq -r '.id')
    local tags=$(echo "$memory_json" | jq -r '.tags[]' 2>/dev/null || true)

    local index_file="$MEMORY_BASE_PATH/$scope/index.json"

    # Create index if it doesn't exist
    if [[ ! -f "$index_file" ]]; then
        local initial_index
        if [[ "$scope" == "shared" ]]; then
            initial_index='{
                "version": "1.0",
                "last_updated": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'",
                "total_memories": 0,
                "categories": {"decisions": [], "learnings": [], "context": [], "general": []},
                "agents": {},
                "tags": {},
                "recent_memories": []
            }'
        else
            initial_index='{
                "version": "1.0",
                "last_updated": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'",
                "agents": {}
            }'
        fi
        atomic_write "$index_file" "$initial_index"
    fi

    # Update index based on operation
    local updated_index
    case "$operation" in
        "add"|"update")
            updated_index=$(cat "$index_file" | jq \
                --arg agent "$agent" \
                --arg scope "$scope" \
                --arg category "$category" \
                --arg memory_id "$memory_id" \
                --arg timestamp "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
                --argjson tags "$(echo "$tags" | jq -R . | jq -s .)" '
                .last_updated = $timestamp |
                if $scope == "shared" then
                    .categories[$category] |= (. + [$memory_id] | unique) |
                    .agents[$agent] = ((.agents[$agent] // 0) + 1) |
                    .recent_memories = ([$memory_id] + .recent_memories)[0:10] |
                    .total_memories = (.categories | to_entries | map(.value | length) | add) |
                    .tags = reduce $tags[] as $tag (.tags; .[$tag] = ((.[$tag] // 0) + 1))
                else
                    .agents[$agent].categories[$category] |= (. + [$memory_id] | unique) |
                    .agents[$agent].recent_memories = ([$memory_id] + (.agents[$agent].recent_memories // []))[0:10] |
                    .agents[$agent].total_memories = (.agents[$agent].categories | to_entries | map(.value | length) | add) |
                    .agents[$agent].tags = reduce $tags[] as $tag (.agents[$agent].tags; .[$tag] = ((.[$tag] // 0) + 1))
                end'
            )
            ;;
        "delete")
            updated_index=$(cat "$index_file" | jq \
                --arg agent "$agent" \
                --arg scope "$scope" \
                --arg category "$category" \
                --arg memory_id "$memory_id" \
                --arg timestamp "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
                --argjson tags "$(echo "$tags" | jq -R . | jq -s .)" '
                .last_updated = $timestamp |
                if $scope == "shared" then
                    .categories[$category] |= (. - [$memory_id]) |
                    .agents[$agent] = ((.agents[$agent] // 1) - 1) |
                    .recent_memories = (.recent_memories - [$memory_id]) |
                    .total_memories = (.categories | to_entries | map(.value | length) | add) |
                    .tags = reduce $tags[] as $tag (.tags; .[$tag] = ((.[$tag] // 1) - 1))
                else
                    .agents[$agent].categories[$category] |= (. - [$memory_id]) |
                    .agents[$agent].recent_memories = (.agents[$agent].recent_memories - [$memory_id]) |
                    .agents[$agent].total_memories = (.agents[$agent].categories | to_entries | map(.value | length) | add) |
                    .agents[$agent].tags = reduce $tags[] as $tag (.agents[$agent].tags; .[$tag] = ((.[$tag] // 1) - 1))
                end'
            )
            ;;
    esac

    atomic_write "$index_file" "$updated_index"
}

# Get memory file path
get_memory_path() {
    local scope="$1"
    local agent="$2"
    local category="$3"
    local memory_id="$4"

    if [[ "$scope" == "shared" ]]; then
        echo "$MEMORY_BASE_PATH/shared/$category/${memory_id}.json"
    else
        echo "$MEMORY_BASE_PATH/private/$agent/${memory_id}.json"
    fi
}

# Ensure directory exists
ensure_directory() {
    local dir="$1"
    if ! is_allowed_path "$dir"; then
        log_error "Directory not allowed: $dir"
        return 1
    fi

    mkdir -p "$dir" 2>/dev/null || {
        log_error "Failed to create directory: $dir"
        return 1
    }
}

# Find memory by ID
find_memory_by_id() {
    local memory_id="$1"
    local agent="$2"

    log_debug "Searching for memory ID: $memory_id, agent: $agent"

    # Search in agent's private memories first
    if [[ -n "$agent" ]]; then
        local private_path="$MEMORY_BASE_PATH/private/$agent"
        if [[ -d "$private_path" ]]; then
            local result=$(find "$private_path" -name "${memory_id}.json" -type f 2>/dev/null | head -1)
            if [[ -n "$result" ]]; then
                echo "$result"
                return
            fi
        fi
    fi

    # Search in shared memories
    local shared_path="$MEMORY_BASE_PATH/shared"
    if [[ -d "$shared_path" ]]; then
        local result=$(find "$shared_path" -name "${memory_id}.json" -type f 2>/dev/null | head -1)
        if [[ -n "$result" ]]; then
            echo "$result"
            return
        fi
    fi

    # Fallback: search all memories (in case of ID pattern match)
    local result=$(find "$MEMORY_BASE_PATH" -name "*${memory_id}*.json" -not -name "index.json" -type f 2>/dev/null | head -1)
    echo "$result"
}
