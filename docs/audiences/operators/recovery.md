---
title: "Production Recovery Procedures"
date: 2025-01-15
layout: page
categories: [Operations, Recovery]
audience: operators
description: "Production state recovery and disaster recovery procedures for Caxton operators"
---

## Production Recovery Overview

This guide provides production-focused recovery procedures for Caxton
operators managing system failures, data loss, and disaster recovery
scenarios. Emphasis is on rapid recovery, data integrity, and business
continuity in production environments.

### Recovery Time Objectives (RTO)

| Scenario | Target RTO | Maximum Downtime | Recovery Complexity |
|----------|------------|------------------|-------------------|
| Single Agent Failure | 2 minutes | 5 minutes | Low |
| Memory System Corruption | 10 minutes | 30 minutes | Medium |
| Full System Failure | 30 minutes | 2 hours | High |
| Disaster Recovery | 4 hours | 24 hours | Very High |

## Production Recovery Scenarios

### Agent Recovery Operations

#### Single Configuration Agent Recovery

```bash
#!/bin/bash
# Single config agent recovery procedure

AGENT_NAME="$1"
RECOVERY_REASON="${2:-unspecified}"

echo "=== CONFIG AGENT RECOVERY: $AGENT_NAME ==="
echo "Reason: $RECOVERY_REASON"
echo "Started: $(date)"

# Step 1: Assess agent state
echo "1. Assessing agent state..."
AGENT_STATUS=$(caxton agents status "$AGENT_NAME" --json)
echo "Current status: $(echo "$AGENT_STATUS" | jq -r '.state')"

# Step 2: Stop problematic agent
echo "2. Stopping agent safely..."
caxton agents stop "$AGENT_NAME" --graceful --timeout 30s

# Step 3: Backup current state if possible
if caxton agents backup "$AGENT_NAME" --quick-check; then
    echo "3. Creating recovery backup..."
    caxton agents backup "$AGENT_NAME" --state-only \
        > "/tmp/recovery-backup-$AGENT_NAME-$(date +%s).json"
else
    echo "3. Skipping backup (state corrupted)"
fi

# Step 4: Restore from last known good configuration
echo "4. Restoring from last known good..."
LAST_GOOD=$(caxton agents history "$AGENT_NAME" --last-working)
caxton agents restore "$AGENT_NAME" --from-backup "$LAST_GOOD"

# Step 5: Verify configuration integrity
echo "5. Verifying configuration..."
if caxton agents validate "$AGENT_NAME" --comprehensive; then
    echo "Configuration validated successfully"
else
    echo "ERROR: Configuration validation failed"
    exit 1
fi

# Step 6: Restore memory context
echo "6. Restoring memory context..."
caxton memory restore-context "$AGENT_NAME" --from-backup "$LAST_GOOD"

# Step 7: Start agent
echo "7. Starting recovered agent..."
caxton agents start "$AGENT_NAME" --verify-health

# Step 8: Resume processing
echo "8. Resuming message processing..."
caxton agents resume "$AGENT_NAME" --drain-queue

echo "=== RECOVERY COMPLETE: $AGENT_NAME ==="
echo "Recovery time: $(date)"
```

#### WASM Agent Recovery Procedure

