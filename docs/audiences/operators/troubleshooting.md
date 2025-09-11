---
title: "Production Troubleshooting Guide"
date: 2025-01-15
layout: page
categories: [Operations, Troubleshooting]
audience: operators
description: "Production error handling and troubleshooting procedures for Caxton operators"
---

## Production Troubleshooting Overview

This guide provides comprehensive troubleshooting procedures for Caxton
operators managing production deployments. Focus is on rapid issue
identification, systematic debugging, and quick resolution procedures for
common production problems.

### Troubleshooting Priority Matrix

| Issue Type | Severity | Response Time | Resolution Target |
|------------|----------|---------------|-------------------|
| Service Down | P0 | Immediate | 15 minutes |
| Performance Degradation | P1 | 5 minutes | 1 hour |
| Configuration Issues | P2 | 15 minutes | 4 hours |
| Monitoring Alerts | P3 | 1 hour | 24 hours |

## Production Issue Identification

### Health Check Dashboard

```bash
#!/bin/bash
# Production health check script
# Run this first for any production issue

echo "=== CAXTON PRODUCTION HEALTH CHECK ==="
echo "Timestamp: $(date)"
echo ""

# 1. System status
echo "1. System Status:"
caxton status --production-check --timeout 10s

# 2. Agent health summary
echo "2. Agent Health Summary:"
caxton agents health-summary --critical-only

# 3. Resource utilization
echo "3. Resource Utilization:"
caxton resources status --alerts-only

# 4. Memory system status
echo "4. Memory System Status:"
caxton memory health-check --embedded --quick

# 5. API endpoint status
echo "5. API Status:"
curl -f -m 5 https://caxton.yourdomain.com/health || echo "API UNREACHABLE"

echo ""
echo "=== HEALTH CHECK COMPLETE ==="
```

### Rapid Issue Classification

#### P0 - Service Completely Down

**Symptoms:**

- API endpoints returning 5xx errors
- No agent responses to any requests
- Caxton process not running
- Database connection failures

**Immediate Actions:**

```bash
# Check if process is running
systemctl status caxton

# Check process logs immediately
journalctl -u caxton -f --lines=50

# Check resource exhaustion
free -h && df -h

# Network connectivity
ss -tuln | grep :8443
```

#### P1 - Performance Degradation

**Symptoms:**

- Response times > 5 seconds
- High error rates (>1%)
- Memory/CPU usage >80%
- Queue backlogs growing

**Immediate Actions:**

```bash
# Performance metrics snapshot
caxton metrics snapshot --performance --last-5min

# Resource usage analysis
caxton resources analyze --bottlenecks

# Agent performance breakdown
caxton agents performance-summary --slow-only
```

#### P2 - Configuration Issues

**Symptoms:**

- Agent deployment failures
- Configuration validation errors
- Hot reload failures
- Tool access issues

**Immediate Actions:**

```bash
# Configuration validation
caxton agents validate-all --failed-only

# Recent configuration changes
caxton config audit-log --last-1h

# Agent status summary
caxton agents status --error-states-only
```

## Common Production Issues

### Configuration Agent Issues

#### Issue: Agent Not Responding

**Symptoms:**

- Agent shows as active but not processing messages
- Requests timeout after 30 seconds
- No error logs from agent

**Diagnostic Steps:**

```bash
# 1. Check agent status details
caxton agents status agent-name --detailed

# 2. Check conversation queue
caxton conversations queue-status agent-name

# 3. Check memory system connectivity
caxton memory connection-test agent-name

# 4. Check tool availability
caxton tools status --agent agent-name

# 5. Review recent logs
caxton agents logs agent-name --level error --last-10min
```

**Common Causes and Solutions:**

1. **Memory System Disconnection**

   ```bash
   # Solution: Reconnect memory system
   caxton agents reconnect-memory agent-name
   caxton memory health-check agent-name
   ```

2. **Tool Access Issues**

   ```bash
   # Solution: Refresh tool connections
   caxton tools reconnect --agent agent-name
   caxton tools test-connectivity --agent agent-name
   ```

3. **Queue Backlog**

   ```bash
   # Solution: Clear queue or scale agent
   caxton conversations drain-queue agent-name
   # OR
   caxton agents scale agent-name --instances +1
   ```

#### Issue: Configuration Hot Reload Failing

**Symptoms:**

- Configuration changes not taking effect
- Validation errors on reload
- Agent stuck in "reloading" state

**Diagnostic Steps:**

