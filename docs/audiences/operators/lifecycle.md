---
title: "Production Agent Lifecycle Management"
date: 2025-01-15
layout: page
categories: [Operations, Lifecycle]
audience: operators
description: "Production agent deployment, monitoring, and maintenance procedures"
---

## Production Lifecycle Operations

This guide provides production-focused agent lifecycle management for Caxton
operators managing both configuration and WASM agents in production
environments. Emphasis is on deployment patterns, monitoring, and operational
procedures.

### Production Lifecycle Overview

| Phase | Config Agents | WASM Agents | Operator Impact |
|-------|---------------|-------------|-----------------|
| Deploy | 30s instant | 2-5min validation | Low/Medium |
| Update | Hot reload | Blue-green/canary | Minimal/Medium |
| Monitor | Memory/tools | Resource/sandbox | Continuous |
| Scale | Horizontal | Vertical limits | Capacity planning |
| Recover | Config rollback | Binary rollback | Fast/Medium |

## Production Deployment Patterns

### Configuration Agent Production Deployment

#### Standard Configuration Deployment

```bash
#!/bin/bash
# Production config agent deployment script

set -euo pipefail

AGENT_NAME="$1"
CONFIG_FILE="$2"
ENVIRONMENT="production"

echo "=== DEPLOYING CONFIG AGENT: $AGENT_NAME ==="

# 1. Pre-deployment validation
echo "1. Validating configuration..."
caxton agents validate "$CONFIG_FILE" --strict --security-check

# 2. Deploy to staging first
echo "2. Staging deployment..."
caxton agents deploy "$CONFIG_FILE" --environment staging --test-mode

# 3. Run smoke tests
echo "3. Running smoke tests..."
caxton agents test "$AGENT_NAME" --basic-functionality --staging

# 4. Production deployment with monitoring
echo "4. Production deployment..."
caxton agents deploy "$CONFIG_FILE" --environment production --monitor

# 5. Health check
echo "5. Health verification..."
caxton agents health-check "$AGENT_NAME" --timeout 30s

echo "=== DEPLOYMENT COMPLETE ==="
```

#### Blue-Green Configuration Deployment

```bash
#!/bin/bash
# Blue-green deployment for configuration agents

AGENT_NAME="$1"
NEW_CONFIG="$2"

# Deploy to green environment
caxton agents deploy-green "$AGENT_NAME" --config "$NEW_CONFIG"

# Verify green environment health
caxton agents test "$AGENT_NAME" --environment green --comprehensive

# Switch traffic to green
caxton traffic switch-to-green "$AGENT_NAME"

# Monitor for issues
sleep 60
if caxton agents health-check "$AGENT_NAME" --environment green; then
    echo "Green deployment successful"
    caxton agents retire-blue "$AGENT_NAME"
else
    echo "Green deployment failed, rolling back"
    caxton traffic switch-to-blue "$AGENT_NAME"
    exit 1
fi
```

### WASM Agent Production Deployment

#### Canary WASM Deployment

```bash
#!/bin/bash
# Canary deployment for WASM agents

AGENT_NAME="$1"
WASM_MODULE="$2"
CANARY_PERCENTAGE=10

echo "=== CANARY WASM DEPLOYMENT: $AGENT_NAME ==="

# 1. Validate WASM module
echo "1. WASM validation..."
caxton wasm validate "$WASM_MODULE" --security-strict --resource-check

# 2. Deploy canary instance
echo "2. Canary deployment ($CANARY_PERCENTAGE% traffic)..."
caxton wasm deploy-canary "$AGENT_NAME" \
    --module "$WASM_MODULE" \
    --traffic-percentage $CANARY_PERCENTAGE \
    --monitor-metrics response_time,error_rate,resource_usage

# 3. Monitor canary metrics
echo "3. Monitoring canary performance..."
sleep 300  # 5 minute observation

# 4. Evaluate canary performance
if caxton wasm canary-health "$AGENT_NAME" --pass-criteria "error_rate<0.1%,response_time<100ms"; then
    echo "Canary passed, increasing traffic to 50%..."
    caxton wasm update-canary-traffic "$AGENT_NAME" --percentage 50

    sleep 300

    if caxton wasm canary-health "$AGENT_NAME" --pass-criteria "error_rate<0.1%"; then
        echo "Canary successful, promoting to 100%"
        caxton wasm promote-canary "$AGENT_NAME"
    else
        echo "Canary degradation at 50%, rolling back"
        caxton wasm rollback-canary "$AGENT_NAME"
        exit 1
    fi
else
    echo "Canary failed initial check, rolling back"
    caxton wasm rollback-canary "$AGENT_NAME"
    exit 1
fi

echo "=== CANARY DEPLOYMENT COMPLETE ==="
```

