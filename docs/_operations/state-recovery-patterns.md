---
title: "State Recovery Patterns for Agents"
date: 2025-01-15
layout: page
categories: [Operations]
---

## Overview

This document outlines comprehensive patterns and procedures for recovering
agent state and system data after crashes, restarts, or failures in the
embedded, zero-dependency architecture (ADRs 28-30). These patterns ensure
system resilience and maintain operational continuity for both configuration
agents and embedded memory systems.

## Recovery Scenarios (Embedded Architecture)

### 1. Server Process Failure

When the single Caxton server process crashes or is terminated.

### 2. Configuration Agent Failure

When configuration agent hot-reload fails or agent configs become corrupted.

### 3. Embedded Memory System Corruption

When SQLite database or embedding model data becomes corrupted.

### 4. Configuration File Corruption

When agent configuration files are corrupted or missing.

### 5. Embedding Model Corruption

When the All-MiniLM-L6-v2 model files become corrupted or inaccessible.

### 6. Partial System Degradation

When specific components fail while the server continues running.

### 7. Complete System Recovery

When recovering from total system failure including data corruption.

## State Recovery Patterns (Embedded Architecture)

### Pattern 1: Server Process Recovery

#### Overview

Recover from single server process failure with embedded data integrity.

#### Implementation

```bash
#!/bin/bash
# Server process recovery for embedded architecture

recover_server_process() {
    local DATA_DIR="/var/lib/caxton/data"
    local CONFIG_DIR="/var/lib/caxton/agents"
    local LOG_DIR="/var/lib/caxton/logs"

    echo "Starting Caxton server recovery..."

    # 1. Check embedded data integrity
    if ! check_embedded_data_integrity "$DATA_DIR"; then
        echo "Data corruption detected, attempting repair..."
        repair_embedded_data "$DATA_DIR" || return 1
    fi

    # 2. Validate configuration agent files
    if ! validate_config_agents "$CONFIG_DIR"; then
        echo "Configuration validation failed, restoring from backup..."
        restore_config_agents "$CONFIG_DIR" || return 1
    fi

    # 3. Check embedding model integrity
    if ! check_embedding_model; then
        echo "Embedding model corrupted, re-downloading..."
        restore_embedding_model || return 1
    fi

    # 4. Start server with recovery mode
    systemctl start caxton

    # 5. Verify server startup
    if wait_for_server_startup; then
        echo "Server recovery completed successfully"
        return 0
    else
        echo "Server recovery failed"
        return 1
    fi
}

check_embedded_data_integrity() {
    local data_dir=$1

    # Check SQLite database integrity
    if [[ -f "$data_dir/memory.db" ]]; then
        sqlite3 "$data_dir/memory.db" "PRAGMA integrity_check;" | grep -q "ok"
    else
        echo "SQLite database missing"
        return 1
    fi
}

validate_config_agents() {
    local config_dir=$1
    local all_valid=true

    for config_file in "$config_dir"/*.md; do
        if [[ -f "$config_file" ]]; then
            if ! caxton agents validate "$config_file"; then
                echo "Invalid configuration: $config_file"
                all_valid=false
            fi
        fi
    done

    [[ "$all_valid" == "true" ]]
}
```text

```text

```text

#### Automated Recovery Strategies

##### Systemd Recovery Configuration

```ini
# /etc/systemd/system/caxton.service
[Unit]
Description=Caxton Multi-Agent Server
After=network.target

[Service]
Type=simple
User=caxton
Group=caxton
ExecStart=/usr/local/bin/caxton start --config /etc/caxton/caxton.yaml
Restart=always
RestartSec=5
StartLimitBurst=5
StartLimitIntervalSec=60

# Recovery configuration
ExecStartPre=/usr/local/bin/caxton-recovery-check
TimeoutStartSec=30
WatchdogSec=60

# Security settings
NoNewPrivileges=true
ProtectSystem=strict

[Install]
WantedBy=multi-user.target
```text

##### Health Check Based Recovery

```bash
#!/bin/bash
# /usr/local/bin/caxton-recovery-check

perform_recovery_checks() {
    local checks_passed=true

    # Check 1: Data directory accessibility
    if [[ ! -w "/var/lib/caxton/data" ]]; then
        echo "Data directory not writable"
        checks_passed=false
    fi

    # Check 2: SQLite database integrity
    if [[ -f "/var/lib/caxton/data/memory.db" ]]; then
        if ! sqlite3 "/var/lib/caxton/data/memory.db" "PRAGMA quick_check;" | grep -q "ok"; then
            echo "SQLite integrity check failed, attempting repair..."
            repair_sqlite_database
        fi
    fi

    # Check 3: Configuration agent validation
    if ! validate_all_config_agents; then
        echo "Configuration validation failed"
        checks_passed=false
    fi

    # Check 4: Embedding model accessibility
    if ! check_embedding_model_files; then
        echo "Embedding model files corrupted or missing"
        download_embedding_model
    fi

    [[ "$checks_passed" == "true" ]]
}
```text