```bash
#!/bin/bash
# WASM agent recovery with rollback capability

AGENT_NAME="$1"
RECOVERY_TYPE="${2:-automatic}"

echo "=== WASM AGENT RECOVERY: $AGENT_NAME ==="

# Step 1: Emergency isolation
echo "1. Emergency isolation..."
caxton wasm isolate "$AGENT_NAME" --immediate --preserve-state

# Step 2: Collect crash diagnostics
echo "2. Collecting crash diagnostics..."
caxton wasm crash-dump "$AGENT_NAME" \
    > "/tmp/crash-dump-$AGENT_NAME-$(date +%s).json"

# Step 3: Resource cleanup
echo "3. Cleaning up resources..."
caxton wasm cleanup-resources "$AGENT_NAME" --force

# Step 4: Determine recovery strategy
echo "4. Determining recovery strategy..."
case "$RECOVERY_TYPE" in
    "rollback")
        echo "Rolling back to previous working version..."
        PREVIOUS_VERSION=$(caxton wasm version-history "$AGENT_NAME" --last-working)
        caxton wasm rollback "$AGENT_NAME" --to-version "$PREVIOUS_VERSION"
        ;;
    "rebuild")
        echo "Rebuilding from source..."
        caxton wasm rebuild "$AGENT_NAME" --from-source --same-config
        ;;
    "automatic"|*)
        echo "Attempting automatic recovery..."
        if caxton wasm auto-recover "$AGENT_NAME" --with-diagnostics; then
            echo "Automatic recovery successful"
        else
            echo "Automatic recovery failed, falling back to rollback"
            PREVIOUS_VERSION=$(caxton wasm version-history "$AGENT_NAME" --last-working)
            caxton wasm rollback "$AGENT_NAME" --to-version "$PREVIOUS_VERSION"
        fi
        ;;
esac

# Step 5: Security validation
echo "5. Security validation..."
caxton wasm security-validate "$AGENT_NAME" --full-check

# Step 6: Resource limit verification
echo "6. Verifying resource limits..."
caxton wasm verify-limits "$AGENT_NAME" --conservative

# Step 7: Controlled restart
echo "7. Controlled restart..."
caxton wasm start "$AGENT_NAME" --monitored --resource-tracking

echo "=== WASM RECOVERY COMPLETE ==="
```

#### Bulk Agent Recovery

```bash
#!/bin/bash
# Bulk agent recovery for multiple failed agents

echo "=== BULK AGENT RECOVERY ==="

# Identify failed agents
FAILED_AGENTS=$(caxton agents list --status failed,crashed --names-only)
AGENT_COUNT=$(echo "$FAILED_AGENTS" | wc -l)

echo "Recovering $AGENT_COUNT failed agents..."

# Create recovery plan
echo "Creating recovery plan..."
for agent in $FAILED_AGENTS; do
    AGENT_TYPE=$(caxton agents type "$agent")
    PRIORITY=$(caxton agents priority "$agent")
    echo "$agent:$AGENT_TYPE:$PRIORITY"
done | sort -t: -k3 -nr > /tmp/recovery-plan.txt

# Execute recovery by priority
echo "Executing priority-based recovery..."
while IFS=: read -r agent type priority; do
    echo "Recovering $agent (type: $type, priority: $priority)..."

    if [ "$type" = "config" ]; then
        /usr/local/bin/recover-config-agent.sh "$agent" "bulk_recovery" &
    else
        /usr/local/bin/recover-wasm-agent.sh "$agent" "automatic" &
    fi

    # Limit concurrent recoveries
    if [ "$(jobs -r | wc -l)" -ge 5 ]; then
        wait
    fi
done < /tmp/recovery-plan.txt

# Wait for all recoveries to complete
wait

echo "=== BULK RECOVERY COMPLETE ==="
```

### Memory System Recovery

#### SQLite Database Recovery

