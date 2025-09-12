---
title: "Installation Guide"
date: 2025-09-10
layout: page
audience: operators
navigation_order: 1
categories: [Operators, Installation]
---

## Production-ready installation with embedded memory system

Caxton is a standalone server designed for production deployment with zero
external dependencies. The embedded memory system uses SQLite + local
embeddings, eliminating the need for external databases or vector stores
while providing enterprise-grade reliability and performance.

## System Requirements

### Minimum Requirements

- **Operating System**: Linux, macOS, or Windows
- **Memory**: 2GB RAM (for development and testing)
- **Disk Space**: 500MB for Caxton binary and embedded memory system
- **Network**: Ports 8080 (API) and 9090 (metrics) available
- **Dependencies**: None - Caxton includes everything needed

### Production Requirements

- **Memory**: 8GB+ RAM (supports 100K+ entities efficiently)
- **CPU**: 4+ cores (for concurrent agent processing)
- **Disk Space**: 10GB+ (with log rotation and memory growth)
- **Network**: Load balancer compatibility, TLS termination
- **Monitoring**: Prometheus endpoint available on port 9090

## Quick Install

### Linux/macOS (Recommended)

```bash
# Install latest stable release
curl -sSL https://caxton.io/install.sh | sh
```

This script will:

1. Detect your OS and architecture
2. Download the appropriate Caxton binary (includes embedded memory system)
3. Install to `/usr/local/bin/caxton`
4. Set up systemd service (Linux) or launchd (macOS)
5. Create default configuration with embedded memory backend
6. Configure log rotation and monitoring

### Manual Installation

#### Download Binary

```bash
# Linux x86_64
curl -L https://github.com/caxton/caxton/releases/latest/download/caxton-linux-x86_64.tar.gz | tar xz

# Linux ARM64 (for ARM servers)
curl -L https://github.com/caxton/caxton/releases/latest/download/caxton-linux-arm64.tar.gz | tar xz

# macOS x86_64 (Intel)
curl -L https://github.com/caxton/caxton/releases/latest/download/caxton-darwin-x86_64.tar.gz | tar xz

# macOS ARM64 (Apple Silicon)
curl -L https://github.com/caxton/caxton/releases/latest/download/caxton-darwin-arm64.tar.gz | tar xz

# Windows x86_64
curl -L https://github.com/caxton/caxton/releases/latest/download/caxton-windows-x86_64.zip -o caxton.zip
unzip caxton.zip
```

#### Install Binary

```bash
# Install to system PATH
sudo mv caxton /usr/local/bin/caxton
sudo chmod +x /usr/local/bin/caxton

# Verify installation
caxton --version
```

Expected output:

```text
Caxton 1.0.0 (embedded memory: SQLite + all-MiniLM-L6-v2)
```

### Package Managers

#### Homebrew (macOS/Linux)

```bash
# Add Caxton tap
brew tap caxton/caxton

# Install Caxton with embedded memory
brew install caxton

# Enable service auto-start
brew services start caxton
```

#### APT (Ubuntu/Debian)

```bash
# Add Caxton repository
curl -fsSL https://pkg.caxton.io/gpg | sudo gpg --dearmor -o /usr/share/keyrings/caxton.gpg
echo "deb [signed-by=/usr/share/keyrings/caxton.gpg] https://pkg.caxton.io/apt stable main" | sudo tee /etc/apt/sources.list.d/caxton.list

# Install Caxton
sudo apt update
sudo apt install caxton

# Enable systemd service
sudo systemctl enable caxton
sudo systemctl start caxton
```

#### YUM/DNF (RHEL/CentOS/Fedora)

```bash
# Add Caxton repository
sudo tee /etc/yum.repos.d/caxton.repo << EOF
[caxton]
name=Caxton Repository
baseurl=https://pkg.caxton.io/rpm/stable
enabled=1
gpgcheck=1
gpgkey=https://pkg.caxton.io/gpg
EOF

# Install Caxton
sudo yum install caxton
# or
sudo dnf install caxton

# Enable systemd service
sudo systemctl enable caxton
sudo systemctl start caxton
```

