---
title: "Production Deployment Guide"
layout: documentation
description: "Deploy Caxton from zero-dependency embedded setup to enterprise scale with embedded memory and configuration-driven agents"
date: 2025-09-10
categories: [Website]
---

Deploy Caxton using the zero-dependency embedded approach that scales from
single binary to enterprise infrastructure without architectural changes.

## Deployment Philosophy

Caxton follows a **grow-with-you architecture**:

1. **Start Simple**: Single binary with embedded memory (SQLite + local embeddings)
2. **Scale Gradually**: Add external backends when you exceed ~100K entities
3. **Enterprise Ready**: Multi-tenant, cloud-native deployment for large scale

## Zero-Dependency Embedded Deployment

### Minimum Requirements (Embedded)

Perfect for getting started and small-to-medium deployments:

- **CPU**: 2 cores (x86_64 or ARM64)
- **Memory**: 4 GB RAM (including ~200MB for embedding model)
- **Storage**: 10 GB SSD
- **Network**: Any internet connection
- **OS**: Linux, macOS, or Windows
- **External Dependencies**: **NONE** - works immediately

### Recommended Requirements (Embedded)

For production workloads up to 100K entities:

- **CPU**: 4+ cores with good single-thread performance
- **Memory**: 8+ GB RAM
- **Storage**: 50+ GB SSD (for agent memory growth)
- **Network**: 1 Gbps connection
- **OS**: Linux (Ubuntu 22.04+, RHEL 8+, CentOS 8+)

### Enterprise Scale Requirements

When you outgrow embedded memory (~100K entities):

- **CPU**: 16+ cores with AVX2 support
- **Memory**: 32+ GB RAM
- **Storage**: 200+ GB NVMe SSD
- **Network**: 10 Gbps connection with low latency
- **External**: Neo4j/Qdrant clusters, Kubernetes infrastructure

## Quick Start (Zero Dependencies)

### Single Binary Deployment

Get running in under 5 minutes with zero external dependencies:

```bash
# Download and run - that's it!
curl -L https://releases.caxton.dev/latest/caxton-linux -o caxton
chmod +x caxton
./caxton serve

# Caxton is now running with:
# - Embedded SQLite memory
# - Local embedding model (downloads automatically)
# - Configuration agent runtime
# - Web UI at http://localhost:8080
```

### Docker Deployment (Embedded)

#### Single Container Setup

```bash
# Pull and run - embedded memory included
docker run -d \
  --name caxton \
  -p 8080:8080 \
  -v caxton-data:/var/lib/caxton \
  caxton/caxton:latest

# Zero configuration required
# Memory and agents persist in caxton-data volume
```

### Configuration Agent Deployment

Deploy your first agent in minutes:

```bash
# 1. Create your agent file
cat > my-agent.md << 'EOF'
---
name: MyFirstAgent
capabilities: [greeting, help]
tools: [http_client]
memory_enabled: true
system_prompt: |
  You are a helpful assistant who greets users and provides basic help.
---

# My First Agent

I can greet users and provide helpful information.
EOF

# 2. Deploy agent
./caxton agent deploy my-agent.md

# 3. Test agent (via API or Web UI)
curl -X POST http://localhost:8080/api/messages \
  -H "Content-Type: application/json" \
  -d '{
    "capability": "greeting",
    "content": {"message": "Hello!"}
  }'
```

**That's it!** Your agent is running with:

- Embedded memory (no external database)
- Automatic context awareness
- Zero configuration overhead

## Scaling to External Backends

### When to Scale Beyond Embedded

**Consider external backends when you reach:**

- 100K+ entities in memory
- Need for multi-node deployments
- Advanced analytics requirements
- Enterprise compliance needs

### Neo4j Backend (Graph Memory)

```yaml
# caxton.yaml
memory:
  backend: neo4j
  neo4j:
    uri: bolt://neo4j:7687
    username: neo4j
    password: ${NEO4J_PASSWORD}
    database: caxton
```

### Qdrant Backend (Vector Memory)

```yaml
# caxton.yaml
memory:
  backend: qdrant
  qdrant:
    url: http://qdrant:6333
    collection: caxton_memory
    vector_size: 384
```

  --restart unless-stopped \
  -p 8080:8080 \
  -p 9090:9090 \
  -v /opt/caxton/data:/data \
  -e CAXTON_CONFIG_PATH=/data/config.toml \
  caxton/caxton:latest

