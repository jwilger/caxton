---
layout: adr
title: "0018. Operational Procedures"
status: accepted
date: 2025-08-09
---

# ADR-0018: Operational Procedures

## Status
Accepted

## Context
Caxton's distributed architecture requires well-defined operational procedures for deployment, maintenance, and incident response. With the coordination-first architecture (ADR-0014) and security requirements (ADR-0016), we need clear procedures for bootstrapping clusters, performing upgrades, handling failures, and maintaining system health.

These procedures must be simple enough for small teams while being robust enough for production environments.

## Decision

### Cluster Bootstrap Procedure

#### Initial Cluster Formation
```bash
#!/bin/bash
# Bootstrap script for initial cluster formation

# Step 1: Generate cluster certificates
caxton security init-ca \
  --ca-cert /etc/caxton/ca.crt \
  --ca-key /etc/caxton/ca.key \
  --validity 3650d

# Step 2: Initialize first node (seed node)
caxton init \
  --node-id node-1 \
  --bind-addr 0.0.0.0:7946 \
  --api-addr 0.0.0.0:8080 \
  --data-dir /var/lib/caxton \
  --cert-dir /etc/caxton/certs \
  --bootstrap-expect 3  # Expected cluster size

# Step 3: Generate node certificate
caxton security gen-cert \
  --node-id node-1 \
  --ca-cert /etc/caxton/ca.crt \
  --ca-key /etc/caxton/ca.key \
  --output /etc/caxton/certs/

# Step 4: Start seed node
caxton server start \
  --config /etc/caxton/config.yaml \
  --bootstrap
```

#### Adding Nodes to Cluster
```bash
# On new node:

# Step 1: Copy CA certificate
scp seed-node:/etc/caxton/ca.crt /etc/caxton/

# Step 2: Generate node certificate
caxton security gen-cert \
  --node-id node-2 \
  --ca-cert /etc/caxton/ca.crt \
  --ca-key /etc/caxton/ca.key \
  --output /etc/caxton/certs/

# Step 3: Join cluster
caxton server start \
  --config /etc/caxton/config.yaml \
  --join node-1.example.com:7946,node-2.example.com:7946

# Step 4: Verify cluster membership
caxton cluster members
```

#### Automated Bootstrap
```rust
pub struct ClusterBootstrap {
    config: BootstrapConfig,
    state: BootstrapState,
}

impl ClusterBootstrap {
    pub async fn bootstrap(&mut self) -> Result<()> {
        // Phase 1: Pre-flight checks
        self.verify_network_connectivity().await?;
        self.verify_certificates().await?;
        self.verify_data_directories().await?;

        // Phase 2: Seed node initialization
        if self.is_seed_node() {
            self.initialize_seed_node().await?;
            self.wait_for_quorum().await?;
        } else {
            // Phase 3: Join existing cluster
            self.discover_seed_nodes().await?;
            self.join_cluster().await?;
            self.synchronize_state().await?;
        }

        // Phase 4: Post-bootstrap validation
        self.verify_cluster_health().await?;
        self.run_smoke_tests().await?;

        Ok(())
    }
}
```

### Rolling Upgrade Procedure

#### Zero-Downtime Upgrade Strategy
```yaml
upgrade_strategy:
  type: rolling
  phases:
    - name: pre_upgrade_validation
      steps:
        - verify_cluster_health
        - backup_critical_state
        - verify_version_compatibility

    - name: canary_deployment
      steps:
        - select_canary_node
        - drain_canary_traffic
        - upgrade_canary
        - validate_canary
        - monitor_canary_24h

    - name: rolling_upgrade
      steps:
        - foreach_node:
            - drain_traffic
            - checkpoint_state
            - perform_upgrade
            - health_check
            - restore_traffic
        - batch_size: 1
        - batch_delay: 5m

    - name: post_upgrade_validation
      steps:
        - verify_cluster_consistency
        - run_integration_tests
        - update_documentation
```

#### Upgrade Automation Script
```rust
pub struct UpgradeOrchestrator {
    cluster: ClusterHandle,
    strategy: UpgradeStrategy,
}

impl UpgradeOrchestrator {
    pub async fn perform_upgrade(&self, target_version: Version) -> Result<()> {
        // Pre-upgrade checks
        self.verify_upgrade_path(target_version).await?;
        let snapshot = self.create_cluster_snapshot().await?;

        // Canary phase
        if self.strategy.use_canary {
            let canary = self.select_canary_node().await?;
            self.upgrade_node(canary, target_version).await?;
            self.monitor_canary(Duration::from_secs(86400)).await?; // 24h
        }

        // Rolling upgrade
        let nodes = self.cluster.nodes_by_upgrade_order().await?;
        for batch in nodes.chunks(self.strategy.batch_size) {
            for node in batch {
                self.drain_node_traffic(node).await?;
                self.upgrade_node(node, target_version).await?;
                self.verify_node_health(node).await?;
                self.restore_node_traffic(node).await?;
            }

            // Wait between batches
            tokio::time::sleep(self.strategy.batch_delay).await;

            // Verify cluster health after each batch
            self.verify_cluster_health().await?;
        }

        // Post-upgrade validation
        self.verify_version_consistency().await?;
        self.run_smoke_tests().await?;

        Ok(())
    }
}
```