#### Arch Linux (AUR)

```bash
# Using yay
yay -S caxton

# Using paru
paru -S caxton

# Manual installation
git clone https://aur.archlinux.org/caxton.git
cd caxton
makepkg -si

# Enable systemd service
sudo systemctl enable caxton
```

## Container Installation

### Docker

```bash
# Run Caxton in Docker (embedded memory persisted)
docker run -d \
  --name caxton \
  -p 8080:8080 \
  -p 9090:9090 \
  -v caxton-data:/data \
  -v caxton-logs:/var/log/caxton \
  --restart unless-stopped \
  caxton/caxton:latest

# Verify container is running
docker ps | grep caxton
```

### Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'
services:
  caxton:
    image: caxton/caxton:latest
    ports:
      - "8080:8080"
      - "9090:9090"
    volumes:
      - caxton-data:/data
      - caxton-logs:/var/log/caxton
      - ./config.yaml:/etc/caxton/config.yaml:ro
    environment:
      - CAXTON_CONFIG=/etc/caxton/config.yaml
      - CAXTON_LOG_LEVEL=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "caxton", "health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    deploy:
      resources:
        limits:
          memory: 4G
          cpus: '2.0'
        reservations:
          memory: 1G
          cpus: '0.5'

volumes:
  caxton-data:
    driver: local
  caxton-logs:
    driver: local
```

Start with:

```bash
docker-compose up -d
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: caxton
  labels:
    app: caxton
spec:
  replicas: 3
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
          name: http
        - containerPort: 9090
          name: metrics
        volumeMounts:
        - name: caxton-data
          mountPath: /data
        - name: config
          mountPath: /etc/caxton/config.yaml
          subPath: config.yaml
        - name: logs
          mountPath: /var/log/caxton
        env:
        - name: CAXTON_CONFIG
          value: /etc/caxton/config.yaml
        - name: CAXTON_LOG_LEVEL
          value: info
        resources:
          limits:
            memory: "4Gi"
            cpu: "2000m"
          requests:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /api/v1/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 30
          timeoutSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/v1/health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
          timeoutSeconds: 5
        securityContext:
          runAsNonRoot: true
          runAsUser: 1000
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
      volumes:
      - name: caxton-data
        persistentVolumeClaim:
          claimName: caxton-pvc
      - name: config
        configMap:
          name: caxton-config
      - name: logs
        emptyDir: {}
      securityContext:
        fsGroup: 1000
---
apiVersion: v1
kind: Service
metadata:
  name: caxton-service
spec:
  selector:
    app: caxton
  ports:
  - name: http
    port: 80
    targetPort: 8080
  - name: metrics
    port: 9090
    targetPort: 9090
  type: LoadBalancer
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: caxton-pvc
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 50Gi
  storageClassName: fast-ssd
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: caxton-config
data:
  config.yaml: |
    server:
      host: 0.0.0.0
      port: 8080
      metrics_port: 9090
    memory:
      backend: embedded
      embedded:
        database_path: "/data/caxton.db"
        max_entities: 100000
    observability:
      logging:
        level: info
        format: json
```

## Building from Source

For development or customization:

```bash
# Prerequisites
- Rust 1.70+
- Git

# Clone repository
git clone https://github.com/caxton/caxton.git
cd caxton

# Build with embedded memory system
cargo build --release --features embedded-memory

# Install locally
cargo install --path . --features embedded-memory

# Verify installation
caxton --version
```

## Production Setup

### 1. Create Production Directories

```bash
# System-wide production installation
sudo mkdir -p /etc/caxton
sudo mkdir -p /var/lib/caxton
sudo mkdir -p /var/log/caxton

# Set proper ownership
sudo useradd --system --home-dir /var/lib/caxton --shell /bin/false caxton
sudo chown -R caxton:caxton /var/lib/caxton /var/log/caxton
sudo chmod 755 /etc/caxton
```

### 2. Generate Production Configuration

```bash
# Generate production configuration
sudo caxton config init --config /etc/caxton/config.yaml --profile production