```bash

#### Docker Compose Configuration

```yaml
# docker-compose.yml
version: '3.8'
services:
  caxton-runtime:
    image: caxton/caxton:latest
    restart: unless-stopped
    ports:
      - "8080:8080"  # HTTP API
      - "9090:9090"  # Metrics
      - "4317:4317"  # OTLP gRPC
    volumes:
      - ./config:/config:ro
      - caxton-data:/data
    environment:
      - CAXTON_CONFIG_PATH=/config/production.toml
      - CAXTON_LOG_LEVEL=info
      - CAXTON_METRICS_ENABLED=true
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  caxton-data:
```

### Kubernetes Deployment

#### Namespace and ConfigMap

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: caxton-system

---
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: caxton-config
  namespace: caxton-system
data:
  production.toml: |
    [runtime]
    max_agents = 1000
    wasm_memory_limit = "512MB"
    execution_timeout = "30s"

    [networking]
    bind_address = "0.0.0.0:8080"
    metrics_address = "0.0.0.0:9090"

    [observability]
    enable_tracing = true
    otlp_endpoint = "http://jaeger-collector:14268/api/traces"

    [coordination]
    # Local state storage
    local_state_path = "/data/local.db"

    # Cluster coordination
    cluster_enabled = true
    bind_addr = "0.0.0.0:7946"
    gossip_interval = "200ms"
```

#### Deployment Configuration

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: caxton-runtime
  namespace: caxton-system
spec:
  replicas: 3
  selector:
    matchLabels:
      app: caxton-runtime
  template:
    metadata:
      labels:
        app: caxton-runtime
    spec:
      containers:
      - name: caxton
        image: caxton/caxton:latest
        ports:
        - containerPort: 8080
        - containerPort: 9090
        env:
        - name: CAXTON_CONFIG_PATH
          value: "/config/production.toml"
        volumeMounts:
        - name: config-volume
          mountPath: /config
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config-volume
        configMap:
          name: caxton-config

---
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: caxton-service
  namespace: caxton-system
spec:
  selector:
    app: caxton-runtime
  ports:
  - name: http
    port: 8080
    targetPort: 8080
  - name: metrics
    port: 9090
    targetPort: 9090
  type: ClusterIP
```

### Bare Metal Installation

#### System Preparation

```bash
# Install dependencies
sudo apt update && sudo apt install -y \
  curl wget \
  build-essential \
  pkg-config \
  libssl-dev

# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install WebAssembly targets
rustup target add wasm32-wasi
rustup target add wasm32-unknown-unknown
```

#### Binary Installation

```bash
# Download latest release
CAXTON_VERSION="v0.2.0"
curl -L "https://github.com/caxton/caxton/releases/download/${CAXTON_VERSION}/caxton-linux-amd64.tar.gz" \
  | tar xz -C /usr/local/bin/

# Create system user
sudo useradd --system --shell /bin/false --home-dir /opt/caxton caxton

# Create directories
sudo mkdir -p /opt/caxton/{bin,config,data,logs}
sudo chown -R caxton:caxton /opt/caxton

# Create systemd service
sudo tee /etc/systemd/system/caxton.service > /dev/null << EOF
[Unit]
Description=Caxton Multi-Agent Runtime
After=network.target

[Service]
Type=exec
User=caxton
Group=caxton
ExecStart=/usr/local/bin/caxton --config /opt/caxton/config/production.toml
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=caxton

# Security settings
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/opt/caxton/data /opt/caxton/logs

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable caxton
sudo systemctl start caxton
```

## Configuration Best Practices

### Production Configuration

```toml
# /opt/caxton/config/production.toml

[runtime]
# Agent execution limits
max_agents = 1000
max_concurrent_executions = 100
wasm_memory_limit = "512MB"
wasm_stack_limit = "1MB"
execution_timeout = "30s"
agent_idle_timeout = "300s"

# Resource management
cpu_quota = "8.0"  # CPU cores
memory_limit = "16GB"
temp_storage_limit = "10GB"

[networking]
# Bind addresses
bind_address = "0.0.0.0:8080"
metrics_address = "0.0.0.0:9090"
admin_address = "127.0.0.1:8081"

# Connection limits
max_connections = 10000
connection_timeout = "30s"
request_timeout = "60s"
keepalive_timeout = "60s"

# TLS configuration
tls_enabled = true
tls_cert_path = "/opt/caxton/config/server.crt"
tls_key_path = "/opt/caxton/config/server.key"
tls_ca_path = "/opt/caxton/config/ca.crt"

[coordination]
# Local state storage (per instance)
local_state_path = "/opt/caxton/data/local.db"
journal_mode = "WAL"