## Production Monitoring and Observability

### Configuration Agent Production Metrics

#### Essential Configuration Metrics Dashboard

```yaml
# Grafana dashboard for config agents
dashboards:
  config_agents:
    panels:
      - title: "Configuration Reload Success Rate"
        target: "rate(caxton_config_reloads_successful_total[5m]) / rate(caxton_config_reloads_total[5m])"
        threshold: 0.95

      - title: "Memory System Usage by Agent"
        target: "caxton_memory_entities_total by (agent_name)"
        threshold: 10000

      - title: "Tool Call Latency (P95)"
        target: "histogram_quantile(0.95, rate(caxton_tool_call_duration_seconds_bucket[5m]))"
        threshold: 10.0

      - title: "Agent Response Time Distribution"
        target: "caxton_agent_response_duration_seconds_bucket"
        type: heatmap
```

#### Configuration Agent Alerting

```yaml
# Production alerts for configuration agents
groups:
- name: caxton_config_production
  rules:
  - alert: ConfigAgentHighReloadFailureRate
    expr: rate(caxton_config_reload_failures_total[5m]) > 0.1
    for: 2m
    labels:
      severity: warning
      component: config_agent
    annotations:
      summary: "High configuration reload failure rate"
      description: "{{ $labels.agent_name }} failing {{ $value }} reloads/sec"
      runbook_url: "https://ops.company.com/caxton/config-reload-failures"

  - alert: ConfigAgentMemoryExhaustion
    expr: caxton_memory_entities_count > 50000
    for: 5m
    labels:
      severity: critical
      component: memory
    annotations:
      summary: "Configuration agent approaching memory limits"
      description: "Agent {{ $labels.agent_name }} has {{ $value }} entities"

  - alert: ConfigAgentToolCallsStuck
    expr: caxton_tool_calls_active > 100
    for: 10m
    labels:
      severity: warning
      component: tools
    annotations:
      summary: "High number of active tool calls"
      description: "{{ $value }} tool calls active for >10min"
```

### WASM Agent Production Metrics

#### WASM Resource Monitoring Dashboard

```yaml
dashboards:
  wasm_agents:
    panels:
      - title: "WASM Memory Usage"
        target: "caxton_wasm_memory_bytes / caxton_wasm_memory_limit_bytes"
        threshold: 0.8

      - title: "WASM Fuel Consumption Rate"
        target: "rate(caxton_wasm_fuel_consumed_total[5m])"
        threshold: 1000000

      - title: "Sandbox Security Violations"
        target: "increase(caxton_wasm_security_violations_total[1h])"
        threshold: 0

      - title: "WASM Agent Restart Frequency"
        target: "rate(caxton_wasm_agent_restarts_total[1h])"
        threshold: 0.1
```

#### WASM Agent Critical Alerts

```yaml
groups:
- name: caxton_wasm_production
  rules:
  - alert: WASMMemoryLimitApproached
    expr: caxton_wasm_memory_bytes / caxton_wasm_memory_limit_bytes > 0.9
    for: 1m
    labels:
      severity: warning
      component: wasm_resource
    annotations:
      summary: "WASM agent approaching memory limit"
      description: "Agent {{ $labels.agent_name }} at {{ $value }}% memory"

  - alert: WASMSecurityViolation
    expr: increase(caxton_wasm_security_violations_total[1m]) > 0
    for: 0m
    labels:
      severity: critical
      component: wasm_security
    annotations:
      summary: "WASM sandbox security violation"
      description: "{{ $value }} violations in agent {{ $labels.agent_name }}"
      runbook_url: "https://ops.company.com/caxton/wasm-security-incident"

  - alert: WASMAgentCrashLoop
    expr: rate(caxton_wasm_agent_restarts_total[10m]) > 0.1
    for: 5m
    labels:
      severity: critical
      component: wasm_stability
    annotations:
      summary: "WASM agent in crash loop"
      description: "Agent {{ $labels.agent_name }} restarting {{ $value }}/sec"
```

### Production Health Checks

#### Automated Health Monitoring

