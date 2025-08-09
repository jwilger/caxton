# Clustering and Distributed Operations

This guide covers running Caxton in a distributed cluster configuration for high availability and scalability.

## Overview

Caxton uses a **coordination-first architecture** that requires no external dependencies like databases or message queues. Each Caxton instance:

- Maintains its own local state using embedded SQLite
- Coordinates with other instances via the SWIM gossip protocol
- Automatically discovers and routes messages to agents across the cluster
- Handles network partitions gracefully with degraded mode operation

For architectural details, see:
- [ADR-0014: Coordination-First Architecture](../adr/0014-coordination-first-architecture.md)
- [ADR-0015: Distributed Protocol Architecture](../adr/0015-distributed-protocol-architecture.md)

## Starting a Cluster

### Bootstrap First Node

The first node acts as the seed for cluster formation:

```bash
# Start the seed node
caxton server start \
  --node-id node-1 \
  --bind-addr 0.0.0.0:7946 \
  --api-addr 0.0.0.0:8080 \
  --bootstrap
```

### Join Additional Nodes

Other nodes join by connecting to the seed:

```bash
# On node 2
caxton server start \
  --node-id node-2 \
  --bind-addr 0.0.0.0:7946 \
  --api-addr 0.0.0.0:8080 \
  --join node-1.example.com:7946

# On node 3
caxton server start \
  --node-id node-3 \
  --bind-addr 0.0.0.0:7946 \
  --api-addr 0.0.0.0:8080 \
  --join node-1.example.com:7946,node-2.example.com:7946
```

### Verify Cluster Status

```bash
# Check cluster membership
caxton cluster members

# Example output:
NODE-ID    STATUS    ADDRESS           AGENTS    CPU    MEMORY
node-1     alive     10.0.1.10:7946    42        15%    2.1GB
node-2     alive     10.0.1.11:7946    38        12%    1.8GB
node-3     alive     10.0.1.12:7946    40        18%    2.3GB
```

## Configuration

### Cluster Configuration File

Create `/etc/caxton/cluster.yaml`:

```yaml
coordination:
  cluster:
    # SWIM protocol settings
    bind_addr: 0.0.0.0:7946
    advertise_addr: ${HOSTNAME}:7946

    # Seed nodes for joining
    seeds:
      - caxton-1.example.com:7946
      - caxton-2.example.com:7946
      - caxton-3.example.com:7946

    # Gossip parameters
    gossip_interval: 200ms
    gossip_fanout: 3
    probe_interval: 1s
    probe_timeout: 500ms

  # Partition handling
  partition:
    detection_timeout: 5s
    quorum_size: 2
    degraded_mode: true
    queue_writes: true
```

### Security Configuration

Enable mTLS for secure inter-node communication:

```yaml
security:
  cluster:
    mtls:
      enabled: true
      ca_cert: /etc/caxton/ca.crt
      node_cert: /etc/caxton/certs/node.crt
      node_key: /etc/caxton/certs/node.key
      verify_peer: true
```

See [ADR-0016: Security Architecture](../adr/0016-security-architecture.md) for details.

## Agent Distribution

Agents are automatically distributed across the cluster:

```bash
# Deploy an agent (automatically placed on optimal node)
caxton deploy agent.wasm --name my-agent

# Deploy with placement preferences
caxton deploy agent.wasm \
  --name my-agent \
  --placement-strategy least-loaded \
  --prefer-nodes node-1,node-2

# Force deployment to specific node
caxton deploy agent.wasm \
  --name my-agent \
  --target-node node-3
```

### Agent Discovery

Agents can communicate regardless of which node they're on:

```bash
# Send message to agent (routing handled automatically)
caxton message send \
  --to remote-agent \
  --content "Hello from anywhere in the cluster!"

# The cluster automatically:
# 1. Discovers which node hosts 'remote-agent'
# 2. Routes the message through the cluster
# 3. Delivers to the target agent
```

## High Availability

### Automatic Failover

When a node fails, its agents are automatically redistributed:

```bash
# Monitor failover behavior
caxton cluster watch

# Example during node failure:
[INFO] Node node-2 detected as failed
[INFO] Redistributing 38 agents from node-2
[INFO] Agent 'processor-1' migrated to node-1
[INFO] Agent 'worker-5' migrated to node-3
[INFO] All agents successfully redistributed (2.3s)
```

### Network Partition Handling

Caxton handles network partitions gracefully:

#### Majority Partition
Nodes in the majority partition continue normal operations:

```bash
# On majority side (2 of 3 nodes)
caxton cluster status
# Status: HEALTHY (majority partition)
# Operations: READ-WRITE
# Nodes: 2/3 active
```

#### Minority Partition
Nodes in the minority enter degraded mode:

```bash
# On minority side (1 of 3 nodes)
caxton cluster status
# Status: DEGRADED (minority partition)
# Operations: READ-ONLY
# Nodes: 1/3 active
# Queued writes: 42
```