```bash
#!/bin/bash
# SQLite embedded database recovery

echo "=== MEMORY SYSTEM RECOVERY ==="

# Step 1: Stop all services using the database
echo "1. Stopping services..."
systemctl stop caxton
sleep 5

# Step 2: Database integrity check
echo "2. Checking database integrity..."
DB_PATH="/var/lib/caxton/data/memory.db"
if sqlite3 "$DB_PATH" "PRAGMA integrity_check;" | grep -q "ok"; then
    echo "Database integrity OK"
    CORRUPTION_LEVEL="none"
else
    echo "Database corruption detected"
    CORRUPTION_LEVEL="detected"
fi

# Step 3: Backup current database
echo "3. Backing up current database..."
cp "$DB_PATH" "$DB_PATH.recovery-backup-$(date +%s)"

# Step 4: Recovery based on corruption level
case "$CORRUPTION_LEVEL" in
    "none")
        echo "4. No corruption, checking WAL recovery..."
        sqlite3 "$DB_PATH" "PRAGMA wal_checkpoint(FULL);"
        ;;
    "detected")
        echo "4. Attempting database recovery..."

        # Try to recover from WAL
        if [ -f "$DB_PATH-wal" ]; then
            echo "Recovering from WAL file..."
            sqlite3 "$DB_PATH" "PRAGMA wal_checkpoint(RESTART);"
        fi

        # If still corrupted, try backup recovery
        if ! sqlite3 "$DB_PATH" "PRAGMA integrity_check;" | grep -q "ok"; then
            echo "WAL recovery failed, restoring from backup..."
            LATEST_BACKUP=$(ls -t /backup/caxton/caxton-*.tar.gz.gpg | head -1)

            if [ -n "$LATEST_BACKUP" ]; then
                caxton backup restore "$LATEST_BACKUP" --memory-only
            else
                echo "No backup available, rebuilding empty database..."
                caxton memory reinitialize --confirm-data-loss
            fi
        fi
        ;;
esac

# Step 5: Rebuild indexes
echo "5. Rebuilding indexes..."
caxton memory reindex --full --optimize

# Step 6: Verify recovery
echo "6. Verifying recovery..."
systemctl start caxton
sleep 10

if caxton memory health-check --comprehensive; then
    echo "Memory system recovery successful"
else
    echo "Recovery verification failed"
    exit 1
fi

echo "=== MEMORY RECOVERY COMPLETE ==="
```

#### Memory Data Recovery and Rebuild

```bash
#!/bin/bash
# Memory data recovery with entity reconstruction

echo "=== MEMORY DATA RECOVERY ==="

# Step 1: Export recoverable data
echo "1. Exporting recoverable data..."
caxton memory export-partial --corrupted-db --format json \
    > "/tmp/partial-memory-export-$(date +%s).json"

# Step 2: Analyze data loss extent
echo "2. Analyzing data loss..."
caxton memory analyze-loss --export-file /tmp/partial-memory-export-*.json

# Step 3: Reinitialize memory system
echo "3. Reinitializing memory system..."
caxton service stop
caxton memory reinitialize --preserve-schema
caxton service start

# Step 4: Restore recoverable data
echo "4. Restoring recoverable data..."
caxton memory import /tmp/partial-memory-export-*.json --validate --skip-corrupted

# Step 5: Rebuild agent memory contexts
echo "5. Rebuilding agent memory contexts..."
caxton agents list --names-only | while read -r agent; do
    echo "Rebuilding memory context for $agent..."
    caxton memory rebuild-context "$agent" --from-conversations
done

# Step 6: Verify memory integrity
echo "6. Verifying memory integrity..."
caxton memory integrity-check --full --repair-minor

echo "=== MEMORY DATA RECOVERY COMPLETE ==="
```

### System-Wide Recovery

#### Full System Recovery from Backup