```bash
# 1. Check reload status
caxton agents reload-status agent-name

# 2. Validate configuration syntax
caxton agents validate config-file.md --detailed

# 3. Check file permissions
ls -la /var/lib/caxton/agents/

# 4. Review reload logs
grep "hot_reload" /var/log/caxton/agents.log | tail -20
```

**Common Solutions:**

```bash
# Fix 1: File permission issues
sudo chown caxton:caxton /var/lib/caxton/agents/*.md
sudo chmod 644 /var/lib/caxton/agents/*.md

# Fix 2: YAML syntax errors
caxton agents validate config-file.md --fix-suggestions

# Fix 3: Capability conflicts
caxton capabilities check-conflicts agent-name

# Fix 4: Force reload with fallback
caxton agents reload agent-name --force --fallback-on-error
```

#### Issue: Memory Scope Violations

**Symptoms:**

- "Memory access denied" errors
- Agents unable to store/retrieve memories
- Cross-agent memory conflicts

**Diagnostic Steps:**

```bash
# 1. Check memory scope configuration
caxton memory scope-status --all-agents

# 2. Review scope violations
caxton memory violations --last-24h

# 3. Check memory usage by scope
caxton memory usage --by-scope --detailed

# 4. Verify scope permissions
caxton memory permissions-check agent-name
```

**Solutions:**

```bash
# Fix 1: Reconfigure memory scopes
caxton memory scope-reset agent-name --scope workspace

# Fix 2: Clean conflicting memories
caxton memory cleanup --scope-conflicts --agent agent-name

# Fix 3: Rebuild memory indexes
caxton memory reindex --agent agent-name --scope-aware
```

### WASM Agent Issues

#### Issue: WASM Agent Sandbox Violations

**Symptoms:**

- Agent terminated unexpectedly
- Security violation alerts
- Resource limit exceeded errors

**Diagnostic Steps:**

```bash
# 1. Check security violations
caxton wasm security-audit agent-name --recent

# 2. Review resource usage
caxton wasm resource-usage agent-name --detailed

# 3. Check sandbox integrity
caxton wasm sandbox-status agent-name

# 4. Analyze violation patterns
caxton wasm violations-analysis agent-name --pattern-detection
```

**Solutions:**

```bash
# Fix 1: Adjust resource limits
caxton wasm update-limits agent-name --memory +50% --fuel +25%

# Fix 2: Update security policy
caxton wasm update-security-policy agent-name --less-restrictive

# Fix 3: Rebuild sandbox
caxton wasm rebuild-sandbox agent-name --clean-state

# Fix 4: Rollback to previous version
caxton wasm rollback agent-name --to-last-working
```

#### Issue: WASM Module Loading Failures

**Symptoms:**

- Agent deployment fails at WASM loading stage
- "Invalid WASM module" errors
- Module validation timeouts

**Diagnostic Steps:**

```bash
# 1. Validate WASM module
caxton wasm validate module.wasm --comprehensive

# 2. Check module compatibility
caxton wasm compatibility-check module.wasm --runtime-version

# 3. Resource requirements analysis
caxton wasm analyze-requirements module.wasm

# 4. Compare with working version
caxton wasm diff module.wasm previous-working.wasm
```

**Solutions:**

```bash
# Fix 1: Recompile WASM module
caxton wasm recompile source/ --target production

# Fix 2: Update runtime compatibility
caxton wasm update-runtime --compatible-with module.wasm

# Fix 3: Reduce module size
caxton wasm optimize module.wasm --size-optimized

# Fix 4: Use fallback deployment
caxton wasm deploy-fallback agent-name --previous-version
```

### Memory System Issues

#### Issue: Embedded SQLite Database Corruption

**Symptoms:**

- Database locked errors
- Memory queries returning inconsistent results
- "Database is corrupt" messages

**Diagnostic Steps:**

```bash
# 1. Check database integrity
caxton memory integrity-check --full

# 2. Review WAL file status
caxton memory wal-status --embedded

# 3. Check disk space and permissions
df -h /var/lib/caxton/data/
ls -la /var/lib/caxton/data/memory.db*

# 4. Analyze corruption patterns
caxton memory corruption-analysis --export-report
```

**Solutions:**

```bash
# Fix 1: Database recovery
caxton service stop
caxton memory recover --from-wal --backup-first
caxton service start

# Fix 2: Rebuild from backup
caxton memory restore --from-backup latest --verify-integrity

# Fix 3: Emergency data export/import
caxton memory export --json emergency-export.json
caxton memory reinitialize --empty
caxton memory import emergency-export.json --validate

# Fix 4: Rebuild indexes
caxton memory reindex --full --embedded --optimize
```