### Pattern 2: Embedded Memory System Recovery

#### Overview

Recover SQLite database and embedding model data from corruption or failure.

#### Implementation

```bash
#!/bin/bash
# Embedded memory system recovery

recover_embedded_memory_system() {
    local data_dir="/var/lib/caxton/data"
    local backup_dir="/backup/caxton"

    echo "Recovering embedded memory system..."

    # 1. Stop server to ensure consistent recovery
    systemctl stop caxton

    # 2. Assess damage
    local recovery_needed=false

    if ! sqlite3 "$data_dir/memory.db" "PRAGMA integrity_check;" | grep -q "ok"; then
        echo "SQLite database corruption detected"
        recovery_needed=true
    fi

    if ! check_embedding_cache_integrity; then
        echo "Embedding cache corruption detected"
        recovery_needed=true
    fi

    if [[ "$recovery_needed" == "true" ]]; then
        # 3. Attempt repair first
        if attempt_sqlite_repair "$data_dir/memory.db"; then
            echo "SQLite repair successful"
        else
            echo "SQLite repair failed, restoring from backup"
            restore_from_backup "$backup_dir" "$data_dir"
        fi

        # 4. Rebuild embedding cache if needed
        if ! check_embedding_cache_integrity; then
            rebuild_embedding_cache
        fi
    fi

    # 5. Start server and verify
    systemctl start caxton

    if wait_for_memory_system_ready; then
        echo "Memory system recovery completed"
        return 0
    else
        echo "Memory system recovery failed"
        return 1
    fi
}

attempt_sqlite_repair() {
    local db_file=$1
    local temp_db="${db_file}.repair"

    # Create repair database
    sqlite3 "$temp_db" << 'EOF'
.recover /var/lib/caxton/data/memory.db
EOF

    # Verify repair worked
    if sqlite3 "$temp_db" "PRAGMA integrity_check;" | grep -q "ok"; then
        mv "$temp_db" "$db_file"
        return 0
    else
        rm -f "$temp_db"
        return 1
    fi
}

rebuild_embedding_cache() {
    echo "Rebuilding embedding cache..."

    # Clear corrupted cache
    rm -rf /var/lib/caxton/data/embedding_cache/*

    # Start server to rebuild cache on demand
    systemctl start caxton

    # Wait for cache rebuild (embeddings generated as needed)
    sleep 10

    echo "Embedding cache rebuild initiated"
}
```text

```text

#### Memory System Recovery Optimization

```bash
# Advanced SQLite recovery procedures

optimized_sqlite_recovery() {
    local db_file="/var/lib/caxton/data/memory.db"
    local backup_file="/backup/caxton/latest-memory.db"

    # Strategy 1: Incremental recovery
    if attempt_incremental_recovery "$db_file"; then
        echo "Incremental recovery successful"
        return 0
    fi

    # Strategy 2: Selective table recovery
    if attempt_selective_recovery "$db_file"; then
        echo "Selective recovery successful"
        return 0
    fi

    # Strategy 3: Full backup restoration
    if attempt_backup_restoration "$backup_file" "$db_file"; then
        echo "Backup restoration successful"
        return 0
    fi

    # Strategy 4: Fresh start with schema recreation
    echo "All recovery strategies failed, creating fresh database"
    create_fresh_database "$db_file"
}

attempt_incremental_recovery() {
    local db_file=$1

    # Use SQLite's recovery features
    sqlite3 "$db_file" << 'EOF'
PRAGMA writable_schema = ON;
UPDATE sqlite_master SET sql = (
    SELECT sql FROM sqlite_master
    WHERE name = sqlite_master.name
) WHERE type = 'table';
PRAGMA writable_schema = OFF;
PRAGMA integrity_check;
EOF

    # Check if recovery worked
    sqlite3 "$db_file" "PRAGMA integrity_check;" | grep -q "ok"
}