```bash
#!/bin/bash
# Complete system recovery procedure

BACKUP_FILE="$1"
RECOVERY_TYPE="${2:-full}"

echo "=== FULL SYSTEM RECOVERY ==="
echo "Backup: $BACKUP_FILE"
echo "Type: $RECOVERY_TYPE"

# Step 1: Validate backup
echo "1. Validating backup..."
if ! caxton backup validate "$BACKUP_FILE" --comprehensive; then
    echo "ERROR: Backup validation failed"
    exit 1
fi

# Step 2: Stop all services
echo "2. Stopping all services..."
systemctl stop caxton
systemctl stop prometheus || true
systemctl stop grafana-server || true

# Step 3: Create pre-recovery snapshot
echo "3. Creating pre-recovery snapshot..."
tar -czf "/tmp/pre-recovery-$(date +%s).tar.gz" \
    /var/lib/caxton/ \
    /etc/caxton/ \
    2>/dev/null || echo "Warning: Some files may not be backed up"

# Step 4: Restore from backup
echo "4. Restoring from backup..."
case "$RECOVERY_TYPE" in
    "full")
        caxton backup restore "$BACKUP_FILE" --full-system --overwrite
        ;;
    "config_only")
        caxton backup restore "$BACKUP_FILE" --config-only
        ;;
    "data_only")
        caxton backup restore "$BACKUP_FILE" --data-only
        ;;
esac

# Step 5: Update file permissions
echo "5. Updating file permissions..."
chown -R caxton:caxton /var/lib/caxton/
chmod 700 /var/lib/caxton/data/
chmod 755 /var/lib/caxton/agents/
chmod 644 /var/lib/caxton/agents/*.md

# Step 6: Configuration validation
echo "6. Validating configuration..."
caxton config validate --all --fix-permissions

# Step 7: Memory system recovery
echo "7. Recovering memory system..."
caxton memory recovery-check --auto-repair

# Step 8: Start services
echo "8. Starting services..."
systemctl start caxton

# Step 9: Health verification
echo "9. Health verification..."
sleep 30
if caxton health-check --comprehensive --timeout 60s; then
    echo "System recovery successful"
else
    echo "System recovery verification failed"
    exit 1
fi

# Step 10: Agent restoration
echo "10. Restoring agents..."
caxton agents restore-all --from-backup --verify-health

echo "=== SYSTEM RECOVERY COMPLETE ==="
```

#### Disaster Recovery Procedure

```bash
#!/bin/bash
# Disaster recovery for complete infrastructure loss

DISASTER_TYPE="$1"  # fire, flood, hardware_failure, security_breach
RECOVERY_SITE="$2"  # primary, secondary, cloud

echo "=== DISASTER RECOVERY PROCEDURE ==="
echo "Disaster Type: $DISASTER_TYPE"
echo "Recovery Site: $RECOVERY_SITE"
echo "Started: $(date)"

# Step 1: Activate disaster recovery team
echo "1. Activating disaster recovery team..."
/usr/local/bin/notify-disaster-team.sh "$DISASTER_TYPE" "$RECOVERY_SITE"

# Step 2: Secure backup access
echo "2. Securing backup access..."
case "$RECOVERY_SITE" in
    "cloud")
        aws s3 sync s3://caxton-disaster-backups/ /tmp/disaster-recovery/
        ;;
    "secondary")
        rsync -avz disaster-backup-server:/caxton-backups/ /tmp/disaster-recovery/
        ;;
esac

# Step 3: Provision new infrastructure
echo "3. Provisioning infrastructure..."
if [ "$RECOVERY_SITE" = "cloud" ]; then
    terraform apply -var="disaster_recovery=true" \
                   -var="backup_location=/tmp/disaster-recovery"
fi

# Step 4: Install Caxton
echo "4. Installing Caxton..."
wget -O /tmp/caxton-latest.tar.gz \
    https://github.com/your-org/caxton/releases/latest/download/caxton-linux-x64.tar.gz
tar -xzf /tmp/caxton-latest.tar.gz -C /usr/local/bin/

# Step 5: Restore latest backup
echo "5. Restoring from latest backup..."
LATEST_BACKUP=$(ls -t /tmp/disaster-recovery/caxton-*.tar.gz.gpg | head -1)
/usr/local/bin/full-system-recovery.sh "$LATEST_BACKUP" full

# Step 6: Update DNS and networking
echo "6. Updating DNS and network configuration..."
case "$RECOVERY_SITE" in
    "cloud")
        aws route53 change-resource-record-sets \
            --hosted-zone-id Z123456789 \
            --change-batch file://dns-failover.json
        ;;
    "secondary")
        /usr/local/bin/update-dns-failover.sh secondary
        ;;
esac

# Step 7: Validate full system operation
echo "7. Validating full system operation..."
caxton system-test --disaster-recovery --comprehensive

# Step 8: Notify stakeholders
echo "8. Notifying stakeholders..."
/usr/local/bin/notify-recovery-complete.sh "$DISASTER_TYPE" "$RECOVERY_SITE"

echo "=== DISASTER RECOVERY COMPLETE ==="
echo "Completed: $(date)"
```