### Backup and Recovery Procedures

#### Backup Strategy
```yaml
backup:
  schedule:
    full_backup:
      frequency: daily
      time: 02:00
      retention: 30d

    incremental_backup:
      frequency: hourly
      retention: 7d

  destinations:
    - type: local
      path: /backup/caxton
    - type: s3
      bucket: caxton-backups
      region: us-east-1

  components:
    - agent_registry
    - agent_state
    - message_queues
    - configuration
    - certificates
```

#### Backup Implementation
```rust
pub struct BackupManager {
    config: BackupConfig,
    storage: BackupStorage,
}

impl BackupManager {
    pub async fn perform_backup(&self, backup_type: BackupType) -> Result<BackupId> {
        let backup_id = BackupId::new();

        // Create backup manifest
        let manifest = BackupManifest {
            id: backup_id.clone(),
            timestamp: Utc::now(),
            backup_type,
            node_id: self.node_id(),
            version: env!("CARGO_PKG_VERSION"),
        };

        // Backup components
        let components = match backup_type {
            BackupType::Full => {
                self.backup_agent_registry(&backup_id).await?;
                self.backup_agent_state(&backup_id).await?;
                self.backup_configuration(&backup_id).await?;
                self.backup_certificates(&backup_id).await?;
            }
            BackupType::Incremental => {
                self.backup_changed_agents(&backup_id).await?;
                self.backup_message_queues(&backup_id).await?;
            }
        };

        // Store backup
        self.storage.store_backup(backup_id, manifest, components).await?;

        // Verify backup integrity
        self.verify_backup(&backup_id).await?;

        Ok(backup_id)
    }
}
```

#### Recovery Procedures
```rust
pub struct RecoveryManager {
    backup_storage: BackupStorage,
}

impl RecoveryManager {
    pub async fn restore_from_backup(&self, backup_id: BackupId) -> Result<()> {
        // Verify backup integrity
        let manifest = self.backup_storage.get_manifest(&backup_id).await?;
        self.verify_backup_integrity(&backup_id, &manifest).await?;

        // Stop services
        self.stop_agent_runtime().await?;

        // Restore data
        self.restore_agent_registry(&backup_id).await?;
        self.restore_agent_state(&backup_id).await?;
        self.restore_configuration(&backup_id).await?;

        // Restart services
        self.start_agent_runtime().await?;

        // Verify restoration
        self.verify_restoration().await?;

        Ok(())
    }
}
```

### Monitoring and Alerting

#### Health Check Procedures
```yaml
health_checks:
  cluster_health:
    interval: 30s
    checks:
      - cluster_membership
      - gossip_convergence
      - partition_detection

  node_health:
    interval: 10s
    checks:
      - cpu_usage < 80%
      - memory_usage < 90%
      - disk_usage < 85%
      - agent_count < max_agents

  agent_health:
    interval: 60s
    checks:
      - agent_responsiveness
      - message_processing_time
      - error_rate < 1%
```

#### Alert Configuration
```yaml
alerts:
  critical:
    - name: cluster_split_brain
      condition: partition_detected
      action: page_oncall

    - name: node_down
      condition: node_unreachable > 60s
      action: page_oncall

    - name: high_error_rate
      condition: error_rate > 5%
      action: page_oncall

  warning:
    - name: high_cpu_usage
      condition: cpu_usage > 70%
      duration: 5m
      action: send_alert

    - name: disk_space_low
      condition: disk_usage > 80%
      action: send_alert

    - name: certificate_expiring
      condition: cert_expiry < 30d
      action: send_alert
```

### Incident Response Procedures