# Secure configuration file
sudo chown root:caxton /etc/caxton/config.yaml
sudo chmod 640 /etc/caxton/config.yaml
```

Production configuration includes:

```yaml
server:
  host: 0.0.0.0
  port: 8080
  metrics_port: 9090
  dashboard_enabled: false  # Disable in production

memory:
  backend: embedded
  embedded:
    database_path: "/var/lib/caxton/memory.db"
    embedding_model: "all-MiniLM-L6-v2"
    max_entities: 500000  # Higher production limit
    cleanup_interval: 1h

observability:
  logging:
    level: info
    format: json
    file: "/var/log/caxton/caxton.log"
    rotation:
      max_size: "100MB"
      max_files: 10
  metrics:
    enabled: true
  tracing:
    enabled: true
    jaeger_endpoint: "http://jaeger:14268/api/traces"

security:
  tls:
    enabled: true
    cert_path: "/etc/caxton/tls/cert.pem"
    key_path: "/etc/caxton/tls/key.pem"
  rate_limiting:
    enabled: true
    requests_per_minute: 100
  cors:
    enabled: true
    allowed_origins: ["https://your-domain.com"]
```

### 3. Start Caxton Server

```bash
# Start server (foreground for testing)
sudo -u caxton caxton server start --config /etc/caxton/config.yaml

# Start server (background with systemd)
sudo systemctl start caxton
sudo systemctl enable caxton

# Verify server is running
curl http://localhost:8080/api/v1/health
```

Expected response:

```json
{
  "status": "healthy",
  "version": "1.0.0",
  "memory_backend": "embedded",
  "memory_entities": 0,
  "uptime": "0s"
}
```

### 4. Configure Monitoring

```bash
# Add Prometheus scraping target
echo '
  - job_name: caxton
    static_configs:
      - targets: ["localhost:9090"]
' >> /etc/prometheus/prometheus.yml

# Restart Prometheus
sudo systemctl restart prometheus
```

### 5. Set Up Log Rotation

```bash
# Configure logrotate
sudo tee /etc/logrotate.d/caxton << EOF
/var/log/caxton/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 0644 caxton caxton
    postrotate
        /bin/systemctl reload caxton
    endscript
}
EOF
```

## System Service Setup

### Linux (systemd)

Create `/etc/systemd/system/caxton.service`:

```ini
[Unit]
Description=Caxton Multi-Agent System
Documentation=https://caxton.io/docs
After=network.target
Wants=network.target

[Service]
Type=simple
User=caxton
Group=caxton
ExecStart=/usr/local/bin/caxton server start --config /etc/caxton/config.yaml
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=5s
TimeoutStopSec=20s
TimeoutStartSec=60s

# Security settings
NoNewPrivileges=true
PrivateTmp=true
PrivateDevices=true
ProtectHome=true
ProtectSystem=strict
ReadWritePaths=/var/lib/caxton /var/log/caxton
CapabilityBoundingSet=CAP_NET_BIND_SERVICE
AmbientCapabilities=CAP_NET_BIND_SERVICE

# Resource limits
MemoryLimit=8G
CPUQuota=400%
TasksMax=1000

# Environment
Environment=CAXTON_LOG_LEVEL=info
Environment=CAXTON_METRICS_ENABLED=true

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable caxton
sudo systemctl start caxton

# Verify status
sudo systemctl status caxton
```

### macOS (launchd)

Create `/Library/LaunchDaemons/io.caxton.server.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>io.caxton.server</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/caxton</string>
        <string>server</string>
        <string>start</string>
        <string>--config</string>
        <string>/usr/local/etc/caxton/config.yaml</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/usr/local/var/log/caxton/stdout.log</string>
    <key>StandardErrorPath</key>
    <string>/usr/local/var/log/caxton/stderr.log</string>
    <key>WorkingDirectory</key>
    <string>/usr/local/var/lib/caxton</string>
    <key>UserName</key>
    <string>_caxton</string>
    <key>GroupName</key>
    <string>_caxton</string>
    <key>ThrottleInterval</key>
    <integer>10</integer>