## Production Recovery Testing

### Recovery Testing Schedule

```bash
#!/bin/bash
# Monthly disaster recovery testing

echo "=== DISASTER RECOVERY TESTING ==="

# Test 1: Agent recovery test
echo "1. Testing agent recovery..."
TEST_AGENT="dr-test-agent"
caxton agents create-test-agent "$TEST_AGENT"
caxton agents crash-test "$TEST_AGENT" --simulated
sleep 10
if caxton agents recover-test "$TEST_AGENT"; then
    echo "✓ Agent recovery test passed"
else
    echo "✗ Agent recovery test failed"
fi

# Test 2: Memory system recovery test
echo "2. Testing memory system recovery..."
caxton memory create-test-corruption --simulated
if caxton memory recovery-test --auto-repair; then
    echo "✓ Memory recovery test passed"
else
    echo "✗ Memory recovery test failed"
fi

# Test 3: Backup restoration test
echo "3. Testing backup restoration..."
LATEST_BACKUP=$(ls -t /backup/caxton/*.tar.gz.gpg | head -1)
if caxton backup test-restore "$LATEST_BACKUP" --dry-run; then
    echo "✓ Backup restoration test passed"
else
    echo "✗ Backup restoration test failed"
fi

# Test 4: Full system recovery test
echo "4. Testing full system recovery..."
if caxton system recovery-test --staging-environment; then
    echo "✓ Full system recovery test passed"
else
    echo "✗ Full system recovery test failed"
fi

echo "=== DISASTER RECOVERY TESTING COMPLETE ==="
```

### Recovery Time Measurement

```bash
#!/bin/bash
# Measure and track recovery times

echo "=== RECOVERY TIME MEASUREMENT ==="

measure_recovery_time() {
    local recovery_type="$1"
    local start_time=$(date +%s)

    case "$recovery_type" in
        "single_agent")
            caxton agents recovery-benchmark --type config --simulated
            ;;
        "memory_system")
            caxton memory recovery-benchmark --corruption-level medium
            ;;
        "full_system")
            caxton system recovery-benchmark --from-backup --staging
            ;;
    esac

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    echo "$recovery_type: ${duration}s"
    echo "$(date),$recovery_type,$duration" >> /var/log/caxton/recovery-times.csv
}

# Run benchmarks
measure_recovery_time "single_agent"
measure_recovery_time "memory_system"
measure_recovery_time "full_system"

# Generate report
echo "Recovery time trends:"
tail -30 /var/log/caxton/recovery-times.csv | \
awk -F, '{sum[$2]+=$3; count[$2]++} END {
    for (type in sum) print type ": avg " sum[type]/count[type] "s"
}'
```

## Recovery Verification

### Post-Recovery Health Checks

```bash
#!/bin/bash
# Comprehensive post-recovery health verification

echo "=== POST-RECOVERY HEALTH CHECKS ==="

# Check 1: System status
echo "1. System status check..."
if caxton status --production-ready; then
    echo "✓ System status: HEALTHY"
else
    echo "✗ System status: UNHEALTHY"
    exit 1
fi

# Check 2: All agents operational
echo "2. Agent status check..."
FAILED_AGENTS=$(caxton agents list --status failed --count)
if [ "$FAILED_AGENTS" -eq 0 ]; then
    echo "✓ All agents operational"
else
    echo "✗ $FAILED_AGENTS agents still failed"
    caxton agents list --status failed --details
fi

# Check 3: Memory system integrity
echo "3. Memory system integrity..."
if caxton memory integrity-check --quick; then
    echo "✓ Memory system: HEALTHY"
else
    echo "✗ Memory system: ISSUES DETECTED"
    caxton memory integrity-check --detailed
fi

# Check 4: API functionality
echo "4. API functionality check..."
if curl -f -s https://caxton.yourdomain.com/health >/dev/null; then
    echo "✓ API endpoints: RESPONSIVE"
else
    echo "✗ API endpoints: UNRESPONSIVE"
fi

# Check 5: Performance baseline
echo "5. Performance baseline check..."
BASELINE_RESPONSE_TIME=1.0  # seconds
CURRENT_RESPONSE_TIME=$(caxton performance measure --avg-response-time)
if (( $(echo "$CURRENT_RESPONSE_TIME < $BASELINE_RESPONSE_TIME" | bc -l) )); then
    echo "✓ Performance: WITHIN BASELINE"
else
    echo "⚠ Performance: DEGRADED (${CURRENT_RESPONSE_TIME}s vs ${BASELINE_RESPONSE_TIME}s)"
fi

echo "=== HEALTH CHECKS COMPLETE ==="
```

