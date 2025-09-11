---
title: "Operational Runbook"
date: 2025-01-15
layout: page
categories: [Operations]
---

This runbook provides step-by-step procedures for operating Caxton in
production with the embedded, zero-dependency architecture (ADRs 28-30).

## Quick Reference

| Situation | Command | Page |
|-----------|---------|------|
| Server not responding | `curl http://localhost:8080/api/v1/health` | [Health Checks](#health-checks) |
| Deploy config agent | Create markdown file in agents/ | [Config Agent Deployment](#config-agent-deployment) |
| Deploy WASM agent | `curl -X POST /api/v1/agents` | [WASM Agent Deployment](#wasm-agent-deployment) |
| List agents | `curl /api/v1/agents` | [Agent Management](#agent-management) |
| Memory performance | `caxton memory stats` | [Memory Optimization](#memory-performance) |
| Backup embedded data | `caxton backup --embedded` | [Backup Procedures](#backup-procedures) |
| Hot-reload config | `caxton reload --agent <name>` | [Config Agent Operations](#config-agent-operations) |
| Emergency stop | `caxton shutdown` | [Emergency Procedures](#emergency-procedures) |

## Core Operations

### Health Checks

**Purpose**: Verify Caxton server is running and responsive with embedded
architecture

#### Basic Health Check

```bash
# Check server health (embedded architecture)
curl http://localhost:8080/api/v1/health

# Expected response:
# {"status":"healthy","memory_backend":"embedded","agents":5}

# Check embedded memory system
curl http://localhost:8080/api/v1/health/memory

# Expected response:
# {"status":"healthy","sqlite_status":"ok","embedding_model":"loaded"}

# Automated monitoring script for zero-dependency deployment
#!/bin/bash
while true; do
    if curl -f -s http://localhost:8080/api/v1/health > /dev/null; then
        echo "$(date): Caxton healthy"
    else
        echo "$(date): Caxton DOWN - checking single process..."
        ps aux | grep caxton
    fi
    sleep 30
done
```

#### Troubleshooting API Issues

```bash
# If health check fails (embedded architecture troubleshooting):

# 1. Check if single Caxton process is running
ps aux | grep caxton

# 2. Check if port 8080 is listening
netstat -tlnp | grep 8080

# 3. Check embedded storage integrity
caxton storage verify --sqlite-path ./data/caxton.db

# 4. Check embedding model status
caxton memory model-status

# 5. Check server logs
tail -f /var/log/caxton/caxton.log

# 6. Restart server (zero-dependency)
caxton start
# OR with systemd:
systemctl restart caxton
```

## Agent Operations

### Config Agent Deployment

**Purpose**: Deploy configuration-driven agents (ADR-0028) - the primary
user experience

#### Deploy Config Agent (Primary Method)

```bash
# 1. Create agent configuration file
cat > agents/data-analyzer.md << 'EOF'
---
name: DataAnalyzer
version: "1.0.0"
capabilities:
  - data-analysis
  - report-generation
tools:
  - http_client
  - csv_parser
parameters:
  max_file_size: "10MB"
system_prompt: |
  You are a data analysis expert who helps users understand their data.
---

# DataAnalyzer Agent

This agent specializes in data analysis and can fetch data from URLs,
parse various formats, and generate visualizations.
EOF

# 2. Hot-reload the agent (zero-downtime deployment)
caxton agents reload data-analyzer

# 3. Verify deployment
caxton agents list | grep DataAnalyzer

# 4. Test agent capability
curl -X POST http://localhost:8080/api/v1/agents/data-analyzer/message \
  -H "Content-Type: application/json" \
  -d '{"message": "Analyze sales data at example.com/data.csv"}'
```

#### Deploy Config Agent with Error Handling

```bash
deploy_config_agent() {
    local AGENT_NAME=$1
    local AGENT_FILE=$2

    # Validate configuration file
    if ! caxton agents validate "$AGENT_FILE"; then
        echo "ERROR: Invalid agent configuration"
        return 1
    fi

    # Deploy with hot-reload
    if caxton agents deploy "$AGENT_FILE" --hot-reload; then
        echo "Config agent '$AGENT_NAME' deployed successfully"
        caxton agents status "$AGENT_NAME"
    else
        echo "Deployment failed for '$AGENT_NAME'"
        return 1
    fi
}

# Usage
deploy_config_agent "data-analyzer" "agents/data-analyzer.md"
```

#### Common Deployment Errors

#### Common Config Agent Deployment Errors

| Error | Cause | Solution |
|-------|-------|----------|
| YAML Parse Error | Invalid frontmatter syntax | Validate YAML structure |
| Missing Required Field | name, capabilities missing | Add required YAML fields |
| Invalid Tool Reference | Unknown tool in tools list | Check available tools with `caxton tools list` |
| Capability Conflict | Agent name conflicts with existing | Choose unique agent name |
| File Not Found | Agent file path incorrect | Verify file exists in agents/ directory |

### WASM Agent Deployment (Advanced Use Cases)

**Purpose**: Deploy compiled WebAssembly agents for power users requiring
custom algorithms

#### Deploy WASM Agent

```bash
# Compile your agent to WASM first (example with Rust)
cargo build --target wasm32-wasi --release
cp target/wasm32-wasi/release/my-agent.wasm ./

# Deploy via REST API
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"custom-algorithm-agent\",
    \"wasm_module\": \"$(base64 -w0 my-agent.wasm)\",
    \"resource_limits\": {
      \"max_memory_bytes\": 10485760,
      \"max_fuel\": 1000000,
      \"max_execution_time_ms\": 5000
    }
  }"
```

### Agent Management

**Purpose**: Monitor and manage both config and WASM agents

#### List All Agents

```bash
# Get all agents (config and WASM)
curl http://localhost:8080/api/v1/agents | jq '.'

# Filter by agent type
caxton agents list --type config
caxton agents list --type wasm

# Count agents by type
CONFIG_COUNT=$(caxton agents list --type config --count)
WASM_COUNT=$(caxton agents list --type wasm --count)
echo "Config agents: $CONFIG_COUNT, WASM agents: $WASM_COUNT"

# Monitor agent status
watch -n 5 'caxton agents status --summary'
```

#### Get Agent Details

```bash
# Get specific agent details
AGENT_NAME="data-analyzer"
caxton agents show "$AGENT_NAME"

# Get agent via REST API (both config and WASM)
AGENT_ID="550e8400-e29b-41d4-a716-446655440000"
curl http://localhost:8080/api/v1/agents/$AGENT_ID | jq '.'

# Check agent health and capabilities
check_agent() {
    local AGENT_NAME=$1
    if caxton agents ping "$AGENT_NAME"; then
        echo "Agent $AGENT_NAME is healthy"
        caxton agents capabilities "$AGENT_NAME"
        return 0
    else
        echo "Agent $AGENT_NAME is not responding"
        return 1
    fi
}
```

## Config Agent Operations

**Purpose**: Hot-reload and manage configuration-driven agents

### Config Agent Hot-Reload

```bash
# Reload single agent after config changes
caxton agents reload data-analyzer

# Reload all config agents
caxton agents reload --all-config

# Validate config before reload
caxton agents validate agents/data-analyzer.md
if [ $? -eq 0 ]; then
    caxton agents reload data-analyzer
else
    echo "Configuration validation failed"
fi

# Monitor reload status
caxton agents reload-status data-analyzer
```

### Config Agent Development Workflow

```bash
# Development mode with auto-reload
caxton agents watch agents/ --auto-reload

# Test agent locally
caxton agents test data-analyzer --input "Analyze this CSV data"

# Check agent logs
caxton agents logs data-analyzer --tail 50

# Debug agent capabilities
caxton agents debug data-analyzer --show-tools
```

## Memory Performance

**Purpose**: Monitor and optimize embedded memory system (ADR-0030)

### Embedded Memory Operations

```bash
# Check memory system status
caxton memory status
# Output:
# SQLite database: healthy (1.2MB)
# Embedding model: All-MiniLM-L6-v2 loaded
# Entities: 1,247 stored
# Relations: 3,891 connections
# Vector cache: 89% hit rate

# Memory system statistics
caxton memory stats --detailed
# Semantic search latency P99: 15ms
# Graph traversal P99: 8ms
# Storage usage: 45MB total
# Memory baseline: 203MB (embedding model)

# Optimize memory performance
caxton memory optimize --vacuum-sqlite --rebuild-index

# Monitor memory queries
caxton memory monitor --show-slow-queries
```

### Memory System Scaling

```bash
# Check if approaching embedded limits
caxton memory limits-check
# Entity count: 85,000 / 100,000 (85% of recommended limit)
# WARNING: Consider external backend migration

# Migrate to external backend if needed
caxton memory migrate --to qdrant --config qdrant.yaml
# OR
caxton memory migrate --to neo4j --config neo4j.yaml

# Export memory data for backup/migration
caxton memory export --format json --output memory-backup.json
```

## Initial Setup

### Prerequisites Checklist (Zero-Dependency Deployment)

- [ ] Caxton binary installed (single executable)
- [ ] Port 8080 available (REST API)
- [ ] Write permissions for data directory
- [ ] ~200MB RAM for embedding model
- [ ] Agents directory created (for config agents)
- [ ] Optional: Reverse proxy for production (nginx/caddy)

### First-Time Bootstrap

#### Zero-Dependency Server Bootstrap

```bash
#!/bin/bash
# Bootstrap Caxton with embedded architecture

# 1. Initialize data directory
caxton init --data-dir ./data --agents-dir ./agents

# 2. Start server (embedded memory loads automatically)
caxton start --port 8080 --data-dir ./data

# 3. Verify embedded systems
curl http://localhost:8080/api/v1/health
# Expected: {"status":"healthy","memory_backend":"embedded"}

# 4. Deploy first config agent
cat > agents/greeter.md << 'EOF'
---
name: Greeter
version: "1.0.0"
capabilities:
  - greeting
tools: []
system_prompt: |
  You are a friendly greeter who welcomes users.
---
# Greeter Agent
I help welcome users to Caxton!
EOF

# 5. Load the agent
caxton agents reload greeter
caxton agents list
```

#### Production Configuration

```bash
#!/bin/bash
# Production deployment with embedded architecture

# 1. Create production configuration
cat > caxton.yaml << 'EOF'
server:
  port: 8080
  host: "0.0.0.0"
  data_dir: "/var/lib/caxton/data"
  agents_dir: "/var/lib/caxton/agents"

memory:
  backend: "embedded"  # SQLite + Candle
  sqlite_path: "memory.db"
  embedding_model: "all-MiniLM-L6-v2"
  cleanup_interval: "1h"

logging:
  level: "info"
  file: "/var/log/caxton/caxton.log"

monitoring:
  metrics_port: 9090
  health_check_interval: "30s"
EOF

# 2. Create systemd service
sudo tee /etc/systemd/system/caxton.service << 'EOF'
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

[Install]
WantedBy=multi-user.target
EOF

# 3. Enable and start
sudo systemctl enable caxton
sudo systemctl start caxton
```

## Troubleshooting

### Server Failure (Single Process)

#### Detection

```bash
# Check server process status
ps aux | grep caxton
sudo systemctl status caxton

# Check if process crashed:
# SERVER: stopped    SINCE: 45s ago    AGENTS: 12 (suspended)
```

#### Diagnosis

```bash
# 1. Check system resources
top -bn1 | head -5
df -h /var/lib/caxton/

# 2. Check embedded database integrity
caxton storage verify --sqlite-path /var/lib/caxton/data/memory.db

# 3. Check embedding model status
ls -la /var/lib/caxton/models/

# 4. Review logs for crashes
tail -100 /var/log/caxton/caxton.log | grep -E "ERROR|PANIC|crash"
```

#### Recovery

```bash
# Option 1: Restart the service
sudo systemctl restart caxton

# Option 2: Repair embedded database if corrupted
caxton storage repair --sqlite-path /var/lib/caxton/data/memory.db
sudo systemctl start caxton

# Option 3: Restore from backup if needed
caxton restore --backup /backups/caxton-latest.tar.gz
sudo systemctl start caxton

# Verify recovery
caxton agents list
caxton memory status
```

### Memory Performance Issues

#### Detection

```bash
# Check memory system performance
caxton memory stats

# If performance degrades:
METRIC                    TARGET    ACTUAL    STATUS
Semantic search P99       50ms      150ms     ✗ DEGRADED
SQLite query P99          20ms      80ms      ✗ DEGRADED
Embedding generation      100/s     25/s      ✗ DEGRADED
```

#### Diagnosis

```bash
# 1. Check embedded database performance
caxton memory diagnose --show-slow-queries
# QUERY-TYPE        AVG-LATENCY    COUNT
# semantic_search   120ms          1,247
# graph_traversal   85ms           892

# 2. Check SQLite database size and fragmentation
du -h /var/lib/caxton/data/memory.db
sqlite3 /var/lib/caxton/data/memory.db "PRAGMA integrity_check;"

# 3. Check embedding model memory usage
caxton memory model-stats
# MODEL: All-MiniLM-L6-v2
# MEMORY: 245MB (baseline: 203MB)
# CACHE_HIT_RATE: 67% (target: >85%)

# 4. Check if approaching embedded limits
caxton memory capacity-check
# ENTITIES: 95,000 / 100,000 (95% - approaching limit)
```

#### Mitigation

```bash
# 1. Optimize embedded database
caxton memory optimize --vacuum --reindex

# 2. Clear embedding cache and rebuild
caxton memory cache-clear --rebuild

# 3. Clean up old/unused entities
caxton memory cleanup --remove-orphaned --older-than 30d

# 4. If at capacity limits, migrate to external backend
if [ $(caxton memory capacity-check --percentage) -gt 90 ]; then
    echo "Migrating to external backend..."
    caxton memory migrate --to qdrant --config production-qdrant.yaml
fi
```

### Data Corruption

#### Detection

```bash
# Check data integrity
caxton storage verify --deep

# Output if corrupted:
ERROR: Data corruption detected
SQLite database: 3 corrupted pages
Embedding vectors: 127 orphaned entries
```

#### Automatic Recovery

```bash
# Enable safe mode (read-only) during corruption
caxton storage safe-mode --enable

# Status check in safe mode
caxton status
# Status: SAFE_MODE (read-only)
# Agents: 12 suspended
# Memory: Corruption detected, repairs needed
```

#### Repair Process

```bash
# 1. Attempt automatic repair
caxton storage repair --auto

# 2. If auto-repair fails, restore from backup
caxton storage restore --backup \
  /backups/caxton-$(date -d yesterday +%Y%m%d).tar.gz

# 3. Verify repair success
caxton storage verify --full
# [INFO] Repair completed successfully
# [INFO] SQLite integrity: OK
# [INFO] Embedding vectors: 98,547 verified

# 4. Disable safe mode and resume
caxton storage safe-mode --disable
caxton agents resume --all
```

## Maintenance Procedures

### Server Upgrade (Single Process)

#### Pre-Upgrade Checklist

- [ ] Embedded data backup completed
- [ ] Config agents backed up
- [ ] Upgrade tested in staging
- [ ] Rollback binary available
- [ ] Minimal downtime window scheduled (~30 seconds)

#### Upgrade Process

```bash
# 1. Create backup before upgrade
caxton backup --embedded --output /backups/pre-upgrade-$(date +%Y%m%d).tar.gz

# 2. Stop server gracefully
sudo systemctl stop caxton
# Graceful shutdown: agents suspended, memory synced

# 3. Replace binary
sudo cp /tmp/caxton-v1.2.0 /usr/local/bin/caxton
sudo chmod +x /usr/local/bin/caxton

# 4. Start with embedded data migration if needed
sudo systemctl start caxton
# Auto-migrates embedded schema if needed

# 5. Verify upgrade
caxton version
caxton storage verify
caxton agents list

# 6. Monitor for issues
tail -f /var/log/caxton/caxton.log
```

### Backup Procedures

#### Embedded Data Backup

```bash
# Full backup of embedded systems
caxton backup --embedded \
  --output /backups/caxton-$(date +%Y%m%d-%H%M).tar.gz \
  --compress

# Backup includes:
# - SQLite database
# - Embedding model cache
# - Config agent definitions
# - Server configuration

# Automated daily backup
crontab -e
# Add: 0 2 * * * /usr/local/bin/caxton backup --embedded --output \
#   /backups/daily/caxton-$(date +\%Y\%m\%d).tar.gz

# Verify backup integrity
caxton backup verify /backups/caxton-20250110-0200.tar.gz
```

#### Recovery from Backup

```bash
# 1. Stop server
sudo systemctl stop caxton

# 2. Clear current data
sudo rm -rf /var/lib/caxton/data/*
sudo rm -rf /var/lib/caxton/agents/*

# 3. Restore from backup
caxton restore /backups/caxton-20250110-0200.tar.gz \
  --target /var/lib/caxton/

# 4. Verify restore
caxton storage verify --full
caxton agents validate-all

# 5. Start server
sudo systemctl start caxton

# 6. Verify all agents loaded
caxton agents list
caxton memory status
```

### Config Agent Maintenance

```bash
# 1. Validate all config agents
caxton agents validate-all
# Reports any YAML syntax errors or missing tools

# 2. Update agent tool permissions
caxton agents audit-tools --show-unused
# Lists tools declared but not used

# 3. Backup config agents separately
tar -czf /backups/agents-$(date +%Y%m%d).tar.gz /var/lib/caxton/agents/

# 4. Update all agents from git repository
cd /var/lib/caxton/agents/
git pull origin main
caxton agents reload --all-config
```

## Emergency Procedures

### Emergency Shutdown

```bash
# Graceful emergency stop
caxton shutdown --graceful --timeout 30s
# Suspends agents, flushes memory, saves state

# Immediate emergency stop
caxton shutdown --immediate
# Force kills server process

# Emergency stop with state dump
caxton shutdown --dump-state --output /backup/emergency/$(date +%s).tar.gz
```

### Embedded Data Recovery

```bash
# 1. Detect corruption level
caxton storage diagnose --corruption-level
# CORRUPTION_LEVEL: HIGH (75% of data affected)

# 2. Enable safe mode
caxton storage safe-mode --enable

# 3. Attempt graduated recovery
if [ $(caxton storage corruption-level) -lt 25 ]; then
    # Light corruption: repair in place
    caxton storage repair --in-place
else
    # Heavy corruption: restore from backup
    caxton restore --latest-backup --verify
fi

# 4. Verify recovery
caxton storage verify --comprehensive
```

### Memory System Crisis

```bash
# 1. Check memory usage breakdown
caxton memory usage --detailed
# COMPONENT          USAGE    LIMIT     STATUS
# Embedding model    203MB    -         OK
# SQLite cache       45MB     100MB     OK
# Entity store       156MB    500MB     OK
# Vector cache       89MB     200MB     OK

# 2. If approaching limits, emergency cleanup
if [ $(caxton memory usage --percentage) -gt 95 ]; then
    # Emergency memory cleanup
    caxton memory cleanup --aggressive --force
    caxton memory cache-clear --all
fi

# 3. If still critical, migrate to external backend
caxton memory emergency-migrate --to qdrant --minimal-downtime
```

## Monitoring and Alerting

### Key Metrics to Watch (Embedded Architecture)

```bash
# Server health (single process)
curl -s localhost:9090/metrics | grep caxton_server_
# caxton_server_uptime_seconds 86400
# caxton_server_restarts_total 0
# caxton_server_memory_usage_bytes 256000000

# Embedded memory system
curl -s localhost:9090/metrics | grep caxton_memory_
# caxton_memory_entities_total 12450
# caxton_memory_relations_total 38912
# caxton_memory_search_latency_p99 0.023
# caxton_memory_sqlite_size_bytes 45678912

# Agent performance (config + WASM)
curl -s localhost:9090/metrics | grep caxton_agent_
# caxton_agent_config_count 8
# caxton_agent_wasm_count 2
# caxton_agent_reload_success_total 156
# caxton_agent_response_latency_p99 0.089
```

### Alert Response

#### Critical Alerts

##### Memory System Failure

```bash
# Immediate response
caxton memory emergency-diagnosis

# Check specific failure mode
caxton storage verify --quick

# Initiate recovery
if caxton storage can-repair; then
    caxton storage repair --emergency
else
    caxton restore --latest-backup --force
fi
```

##### Config Agent Failures

```bash
# Stop problematic agents
caxton agents suspend --failing-only

# Identify configuration issues
caxton agents diagnose --show-errors

# Gradual restart with validation
caxton agents reload --validate-first --gradual
```

### Debug Commands (Embedded Architecture)

```bash
# Trace agent conversations
caxton trace --agent data-analyzer --conversation-id abc-123

# Profile memory system performance
caxton memory profile --duration 60s --include-queries

# Dump complete system state
caxton debug dump --embedded --output debug-$(date +%s).tar.gz

# Analyze config agent patterns
caxton agents analyze --pattern "error|timeout" --last 1h

# Check configuration consistency
caxton config verify --agents-dir /var/lib/caxton/agents/

# SQLite database analysis
caxton storage analyze --show-indexes --show-fragmentation
```

### Common Issues (Embedded Architecture)

| Symptom | Likely Cause | Solution |
|---------|--------------|----------|
| Config agents not loading | YAML syntax error | Run `caxton agents validate-all` |
| Slow memory searches | SQLite fragmentation | Run `caxton memory optimize --vacuum` |
| High memory usage | Embedding cache full | Run `caxton memory cache-clear` |
| Server won't start | Data corruption | Run `caxton storage verify --repair` |
| Agent hot-reload fails | File permissions | Check write access to agents/ directory |
| Performance degraded | Approaching capacity limits | Check `caxton memory capacity-check` |

## Best Practices (Embedded Architecture)

1. **Backup embedded data daily** (automated cron job)
2. **Monitor SQLite database size** (approaching 100K entities limit)
3. **Validate config agents before deployment** (YAML lint)
4. **Use semantic versioning for agent configs** (track changes)
5. **Monitor memory system performance** (search latency)
6. **Plan external backend migration** (before hitting capacity)
7. **Test hot-reload in staging** (validate config changes)
8. **Maintain embedding cache efficiency** (>85% hit rate)

## References

- [Configuration Agent Guide](../user-guide/config-agents.md)
- [Embedded Memory System](../adr/0030-embedded-memory-system.md)
- [Configuration-Driven Architecture](../adr/0028-configuration-driven-agent-architecture.md)
- [FIPA Lightweight Messaging](../adr/0029-fipa-acl-lightweight-messaging.md)
- [Performance Tuning](performance-tuning.md)
- [Agent Lifecycle Management](agent-lifecycle-management.md)