#### Runbook: Node Failure
```markdown
## Node Failure Response

### Detection
- Alert: "Node <node-id> unreachable"
- Dashboard: Node shown as red in cluster view

### Diagnosis
1. Check network connectivity: `ping <node-ip>`
2. Check node process: `ssh <node> 'ps aux | grep caxton'`
3. Check logs: `ssh <node> 'tail -f /var/log/caxton/caxton.log'`
4. Check system resources: `ssh <node> 'top'`

### Mitigation
1. If process crashed:
   ```bash
   ssh <node> 'systemctl restart caxton'
   ```

2. If out of memory:
   ```bash
   ssh <node> 'systemctl stop caxton'
   ssh <node> 'echo 3 > /proc/sys/vm/drop_caches'
   ssh <node> 'systemctl start caxton'
   ```

3. If network partition:
   - Wait for automatic healing (5 minutes)
   - If not healed, restart SWIM: `caxton cluster rejoin`

### Recovery
1. Verify node rejoined: `caxton cluster members`
2. Verify agent redistribution: `caxton agents list`
3. Check for message loss: `caxton messages stats`

### Post-Incident
1. Analyze root cause from logs
2. Update monitoring if new failure mode
3. Document in incident log
```

#### Runbook: Performance Degradation
```markdown
## Performance Degradation Response

### Detection
- Alert: "P99 latency > 100ms"
- Dashboard: Latency graphs showing spike

### Diagnosis
1. Check message queue depth: `caxton queue stats`
2. Check agent count: `caxton agents count`
3. Check slow agents: `caxton agents slow --threshold 10s`
4. Check network latency: `caxton cluster ping`

### Mitigation
1. If queue overflow:
   ```bash
   caxton queue drain --timeout 60s
   ```

2. If too many agents:
   ```bash
   caxton agents suspend --inactive-for 1h
   ```

3. If slow agents:
   ```bash
   caxton agents restart --slow-threshold 10s
   ```

### Recovery
1. Monitor latency metrics return to normal
2. Verify no message loss
3. Review agent performance

### Post-Incident
1. Analyze agent behavior patterns
2. Consider scaling cluster
3. Update resource limits
```

### Maintenance Procedures

#### Regular Maintenance Tasks
```yaml
maintenance:
  daily:
    - rotate_logs
    - cleanup_temp_files
    - verify_backups

  weekly:
    - compact_database
    - analyze_performance_trends
    - review_error_logs

  monthly:
    - certificate_rotation_check
    - security_audit
    - capacity_planning_review

  quarterly:
    - disaster_recovery_drill
    - upgrade_planning
    - architecture_review
```

#### Maintenance Mode
```rust
pub struct MaintenanceMode {
    duration: Duration,
    allowed_operations: Vec<Operation>,
}

impl MaintenanceMode {
    pub async fn enter(&self) -> Result<()> {
        // Notify cluster of maintenance
        self.broadcast_maintenance_start().await?;

        // Drain new messages
        self.stop_accepting_messages().await?;

        // Wait for in-flight to complete
        self.wait_for_message_drain().await?;

        // Suspend non-critical agents
        self.suspend_agents(AgentPriority::Low).await?;

        Ok(())
    }

    pub async fn exit(&self) -> Result<()> {
        // Resume agents
        self.resume_agents().await?;

        // Start accepting messages
        self.start_accepting_messages().await?;

        // Notify cluster maintenance complete
        self.broadcast_maintenance_end().await?;

        // Verify normal operations
        self.run_health_checks().await?;

        Ok(())
    }
}
```

### Debugging Procedures

#### Debug Data Collection
```bash
#!/bin/bash
# Collect debug bundle

caxton debug bundle \
  --include-logs \
  --include-metrics \
  --include-config \
  --include-traces \
  --duration 1h \
  --output /tmp/caxton-debug.tar.gz
```

#### Live Debugging
```rust
pub struct DebugInterface {
    pub async fn trace_message(&self, message_id: MessageId) -> MessageTrace {
        // Real-time message tracing
    }

    pub async fn profile_agent(&self, agent_id: AgentId) -> AgentProfile {
        // CPU and memory profiling
    }

    pub async fn dump_state(&self) -> SystemState {
        // Complete state dump
    }
}
```

## Consequences

### Positive
- **Clear procedures**: Operations team has step-by-step guides
- **Automation-ready**: Procedures can be scripted
- **Reduced MTTR**: Quick incident response
- **Safe operations**: Risk mitigation built-in

### Negative
- **Documentation overhead**: Procedures must be maintained
- **Training required**: Teams need procedure familiarity
- **Complexity**: Many procedures to master

### Neutral
- Standard DevOps practices
- Similar to other distributed systems
- Can be adapted to organization needs

## References
- [Site Reliability Engineering](https://sre.google/sre-book/table-of-contents/)
- [The Checklist Manifesto](https://atulgawande.com/book/the-checklist-manifesto/)
- [ADR-0014: Coordination-First Architecture](0014-coordination-first-architecture.md)
- [ADR-0016: Security Architecture](0016-security-architecture.md)
- [ADR-0017: Performance Requirements](0017-performance-requirements.md)
