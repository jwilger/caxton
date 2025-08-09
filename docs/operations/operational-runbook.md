# Operational Runbook

This runbook provides step-by-step procedures for operating Caxton in production. For architectural decisions behind these procedures, see [ADR-0018: Operational Procedures](../adr/0018-operational-procedures.md).

## Quick Reference

| Situation | Command | Page |
|-----------|---------|------|
| Node down | `caxton cluster members` | [Node Failure](#node-failure) |
| High latency | `caxton cluster performance` | [Performance Issues](#performance-degradation) |
| Network partition | `caxton cluster detect-partition` | [Partition Handling](#network-partition) |
| Upgrade cluster | `caxton cluster upgrade` | [Rolling Upgrade](#rolling-upgrade) |
| Emergency stop | `caxton emergency stop` | [Emergency Procedures](#emergency-procedures) |

## Initial Setup

### Prerequisites Checklist

- [ ] TLS certificates generated
- [ ] Network ports open (8080, 7946, 9090)
- [ ] Storage directories created
- [ ] Configuration files in place
- [ ] Monitoring endpoints configured

### First-Time Bootstrap

```bash
#!/bin/bash
# Bootstrap new cluster

# 1. Generate certificates
caxton security init-ca --output /etc/caxton/certs/

# 2. Create configuration
cat > /etc/caxton/config.yaml << EOF
coordination:
  cluster:
    bind_addr: 0.0.0.0:7946
    bootstrap_expect: 3
security:
  cluster:
    mtls:
      enabled: true
      ca_cert: /etc/caxton/certs/ca.crt
EOF

# 3. Start seed node
caxton server start --bootstrap --config /etc/caxton/config.yaml
```

## Common Operations

### Node Failure

#### Detection
```bash
# Check node status
caxton cluster members

# If node shows as 'failed' or 'suspect':
NODE-ID    STATUS    LAST-SEEN    AGENTS
node-2     suspect   45s ago      38
```

#### Diagnosis
```bash
# 1. Try to ping the node
ping node-2.example.com

# 2. Check if Caxton process is running
ssh node-2 'systemctl status caxton'

# 3. Check system resources
ssh node-2 'top -bn1 | head -5'

# 4. Review logs
ssh node-2 'tail -100 /var/log/caxton/caxton.log'
```

#### Recovery
```bash
# Option 1: Restart the service
ssh node-2 'systemctl restart caxton'

# Option 2: Rejoin cluster manually
ssh node-2 'caxton server start --join node-1:7946'

# Option 3: If unrecoverable, remove and replace
caxton cluster remove --node node-2 --force
# Then bootstrap new node
```

### Performance Degradation

#### Detection
```bash
# Check performance metrics
caxton cluster performance

# If latencies exceed targets:
METRIC                TARGET    ACTUAL    STATUS
Message routing P99   1ms       12.3ms    ✗ DEGRADED
```

#### Diagnosis
```bash
# 1. Check message queue depth
caxton queue stats
# Queue depth > 10000 indicates backlog

# 2. Identify slow agents
caxton agents slow --threshold 100ms
# AGENT-ID          AVG-LATENCY    P99-LATENCY
# processor-5       250ms          1.2s

# 3. Check resource utilization
caxton resources
# NODE      CPU    MEMORY    AGENTS
# node-1    89%    7.2GB     3421
```

#### Mitigation
```bash
# 1. Suspend slow agents
caxton agent suspend processor-5

# 2. Scale horizontally
caxton cluster add-node new-node:7946

# 3. Rebalance agents
caxton cluster rebalance --strategy least-loaded

# 4. Clear message backlog if needed
caxton queue drain --timeout 60s
```

### Network Partition

#### Detection
```bash
# Check for partitions
caxton cluster detect-partition

# Output if partitioned:
WARNING: Network partition detected
Partition A: [node-1, node-2] (majority)
Partition B: [node-3] (minority)
```

#### During Partition

**On Majority Side:**
```bash
# Verify majority status
caxton cluster status
# Status: HEALTHY (majority partition)

# Continue operations normally
# Minority nodes marked as failed after timeout
```

**On Minority Side:**
```bash
# Check degraded mode
caxton cluster status
# Status: DEGRADED (minority partition)
# Operations: READ-ONLY
# Queued writes: 142

# Monitor queue growth
watch -n 5 'caxton queue stats'
```

#### Healing Partition
```bash
# 1. Fix network issue
# (resolve firewall/routing/dns problem)

# 2. Verify connectivity restored
caxton cluster ping node-3

# 3. Monitor automatic healing
caxton cluster watch
# [INFO] Partition healed, merging state
# [INFO] Replaying 142 queued messages
# [INFO] Cluster synchronized

# 4. Verify consistency
caxton cluster verify
```

## Maintenance Procedures

### Rolling Upgrade

#### Pre-Upgrade Checklist
- [ ] Backup completed
- [ ] Upgrade tested in staging
- [ ] Rollback plan documented
- [ ] Maintenance window scheduled

#### Upgrade Process
```bash
# 1. Start upgrade coordinator
caxton cluster upgrade start --version v1.2.0

# 2. Upgrade will proceed automatically:
# - Select canary node
# - Drain traffic from canary
# - Upgrade canary
# - Monitor for 24h (or --canary-duration)
# - Proceed with remaining nodes

# 3. Monitor progress
caxton cluster upgrade status
# PHASE: Rolling upgrade
# PROGRESS: 2/5 nodes upgraded
# CANARY: Healthy for 23h 45m
# ETA: 45 minutes

# 4. If issues detected, rollback
caxton cluster upgrade rollback
```

### Backup and Recovery

#### Scheduled Backup
```bash
# Create full backup
caxton backup create \
  --type full \
  --destination s3://backups/caxton/$(date +%Y%m%d)/ \
  --compress

# Verify backup
caxton backup verify --id backup-20240115-0200
```

#### Recovery from Backup
```bash
# 1. Stop cluster
caxton cluster stop --all

# 2. Restore from backup
caxton backup restore \
  --id backup-20240115-0200 \
  --target /var/lib/caxton/

# 3. Start cluster
caxton cluster start --verify
```

### Certificate Rotation

```bash
# 1. Check certificate expiry
caxton security cert-status
# NODE-ID    EXPIRES-IN    STATUS
# node-1     25 days       WARNING
# node-2     25 days       WARNING

# 2. Generate new certificates
caxton security rotate-certs \
  --ca-cert /etc/caxton/ca.crt \
  --ca-key /etc/caxton/ca.key

# 3. Rolling restart with new certs
caxton cluster rolling-restart --reason cert-rotation
```

## Emergency Procedures

### Emergency Stop
```bash
# Stop all agents immediately
caxton emergency stop --all-agents

# Stop entire cluster
caxton emergency stop --cluster

# Stop with state preservation
caxton emergency stop --preserve-state --dump-to /backup/emergency/
```

### Data Corruption Recovery
```bash
# 1. Identify corrupted node
caxton cluster verify --deep
# ERROR: Node-2 state corruption detected

# 2. Isolate corrupted node
caxton cluster isolate --node node-2

# 3. Rebuild from peers
caxton cluster rebuild --node node-2 --from-peers

# 4. Verify and rejoin
caxton cluster verify --node node-2
caxton cluster rejoin --node node-2
```

### Memory Exhaustion
```bash
# 1. Identify memory usage
caxton memory stats
# NODE      USED     LIMIT    AGENTS
# node-1    15.2GB   16GB     4521

# 2. Suspend low-priority agents
caxton agents suspend --priority low

# 3. Force garbage collection
caxton memory gc --aggressive

# 4. If still critical, shed load
caxton cluster shed-load --percentage 20
```

## Monitoring and Alerting

### Key Metrics to Watch

```bash
# Cluster health
curl -s localhost:9090/metrics | grep caxton_cluster_
# caxton_cluster_nodes_alive 5
# caxton_cluster_nodes_failed 0
# caxton_cluster_gossip_latency_ms 1.2

# Agent performance
curl -s localhost:9090/metrics | grep caxton_agent_
# caxton_agent_message_latency_p99 0.067
# caxton_agent_crashes_total 0
# caxton_agent_memory_used_bytes 5242880

# System resources
curl -s localhost:9090/metrics | grep caxton_system_
# caxton_system_cpu_usage_percent 34.5
# caxton_system_memory_available_bytes 8589934592
```

### Alert Response

#### Critical Alerts

**Cluster Split Brain**
```bash
# Immediate response
caxton cluster detect-partition --resolve-strategy manual

# Identify correct partition
caxton cluster compare-state

# Force resolution
caxton cluster resolve-partition --prefer majority
```

**Agent Storm (Cascading Failures)**
```bash
# Stop cascade
caxton circuit-breaker activate --all

# Identify root cause
caxton trace --error-cascade --last 5m

# Gradual recovery
caxton circuit-breaker reset --gradual --duration 10m
```

## Troubleshooting

### Debug Commands

```bash
# Trace specific conversation
caxton trace --conversation-id abc-123

# Profile agent performance
caxton profile --agent processor-1 --duration 60s

# Dump system state
caxton debug dump --output debug-$(date +%s).tar.gz

# Analyze message patterns
caxton analyze messages --pattern "timeout|error" --last 1h

# Check configuration drift
caxton config diff --baseline /etc/caxton/config.yaml
```

### Common Issues

| Symptom | Likely Cause | Solution |
|---------|--------------|----------|
| Agents not discovered | Gossip not converging | Check network, increase gossip_interval |
| High memory usage | Message queue backlog | Check slow agents, increase throughput |
| Cluster won't form | mTLS mismatch | Verify certificates on all nodes |
| Degraded performance | Resource exhaustion | Add nodes or reduce agent count |
| Messages lost | Partition during write | Check partition logs, replay from queue |

## Best Practices

1. **Always backup before upgrades**
2. **Test in staging first**
3. **Monitor key metrics continuously**
4. **Document all changes**
5. **Keep runbook updated**
6. **Practice emergency procedures**
7. **Maintain 20% resource headroom**

## References

- [ADR-0018: Operational Procedures](../adr/0018-operational-procedures.md)
- [Clustering Guide](../user-guide/clustering.md)
- [Performance Requirements](../adr/0017-performance-requirements.md)
- [Security Architecture](../adr/0016-security-architecture.md)