# Cluster coordination
cluster_enabled = true
bind_addr = "0.0.0.0:7946"
advertise_addr = "auto"
seeds = [
  "caxton-node-1:7946",
  "caxton-node-2:7946",
  "caxton-node-3:7946"
]
gossip_interval = "200ms"
probe_interval = "1s"

[observability]
# Logging
log_level = "info"
log_format = "json"
log_file = "/opt/caxton/logs/caxton.log"
log_rotation = "daily"
log_retention_days = 30

# Metrics
enable_metrics = true
metrics_prefix = "caxton"
histogram_buckets = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]

# Tracing
enable_tracing = true
trace_sample_rate = 0.1
otlp_endpoint = "http://jaeger-collector:14268/api/traces"
otlp_timeout = "10s"

[security]
# Authentication
auth_enabled = true
jwt_secret = "${JWT_SECRET}"
jwt_expiry = "1h"
api_key_header = "X-API-Key"

# Rate limiting
rate_limit_enabled = true
rate_limit_per_second = 100
rate_limit_burst = 200

# CORS
cors_enabled = true
cors_origins = ["https://dashboard.example.com"]
cors_methods = ["GET", "POST", "PUT", "DELETE"]
```

### Environment Variables

```bash
# /opt/caxton/config/caxton.env
CAXTON_CONFIG_PATH=/opt/caxton/config/production.toml
CAXTON_LOG_LEVEL=info
JWT_SECRET=your-jwt-secret
OTLP_ENDPOINT=http://jaeger:14268/api/traces
```

## High Availability Setup

### Multi-Node Cluster

#### Load Balancer Configuration (HAProxy)

```haproxy
# /etc/haproxy/haproxy.cfg
global
    daemon
    log stdout local0 info

defaults
    log global
    option httplog
    option dontlognull
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms

frontend caxton_frontend
    bind *:80
    bind *:443 ssl crt /etc/ssl/certs/caxton.pem
    redirect scheme https if !{ ssl_fc }
    default_backend caxton_backend

backend caxton_backend
    balance roundrobin
    option httpchk GET /health
    server caxton1 10.0.1.10:8080 check
    server caxton2 10.0.1.11:8080 check
    server caxton3 10.0.1.12:8080 check

frontend caxton_metrics
    bind *:9090
    default_backend caxton_metrics_backend

backend caxton_metrics_backend
    balance roundrobin
    server caxton1 10.0.1.10:9090 check
    server caxton2 10.0.1.11:9090 check
    server caxton3 10.0.1.12:9090 check
```

#### Cluster Coordination Setup

```bash
# Each Caxton instance automatically discovers others via SWIM protocol
# No external coordination service required
# Instances share agent registry through gossip
# Message routing works without shared state
```

### Health Checks and Failover

```yaml
# kubernetes/healthcheck-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: caxton-healthcheck
  namespace: caxton-system
spec:
  selector:
    app: caxton-runtime
  ports:
  - port: 8080
    targetPort: 8080
  type: LoadBalancer
  externalTrafficPolicy: Local  # Preserve source IP
  healthCheckNodePort: 32000   # Custom health check port
```

## Load Balancing

### Nginx Configuration

```nginx
# /etc/nginx/sites-available/caxton
upstream caxton_backend {
    least_conn;
    server 10.0.1.10:8080 max_fails=3 fail_timeout=30s;
    server 10.0.1.11:8080 max_fails=3 fail_timeout=30s;
    server 10.0.1.12:8080 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    listen 443 ssl http2;
    server_name caxton.example.com;

    ssl_certificate /etc/ssl/certs/caxton.crt;
    ssl_certificate_key /etc/ssl/private/caxton.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;

    location / {
        proxy_pass http://caxton_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    location /health {
        access_log off;
        proxy_pass http://caxton_backend/health;
    }
}
```

### Session Affinity

For stateful agent sessions, configure session affinity:

```nginx
# Add to upstream block
upstream caxton_backend {
    ip_hash;  # Route based on client IP
    server 10.0.1.10:8080;
    server 10.0.1.11:8080;
    server 10.0.1.12:8080;
}

# Or use consistent hashing
upstream caxton_backend {
    hash $request_uri consistent;
    server 10.0.1.10:8080;
    server 10.0.1.11:8080;
    server 10.0.1.12:8080;
}
```

## Backup Strategies

### Data Backup

#### Local State Backup Script

```bash
#!/bin/bash
# /opt/caxton/scripts/backup-state.sh

BACKUP_DIR="/opt/caxton/backups"
DATE=$(date +%Y%m%d_%H%M%S)
STATE_PATH="/opt/caxton/data/local.db"