```bash
#!/bin/bash
# Production health check script (run every 5 minutes)

set -euo pipefail

echo "=== CAXTON HEALTH CHECK $(date) ==="

# Configuration agent health
echo "Checking configuration agents..."
caxton agents list --type config --unhealthy | while read -r agent; do
    echo "UNHEALTHY CONFIG AGENT: $agent"
    caxton agents health-check "$agent" --detailed >> /var/log/caxton/health.log
done

# WASM agent health
echo "Checking WASM agents..."
caxton agents list --type wasm --resource-issues | while read -r agent; do
    echo "WASM RESOURCE ISSUE: $agent"
    caxton wasm resource-status "$agent" --detailed >> /var/log/caxton/health.log
done

# Memory system health
echo "Checking memory system..."
caxton memory health-check --embedded

# Overall system health
echo "Overall system status:"
caxton status --production-check

echo "=== HEALTH CHECK COMPLETE ==="
```

## Production Scaling and Capacity Management

### Configuration Agent Scaling

#### Horizontal Configuration Scaling

```bash
#!/bin/bash
# Scale configuration agents based on load

AGENT_NAME="$1"
TARGET_LOAD_FACTOR="0.7"

# Get current metrics
CURRENT_LOAD=$(caxton agents metrics "$AGENT_NAME" --metric conversation_utilization)
CURRENT_INSTANCES=$(caxton agents count "$AGENT_NAME")

if (( $(echo "$CURRENT_LOAD > $TARGET_LOAD_FACTOR" | bc -l) )); then
    # Scale up
    NEW_INSTANCES=$((CURRENT_INSTANCES + 1))
    echo "Scaling up $AGENT_NAME to $NEW_INSTANCES instances"
    caxton agents scale "$AGENT_NAME" --instances "$NEW_INSTANCES"
elif (( $(echo "$CURRENT_LOAD < 0.3" | bc -l) )) && (( CURRENT_INSTANCES > 1 )); then
    # Scale down
    NEW_INSTANCES=$((CURRENT_INSTANCES - 1))
    echo "Scaling down $AGENT_NAME to $NEW_INSTANCES instances"
    caxton agents scale "$AGENT_NAME" --instances "$NEW_INSTANCES"
fi
```

#### Memory Capacity Planning

```bash
#!/bin/bash
# Memory capacity monitoring for config agents

echo "=== MEMORY CAPACITY ANALYSIS ==="

caxton memory analyze-usage --by-agent --production | \
while IFS=' ' read -r agent entities relations memory_mb; do
    if (( memory_mb > 500 )); then
        echo "CAPACITY WARNING: $agent using ${memory_mb}MB (${entities} entities)"

        # Suggest memory optimization
        caxton memory optimize-suggestions "$agent"
    fi
done
```

### WASM Agent Resource Management

#### WASM Resource Optimization

```bash
#!/bin/bash
# WASM agent resource optimization

AGENT_NAME="$1"

echo "=== WASM RESOURCE OPTIMIZATION: $AGENT_NAME ==="

# Analyze resource usage patterns
caxton wasm analyze-usage "$AGENT_NAME" --period 24h

# Get optimization recommendations
caxton wasm optimize-recommendations "$AGENT_NAME" \
    --criteria memory_efficiency,fuel_efficiency,stability

# Apply conservative optimizations
caxton wasm update-limits "$AGENT_NAME" --auto-optimize --conservative
```

## Production Incident Response

### Configuration Agent Incident Response

#### Configuration Agent Failure Recovery

```bash
#!/bin/bash
# Configuration agent incident response playbook

AGENT_NAME="$1"
INCIDENT_TYPE="$2"

echo "=== CONFIG AGENT INCIDENT RESPONSE: $AGENT_NAME ==="
echo "Incident Type: $INCIDENT_TYPE"
echo "Timestamp: $(date)"

case "$INCIDENT_TYPE" in
    "config_validation_failed")
        echo "1. Rolling back to last known good configuration..."
        caxton agents rollback "$AGENT_NAME" --to-last-good

        echo "2. Validating rollback success..."
        caxton agents health-check "$AGENT_NAME"

        echo "3. Preserving failed config for analysis..."
        caxton agents export-failed-config "$AGENT_NAME" \
            > "/tmp/failed-config-$AGENT_NAME-$(date +%s).yaml"
        ;;

    "memory_system_disconnected")
        echo "1. Checking memory system connectivity..."
        caxton memory connection-test

        echo "2. Attempting memory system reconnection..."
        caxton agents reconnect-memory "$AGENT_NAME"

        echo "3. Verifying memory scope integrity..."
        caxton memory scope-check "$AGENT_NAME"
        ;;

    "tool_calls_failing")
        echo "1. Identifying failing tools..."
        caxton tools diagnose "$AGENT_NAME"

        echo "2. Attempting tool reconnection..."
        caxton tools reconnect --agent "$AGENT_NAME"

        echo "3. Updating tool configuration if needed..."
        caxton agents update-tool-config "$AGENT_NAME" --auto-fix
        ;;
esac

echo "=== INCIDENT RESPONSE COMPLETE ==="
```