#### Issue: Memory Performance Degradation

**Symptoms:**

- Semantic search queries taking >10 seconds
- Memory operations timing out
- High CPU usage during memory operations

**Diagnostic Steps:**

```bash
# 1. Performance metrics analysis
caxton memory performance-metrics --detailed --last-1h

# 2. Query analysis
caxton memory slow-queries --threshold 5s

# 3. Index health check
caxton memory index-analysis --fragmentation

# 4. Resource utilization
caxton memory resource-usage --embedded --breakdown
```

**Solutions:**

```bash
# Fix 1: Optimize indexes
caxton memory optimize --indexes --background

# Fix 2: Memory cleanup
caxton memory cleanup --vacuum --embedded --compact

# Fix 3: Query optimization
caxton memory analyze-queries --optimize-suggestions

# Fix 4: Resource allocation
caxton memory adjust-cache --size +512MB --embedded
```

### API and Network Issues

#### Issue: API Endpoints Returning 5xx Errors

**Symptoms:**

- HTTP 500/503 responses
- Connection refused errors
- SSL/TLS handshake failures

**Diagnostic Steps:**

```bash
# 1. Test API endpoints
curl -v https://caxton.yourdomain.com/health
curl -v https://caxton.yourdomain.com/api/v1/agents

# 2. Check API server status
caxton api status --detailed

# 3. Review API logs
caxton api logs --level error --last-10min

# 4. Network connectivity test
caxton network diagnose --api-endpoints
```

**Solutions:**

```bash
# Fix 1: Restart API server
caxton api restart --graceful

# Fix 2: Certificate renewal
caxton tls renew-certificates --api

# Fix 3: Clear API cache
caxton api cache-clear --restart-workers

# Fix 4: Scale API workers
caxton api scale --workers +2
```

#### Issue: High API Latency

**Symptoms:**

- Response times >5 seconds
- Request queue building up
- Client timeouts

**Diagnostic Steps:**

```bash
# 1. API performance metrics
caxton api metrics --performance --last-15min

# 2. Request tracing
caxton api trace-slow-requests --threshold 2s

# 3. Resource bottleneck analysis
caxton api bottleneck-analysis --detailed

# 4. Database connection pool status
caxton api connection-pool-status
```

**Solutions:**

```bash
# Fix 1: Optimize database queries
caxton api optimize-queries --slow-only

# Fix 2: Increase connection pool
caxton api tune --connection-pool-size +10

# Fix 3: Add API caching
caxton api cache-enable --ttl 60s --endpoints /agents,/status

# Fix 4: Load balancing
caxton api load-balance --distribute-load
```

## Production Monitoring Integration

### Alerting Response Procedures

#### Critical Alert Response

```bash
#!/bin/bash
# Critical alert response script
# Called automatically by monitoring system

ALERT_TYPE="$1"
COMPONENT="$2"
SEVERITY="$3"

echo "=== CRITICAL ALERT RESPONSE ==="
echo "Type: $ALERT_TYPE"
echo "Component: $COMPONENT"
echo "Severity: $SEVERITY"
echo "Timestamp: $(date)"

case "$ALERT_TYPE" in
    "service_down")
        # Immediate service recovery
        caxton service restart --emergency
        caxton health-check --wait-ready --timeout 30s
        ;;

    "memory_corruption")
        # Memory system emergency recovery
        caxton service stop
        caxton memory emergency-backup
        caxton memory recover --from-backup --verify
        caxton service start --verify-health
        ;;

    "security_violation")
        # Security incident response
        caxton security incident-response --type "$COMPONENT"
        caxton agents suspend --security-risk
        caxton security audit-all --emergency
        ;;
esac

echo "=== ALERT RESPONSE COMPLETE ==="
```

#### Monitoring Integration Scripts

```bash
# Prometheus alert handler
#!/bin/bash
ALERT_JSON="$1"

# Parse Prometheus alert
ALERT_NAME=$(echo "$ALERT_JSON" | jq -r '.alerts[0].labels.alertname')
INSTANCE=$(echo "$ALERT_JSON" | jq -r '.alerts[0].labels.instance')
SEVERITY=$(echo "$ALERT_JSON" | jq -r '.alerts[0].labels.severity')

# Route to appropriate handler
case "$ALERT_NAME" in
    "CaxtonConfigValidationFailures")
        /usr/local/bin/handle-config-failures.sh "$INSTANCE"
        ;;
    "CaxtonMemoryScopeViolation")
        /usr/local/bin/handle-memory-violations.sh "$INSTANCE"
        ;;
    "CaxtonWASMSecurityViolation")
        /usr/local/bin/handle-wasm-security.sh "$INSTANCE"
        ;;
esac
```