</dict>
</plist>
```

Load and start:

```bash
sudo launchctl load /Library/LaunchDaemons/io.caxton.server.plist
sudo launchctl start io.caxton.server
```

## Embedded Memory System Configuration

### Default Configuration

No external dependencies required:

```yaml
memory:
  backend: embedded
  embedded:
    database_path: "/var/lib/caxton/caxton.db"  # SQLite database location
    embedding_model: "all-MiniLM-L6-v2"        # Local embedding model (~23MB)
    max_entities: 500000                        # Production scaling limit
    cleanup_interval: 1h                       # Automatic cleanup
    semantic_threshold: 0.6                    # Similarity threshold
    backup_interval: 24h                       # Database backup frequency
```

### Performance Characteristics

**Startup time**: ~10 seconds (loads embedding model)
**Memory usage**: ~400MB baseline (production)
**Entity storage**: ~2.5KB per entity (including embeddings)
**Query performance**:

- 10-50ms for semantic search up to 100K entities
- 50-200ms for semantic search up to 500K entities
**Throughput**: 1000+ entities per second insertion rate

### Backup and Recovery

```bash
# Manual backup
sudo -u caxton caxton memory backup --output /backup/caxton-$(date +%Y%m%d).db

# Automated backup script
sudo tee /usr/local/bin/caxton-backup.sh << 'EOF'
#!/bin/bash
BACKUP_DIR="/backup/caxton"
DATE=$(date +%Y%m%d_%H%M%S)
mkdir -p "$BACKUP_DIR"
sudo -u caxton caxton memory backup --output "$BACKUP_DIR/caxton-$DATE.db"
find "$BACKUP_DIR" -name "caxton-*.db" -mtime +30 -delete
EOF
sudo chmod +x /usr/local/bin/caxton-backup.sh

# Add to crontab
echo "0 2 * * * /usr/local/bin/caxton-backup.sh" | sudo crontab -u root -
```

## Optional External Memory Backends

For enterprise deployments requiring shared memory across clusters:

### Neo4j Backend

```yaml
memory:
  backend: neo4j
  neo4j:
    uri: "bolt://neo4j-cluster.internal:7687"
    username: "caxton"
    password: "${NEO4J_PASSWORD}"
    database: "caxton"
    pool_size: 10
    timeout: 30s
```

### Qdrant Backend

```yaml
memory:
  backend: qdrant
  qdrant:
    host: "qdrant-cluster.internal"
    port: 6333
    collection_name: "caxton_memory"
    vector_size: 384
    api_key: "${QDRANT_API_KEY}"
```

## Verification and Testing

### Health Check

```bash
# Basic health check
caxton health

# Detailed system status
caxton status

# Memory system health
caxton memory status

# Server connectivity
curl http://localhost:8080/api/v1/health
```

### Test Configuration Agent

Create a test agent to verify everything is working:

```bash
# Create simple test agent
cat > /tmp/test-agent.toml << EOF
name = "TestAgent"
version = "1.0.0"
capabilities = ["testing"]

system_prompt = '''
You are a test agent. Respond with "Hello, Caxton!" to any message.
'''

documentation = '''
## Test Agent
Simple agent for testing Caxton installation.
'''
EOF

# Deploy test agent
caxton agent deploy /tmp/test-agent.toml

# Send test message
caxton message send \
  --capability "testing" \
  --performative request \
  --content '{"message": "test"}'

# Clean up
caxton agent remove TestAgent
rm /tmp/test-agent.toml
```

### Performance Benchmarks

```bash
# Test embedded memory system performance
caxton benchmark memory --entities 10000

# Test agent deployment performance
caxton benchmark agents --concurrent 10

# Test messaging performance
caxton benchmark messaging --messages 1000 --concurrent 10
```

## Troubleshooting Installation

### Common Issues

#### Port Already in Use

```bash
# Check what's using port 8080
sudo lsof -i :8080

# Use different port
caxton server start --config config.yaml --override server.port=8081
```

#### Permission Denied

```bash
# Check file permissions
ls -la /etc/caxton/config.yaml

# Fix ownership
sudo chown -R caxton:caxton /etc/caxton /var/lib/caxton /var/log/caxton
```

#### Memory System Not Starting

```bash
# Check embedded memory system
caxton memory test