When the partition heals, queued operations are replayed automatically.

## Monitoring

### Cluster Metrics

Key metrics to monitor:

```bash
# Cluster health metrics
curl http://localhost:9090/metrics | grep caxton_cluster

# Key metrics:
caxton_cluster_nodes_total          3
caxton_cluster_nodes_alive          3
caxton_cluster_agents_total         120
caxton_cluster_gossip_latency_ms    0.8
caxton_cluster_convergence_time_ms  423
```

### Performance Monitoring

Monitor cluster performance against targets:

```bash
# Check performance against requirements
caxton cluster performance

# Output:
METRIC                    TARGET      ACTUAL    STATUS
Message routing P50       100μs       87μs      ✓
Message routing P99       1ms         0.9ms     ✓
Agent startup P50         10ms        8.2ms     ✓
Gossip convergence        <5s         2.1s      ✓
```

See [ADR-0017: Performance Requirements](../adr/0017-performance-requirements.md) for targets.

## Operations

### Rolling Upgrades

Perform zero-downtime upgrades:

```bash
# Start upgrade process
caxton cluster upgrade --version v1.2.0

# The cluster will:
# 1. Select a canary node
# 2. Drain traffic from canary
# 3. Upgrade canary node
# 4. Monitor for 24 hours
# 5. Roll out to remaining nodes
```

See [ADR-0018: Operational Procedures](../adr/0018-operational-procedures.md) for details.

### Backup and Recovery

Each node maintains its own state, but cluster-wide backups are coordinated:

```bash
# Create cluster-wide backup
caxton cluster backup --dest s3://backups/caxton/

# Restore from backup
caxton cluster restore --from s3://backups/caxton/2024-01-15/
```

### Scaling

#### Adding Nodes

```bash
# Add new node to running cluster
caxton server start \
  --node-id node-4 \
  --join <any-existing-node>:7946

# Agents automatically rebalance
caxton cluster rebalance --strategy even-distribution
```

#### Removing Nodes

```bash
# Gracefully remove a node
caxton cluster leave --node node-2 --drain-timeout 60s

# Force remove failed node
caxton cluster remove --node node-2 --force
```

## Troubleshooting

### Common Issues

#### Nodes Not Joining

```bash
# Check network connectivity
caxton cluster ping node-2

# Verify gossip encryption keys match
caxton cluster verify-auth

# Check firewall rules (port 7946 must be open)
```

#### Split Brain Detection

```bash
# Check for split brain
caxton cluster detect-partition

# If split brain detected:
WARNING: Potential split brain detected
Partition 1: [node-1, node-2] (majority)
Partition 2: [node-3] (minority)
Action: Node-3 entering degraded mode
```

#### Performance Issues

```bash
# Analyze cluster performance
caxton cluster analyze

# Suggestions:
- High gossip latency: Reduce gossip_fanout
- Slow convergence: Decrease gossip_interval
- Message delays: Check network latency between nodes
```

## Best Practices

1. **Odd Number of Nodes**: Deploy 3, 5, or 7 nodes to avoid split-brain
2. **Geographic Distribution**: Spread nodes across availability zones
3. **Resource Monitoring**: Monitor CPU, memory, and network usage
4. **Regular Backups**: Schedule automated backups
5. **Security**: Always enable mTLS in production
6. **Capacity Planning**: Plan for 2x peak load for headroom

## Advanced Topics

### Multi-Region Deployment

For global deployments:

```yaml
coordination:
  cluster:
    regions:
      - name: us-east
        nodes: [node-1, node-2, node-3]
      - name: eu-west
        nodes: [node-4, node-5, node-6]

    # Cross-region settings
    cross_region:
      latency_aware_routing: true
      prefer_local_region: true
      max_cross_region_latency: 100ms
```

### Custom Partition Strategies

Implement custom partition handling:

```yaml
partition:
  strategy: custom
  custom_handler: /usr/local/bin/partition-handler
  decisions:
    - condition: "nodes < quorum"
      action: "read-only"
    - condition: "nodes == 1"
      action: "local-only"
    - condition: "critical_agents_present"
      action: "continue-critical"
```

## Performance Tuning

### SWIM Protocol Tuning

```yaml
# For small clusters (< 10 nodes)
gossip_interval: 100ms
gossip_fanout: 3

# For medium clusters (10-50 nodes)
gossip_interval: 200ms
gossip_fanout: 4

# For large clusters (> 50 nodes)
gossip_interval: 500ms
gossip_fanout: 5
```

### Network Optimization

```yaml
# Use QUIC for better performance
transport:
  type: quic
  congestion_control: bbr
  max_streams: 100
```

## Next Steps

- [Production Deployment Guide](../operations/production-deployment.md)
- [Security Best Practices](../operations/devops-security-guide.md)
- [Performance Benchmarking](../benchmarks/performance-benchmarking-guide.md)
- [Monitoring Integration](../monitoring/metrics-integration-guide.md)