### Log Analysis Automation

```bash
#!/bin/bash
# Automated log analysis for problem detection

echo "=== AUTOMATED LOG ANALYSIS ==="

# 1. Error pattern detection
echo "Detecting error patterns..."
caxton logs analyze --patterns --last-1h | \
while read -r pattern count; do
    if [ "$count" -gt 10 ]; then
        echo "HIGH ERROR PATTERN: $pattern ($count occurrences)"
        caxton troubleshoot pattern "$pattern" --suggest-fixes
    fi
done

# 2. Performance anomaly detection
echo "Checking performance anomalies..."
caxton logs analyze --performance-anomalies --threshold 2-sigma

# 3. Security event correlation
echo "Analyzing security events..."
caxton logs security-correlation --export-incidents

# 4. Resource trend analysis
echo "Resource trend analysis..."
caxton logs resource-trends --predict-exhaustion
```

## Escalation Procedures

### Internal Escalation Path

1. **L1 - Operations Team** (0-15 minutes)
   - Run standard troubleshooting procedures
   - Check known issues and solutions
   - Apply automated fixes

2. **L2 - Senior Operations** (15-60 minutes)
   - Complex configuration issues
   - Performance tuning requirements
   - Cross-system integration problems

3. **L3 - Development Team** (1-4 hours)
   - Core system bugs
   - WASM runtime issues
   - Architecture-level problems

### External Escalation

```bash
# Create escalation report
#!/bin/bash
ISSUE_ID="$1"
SEVERITY="$2"

echo "Creating escalation report for issue $ISSUE_ID..."

# Collect diagnostic data
caxton diagnostics export --issue "$ISSUE_ID" --comprehensive \
    > "escalation-$ISSUE_ID-$(date +%s).json"

# System state snapshot
caxton snapshot create --full-system \
    "escalation-snapshot-$ISSUE_ID-$(date +%s)"

# Recent logs
caxton logs export --last-2h --all-components \
    > "escalation-logs-$ISSUE_ID-$(date +%s).log"

echo "Escalation package ready for issue $ISSUE_ID"
```

## Recovery Verification

### Post-Fix Verification Checklist

```bash
#!/bin/bash
# Post-resolution verification script

echo "=== POST-FIX VERIFICATION ==="

# 1. System health check
echo "1. System Health..."
caxton health-check --comprehensive --timeout 60s

# 2. Agent functionality test
echo "2. Agent Functionality..."
caxton agents test-all --basic-functions --timeout 30s

# 3. Performance verification
echo "3. Performance Check..."
caxton performance baseline-compare --tolerance 10%

# 4. Memory system integrity
echo "4. Memory Integrity..."
caxton memory integrity-check --quick

# 5. API endpoint verification
echo "5. API Endpoints..."
caxton api test-endpoints --production-suite

# 6. Security posture check
echo "6. Security Status..."
caxton security status --all-components

echo "=== VERIFICATION COMPLETE ==="
```

## Prevention Strategies

### Proactive Monitoring Setup

```yaml
# Enhanced monitoring configuration
monitoring:
  predictive_alerts:
    - name: "memory_growth_trend"
      query: "predict_linear(caxton_memory_usage[2h], 3600)"
      threshold: 0.8

    - name: "performance_degradation"
      query: "rate(caxton_response_time_seconds[10m])"
      threshold: 2.0

  health_checks:
    - endpoint: "/health/deep"
      interval: 30s
      timeout: 5s

    - endpoint: "/agents/health-summary"
      interval: 60s
      timeout: 10s
```

### Automated Issue Prevention

```bash
#!/bin/bash
# Preventive maintenance script (run hourly)

echo "=== PREVENTIVE MAINTENANCE ==="

# 1. Resource cleanup
caxton resources cleanup --auto-optimize

# 2. Memory system maintenance
caxton memory maintenance --auto-compact --background

# 3. Configuration validation
caxton agents validate-all --fix-minor-issues

# 4. Security audit
caxton security audit --quick --fix-permissions

# 5. Performance optimization
caxton performance auto-tune --conservative

echo "=== PREVENTIVE MAINTENANCE COMPLETE ==="
```

This production troubleshooting guide provides systematic approaches for
identifying, diagnosing, and resolving common Caxton production issues,
with emphasis on rapid resolution and prevention of recurring problems.
