---
title: "For Operators: Deploying and Maintaining Caxton"
date: 2025-01-15
layout: page
categories: [Audiences, Operators]
audience: operators
description: "Deploy, monitor, and maintain Caxton in production with comprehensive operational procedures, scaling strategies, and troubleshooting guides."
---

## Welcome, Operator

You're responsible for **deploying and maintaining Caxton in production**. This
path provides comprehensive operational procedures, monitoring strategies,
scaling guidance, and troubleshooting resources for production-grade Caxton
deployments.

## What You'll Master

- ✅ Zero-dependency deployment with embedded memory system
- ✅ Production configuration and security hardening
- ✅ Monitoring, alerting, and observability best practices
- ✅ Backup and disaster recovery procedures
- ✅ Performance tuning and scaling strategies
- ✅ Troubleshooting and incident response
- ✅ Upgrade procedures and maintenance schedules

## Operational Journey

### 🚀 Production Deployment (2 hours)

Get Caxton running in production with proper configuration and monitoring.

1. **[Installation Guide](../../getting-started/installation.md)** (15 min)
   - Production installation options
   - System requirements and dependencies
   - Security considerations

2. **[Configuration Guide](../../getting-started/configuration.md)** (45 min)
   - Production configuration patterns
   - Environment-specific settings
   - Security hardening checklist

3. **[Operational Runbook](../../operations/operational-runbook.md)** (60 min)
   - Complete production setup procedures
   - Initial bootstrap and health checks
   - Common operational tasks

### 📊 Monitoring and Observability (1.5 hours)

Set up comprehensive monitoring for production Caxton deployments.

1. **[DevOps Security Guide](../../operations/devops-security-guide.md)** (30 min)
   - Security monitoring and alerting
   - Access control and audit logging
   - Compliance and governance

2. **[Performance Tuning](../../operations/performance-tuning.md)** (45 min)
   - Memory system optimization
   - Agent performance monitoring
   - Resource utilization tuning

3. **[Metrics Integration](../../docs/monitoring/metrics-integration-guide.md)**
   (15 min)
   - Prometheus metrics setup
   - Grafana dashboard configuration
   - Key performance indicators

### 🔧 Maintenance and Troubleshooting (2 hours)

Master ongoing maintenance, upgrades, and issue resolution.

1. **[Agent Lifecycle Management](../../operations/agent-lifecycle-management.md)**
   (45 min)
   - Blue-green deployments
   - Canary releases and rollbacks
   - Hot-reload strategies for config agents

2. **[State Recovery Patterns](../../operations/state-recovery-patterns.md)**
   (45 min)
   - Data backup and restoration
   - Embedded database recovery
   - Agent state consistency

3. **[Error Handling Guide](../../operations/error-handling-guide.md)** (30 min)
   - Common error patterns and resolutions
   - Escalation procedures
   - Automated recovery strategies

## Deployment Architectures

### Single Node (Zero Dependencies)

**Best for**: Development, small teams, proof-of-concept deployments

```yaml
# caxton.yaml - Minimal production configuration
server:
  port: 8080
  host: "0.0.0.0"
  data_dir: "/var/lib/caxton"
  agents_dir: "/var/lib/caxton/agents"

memory:
  backend: "embedded"  # SQLite + Candle
  sqlite_path: "memory.db"
  embedding_model: "all-MiniLM-L6-v2"
  max_entities: 100000  # Recommended limit

logging:
  level: "info"
  format: "json"
  file: "/var/log/caxton/caxton.log"

observability:
  metrics_enabled: true
  metrics_port: 9090
  tracing_enabled: true
```

**Deployment**:

```bash
# Systemd service setup
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
Environment=RUST_LOG=info

# Resource limits
LimitNOFILE=65536
LimitCORE=0

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=/var/lib/caxton /var/log/caxton

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable caxton
sudo systemctl start caxton
```

### Docker Deployment

**Best for**: Containerized environments, easy deployment

```yaml
# docker-compose.yml
version: '3.8'

services:
  caxton:
    image: caxton/caxton:latest
    container_name: caxton-server
    ports:
      - "8080:8080"    # API
      - "9090:9090"    # Metrics
    environment:
      - CAXTON_CONFIG_PATH=/etc/caxton/config.yaml
      - RUST_LOG=info
    volumes:
      - ./config/caxton.yaml:/etc/caxton/config.yaml:ro
      - ./agents:/var/lib/caxton/agents:ro
      - caxton-data:/var/lib/caxton
    healthcheck:
      test: ["CMD", "caxton", "health"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped
    logging:
      driver: "json-file"
      options:
        max-size: "100m"
        max-file: "3"

  # Optional: Monitoring stack
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9091:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
    profiles: ["monitoring"]

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-data:/var/lib/grafana
    profiles: ["monitoring"]

volumes:
  caxton-data:
  grafana-data:
```