attempt_selective_recovery() {
    local db_file=$1
    local temp_db="${db_file}.selective"

    # Extract recoverable data
    sqlite3 "$db_file" << EOF
.output /tmp/entities_dump.sql
.dump entities
.output /tmp/relations_dump.sql
.dump relations
.output /tmp/observations_dump.sql
.dump observations
EOF

    # Create new database with recovered data
    sqlite3 "$temp_db" << EOF
.read /tmp/entities_dump.sql
.read /tmp/relations_dump.sql
.read /tmp/observations_dump.sql
EOF

    # Verify selective recovery
    if sqlite3 "$temp_db" "PRAGMA integrity_check;" | grep -q "ok"; then
        mv "$temp_db" "$db_file"
        rm -f /tmp/*_dump.sql
        return 0
    else
        rm -f "$temp_db" /tmp/*_dump.sql
        return 1
    fi
}
```text

```text

### Pattern 3: Configuration Agent Recovery

#### Overview

Recover configuration agents from file corruption, validation failures, or hot-reload issues.

#### Implementation

```bash
#!/bin/bash
# Configuration agent recovery procedures

recover_configuration_agents() {
    local agents_dir="/var/lib/caxton/agents"
    local backup_dir="/backup/caxton/agents"

    echo "Recovering configuration agents..."

    # 1. Validate all existing configurations
    local corrupted_agents=()
    for config_file in "$agents_dir"/*.md; do
        if [[ -f "$config_file" ]]; then
            if ! caxton agents validate "$config_file"; then
                corrupted_agents+=("$(basename "$config_file")")
            fi
        fi
    done

    # 2. Recover corrupted agents
    for agent_file in "${corrupted_agents[@]}"; do
        echo "Recovering corrupted agent: $agent_file"
        recover_single_agent "$agent_file" "$agents_dir" "$backup_dir"
    done

    # 3. Check for missing agents
    check_for_missing_agents "$agents_dir" "$backup_dir"

    # 4. Perform hot-reload validation
    echo "Validating recovered agents..."
    if caxton agents validate-all; then
        echo "All agents validated successfully"
        return 0
    else
        echo "Some agents still have validation issues"
        return 1
    fi
}

recover_single_agent() {
    local agent_file=$1
    local agents_dir=$2
    local backup_dir=$3
    local full_path="$agents_dir/$agent_file"

    # Strategy 1: Restore from git history (if using git)
    if [[ -d "$agents_dir/.git" ]]; then
        echo "Attempting git recovery for $agent_file"
        cd "$agents_dir"
        if git checkout HEAD~1 "$agent_file" 2>/dev/null; then
            if caxton agents validate "$agent_file"; then
                echo "Git recovery successful for $agent_file"
                return 0
            else
                # Try earlier versions
                for i in {2..5}; do
                    if git checkout HEAD~$i "$agent_file" 2>/dev/null; then
                        if caxton agents validate "$agent_file"; then
                            echo "Git recovery successful (HEAD~$i) for $agent_file"
                            return 0
                        fi
                    fi
                done
            fi
        fi
    fi

    # Strategy 2: Restore from backup
    if [[ -f "$backup_dir/$agent_file" ]]; then
        echo "Restoring $agent_file from backup"
        cp "$backup_dir/$agent_file" "$full_path"
        if caxton agents validate "$agent_file"; then
            echo "Backup restoration successful for $agent_file"
            return 0
        fi
    fi

    # Strategy 3: Attempt automatic repair
    echo "Attempting automatic repair for $agent_file"
    if attempt_config_repair "$full_path"; then
        echo "Automatic repair successful for $agent_file"
        return 0
    fi

    # Strategy 4: Create minimal working version
    echo "Creating minimal working version for $agent_file"
    create_minimal_agent_config "$full_path"
}

attempt_config_repair() {
    local config_file=$1

    # Common repair strategies for YAML corruption

    # Fix 1: Remove null bytes
    sed -i 's/\x0//g' "$config_file"

    # Fix 2: Fix common YAML syntax errors
    sed -i 's/: $/: ""/g' "$config_file"  # Empty values
    sed -i 's/^\t/  /g' "$config_file"    # Convert tabs to spaces

    # Fix 3: Ensure proper YAML structure
    if ! grep -q '^---$' "$config_file"; then
        echo "---" | cat - "$config_file" > temp && mv temp "$config_file"
    fi

    # Test if repair worked
    caxton agents validate "$config_file"
}

create_minimal_agent_config() {
    local config_file=$1
    local agent_name=$(basename "$config_file" .md)

    cat > "$config_file" << EOF
---
name: $agent_name
version: "1.0.0"
capabilities:
  - general
tools: []
parameters: {}
resource_limits:
  memory_scope: "agent"
  max_conversations: 10
system_prompt: |
  You are a general purpose agent. Your configuration was recovered
  from corruption and needs to be properly configured.
---

# $agent_name Agent

This agent configuration was automatically recovered from corruption.
Please update the configuration with proper capabilities and tools.
EOF

    echo "Created minimal configuration for $agent_name"
}
```

```text

### Pattern 4: Hot-Reload Failure Recovery

#### Overview

Recover from failed hot-reload operations that leave agents in inconsistent states.

#### Implementation

```bash
#!/bin/bash
# Hot-reload failure recovery

recover_from_hotreload_failure() {
    local agent_name=$1
    local agents_dir="/var/lib/caxton/agents"

    echo "Recovering from hot-reload failure for agent: $agent_name"

    # 1. Check agent status
    local agent_status=$(caxton agents status "$agent_name" --format json 2>/dev/null)

    if [[ -z "$agent_status" ]]; then
        echo "Agent $agent_name not found in system"
        return 1
    fi

    # 2. Identify failure type
    local failure_type=$(identify_hotreload_failure_type "$agent_name")

    case "$failure_type" in
        "validation_failure")
            recover_from_validation_failure "$agent_name"
            ;;
        "memory_corruption")
            recover_from_memory_corruption "$agent_name"
            ;;
        "tool_loading_failure")
            recover_from_tool_failure "$agent_name"
            ;;
        "conversation_state_loss")
            recover_conversation_state "$agent_name"
            ;;
        *)
            echo "Unknown failure type: $failure_type"
            perform_full_agent_recovery "$agent_name"
            ;;
    esac
}

identify_hotreload_failure_type() {
    local agent_name=$1

    # Check logs for failure patterns
    local log_file="/var/log/caxton/caxton.log"

    if grep -q "validation_failed.*$agent_name" "$log_file"; then
        echo "validation_failure"
    elif grep -q "memory_corruption.*$agent_name" "$log_file"; then
        echo "memory_corruption"
    elif grep -q "tool_loading_error.*$agent_name" "$log_file"; then
        echo "tool_loading_failure"
    elif grep -q "conversation_state_lost.*$agent_name" "$log_file"; then
        echo "conversation_state_loss"
    else
        echo "unknown"
    fi
}

recover_from_validation_failure() {
    local agent_name=$1
    local config_file="/var/lib/caxton/agents/${agent_name}.md"

    echo "Recovering from validation failure for $agent_name"

    # 1. Suspend agent to prevent further issues
    caxton agents suspend "$agent_name" 2>/dev/null

    # 2. Restore configuration from known good state
    if restore_agent_config_from_backup "$agent_name"; then
        echo "Configuration restored from backup"
    elif restore_agent_config_from_git "$agent_name"; then
        echo "Configuration restored from git history"
    else
        echo "Creating safe minimal configuration"
        create_safe_minimal_config "$agent_name"
    fi

    # 3. Validate restored configuration
    if caxton agents validate "$config_file"; then
        echo "Validation successful, resuming agent"
        caxton agents resume "$agent_name"
    else
        echo "Validation still failing, keeping agent suspended"
        return 1
    fi
}

recover_from_memory_corruption() {
    local agent_name=$1

    echo "Recovering from memory corruption for $agent_name"

    # 1. Clear agent's memory scope
    caxton memory clear-agent-scope "$agent_name"

    # 2. Reload agent configuration
    caxton agents reload "$agent_name" --clear-memory

    # 3. Verify memory system integrity
    if caxton memory verify-agent "$agent_name"; then
        echo "Memory corruption recovery successful"
        return 0
    else
        echo "Memory corruption recovery failed"
        return 1
    fi
}

recover_from_tool_failure() {
    local agent_name=$1
    local config_file="/var/lib/caxton/agents/${agent_name}.md"

    echo "Recovering from tool loading failure for $agent_name"

    # 1. Check which tools are failing
    local failing_tools=$(caxton tools check-agent "$agent_name" --failing-only)

    # 2. Create backup of current config
    cp "$config_file" "${config_file}.tool-failure-backup"

    # 3. Remove failing tools from configuration
    remove_failing_tools_from_config "$config_file" "$failing_tools"

    # 4. Reload agent with reduced tool set
    if caxton agents reload "$agent_name"; then
        echo "Agent recovered with reduced tool set"
        echo "Failing tools removed: $failing_tools"
        return 0
    else
        # Restore original config
        mv "${config_file}.tool-failure-backup" "$config_file"
        echo "Tool failure recovery unsuccessful"
        return 1
    fi
}
```

```text

### Pattern 5: Embedding Model Recovery

#### Overview

Recover from embedding model corruption or missing model files.

#### Implementation

```bash
#!/bin/bash
# Embedding model recovery

recover_embedding_model() {
    local model_dir="/var/lib/caxton/models"
    local model_name="all-MiniLM-L6-v2"

    echo "Recovering embedding model: $model_name"

    # 1. Check model integrity
    if check_embedding_model_integrity "$model_dir/$model_name"; then
        echo "Embedding model is intact, no recovery needed"
        return 0
    fi

    echo "Embedding model corruption detected, recovering..."

    # 2. Stop server to prevent model access during recovery
    systemctl stop caxton

    # 3. Try recovery strategies in order
    if restore_model_from_backup "$model_name" "$model_dir"; then
        echo "Model restored from backup"
    elif download_fresh_model "$model_name" "$model_dir"; then
        echo "Fresh model downloaded"
    else
        echo "Model recovery failed"
        return 1
    fi

    # 4. Verify recovered model
    if verify_model_functionality "$model_dir/$model_name"; then
        echo "Model recovery successful"
    else
        echo "Model recovery verification failed"
        return 1
    fi

    # 5. Clear embedding cache (will be rebuilt)
    rm -rf /var/lib/caxton/data/embedding_cache/*

    # 6. Restart server
    systemctl start caxton

    # 7. Wait for model loading
    if wait_for_embedding_model_ready; then
        echo "Embedding model recovery completed successfully"
        return 0
    else
        echo "Embedding model failed to initialize after recovery"
        return 1
    fi
}

check_embedding_model_integrity() {
    local model_path=$1

    # Check if model files exist
    if [[ ! -d "$model_path" ]]; then
        echo "Model directory missing: $model_path"
        return 1
    fi

    # Check for required model files
    local required_files=("config.json" "pytorch_model.bin" "tokenizer.json" "vocab.txt")
    for file in "${required_files[@]}"; do
        if [[ ! -f "$model_path/$file" ]]; then
            echo "Missing model file: $file"
            return 1
        fi
    done

    # Check file sizes (basic corruption check)
    local config_size=$(stat -f%z "$model_path/config.json" 2>/dev/null || stat -c%s "$model_path/config.json")
    local model_size=$(stat -f%z "$model_path/pytorch_model.bin" 2>/dev/null || stat -c%s "$model_path/pytorch_model.bin")

    if [[ "$config_size" -lt 100 ]]; then
        echo "Config file too small, likely corrupted"
        return 1
    fi

    if [[ "$model_size" -lt 10000000 ]]; then  # 10MB minimum for the model
        echo "Model file too small, likely corrupted"
        return 1
    fi

    return 0
}

download_fresh_model() {
    local model_name=$1
    local model_dir=$2

    echo "Downloading fresh embedding model..."

    # Create temporary download directory
    local temp_dir=$(mktemp -d)

    # Download model using huggingface-hub or direct download
    if command -v huggingface-cli &> /dev/null; then
        huggingface-cli download "sentence-transformers/$model_name" --local-dir "$temp_dir"
    else
        # Fallback: direct download key files
        download_model_files "$model_name" "$temp_dir"
    fi

    # Verify downloaded model
    if check_embedding_model_integrity "$temp_dir"; then
        rm -rf "$model_dir/$model_name"
        mv "$temp_dir" "$model_dir/$model_name"
        echo "Model download successful"
        return 0
    else
        rm -rf "$temp_dir"
        echo "Downloaded model failed integrity check"
        return 1
    fi
}

verify_model_functionality() {
    local model_path=$1

    # Create a simple test script to verify model works
    cat > /tmp/test_embedding.py << 'EOF'
import sys
import torch
from sentence_transformers import SentenceTransformer

try:
    model = SentenceTransformer(sys.argv[1])
    embeddings = model.encode(["test sentence"])
    print(f"Model working: embedding shape {embeddings.shape}")
except Exception as e:
    print(f"Model test failed: {e}")
    sys.exit(1)
EOF

    if python3 /tmp/test_embedding.py "$model_path"; then
        rm -f /tmp/test_embedding.py
        return 0
    else
        rm -f /tmp/test_embedding.py
        return 1
    fi
}
```

```text

## Recovery Strategies by Scenario (Embedded Architecture)

### Scenario 1: Server Process Crash

```bash
#!/bin/bash
# Complete server process crash recovery

recover_from_server_crash() {
    echo "Initiating server crash recovery..."

    # 1. Assess system state
    assess_crash_damage

    # 2. Check embedded data integrity
    if ! check_embedded_data_integrity "/var/lib/caxton/data"; then
        echo "Data corruption detected, repairing..."
        repair_embedded_data "/var/lib/caxton/data"
    fi

    # 3. Validate configuration agents
    if ! validate_all_config_agents "/var/lib/caxton/agents"; then
        echo "Configuration issues detected, repairing..."
        repair_config_agents "/var/lib/caxton/agents"
    fi

    # 4. Check embedding model
    if ! check_embedding_model_integrity "/var/lib/caxton/models/all-MiniLM-L6-v2"; then
        echo "Embedding model issues detected, recovering..."
        recover_embedding_model
    fi

    # 5. Start server in recovery mode
    echo "Starting server in recovery mode..."
    systemctl start caxton

    # 6. Verify system recovery
    if verify_full_system_recovery; then
        echo "Server crash recovery completed successfully"
        return 0
    else
        echo "Server crash recovery failed"
        return 1
    fi
}

assess_crash_damage() {
    echo "Assessing crash damage..."

    # Check for core dumps
    if [[ -f "/var/lib/caxton/core" ]]; then
        echo "Core dump found, crash was severe"
        mv "/var/lib/caxton/core" "/var/log/caxton/core.$(date +%Y%m%d-%H%M)"
    fi

    # Check log file for crash indicators
    local log_file="/var/log/caxton/caxton.log"
    if grep -q "PANIC\|SEGFAULT\|SIGKILL" "$log_file"; then
        echo "Severe crash indicators found in logs"
    fi

    # Check file system corruption
    if ! fsck -n /var/lib/caxton/data &>/dev/null; then
        echo "File system corruption detected"
    fi
}

verify_full_system_recovery() {
    local max_wait=60
    local wait_time=0

    echo "Verifying system recovery..."

    # Wait for server startup
    while [[ $wait_time -lt $max_wait ]]; do
        if curl -sf http://localhost:8080/api/v1/health > /dev/null; then
            break
        fi
        sleep 5
        wait_time=$((wait_time + 5))
    done

    if [[ $wait_time -ge $max_wait ]]; then
        echo "Server failed to start within $max_wait seconds"
        return 1
    fi

    # Verify embedded memory system
    if ! caxton memory status | grep -q "healthy"; then
        echo "Memory system not healthy after recovery"
        return 1
    fi

    # Verify configuration agents
    local agent_count=$(caxton agents list --count)
    if [[ "$agent_count" -eq 0 ]]; then
        echo "No agents loaded after recovery"
        return 1
    fi

    # Verify embedding model
    if ! caxton memory model-status | grep -q "loaded"; then
        echo "Embedding model not loaded after recovery"
        return 1
    fi

    echo "All systems verified healthy after recovery"
    return 0
}
```

```text

### Scenario 2: Configuration Agent Corruption

```bash
#!/bin/bash
# Comprehensive configuration agent corruption recovery

recover_from_config_corruption() {
    local agents_dir="/var/lib/caxton/agents"

    echo "Recovering from configuration agent corruption..."

    # 1. Stop server to prevent further corruption
    systemctl stop caxton

    # 2. Identify corrupted agents
    local corrupted_agents=()
    for config_file in "$agents_dir"/*.md; do
        if [[ -f "$config_file" ]]; then
            local agent_name=$(basename "$config_file" .md)
            if ! caxton agents validate "$config_file"; then
                corrupted_agents+=("$agent_name")
            fi
        fi
    done

    echo "Found ${#corrupted_agents[@]} corrupted agents"

    # 3. Recovery strategies for each corrupted agent
    local recovered_count=0
    for agent_name in "${corrupted_agents[@]}"; do
        echo "Recovering agent: $agent_name"

        if recover_single_agent "$agent_name" "$agents_dir"; then
            echo "Successfully recovered: $agent_name"
            ((recovered_count++))
        else
            echo "Failed to recover: $agent_name"
        fi
    done

    echo "Recovered $recovered_count out of ${#corrupted_agents[@]} agents"

    # 4. Validate all configurations
    echo "Validating all configurations..."
    local validation_passed=true
    for config_file in "$agents_dir"/*.md; do
        if [[ -f "$config_file" ]]; then
            if ! caxton agents validate "$config_file"; then
                validation_passed=false
            fi
        fi
    done

    if [[ "$validation_passed" == "true" ]]; then
        echo "All configurations validated successfully"
    else
        echo "Some configurations still have issues"
    fi

    # 5. Restart server
    systemctl start caxton

    # 6. Verify agent loading
    if wait_for_agents_to_load; then
        echo "Configuration corruption recovery completed"
        return 0
    else
        echo "Configuration corruption recovery failed"
        return 1
    fi
}

wait_for_agents_to_load() {
    local max_wait=60
    local wait_time=0

    while [[ $wait_time -lt $max_wait ]]; do
        if curl -sf http://localhost:8080/api/v1/health > /dev/null; then
            local agent_count=$(caxton agents list --count 2>/dev/null || echo "0")
            if [[ "$agent_count" -gt 0 ]]; then
                echo "Agents loaded successfully: $agent_count"
                return 0
            fi
        fi
        sleep 5
        wait_time=$((wait_time + 5))
    done

    return 1
}
```

```text

### Scenario 3: Memory System Corruption

```bash
#!/bin/bash
# Memory system corruption recovery

recover_from_memory_corruption() {
    local data_dir="/var/lib/caxton/data"
    local backup_dir="/backup/caxton"

    echo "Recovering from memory system corruption..."

    # 1. Stop server for consistent recovery
    systemctl stop caxton

    # 2. Assess corruption extent
    local corruption_assessment=$(assess_memory_corruption "$data_dir")
    echo "Corruption assessment: $corruption_assessment"

    case "$corruption_assessment" in
        "minor")
            recover_minor_memory_corruption "$data_dir"
            ;;
        "major")
            recover_major_memory_corruption "$data_dir" "$backup_dir"
            ;;
        "catastrophic")
            recover_catastrophic_memory_corruption "$data_dir" "$backup_dir"
            ;;
        *)
            echo "Unable to assess corruption level"
            return 1
            ;;
    esac

    local recovery_status=$?

    # 3. Start server and verify recovery
    systemctl start caxton

    if [[ $recovery_status -eq 0 ]] && verify_memory_system_health; then
        echo "Memory corruption recovery successful"
        return 0
    else
        echo "Memory corruption recovery failed"
        return 1
    fi
}

assess_memory_corruption() {
    local data_dir=$1
    local db_file="$data_dir/memory.db"

    # Check if database exists
    if [[ ! -f "$db_file" ]]; then
        echo "catastrophic"
        return
    fi

    # Run integrity check
    local integrity_result=$(sqlite3 "$db_file" "PRAGMA integrity_check;" 2>/dev/null)

    if [[ "$integrity_result" == "ok" ]]; then
        echo "none"
        return
    fi

    # Count corrupted pages
    local corruption_count=$(echo "$integrity_result" | grep -c "corrupt")

    if [[ $corruption_count -le 5 ]]; then
        echo "minor"
    elif [[ $corruption_count -le 20 ]]; then
        echo "major"
    else
        echo "catastrophic"
    fi
}

recover_minor_memory_corruption() {
    local data_dir=$1
    local db_file="$data_dir/memory.db"

    echo "Attempting minor corruption repair..."

    # Try SQLite's built-in repair
    sqlite3 "$db_file" << 'EOF'
PRAGMA writable_schema = ON;
UPDATE sqlite_master SET sql = (
    SELECT sql FROM sqlite_master
    WHERE name = sqlite_master.name
) WHERE type = 'table';
PRAGMA writable_schema = OFF;
REINDEX;
ANALYZE;
EOF

    # Verify repair
    if sqlite3 "$db_file" "PRAGMA integrity_check;" | grep -q "ok"; then
        echo "Minor corruption repair successful"
        return 0
    else
        echo "Minor corruption repair failed"
        return 1
    fi
}

recover_major_memory_corruption() {
    local data_dir=$1
    local backup_dir=$2
    local db_file="$data_dir/memory.db"

    echo "Attempting major corruption recovery..."

    # Try selective data recovery
    local temp_db="${db_file}.recovery"

    # Extract recoverable data
    sqlite3 "$db_file" << EOF
.mode insert
.output ${temp_db}.entities.sql
SELECT * FROM entities WHERE rowid NOT IN (
    SELECT rowid FROM entities WHERE
    name IS NULL OR entity_type IS NULL
);
.output ${temp_db}.relations.sql
SELECT * FROM relations WHERE from_entity IS NOT NULL AND to_entity IS NOT NULL;
.output ${temp_db}.observations.sql
SELECT * FROM observations WHERE entity_id IS NOT NULL;
EOF

    # Create new database with recovered data
    caxton storage init-schema "$temp_db"
    sqlite3 "$temp_db" ".read ${temp_db}.entities.sql"
    sqlite3 "$temp_db" ".read ${temp_db}.relations.sql"
    sqlite3 "$temp_db" ".read ${temp_db}.observations.sql"

    # Verify recovered database
    if sqlite3 "$temp_db" "PRAGMA integrity_check;" | grep -q "ok"; then
        mv "$db_file" "${db_file}.corrupt.$(date +%s)"
        mv "$temp_db" "$db_file"
        rm -f ${temp_db}.*.sql
        echo "Major corruption recovery successful"
        return 0
    else
        rm -f "$temp_db" ${temp_db}.*.sql
        echo "Major corruption recovery failed, trying backup restore"
        restore_from_backup "$backup_dir" "$data_dir"
    fi
}

recover_catastrophic_memory_corruption() {
    local data_dir=$1
    local backup_dir=$2

    echo "Attempting catastrophic corruption recovery..."

    # Total data loss - restore from backup
    if [[ -f "$backup_dir/latest-memory.db" ]]; then
        echo "Restoring from backup..."
        cp "$backup_dir/latest-memory.db" "$data_dir/memory.db"

        # Verify backup integrity
        if sqlite3 "$data_dir/memory.db" "PRAGMA integrity_check;" | grep -q "ok"; then
            echo "Catastrophic recovery from backup successful"
            return 0
        fi
    fi

    # Last resort - create fresh database
    echo "Creating fresh database (data loss)..."
    rm -f "$data_dir/memory.db"
    caxton storage init-schema "$data_dir/memory.db"

    echo "Fresh database created - all previous memory data lost"
    return 0
}

verify_memory_system_health() {
    local max_wait=30
    local wait_time=0

    while [[ $wait_time -lt $max_wait ]]; do
        if caxton memory status 2>/dev/null | grep -q "healthy"; then
            return 0
        fi
        sleep 5
        wait_time=$((wait_time + 5))
    done

    return 1
}
```

```text

## Recovery Testing

### Chaos Engineering Tests

```rust
#[cfg(test)]
mod recovery_tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_crash_recovery() {
        let mut system = TestSystem::new();
        let agent = system.spawn_agent("test-agent").await;

        // Create some state
        agent.process_messages(generate_test_messages(100)).await;

        // Simulate crash
        system.crash_agent(&agent.id).await;

        // Attempt recovery
        let recovered = recover_crashed_agent(agent.id.clone()).await.unwrap();

        // Verify state consistency
        assert_eq!(recovered.processed_count(), 100);
        assert!(recovered.is_consistent());
    }

    #[tokio::test]
    async fn test_partition_recovery() {
        let mut cluster = TestCluster::new(5);

        // Create partition
        let (partition_a, partition_b) = cluster.create_partition(2, 3).await;

        // Process messages in both partitions
        partition_a.process_messages(100).await;
        partition_b.process_messages(100).await;

        // Heal partition
        cluster.heal_partition().await;

        // Verify eventual consistency
        tokio::time::sleep(Duration::from_secs(5)).await;
        assert!(cluster.is_consistent());
    }
}
```

### Recovery Benchmarks

```rust
#[bench]
fn bench_checkpoint_recovery(b: &mut Bencher) {
    let checkpoint = create_large_checkpoint(1_000_000_events);

    b.iter(|| {
        let _ = Agent::restore_from_checkpoint(checkpoint.clone());
    });
}

#[bench]
fn bench_event_replay(b: &mut Bencher) {
    let events = generate_events(10_000);

    b.iter(|| {
        let _ = replay_events(events.clone());
    });
}
```

## Monitoring Recovery Operations

### Key Metrics

```rust
pub struct RecoveryMetrics {
    pub recovery_time: Histogram,
    pub recovered_agents: Counter,
    pub failed_recoveries: Counter,
    pub data_loss_events: Counter,
    pub checkpoint_size: Histogram,
    pub replay_speed: Gauge,
}
```

### Recovery Dashboards

```yaml
recovery_dashboard:
  panels:
    - title: "Recovery Time (p95)"
      query: "histogram_quantile(0.95, caxton_recovery_time_seconds)"

    - title: "Recovery Success Rate"
      query: "rate(caxton_recovered_agents[5m]) / rate(caxton_recovery_attempts[5m])"

    - title: "Data Loss Events"
      query: "increase(caxton_data_loss_events[1h])"

    - title: "Checkpoint Sizes"
      query: "caxton_checkpoint_size_bytes"
```

## Best Practices

### 1. Checkpoint Frequency

- Balance between recovery time and overhead
- More frequent for critical agents
- Less frequent for stateless agents

### 2. State Minimization

- Keep agent state minimal
- Store only essential data
- Use references for large objects

### 3. Idempotent Operations

- Ensure operations can be safely retried
- Use unique operation IDs
- Check for duplicate processing

### 4. Graceful Degradation

- Continue operating with reduced functionality
- Prioritize critical operations
- Queue non-critical work for later

### 5. Testing Recovery Paths

- Regular disaster recovery drills
- Automated chaos testing
- Monitor recovery metrics

## Troubleshooting Guide

### Common Issues and Solutions

#### Issue: Slow Recovery

**Symptoms**: Recovery takes longer than RTO **Solutions**:

- Increase checkpoint frequency
- Optimize event replay
- Use parallel recovery
- Add more snapshots

#### Issue: State Inconsistency

**Symptoms**: Agents have different views of state **Solutions**:

- Implement vector clocks
- Use CRDTs for convergence
- Add state reconciliation phase
- Increase consistency checks

#### Issue: Message Loss

**Symptoms**: Missing messages after recovery **Solutions**:

- Implement message persistence
- Add acknowledgment tracking
- Use reliable message queues
- Implement replay from source

## References

- [ADR-0028: Configuration-Driven Agent Architecture](../adr/0028-configuration-driven-agent-architecture.md)
- [ADR-0030: Embedded Memory System](../adr/0030-embedded-memory-system.md)
- [ADR-0029: Lightweight Agent Messaging](../adr/0029-fipa-acl-lightweight-messaging.md)
- [Operational Runbook](operational-runbook.md)
- [Agent Lifecycle Management](agent-lifecycle-management.md)
- [Performance Tuning Guide](performance-tuning.md)