mkdir -p "$BACKUP_DIR"

# Create SQLite backup
sqlite3 "$STATE_PATH" ".backup '$BACKUP_DIR/state_$DATE.db'"

# Compress backup
gzip "$BACKUP_DIR/state_$DATE.db"

# Clean old backups (keep last 7 days)
find "$BACKUP_DIR" -name "state_*.db.gz" -mtime +7 -delete

echo "State backup completed: $BACKUP_DIR/state_$DATE.db.gz"
```

#### Configuration Backup

```bash
#!/bin/bash
# /opt/caxton/scripts/backup-config.sh

BACKUP_DIR="/opt/caxton/backups"
DATE=$(date +%Y%m%d_%H%M%S)
CONFIG_DIR="/opt/caxton/config"

mkdir -p "$BACKUP_DIR"

# Backup configuration files
tar -czf "$BACKUP_DIR/config_$DATE.tar.gz" -C "$(dirname "$CONFIG_DIR")" "$(basename "$CONFIG_DIR")"

# Clean old backups
find "$BACKUP_DIR" -name "config_*.tar.gz" -mtime +30 -delete

echo "Configuration backup completed: $BACKUP_DIR/config_$DATE.tar.gz"
```

### Automated Backup with Systemd

```ini
# /etc/systemd/system/caxton-backup.service
[Unit]
Description=Caxton Backup Service
After=caxton.service

[Service]
Type=oneshot
User=caxton
ExecStart=/opt/caxton/scripts/backup-state.sh
ExecStartPost=/opt/caxton/scripts/backup-config.sh

# /etc/systemd/system/caxton-backup.timer
[Unit]
Description=Run Caxton backup daily
Requires=caxton-backup.service

[Timer]
OnCalendar=daily
Persistent=true

[Install]
WantedBy=timers.target
```

### Disaster Recovery

#### Recovery Procedures

1. **Local State Recovery**:

   ```bash
   # Stop Caxton service
   sudo systemctl stop caxton

   # Restore SQLite database
   gunzip -c state_backup.db.gz > /opt/caxton/data/local.db

   # Set permissions
   sudo chown caxton:caxton /opt/caxton/data/local.db

   # Start service
   sudo systemctl start caxton
   ```

2. **Configuration Recovery**:

   ```bash
   # Extract configuration backup
   tar -xzf config_backup.tar.gz -C /opt/caxton/

   # Set permissions
   sudo chown -R caxton:caxton /opt/caxton/config

   # Restart service
   sudo systemctl restart caxton
   ```

3. **Full System Recovery**:

   ```bash
   # Deploy infrastructure
   kubectl apply -f kubernetes/

   # Wait for pods to be ready
   kubectl wait --for=condition=ready pod -l app=caxton-runtime

   # Restore data
   kubectl exec -it caxton-runtime-0 -- /scripts/restore-data.sh
   ```

## Performance Optimization

### Resource Limits

```toml
# Fine-tuned resource limits
[runtime]
max_agents = 2000
max_concurrent_executions = 200
wasm_memory_limit = "1GB"
agent_startup_timeout = "10s"
agent_shutdown_timeout = "5s"

# Memory management
gc_interval = "60s"
memory_pressure_threshold = 0.8
agent_memory_reclaim = true
```

### Monitoring and Alerts

Set up monitoring for:

- CPU and memory usage
- Agent execution metrics
- Network I/O
- Storage I/O
- Error rates
- Response times

Example Prometheus alert rules are provided in the \[Monitoring Guide\]({{
'/docs/operations/monitoring/' | relative_url }}).

## Troubleshooting

### Common Issues

1. **High Memory Usage**:

   - Check agent memory limits
   - Monitor for memory leaks
   - Adjust garbage collection settings

2. **Agent Startup Failures**:

   - Verify WASM module validity
   - Check resource limits
   - Review error logs

3. **Network Connectivity Issues**:

   - Verify firewall rules
   - Check DNS resolution
   - Test load balancer health

### Debugging Tools

```bash
# Check service status
systemctl status caxton

# View logs
journalctl -u caxton -f

# Monitor metrics
curl http://localhost:9090/metrics

# Agent debugging
caxton debug --agent-id <agent-id>
```

## Security Considerations

- Enable TLS encryption for all communications
- Use strong authentication mechanisms
- Implement proper network segmentation
- Regular security updates and patches
- Monitor for suspicious activity

For detailed security guidelines, see the \[Security Guide\]({{
'/docs/operations/security/' | relative_url }}).