### Kubernetes Deployment

**Best for**: Large-scale deployments, high availability

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: caxton
---
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: caxton-config
  namespace: caxton
data:
  caxton.yaml: |
    server:
      port: 8080
      host: "0.0.0.0"
      data_dir: "/var/lib/caxton"
    memory:
      backend: "embedded"
    observability:
      metrics_enabled: true
      metrics_port: 9090
---
# k8s/statefulset.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: caxton
  namespace: caxton
spec:
  serviceName: caxton
  replicas: 1  # Embedded backend: single instance only
  selector:
    matchLabels:
      app: caxton
  template:
    metadata:
      labels:
        app: caxton
    spec:
      containers:
      - name: caxton
        image: caxton/caxton:latest
        ports:
        - containerPort: 8080
          name: api
        - containerPort: 9090
          name: metrics
        env:
        - name: CAXTON_CONFIG_PATH
          value: /etc/caxton/config.yaml
        volumeMounts:
        - name: config
          mountPath: /etc/caxton
        - name: data
          mountPath: /var/lib/caxton
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
      volumes:
      - name: config
        configMap:
          name: caxton-config
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 10Gi
```

### High Availability with External Backends

**Best for**: Enterprise deployments requiring scale beyond embedded limits

```yaml
# Enterprise deployment with external backends
version: '3.8'

services:
  caxton-1:
    image: caxton/caxton:latest
    environment:
      - CAXTON_MEMORY_BACKEND=qdrant
      - QDRANT_URL=http://qdrant:6333
    depends_on:
      - qdrant
      - prometheus

  caxton-2:
    image: caxton/caxton:latest
    environment:
      - CAXTON_MEMORY_BACKEND=qdrant
      - QDRANT_URL=http://qdrant:6333
    depends_on:
      - qdrant
      - prometheus

  qdrant:
    image: qdrant/qdrant:latest
    ports:
      - "6333:6333"
    volumes:
      - qdrant-data:/qdrant/storage

  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
    depends_on:
      - caxton-1
      - caxton-2

volumes:
  qdrant-data:
```

## Monitoring and Observability

### Key Metrics to Monitor

#### System Health

```bash
# Core system metrics
curl -s localhost:9090/metrics | grep caxton_server_
# caxton_server_uptime_seconds
# caxton_server_memory_usage_bytes
# caxton_server_restarts_total
```

#### Memory System Performance

```bash
# Embedded memory metrics
curl -s localhost:9090/metrics | grep caxton_memory_
# caxton_memory_entities_total
# caxton_memory_search_latency_p99
# caxton_memory_sqlite_size_bytes
# caxton_memory_cache_hit_rate
```

#### Agent Performance

```bash
# Agent runtime metrics
curl -s localhost:9090/metrics | grep caxton_agent_
# caxton_agent_config_count
# caxton_agent_wasm_count
# caxton_agent_response_latency_p99
# caxton_agent_reload_success_total
```

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "caxton_rules.yml"

scrape_configs:
  - job_name: 'caxton'
    static_configs:
      - targets: ['caxton:9090']
    scrape_interval: 5s
    metrics_path: /metrics

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

### Alerting Rules

```yaml
# caxton_rules.yml
groups:
- name: caxton.rules
  rules:
  - alert: CaxtonDown
    expr: up{job="caxton"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Caxton server is down"
      description: "Caxton has been down for more than 1 minute"

  - alert: MemorySystemSlow
    expr: caxton_memory_search_latency_p99 > 0.1
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "Memory system performance degraded"
      description: "P99 search latency is {{ $value }}s"

  - alert: ApproachingMemoryLimit
    expr: (caxton_memory_entities_total / 100000) > 0.9
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Approaching embedded memory limit"
      description: "{{ $value }}% of recommended entity limit reached"

  - alert: ConfigAgentReloadFailure
    expr: increase(caxton_agent_reload_failed_total[5m]) > 0
    for: 1m
    labels:
      severity: warning
    annotations:
      summary: "Config agent reload failures"
      description: "{{ $value }} agent reloads failed in the last 5 minutes"
```

### Grafana Dashboard

Key panels to include:

1. **System Overview**:
   - Server uptime
   - Memory usage
   - CPU utilization
   - Restart count

2. **Agent Performance**:
   - Active agent count (config vs WASM)
   - Message throughput
   - Response latency distribution
   - Error rate

3. **Memory System**:
   - Entity and relation counts
   - Search latency trends
   - Cache hit rates
   - Database size growth

4. **Operational Health**:
   - Deployment success rates
   - Hot-reload performance
   - Error patterns and trends

## Backup and Recovery

### Embedded Data Backup

```bash
#!/bin/bash
# backup-caxton.sh - Complete backup script

BACKUP_DIR="/backups/caxton"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
BACKUP_FILE="$BACKUP_DIR/caxton-backup-$TIMESTAMP.tar.gz"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Stop Caxton gracefully
systemctl stop caxton

# Create comprehensive backup
tar -czf "$BACKUP_FILE" \
    /var/lib/caxton/data/ \
    /var/lib/caxton/agents/ \
    /etc/caxton/ \
    /var/log/caxton/

# Verify backup integrity
if tar -tzf "$BACKUP_FILE" > /dev/null 2>&1; then
    echo "Backup created successfully: $BACKUP_FILE"
else
    echo "Backup verification failed!" >&2
    exit 1
fi

# Restart Caxton
systemctl start caxton

# Cleanup old backups (keep 30 days)
find "$BACKUP_DIR" -name "caxton-backup-*.tar.gz" -mtime +30 -delete

# Test backup (optional)
if [ "$1" = "--test" ]; then
    echo "Testing backup restore..."
    TEMP_DIR=$(mktemp -d)
    tar -xzf "$BACKUP_FILE" -C "$TEMP_DIR"
    echo "Backup test completed successfully"
    rm -rf "$TEMP_DIR"
fi
```

### Automated Backup with Cron

```bash
# Add to /etc/crontab
# Backup daily at 2 AM
0 2 * * * root /usr/local/bin/backup-caxton.sh

# Backup hourly during business hours (production)
0 9-17 * * 1-5 root /usr/local/bin/backup-caxton.sh --incremental
```

### Recovery Procedures

#### Complete System Recovery

```bash
#!/bin/bash
# restore-caxton.sh - Complete system restore

BACKUP_FILE="$1"
if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup-file.tar.gz>"
    exit 1
fi

echo "WARNING: This will overwrite current Caxton installation!"
read -p "Continue? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
fi

# Stop Caxton
systemctl stop caxton

# Backup current state (just in case)
tar -czf "/tmp/caxton-pre-restore-$(date +%s).tar.gz" \
    /var/lib/caxton/ /etc/caxton/ 2>/dev/null || true

# Clear existing data
rm -rf /var/lib/caxton/data/*
rm -rf /var/lib/caxton/agents/*

# Restore from backup
tar -xzf "$BACKUP_FILE" -C /

# Fix permissions
chown -R caxton:caxton /var/lib/caxton
chmod 755 /var/lib/caxton/data
chmod 755 /var/lib/caxton/agents

# Verify database integrity
sqlite3 /var/lib/caxton/data/memory.db "PRAGMA integrity_check;"

# Start Caxton
systemctl start caxton

# Wait for startup
sleep 10

# Verify recovery
if caxton health; then
    echo "Recovery completed successfully"
    caxton agents list
else
    echo "Recovery failed - check logs"
    journalctl -u caxton -n 50
    exit 1
fi
```

## Performance Tuning

### Memory System Optimization

#### Embedded Backend Tuning

```yaml
# caxton.yaml - Performance optimized
memory:
  backend: "embedded"
  sqlite_config:
    cache_size: 100000  # 100MB SQLite cache
    journal_mode: "WAL"  # Write-Ahead Logging
    synchronous: "NORMAL"  # Balance safety/performance
    temp_store: "MEMORY"  # Temp tables in RAM

  embedding_config:
    batch_size: 32  # Process embeddings in batches
    cache_size: 10000  # Cache 10K recent embeddings
    model_device: "cpu"  # or "cuda" if available

  cleanup:
    vacuum_interval: "24h"  # Defragment database
    analyze_interval: "12h"  # Update query statistics
    cleanup_threshold: 0.1  # Trigger cleanup at 10% fragmentation
```

#### Performance Monitoring

```bash
# Monitor memory system performance
caxton memory stats --detailed

# Output:
# SQLite Performance:
#   Cache hit rate: 94.5%
#   Average query time: 2.3ms
#   Database size: 45.2MB
#   Fragmentation: 3.1%
#
# Embedding Performance:
#   Model load time: 234ms
#   Average encoding time: 15ms
#   Cache hit rate: 87.2%
#   Memory usage: 203MB

# Optimize if performance degrades
caxton memory optimize --vacuum --reindex --analyze
```

### Agent Performance Tuning

#### Configuration Agent Optimization

```yaml
# Optimized config agent
name = "OptimizedAgent"
tools = ["http_client"]

[memory]
enabled = true
scope = "agent"  # Minimize scope

[memory.search]
similarity_threshold = 0.8  # Higher threshold
max_results = 5  # Limit results
cache_ttl = "1h"  # Cache search results

[performance]
max_concurrent_requests = 10
request_timeout = "30s"

[performance.llm_config]
max_tokens = 1000  # Limit response size
temperature = 0.1  # Reduce randomness

[tool_config.http_client]
timeout = "10s"
max_connections = 5

system_prompt = '''
Be concise and direct. Avoid verbose explanations.
Focus on actionable insights and recommendations.
'''
```

#### WASM Agent Resource Limits

```rust
// Optimal resource limits for WASM agents
let resource_limits = ResourceLimits {
    max_memory_bytes: ByteSize::mib(10),  // 10MB RAM limit
    max_cpu_millis: CpuMillis::from_secs(5),  // 5 second CPU limit
    max_execution_time: Duration::from_secs(30),  // Total timeout
    max_message_size: ByteSize::kb(100),  // 100KB message limit
};

// Performance monitoring
let performance_config = PerformanceConfig {
    enable_profiling: true,
    sample_rate: 0.1,  // Sample 10% of requests
    memory_tracking: true,
    cpu_tracking: true,
};
```

## Scaling Strategies

### Vertical Scaling (Single Node)

**Memory optimization**:

```bash
# Increase system resources
# Recommended: 4 CPU cores, 8GB RAM minimum

# Tune OS parameters
echo 'vm.swappiness=1' >> /etc/sysctl.conf
echo 'vm.dirty_ratio=15' >> /etc/sysctl.conf
echo 'vm.dirty_background_ratio=5' >> /etc/sysctl.conf

# Increase file handle limits
echo 'caxton soft nofile 65536' >> /etc/security/limits.conf
echo 'caxton hard nofile 65536' >> /etc/security/limits.conf
```

### Horizontal Scaling (External Backend)

**Migration to external memory backend**:

```bash
# 1. Export existing data
caxton memory export --format json --output memory-export.json

# 2. Setup external backend (Qdrant example)
docker run -d --name qdrant \
  -p 6333:6333 \
  -v qdrant-data:/qdrant/storage \
  qdrant/qdrant:latest

# 3. Update configuration
cat > caxton.yaml << 'EOF'
memory:
  backend: "qdrant"
  qdrant_config:
    url: "http://localhost:6333"
    collection: "caxton-memory"
    vector_size: 384  # All-MiniLM-L6-v2
EOF

# 4. Import data to new backend
caxton memory import memory-export.json

# 5. Deploy multiple Caxton instances
# Each instance connects to shared Qdrant backend
```

### Load Balancing

```nginx
# nginx.conf - Load balancer for multiple Caxton instances
upstream caxton_backend {
    server caxton-1:8080 max_fails=3 fail_timeout=30s;
    server caxton-2:8080 max_fails=3 fail_timeout=30s;
    server caxton-3:8080 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    server_name caxton.example.com;

    location /api/v1/ {
        proxy_pass http://caxton_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Health checks
        proxy_next_upstream error timeout http_500 http_502 http_503;
        proxy_connect_timeout 5s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }

    location /health {
        access_log off;
        proxy_pass http://caxton_backend;
    }
}
```

## Security Hardening

### Network Security

```yaml
# Production security configuration
security:
  tls:
    enabled: true
    cert_file: "/etc/ssl/certs/caxton.crt"
    key_file: "/etc/ssl/private/caxton.key"

  authentication:
    enabled: true
    jwt_secret_file: "/etc/caxton/jwt-secret"
    token_expiry: "24h"

  cors:
    allowed_origins: ["https://dashboard.example.com"]
    allowed_methods: ["GET", "POST", "PUT", "DELETE"]

  rate_limiting:
    requests_per_minute: 1000
    burst_size: 100

  api_keys:
    required: true
    key_file: "/etc/caxton/api-keys.yaml"
```

### Firewall Configuration

```bash
# UFW firewall rules
ufw default deny incoming
ufw default allow outgoing

# Allow SSH (adjust port as needed)
ufw allow 22/tcp

# Allow Caxton API
ufw allow 8080/tcp comment 'Caxton API'

# Allow metrics (internal only)
ufw allow from 10.0.0.0/8 to any port 9090 comment 'Prometheus metrics'

# Enable firewall
ufw --force enable
```

### Container Security

```dockerfile
# Dockerfile - Security hardened
FROM debian:bullseye-slim

# Create non-root user
RUN groupadd -r caxton && useradd -r -g caxton caxton

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary and set permissions
COPY --from=builder /usr/local/bin/caxton /usr/local/bin/caxton
RUN chmod +x /usr/local/bin/caxton

# Create data directories
RUN mkdir -p /var/lib/caxton && \
    chown caxton:caxton /var/lib/caxton

# Switch to non-root user
USER caxton

# Security hardening
ENV RUST_LOG=info
ENV RUST_BACKTRACE=0

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD caxton health || exit 1

CMD ["caxton", "start", "--config", "/etc/caxton/config.yaml"]
```

## Troubleshooting Guide

### Common Issues and Solutions

#### Server Won't Start

**Symptoms**: `systemctl start caxton` fails

**Diagnosis**:

```bash
# Check service status
systemctl status caxton

# Check logs
journalctl -u caxton -n 50

# Check configuration
caxton validate --config /etc/caxton/caxton.yaml

# Check permissions
ls -la /var/lib/caxton/
```

**Solutions**:

- Fix configuration syntax errors
- Ensure data directory permissions
- Check available disk space
- Verify SQLite database integrity

#### Memory Performance Degraded

**Symptoms**: Slow agent responses, high latency

**Diagnosis**:

```bash
# Check memory system status
caxton memory stats

# Check database fragmentation
sqlite3 /var/lib/caxton/data/memory.db "PRAGMA integrity_check;"

# Monitor system resources
top -p $(pgrep caxton)
```

**Solutions**:

```bash
# Optimize database
caxton memory optimize --vacuum --analyze

# Clear embedding cache
caxton memory cache-clear --rebuild

# Check if approaching limits
caxton memory capacity-check
```

#### Config Agents Not Loading

**Symptoms**: Agents show as failed in status

**Diagnosis**:

```bash
# Validate all agents
caxton agents validate-all

# Check specific agent
caxton agents show problematic-agent --verbose

# Check agent logs
caxton logs problematic-agent --tail 50
```

**Solutions**:

- Fix TOML syntax errors in agent configuration files
- Ensure required tools are available
- Check file permissions on agent directory
- Verify capability declarations

### Incident Response Procedures

#### Critical System Failure

1. **Immediate Response** (0-5 minutes):

   ```bash
   # Check system health
   caxton health || echo "SYSTEM DOWN"

   # Check process status
   systemctl status caxton

   # Check available resources
   df -h && free -h
   ```

2. **Emergency Stabilization** (5-15 minutes):

   ```bash
   # Enable safe mode if possible
   caxton safe-mode --enable

   # Restart with minimal configuration
   systemctl restart caxton

   # Check logs for root cause
   journalctl -u caxton --since "10 minutes ago"
   ```

3. **Recovery** (15-60 minutes):

   ```bash
   # Restore from backup if needed
   /usr/local/bin/restore-caxton.sh /backups/latest-backup.tar.gz

   # Verify system integrity
   caxton storage verify --comprehensive

   # Resume normal operations
   caxton safe-mode --disable
   ```

#### Performance Degradation

1. **Assessment**:

   ```bash
   # Check key metrics
   curl -s localhost:9090/metrics | grep -E "(latency|error_rate|memory_usage)"

   # Identify bottlenecks
   caxton memory stats
   caxton agents performance-report
   ```

2. **Mitigation**:

   ```bash
   # Quick optimizations
   caxton memory optimize
   caxton agents restart --failing-only

   # Resource management
   systemctl restart caxton
   ```

---

**Ready to deploy?** Start with the **[Operational
Runbook](../../operations/operational-runbook.md)** for comprehensive
production deployment procedures!