### Data Integrity Verification

```bash
#!/bin/bash
# Verify data integrity after recovery

echo "=== DATA INTEGRITY VERIFICATION ==="

# Check 1: Configuration integrity
echo "1. Configuration integrity..."
caxton config integrity-check --all-agents --checksums
if [ $? -eq 0 ]; then
    echo "✓ Configuration integrity: VERIFIED"
else
    echo "✗ Configuration integrity: COMPROMISED"
fi

# Check 2: Memory data consistency
echo "2. Memory data consistency..."
caxton memory consistency-check --full --repair-minor
if [ $? -eq 0 ]; then
    echo "✓ Memory consistency: VERIFIED"
else
    echo "✗ Memory consistency: INCONSISTENT"
fi

# Check 3: Backup verification
echo "3. Backup verification..."
LATEST_BACKUP=$(ls -t /backup/caxton/*.tar.gz.gpg | head -1)
caxton backup verify "$LATEST_BACKUP" --checksum --structure
if [ $? -eq 0 ]; then
    echo "✓ Backup integrity: VERIFIED"
else
    echo "✗ Backup integrity: CORRUPTED"
fi

echo "=== DATA INTEGRITY VERIFICATION COMPLETE ==="
```

## Recovery Documentation

### Recovery Incident Report Template

```markdown
# Caxton Recovery Incident Report

**Incident ID**: REC-{{ date }}-{{ sequence }}
**Severity**: {{ P0|P1|P2|P3 }}
**Recovery Duration**: {{ start_time }} to {{ end_time }}

## Incident Summary
Brief description of the failure and recovery operation.

## Failure Analysis
### Root Cause
Technical analysis of what caused the initial failure.

### Impact Assessment
- **Affected Components**: {{ list }}
- **Data Loss**: {{ none|minimal|significant }}
- **Service Downtime**: {{ duration }}
- **Affected Users**: {{ count }}

## Recovery Actions Taken
### Immediate Response
1. **{{ timestamp }}**: {{ action_taken }}
2. **{{ timestamp }}**: {{ action_taken }}

### Recovery Procedure
1. **{{ step }}**: {{ description }} - {{ duration }}
2. **{{ step }}**: {{ description }} - {{ duration }}

## Recovery Verification
- [ ] System health check passed
- [ ] All agents operational
- [ ] Memory system integrity verified
- [ ] API endpoints responsive
- [ ] Performance within baseline
- [ ] Data integrity confirmed

## Lessons Learned
### What Worked Well
- {{ success_factor }}

### Areas for Improvement
- {{ improvement_area }}

### Recommendations
- [ ] {{ recommendation }}
- [ ] {{ recommendation }}

**Prepared by**: {{ operator_name }}
**Reviewed by**: {{ senior_operator }}
**Date**: {{ report_date }}
```

<function_calls>
<invoke name="TodoWrite">
<parameter name="todos">[
  {"content": "Create troubleshooting guide from error-handling-guide requirements",
   "status": "completed",
   "activeForm": "Created troubleshooting guide for operators"},
  {"content": "Create recovery guide from state-recovery-patterns with
   operational focus",
   "status": "completed",
   "activeForm": "Created recovery guide for operators"},
  {"content": "Store documentation reorganization patterns in memento",
   "status": "in_progress",
   "activeForm": "Storing documentation patterns in memento"}]
