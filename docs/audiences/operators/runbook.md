---
title: "Operations Runbook"
description: "Complete operational procedures for Caxton multi-agent server"
date: 2025-01-15
layout: page
categories: [Operations, SysOps]
nav_order: 1
parent: Operators
---

This runbook provides comprehensive step-by-step procedures for operating
Caxton in production environments. Focused on the embedded, zero-dependency
architecture following ADRs 28-30.

## Quick Reference

| Situation | Command | Section |
|-----------|---------|---------|
| Server health check | `curl localhost:8080/api/v1/health` | [Health](#health-monitoring) |
| Deploy config agent | Create `.toml` in `agents/` | [Config Agents](#config-agent-deployment) |
| Deploy WASM agent | `curl -X POST /api/v1/agents` | [WASM Agents](#wasm-agent-deployment) |
| List all agents | `curl /api/v1/agents` | [Management](#agent-management) |
| Memory diagnostics | `caxton memory stats` | [Memory](#memory-operations) |
| Backup system | `caxton backup --embedded` | [Backup](#backup-procedures) |
| Hot-reload agent | `caxton reload --agent <name>` | [Operations](#config-agent-operations) |
| Emergency stop | `caxton shutdown` | [Emergency](#emergency-procedures) |

## Health Monitoring

### System Health Checks

**Primary health endpoint verification:**

```bash
# Core health check (returns in < 100ms)
curl -f http://localhost:8080/api/v1/health

# Expected response:
# {"status":"healthy","memory_backend":"embedded","agents":5}

# Detailed health with component status
curl http://localhost:8080/api/v1/health/detailed

# Expected response:
# {
#   "status": "healthy",
#   "components": {
#     "memory_system": "ok",
#     "sqlite_db": "ok",
#     "embedding_model": "loaded",
#     "agents": {"config": 8, "wasm": 2}
#   },
#   "uptime_seconds": 3600
# }
```

**Automated monitoring script:**

```bash
#!/bin/bash
# Production health monitoring with alerting
HEALTH_URL="http://localhost:8080/api/v1/health"
LOG_FILE="/var/log/caxton/health.log"
ALERT_EMAIL="ops@company.com"

check_health() {
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    if response=$(curl -f -s --max-time 10 "$HEALTH_URL"); then
        echo "$timestamp: Caxton healthy" >> "$LOG_FILE"
        return 0
    else
        echo "$timestamp: Caxton DOWN - investigating..." >> "$LOG_FILE"

        # Check process status
        if ! pgrep -f caxton > /dev/null; then
            echo "$timestamp: Process not running" >> "$LOG_FILE"
            systemctl status caxton >> "$LOG_FILE"

            # Send alert
            echo "Caxton process down on $(hostname)" | \
                mail -s "CRITICAL: Caxton Down" "$ALERT_EMAIL"
        fi

        return 1
    fi
}

# Run continuously with 30s intervals
while true; do
    check_health
    sleep 30
done
```

### Component Health Verification

```bash
# Memory system health check
curl http://localhost:8080/api/v1/health/memory
# Response: {"sqlite_status":"ok","embedding_model":"loaded","entities":1247}

# Agent system health check
curl http://localhost:8080/api/v1/health/agents
# Response: {"config_agents":8,"wasm_agents":2,"failing":0}

# Resource utilization check
curl http://localhost:8080/api/v1/health/resources
# Response: {"memory_mb":256,"cpu_percent":15,"disk_gb":45}
```

### Troubleshooting Health Issues

```bash
# If health checks fail, systematic diagnosis:

# 1. Verify process is running
ps aux | grep caxton | grep -v grep

# 2. Check port binding
ss -tlnp | grep 8080

# 3. Check log for errors
tail -50 /var/log/caxton/caxton.log | grep -E "ERROR|PANIC|FATAL"

# 4. Check system resources
df -h /var/lib/caxton/
free -h
top -bn1 | head -5

# 5. Verify embedded database
caxton storage verify --quick

# 6. Restart if needed
systemctl restart caxton

# 7. Verify recovery
sleep 10
curl http://localhost:8080/api/v1/health
```

## Config Agent Deployment

### Standard Deployment Process

**Create and deploy configuration-driven agent (5-10 minute process):**

```bash
# 1. Create agent definition file
cat > /var/lib/caxton/agents/data-analyzer.toml << 'EOF'
name = "DataAnalyzer"
version = "1.0.0"
capabilities = ["data-analysis", "report-generation"]
tools = ["http_client", "csv_parser", "chart_generator"]

[memory]
enabled = true
scope = "workspace"

[parameters]
max_file_size = "10MB"
supported_formats = ["csv", "json", "xlsx"]

system_prompt = '''
You are a data analysis expert who helps users understand their data.
You can fetch data from URLs, parse various formats, and create
visualizations.
'''

user_prompt_template = '''
Analyze the following data request: {{request}}
Available context: {{context}}
'''

documentation = '''
# DataAnalyzer Agent

This agent specializes in data analysis tasks and provides:

- HTTP data fetching from URLs
- CSV, JSON, and Excel file parsing
- Statistical analysis and summaries
- Chart and visualization generation
- Report creation with insights
'''
EOF

# 2. Validate configuration (catches 90% of deployment issues)
caxton agents validate /var/lib/caxton/agents/data-analyzer.toml

# 3. Hot-deploy the agent (zero-downtime)
caxton agents deploy data-analyzer --hot-reload

# 4. Verify deployment success
caxton agents status data-analyzer

# 5. Test agent functionality
caxton agents test data-analyzer --query "Analyze sample data"
```

**Production deployment with error handling:**

```bash
deploy_config_agent() {
    local agent_name="$1"
    local agent_file="$2"
    local max_retries=3
    local retry_count=0

    echo "Deploying config agent: $agent_name"

    # Pre-deployment validation
    if ! caxton agents validate "$agent_file"; then
        echo "ERROR: Configuration validation failed"
        caxton agents validate "$agent_file" --verbose
        return 1
    fi

    # Deploy with retry logic
    while [ $retry_count -lt $max_retries ]; do
        if caxton agents deploy "$agent_name" --source "$agent_file" \
           --hot-reload --timeout 30s; then
            echo "SUCCESS: Agent '$agent_name' deployed"

            # Verify deployment
            if caxton agents ping "$agent_name" --timeout 5s; then
                echo "SUCCESS: Agent responding to health checks"
                caxton agents capabilities "$agent_name"
                return 0
            else
                echo "WARNING: Agent deployed but not responding"
            fi
        else
            retry_count=$((retry_count + 1))
            echo "RETRY: Deployment attempt $retry_count failed"
            sleep 5
        fi
    done

    echo "ERROR: Deployment failed after $max_retries attempts"
    return 1
}

# Usage example
deploy_config_agent "data-analyzer" \
    "/var/lib/caxton/agents/data-analyzer.toml"
```

### Common Deployment Issues

| Error Pattern | Root Cause | Resolution |
|---------------|------------|------------|
| `TOML parse error: line 5` | Invalid configuration | Validate TOML syntax with `toml check` |
| `Missing required field: name` | Incomplete config | Add all required fields per TOML schema |
| `Unknown tool: unknown_tool` | Invalid tool reference | Check `caxton tools list` for available tools |
| `Agent name conflict` | Duplicate agent name | Use unique names or unload existing agent |
| `File not found` | Incorrect file path | Verify file exists in `agents/` directory |
| `Permission denied` | File system permissions | Check read access to agent files |

**Diagnostic commands for deployment issues:**

```bash
# Check TOML syntax
toml check /var/lib/caxton/agents/data-analyzer.toml

# Validate against schema
caxton agents validate-schema /var/lib/caxton/agents/data-analyzer.toml

# Check available tools
caxton tools list --available

# Check existing agents for conflicts
caxton agents list --names-only | grep -i analyzer

# Check file permissions
ls -la /var/lib/caxton/agents/data-analyzer.toml
```

## WASM Agent Deployment

### Advanced Use Case Deployment

**Compile and deploy WebAssembly agent (2-4 hour process):**

```bash
# 1. Prepare WASM module (example with Rust)
# In your agent project directory:
cargo build --target wasm32-wasi --release

# 2. Copy WASM binary
cp target/wasm32-wasi/release/custom-agent.wasm \
   /tmp/custom-agent.wasm

# 3. Deploy via REST API with resource limits
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"custom-algorithm-agent\",
    \"type\": \"wasm\",
    \"wasm_module\": \"$(base64 -w0 /tmp/custom-agent.wasm)\",
    \"resource_limits\": {
      \"max_memory_bytes\": 10485760,
      \"max_cpu_millis\": 5000,
      \"max_execution_time_ms\": 10000
    },
    \"capabilities\": [\"custom-computation\"],
    \"security_policy\": {
      \"network_access\": false,
      \"file_system_access\": false
    }
  }"

# 4. Verify WASM deployment
AGENT_ID=$(curl -s http://localhost:8080/api/v1/agents | \
           jq -r '.[] | select(.name=="custom-algorithm-agent") | .id')

curl http://localhost:8080/api/v1/agents/$AGENT_ID

# 5. Test WASM agent
curl -X POST http://localhost:8080/api/v1/agents/$AGENT_ID/execute \
  -H "Content-Type: application/json" \
  -d '{"input": "test computation"}'
```

**WASM deployment monitoring:**

```bash
# Monitor WASM agent performance
curl http://localhost:8080/api/v1/metrics | \
    grep caxton_wasm_agent_ | head -10

# Check resource usage
caxton agents resource-usage --type wasm

# Monitor sandbox isolation
caxton agents sandbox-status --agent custom-algorithm-agent
```

## Agent Management

### Operational Agent Commands

```bash
# List all agents with status
caxton agents list --detailed
# Output format:
# NAME                TYPE    STATUS    UPTIME    MEMORY_MB  CPU_%
# DataAnalyzer       config  running   2h15m     45.2       3.1
# CustomAgent        wasm    running   45m       12.8       1.2

# Get agent details
caxton agents show data-analyzer --include-metrics

# Filter agents by type and status
caxton agents list --type config --status running
caxton agents list --type wasm --status failing

# Count agents by category
echo "Agent Summary:"
echo "Config agents: $(caxton agents count --type config)"
echo "WASM agents: $(caxton agents count --type wasm)"
echo "Running total: $(caxton agents count --status running)"
echo "Failed total: $(caxton agents count --status failed)"
```

### Agent Health Monitoring

```bash
# Continuous agent monitoring
watch -n 10 'caxton agents status --summary'

# Check specific agent health
check_agent_health() {
    local agent_name="$1"
    local timeout="${2:-5}"

    echo "Checking health of agent: $agent_name"

    # Health ping
    if caxton agents ping "$agent_name" --timeout "${timeout}s"; then
        echo "✓ Agent $agent_name is responsive"

        # Get performance metrics
        caxton agents metrics "$agent_name" --last 1h

        # Check capabilities
        echo "Available capabilities:"
        caxton agents capabilities "$agent_name" --list

        return 0
    else
        echo "✗ Agent $agent_name is not responding"

        # Get diagnostic info
        caxton agents diagnose "$agent_name" --show-errors

        return 1
    fi
}

# Monitor all agents
for agent in $(caxton agents list --names-only); do
    check_agent_health "$agent" 3
done
```

## Config Agent Operations

### Hot-Reload Operations

```bash
# Single agent reload
caxton agents reload data-analyzer

# Bulk reload with validation
caxton agents reload --all-config --validate-first

# Staged reload with rollback capability
caxton agents reload data-analyzer --staged --rollback-on-error

# Monitor reload progress
caxton agents reload-status --watch

# Rollback if issues occur
caxton agents rollback data-analyzer --to-previous-version
```

### Development Workflow Support

```bash
# Development mode with auto-reload
caxton agents watch /var/lib/caxton/agents/ --auto-reload \
    --ignore-errors --log-changes

# Test agent locally before deployment
caxton agents test-local ./agents/new-agent.toml \
    --input "test query" --timeout 10s

# Validate configuration changes
caxton agents diff data-analyzer ./agents/data-analyzer.toml

# Check agent logs for issues
caxton agents logs data-analyzer --tail 100 --level error

# Debug agent capabilities and tools
caxton agents debug data-analyzer --show-tools --show-prompts
```

## Memory Operations

### Embedded Memory System Management

```bash
# Check memory system status
caxton memory status --detailed
# Output:
# SQLite database: healthy (2.3MB, 0% fragmentation)
# Embedding model: All-MiniLM-L6-v2 loaded (203MB baseline)
# Entities: 12,450 stored (12% of 100K limit)
# Relations: 38,912 connections
# Vector cache: 89% hit rate (target: >85%)
# Search latency P99: 23ms (target: <50ms)

# Memory performance statistics
caxton memory stats --include-trends
# Shows performance trends over last 24 hours

# Monitor memory queries in real-time
caxton memory monitor --show-slow-queries --threshold 100ms

# Check if approaching capacity limits
caxton memory capacity-check --warn-percentage 80
# Entity count: 85,000 / 100,000 (85% - approaching limit)
# WARNING: Consider external backend migration
```

### Memory System Optimization

```bash
# Full memory optimization
caxton memory optimize --vacuum-sqlite --rebuild-index \
    --clear-cache --recompute-embeddings

# Incremental optimization (zero-downtime)
caxton memory optimize --incremental --max-duration 30s

# Clean up orphaned data
caxton memory cleanup --remove-orphaned --older-than 30d \
    --vacuum-after

# Rebuild search indexes
caxton memory reindex --background --verify-after
```

### Memory Migration Planning

```bash
# Check migration readiness
caxton memory migration-check --target qdrant
# Readiness: 95% (missing: qdrant connection config)
# Estimated downtime: 2-5 minutes
# Estimated migration time: 15-30 minutes

# Prepare for external backend migration
caxton memory export --format migration --target qdrant \
    --output /backup/memory-migration.json

# Test external backend connection
caxton memory test-connection --backend qdrant \
    --config /etc/caxton/qdrant.yaml
```

## Deployment Patterns

### Zero-Dependency Production Deployment

```bash
# Production-ready systemd service
cat > /etc/systemd/system/caxton.service << 'EOF'
[Unit]
Description=Caxton Multi-Agent Orchestration Server
Documentation=https://docs.caxton.dev
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=caxton
Group=caxton
Environment=RUST_LOG=info
Environment=CAXTON_CONFIG=/etc/caxton/caxton.yaml
Environment=CAXTON_DATA_DIR=/var/lib/caxton/data
Environment=CAXTON_AGENTS_DIR=/var/lib/caxton/agents

ExecStart=/usr/local/bin/caxton server start \
  --config ${CAXTON_CONFIG} \
  --data-dir ${CAXTON_DATA_DIR}

# Resource limits
MemoryMax=2G
CPUQuota=200%

# Security hardening
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/lib/caxton /var/log/caxton

# Restart behavior
Restart=always
RestartSec=10
TimeoutStartSec=60
TimeoutStopSec=30

# Health monitoring
ExecReload=/bin/kill -HUP $MAINPID
ExecStop=/usr/local/bin/caxton shutdown --graceful --timeout 30s

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable caxton
sudo systemctl start caxton

# Verify service status
systemctl status caxton --no-pager -l
```

### Docker Production Deployment

```yaml
# docker-compose.yml for production
version: '3.8'

services:
  caxton:
    image: caxton/caxton:latest
    container_name: caxton-server
    ports:
      - "8080:8080"    # API server
      - "9090:9090"    # Metrics endpoint

    environment:
      - RUST_LOG=info
      - CAXTON_CONFIG=/etc/caxton/caxton.yaml
      - OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:14268

    volumes:
      - ./config/caxton.yaml:/etc/caxton/caxton.yaml:ro
      - ./agents:/var/lib/caxton/agents:ro
      - caxton-data:/var/lib/caxton/data
      - caxton-logs:/var/log/caxton

    # Resource limits
    mem_limit: 2g
    cpus: 2.0

    # Health monitoring
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/api/v1/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s

    # Restart policy
    restart: unless-stopped

    # Security
    user: "1000:1000"
    read_only: true
    tmpfs:
      - /tmp:size=100M,noexec

  # Optional: Observability stack
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9091:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
    profiles: ["monitoring"]

  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"
      - "14268:14268"
    profiles: ["monitoring"]

volumes:
  caxton-data:
    driver: local
  caxton-logs:
    driver: local

networks:
  default:
    name: caxton-network
```

### Kubernetes Production Deployment

```yaml
# k8s/caxton-deployment.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: caxton-server
  labels:
    app: caxton
    version: v1.0.0
spec:
  serviceName: caxton
  replicas: 3
  updateStrategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1

  selector:
    matchLabels:
      app: caxton

  template:
    metadata:
      labels:
        app: caxton
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        prometheus.io/path: "/metrics"

    spec:
      serviceAccountName: caxton
      securityContext:
        runAsUser: 1000
        runAsGroup: 1000
        fsGroup: 1000

      containers:
      - name: caxton
        image: caxton/caxton:latest
        ports:
        - containerPort: 8080
          name: api
        - containerPort: 9090
          name: metrics

        env:
        - name: RUST_LOG
          value: "info"
        - name: CAXTON_CONFIG
          value: "/etc/caxton/caxton.yaml"
        - name: CAXTON_DATA_DIR
          value: "/var/lib/caxton/data"

        volumeMounts:
        - name: config
          mountPath: /etc/caxton
          readOnly: true
        - name: data
          mountPath: /var/lib/caxton/data
        - name: agents
          mountPath: /var/lib/caxton/agents
          readOnly: true

        # Resource management
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"

        # Health probes
        livenessProbe:
          httpGet:
            path: /api/v1/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3

        readinessProbe:
          httpGet:
            path: /api/v1/ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          successThreshold: 1

        # Graceful shutdown
        terminationGracePeriodSeconds: 60

        # Security context
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop:
            - ALL

      volumes:
      - name: config
        configMap:
          name: caxton-config
      - name: agents
        configMap:
          name: caxton-agents

  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: fast-ssd
      resources:
        requests:
          storage: 20Gi
```

## Backup Procedures

### Automated Backup Strategy

```bash
#!/bin/bash
# Production backup script with rotation
BACKUP_DIR="/backup/caxton"
RETENTION_DAYS=30
DATE=$(date +%Y%m%d-%H%M%S)
BACKUP_FILE="caxton-backup-$DATE.tar.gz"

# Full system backup
perform_backup() {
    echo "Starting Caxton backup at $(date)"

    # Create backup with verification
    if caxton backup --embedded \
       --output "$BACKUP_DIR/$BACKUP_FILE" \
       --compress --verify; then

        echo "Backup completed: $BACKUP_DIR/$BACKUP_FILE"

        # Test backup integrity
        caxton backup verify "$BACKUP_DIR/$BACKUP_FILE"

        # Clean up old backups
        find "$BACKUP_DIR" -name "caxton-backup-*.tar.gz" \
             -mtime +$RETENTION_DAYS -delete

        # Update latest symlink
        ln -sf "$BACKUP_FILE" "$BACKUP_DIR/caxton-latest.tar.gz"

        echo "Backup rotation completed"
        return 0
    else
        echo "ERROR: Backup failed"
        return 1
    fi
}

# Schedule with crontab
# 0 2 * * * /usr/local/bin/caxton-backup.sh >> /var/log/caxton/backup.log 2>&1

perform_backup
```

### Disaster Recovery Procedures

```bash
# Emergency restoration process
emergency_restore() {
    local backup_file="$1"
    local confirmation="$2"

    if [ "$confirmation" != "CONFIRM" ]; then
        echo "ERROR: Emergency restoration requires CONFIRM parameter"
        echo "Usage: emergency_restore /path/to/backup.tar.gz CONFIRM"
        return 1
    fi

    echo "EMERGENCY RESTORATION STARTING"
    echo "This will overwrite all current data"
    read -p "Type 'yes' to continue: " confirm

    if [ "$confirm" != "yes" ]; then
        echo "Restoration cancelled"
        return 1
    fi

    # Stop service
    systemctl stop caxton

    # Backup current state (just in case)
    mv /var/lib/caxton/data /var/lib/caxton/data.emergency.bak

    # Restore from backup
    if caxton restore "$backup_file" --target /var/lib/caxton/; then
        echo "Restoration completed"

        # Verify restoration
        caxton storage verify --comprehensive

        # Start service
        systemctl start caxton

        # Verify all systems
        sleep 10
        caxton agents list
        caxton memory status

        echo "Emergency restoration successful"
    else
        echo "ERROR: Restoration failed"
        echo "Recovering original data..."
        mv /var/lib/caxton/data.emergency.bak /var/lib/caxton/data
        systemctl start caxton
        return 1
    fi
}

# Usage: emergency_restore /backup/caxton-latest.tar.gz CONFIRM
```

## Emergency Procedures

### Critical System Failures

```bash
# Emergency response checklist
emergency_response() {
    local issue_type="$1"

    echo "=== EMERGENCY RESPONSE: $issue_type ==="
    echo "Time: $(date)"
    echo "Operator: $(whoami)@$(hostname)"

    case "$issue_type" in
        "server_down")
            # Server process failure
            echo "1. Checking process status..."
            systemctl status caxton

            echo "2. Checking system resources..."
            df -h /var/lib/caxton/
            free -h

            echo "3. Checking logs for crash reason..."
            journalctl -u caxton --since "10 minutes ago" --no-pager

            echo "4. Attempting restart..."
            systemctl restart caxton

            sleep 15
            if curl -f http://localhost:8080/api/v1/health; then
                echo "✓ Server recovery successful"
            else
                echo "✗ Server still down - escalating"
                # Send alert, call on-call engineer
            fi
            ;;

        "data_corruption")
            # Data corruption emergency
            echo "1. Enabling safe mode..."
            caxton storage safe-mode --enable

            echo "2. Assessing corruption level..."
            corruption_level=$(caxton storage corruption-level)

            if [ "$corruption_level" -lt 25 ]; then
                echo "3. Attempting repair..."
                caxton storage repair --emergency
            else
                echo "3. Corruption too severe - restoring from backup..."
                latest_backup=$(ls -t /backup/caxton/*.tar.gz | head -1)
                emergency_restore "$latest_backup" CONFIRM
            fi
            ;;

        "memory_critical")
            # Memory system critical failure
            echo "1. Checking memory system status..."
            caxton memory emergency-diagnosis

            echo "2. Attempting emergency cleanup..."
            caxton memory cleanup --aggressive --force

            echo "3. Clearing all caches..."
            caxton memory cache-clear --all

            if [ $(caxton memory usage --percentage) -gt 95 ]; then
                echo "4. Emergency migration to external backend..."
                caxton memory emergency-migrate --to qdrant
            fi
            ;;
    esac
}

# Usage examples:
# emergency_response "server_down"
# emergency_response "data_corruption"
# emergency_response "memory_critical"
```

### System Recovery Verification

```bash
# Post-emergency verification checklist
verify_recovery() {
    echo "=== RECOVERY VERIFICATION ==="

    # 1. Server health
    echo "1. Server Health Check:"
    if curl -f http://localhost:8080/api/v1/health; then
        echo "   ✓ API responding"
    else
        echo "   ✗ API not responding"
        return 1
    fi

    # 2. All agents operational
    echo "2. Agent Status:"
    failed_agents=$(caxton agents list --status failed --count)
    if [ "$failed_agents" -eq 0 ]; then
        echo "   ✓ All agents operational"
    else
        echo "   ✗ $failed_agents agents failed"
        caxton agents list --status failed
    fi

    # 3. Memory system functional
    echo "3. Memory System:"
    if caxton memory status | grep -q "healthy"; then
        echo "   ✓ Memory system healthy"
    else
        echo "   ✗ Memory system issues detected"
        caxton memory diagnose
    fi

    # 4. Performance within acceptable ranges
    echo "4. Performance Check:"
    response_time=$(curl -o /dev/null -s -w '%{time_total}' \
                    http://localhost:8080/api/v1/health)
    if (( $(echo "$response_time < 1.0" | bc -l) )); then
        echo "   ✓ Response time acceptable ($response_time s)"
    else
        echo "   ✗ Response time degraded ($response_time s)"
    fi

    echo "=== RECOVERY VERIFICATION COMPLETE ==="
}
```

## Monitoring and Alerting

### Prometheus Metrics Integration

```yaml
# prometheus.yml configuration
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "caxton_alerts.yml"

scrape_configs:
  - job_name: 'caxton'
    static_configs:
      - targets: ['caxton:9090']
    scrape_interval: 5s
    metrics_path: /metrics
```

```yaml
# caxton_alerts.yml - Critical alerts
groups:
- name: caxton.critical
  rules:
  - alert: CaxtonDown
    expr: up{job="caxton"} == 0
    for: 30s
    labels:
      severity: critical
    annotations:
      summary: "Caxton server is down"
      description: "Caxton server has been down for more than 30 seconds"

  - alert: CaxtonMemoryUsageHigh
    expr: caxton_memory_usage_percentage > 90
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Caxton memory usage is high"
      description: "Memory usage is {{ $value }}% - consider cleanup"

  - alert: CaxtonAgentsFailing
    expr: caxton_agents_failed_total > 0
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "Caxton agents are failing"
      description: "{{ $value }} agents are in failed state"
```

### Key Operational Metrics

```bash
# Critical metrics to monitor
curl -s localhost:9090/metrics | grep -E '^caxton_' | sort

# Server health metrics
# caxton_server_uptime_seconds - Server uptime
# caxton_server_restarts_total - Server restart count
# caxton_server_request_duration_seconds - API response times

# Memory system metrics
# caxton_memory_entities_total - Number of stored entities
# caxton_memory_search_latency_seconds - Search performance
# caxton_memory_usage_percentage - Memory utilization

# Agent metrics
# caxton_agents_config_total - Config agent count
# caxton_agents_wasm_total - WASM agent count
# caxton_agents_failed_total - Failed agent count
# caxton_agent_response_duration_seconds - Agent response times
```

## Best Practices Summary

### Operational Excellence

1. **Monitoring**: Set up comprehensive monitoring with Prometheus metrics
2. **Alerting**: Configure alerts for critical failures and performance degradation
3. **Backups**: Implement automated daily backups with verification
4. **Documentation**: Keep runbooks current and accessible to all operators
5. **Testing**: Regular disaster recovery drills and backup restoration tests
6. **Capacity Planning**: Monitor embedded system limits and plan migrations
7. **Security**: Regular security updates and access reviews
8. **Performance**: Continuous performance monitoring and optimization

### Deployment Guidelines

1. **Zero-Dependency First**: Use embedded architecture for simplicity
2. **Gradual Scaling**: Start with embedded, migrate to external backends as needed
3. **Configuration Validation**: Always validate before deployment
4. **Hot Reload**: Use zero-downtime deployments for config agents
5. **Resource Limits**: Set appropriate limits for WASM agents
6. **Health Checks**: Implement comprehensive health monitoring
7. **Graceful Degradation**: Design for partial system failures
8. **Recovery Planning**: Test all recovery procedures regularly

## References

- [Performance Optimization](performance.md)
- [Security Operations](security.md)
- [Lifecycle Management](lifecycle.md)
- [Troubleshooting Guide](troubleshooting.md)
- [Recovery Procedures](recovery.md)
- [Architecture Overview](../../ARCHITECTURE.md)