### WASM Agent Incident Response

#### WASM Security Violation Response

```bash
#!/bin/bash
# WASM security incident response

AGENT_NAME="$1"
VIOLATION_TYPE="$2"

echo "=== WASM SECURITY INCIDENT: $AGENT_NAME ==="
echo "Violation: $VIOLATION_TYPE"

# Immediate containment
echo "1. CONTAINMENT: Isolating agent..."
caxton wasm isolate "$AGENT_NAME" --immediate

# Evidence collection
echo "2. EVIDENCE: Collecting violation details..."
caxton wasm security-audit "$AGENT_NAME" --export \
    > "/tmp/wasm-violation-$AGENT_NAME-$(date +%s).json"

# Impact assessment
echo "3. ASSESSMENT: Checking sandbox integrity..."
caxton wasm sandbox-integrity-check "$AGENT_NAME"

# Recovery action
if [ "$VIOLATION_TYPE" = "memory_access" ]; then
    echo "4. RECOVERY: Rebuilding sandbox with stricter limits..."
    caxton wasm rebuild-sandbox "$AGENT_NAME" --memory-strict
elif [ "$VIOLATION_TYPE" = "import_violation" ]; then
    echo "4. RECOVERY: Updating import restrictions..."
    caxton wasm update-security-policy "$AGENT_NAME" --restrict-imports
fi

echo "=== SECURITY INCIDENT RESPONSE COMPLETE ==="
```

## Production Maintenance Procedures

### Scheduled Maintenance Tasks

#### Daily Production Maintenance

```bash
#!/bin/bash
# Daily Caxton production maintenance

echo "=== DAILY CAXTON MAINTENANCE $(date) ==="

# 1. Configuration agent maintenance
echo "1. Configuration agent cleanup..."
caxton agents cleanup --remove-inactive --days 7
caxton memory cleanup --compact-entities --remove-orphaned

# 2. WASM agent maintenance
echo "2. WASM agent maintenance..."
caxton wasm cleanup --collect-garbage --optimize-memory
caxton wasm security-audit --all-agents --quick

# 3. System maintenance
echo "3. System maintenance..."
caxton logs rotate --keep-days 30
caxton metrics export --daily-summary

# 4. Health verification
echo "4. Post-maintenance health check..."
caxton status --comprehensive

echo "=== DAILY MAINTENANCE COMPLETE ==="
```

#### Weekly Production Maintenance

```bash
#!/bin/bash
# Weekly Caxton production maintenance

echo "=== WEEKLY CAXTON MAINTENANCE $(date) ==="

# 1. Deep memory system maintenance
echo "1. Memory system deep cleaning..."
caxton memory vacuum --full --embedded-backend
caxton memory reindex --optimize --background

# 2. Security audit
echo "2. Security audit..."
caxton security audit --comprehensive --export-report

# 3. Performance analysis
echo "3. Performance analysis..."
caxton performance analyze --period 7d --export-report

# 4. Backup verification
echo "4. Backup verification..."
caxton backup verify --test-restore --encrypted-backups

# 5. Capacity planning
echo "5. Capacity planning analysis..."
caxton capacity forecast --period 30d --agents-and-memory

echo "=== WEEKLY MAINTENANCE COMPLETE ==="
```

### Production Backup and Recovery

#### Automated Production Backup