# Verify database permissions
ls -la /var/lib/caxton/

# Check disk space
df -h /var/lib/caxton
```

#### Binary Not Found

```bash
# Check installation
which caxton

# Add to PATH if needed
export PATH="/usr/local/bin:$PATH"
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
```

### Diagnostic Commands

```bash
# Comprehensive system check
caxton doctor

# View system information
caxton info

# Check configuration validity
caxton config validate

# View effective configuration
caxton config show

# Test network connectivity
caxton network test
```

### Log Analysis

```bash
# View recent logs
caxton logs --tail 100

# Follow logs in real-time
caxton logs --follow

# Filter by component
caxton logs --component memory --level debug

# Export logs for support
caxton logs --export logs.json --since 1h
```

## Production Scaling Considerations

### Resource Requirements

**Small deployment** (< 10 agents, < 10K entities):

- RAM: 2GB
- CPU: 2 cores
- Storage: 10GB
- Use embedded memory backend

**Medium deployment** (< 100 agents, < 100K entities):

- RAM: 8GB
- CPU: 4 cores
- Storage: 50GB
- Embedded memory recommended

**Large deployment** (> 100 agents, > 100K entities):

- RAM: 16GB+
- CPU: 8 cores+
- Storage: 100GB+
- Consider external memory backends for clustering

### High Availability Setup

```yaml
# Load balancer configuration (nginx example)
upstream caxton {
    server caxton-1:8080 max_fails=3 fail_timeout=30s;
    server caxton-2:8080 max_fails=3 fail_timeout=30s;
    server caxton-3:8080 max_fails=3 fail_timeout=30s;
}

# Shared memory backend
memory:
  backend: qdrant
  qdrant:
    host: "qdrant-cluster.internal"
    port: 6333

# Cluster coordination
cluster:
  enabled: true
  seeds: ["caxton-1:7946", "caxton-2:7946", "caxton-3:7946"]
```

### Monitoring and Alerting

```bash
# Prometheus metrics endpoint
curl http://localhost:9090/metrics

# Key metrics to monitor
# - caxton_agents_total
# - caxton_memory_entities_total
# - caxton_response_time_seconds
# - caxton_errors_total

# Grafana dashboard setup
grafana-cli plugins install caxton-dashboard

# Alert rules example
caxton alert rule create --name "high-memory-usage" --threshold 90%
caxton alert rule create --name "slow-responses" --threshold 5s
caxton alert rule create --name "agent-failures" --threshold 5%
```

## Security Hardening

### TLS Configuration

```yaml
server:
  tls:
    enabled: true
    cert_path: "/etc/caxton/tls/cert.pem"
    key_path: "/etc/caxton/tls/key.pem"
    ca_path: "/etc/caxton/tls/ca.pem"
    min_version: "1.2"
    cipher_suites: ["TLS_AES_256_GCM_SHA384", "TLS_CHACHA20_POLY1305_SHA256"]
```

### Access Control

```yaml
security:
  authentication:
    enabled: true
    jwt_secret: "${JWT_SECRET}"
  authorization:
    enabled: true
    roles_file: "/etc/caxton/roles.yaml"
  rate_limiting:
    enabled: true
    requests_per_minute: 100
    burst: 20
```

### Firewall Configuration

```bash
# UFW (Ubuntu)
sudo ufw allow 8080/tcp
sudo ufw allow 9090/tcp
sudo ufw deny 7946/tcp  # Block cluster port from external access

# iptables
sudo iptables -A INPUT -p tcp --dport 8080 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 9090 -j ACCEPT
```

The zero-dependency embedded memory system makes Caxton incredibly easy to
install and operate in production, while supporting scaling to external
backends as your multi-agent system grows.

## Next Steps

- **[Configuration Guide](configuration.md)** - Complete server and agent
  configuration reference
- **[Operational Runbook](../../operations/operational-runbook.md)** -
  Day-to-day operations procedures
- **[Performance Tuning](../../operations/performance-tuning.md)** -
  Optimization guidelines
- **[Security Guide](../../operations/devops-security-guide.md)** - Security
  best practices