```bash
#!/bin/bash
# Production backup with zero-downtime procedures

set -euo pipefail

BACKUP_DIR="/secure-backup/caxton"
TIMESTAMP=$(date +%Y%m%d-%H%M)
BACKUP_NAME="caxton-production-$TIMESTAMP"

echo "=== STARTING PRODUCTION BACKUP: $BACKUP_NAME ==="

# 1. Pre-backup validation
echo "1. Pre-backup validation..."
caxton status --ready-for-backup
caxton memory consistency-check --quick

# 2. Create online backup (no service interruption)
echo "2. Creating online backup..."
caxton backup create "$BACKUP_NAME" --online-backup --include-memory

# 3. Verify backup integrity
echo "3. Verifying backup integrity..."
caxton backup verify "$BACKUP_NAME" --checksum --test-extract

# 4. Encrypt and store
echo "4. Encrypting and storing backup..."
caxton backup encrypt "$BACKUP_NAME" --production-key
caxton backup store "$BACKUP_NAME" --location "$BACKUP_DIR"

# 5. Cleanup old backups (keep 30 days production)
echo "5. Cleaning old backups..."
find "$BACKUP_DIR" -name "caxton-production-*.tar.gz.gpg" -mtime +30 -delete

echo "=== PRODUCTION BACKUP COMPLETE: $BACKUP_NAME ==="
```

#### Production Recovery Procedures

```bash
#!/bin/bash
# Production recovery from backup

BACKUP_NAME="$1"
RECOVERY_TYPE="${2:-full}"

echo "=== PRODUCTION RECOVERY: $BACKUP_NAME ==="
echo "Recovery Type: $RECOVERY_TYPE"

# 1. Pre-recovery checks
echo "1. Pre-recovery validation..."
caxton backup verify "$BACKUP_NAME" --production-environment

# 2. Create recovery point
echo "2. Creating recovery point..."
caxton backup create "pre-recovery-$(date +%s)" --quick

# 3. Perform recovery
case "$RECOVERY_TYPE" in
    "full")
        echo "3. Performing full system recovery..."
        caxton service stop --graceful
        caxton backup restore "$BACKUP_NAME" --full-system
        caxton service start --verify-health
        ;;
    "config_agents_only")
        echo "3. Recovering configuration agents only..."
        caxton backup restore "$BACKUP_NAME" --config-agents-only --online
        ;;
    "memory_system_only")
        echo "3. Recovering memory system only..."
        caxton backup restore "$BACKUP_NAME" --memory-only --rebuild-indexes
        ;;
esac

# 4. Post-recovery validation
echo "4. Post-recovery validation..."
caxton status --comprehensive --post-recovery-check
caxton agents health-check --all --detailed

echo "=== PRODUCTION RECOVERY COMPLETE ==="
```

## Production Troubleshooting Runbooks

### Common Production Issues

#### Configuration Agent Performance Degradation

```bash
#!/bin/bash
# Configuration agent performance troubleshooting

AGENT_NAME="$1"

echo "=== CONFIG AGENT PERFORMANCE TROUBLESHOOTING: $AGENT_NAME ==="

# 1. Check basic metrics
echo "1. Basic performance metrics..."
caxton agents metrics "$AGENT_NAME" --detailed --last-1h

# 2. Memory system performance
echo "2. Memory system analysis..."
caxton memory performance-analysis "$AGENT_NAME"

# 3. Tool call analysis
echo "3. Tool call performance..."
caxton tools performance-analysis "$AGENT_NAME"

# 4. Conversation queue analysis
echo "4. Conversation queue status..."
caxton conversations queue-analysis "$AGENT_NAME"

# 5. Optimization recommendations
echo "5. Performance optimization suggestions..."
caxton agents optimize-recommendations "$AGENT_NAME" --production
```

#### WASM Agent Resource Exhaustion

```bash
#!/bin/bash
# WASM agent resource troubleshooting

AGENT_NAME="$1"

echo "=== WASM RESOURCE TROUBLESHOOTING: $AGENT_NAME ==="

# 1. Resource usage analysis
echo "1. Current resource usage..."
caxton wasm resource-usage "$AGENT_NAME" --detailed

# 2. Memory leak detection
echo "2. Memory leak analysis..."
caxton wasm memory-analysis "$AGENT_NAME" --leak-detection

# 3. Fuel consumption patterns
echo "3. Fuel consumption analysis..."
caxton wasm fuel-analysis "$AGENT_NAME" --pattern-detection

# 4. Resource optimization
echo "4. Resource optimization..."
caxton wasm optimize-resources "$AGENT_NAME" --conservative --test-first
```

This production lifecycle guide provides comprehensive operational procedures
for managing Caxton agents in production environments, with focus on
deployment patterns, monitoring, incident response, and maintenance procedures
essential for production operations.
