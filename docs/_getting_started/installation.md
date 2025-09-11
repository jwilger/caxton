---
title: "Installation Guide"
date: 2025-09-10
layout: page
categories: [Getting Started]
---

## Zero-dependency installation with embedded memory system

Caxton is a standalone server that runs with no external dependencies. The
embedded memory system uses SQLite + local embeddings, eliminating the need for
external databases or vector stores.

## System Requirements

- **Operating System**: Linux, macOS, or Windows
- **Memory**: Minimum 2GB RAM (4GB+ recommended for production)
- **Disk Space**: 500MB for Caxton binary and embedded memory system
- **Network**: Ports 8080 (API) and 9090 (metrics) available
- **Dependencies**: None - Caxton includes everything needed

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

### Manual Installation

#### Download Binary

```bash
# Linux x86_64
curl -L https://github.com/caxton/caxton/releases/latest/download/caxton-linux-x86_64.tar.gz | tar xz

# Linux ARM64
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
```

#### APT (Ubuntu/Debian)

```bash
# Add Caxton repository
curl -fsSL https://pkg.caxton.io/gpg | sudo gpg --dearmor -o /usr/share/keyrings/caxton.gpg
echo "deb [signed-by=/usr/share/keyrings/caxton.gpg] https://pkg.caxton.io/apt stable main" | sudo tee /etc/apt/sources.list.d/caxton.list

# Install Caxton
sudo apt update
sudo apt install caxton
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
```

### Container Installation

#### Docker

```bash
# Run Caxton in Docker (embedded memory persisted)
docker run -d \
  --name caxton \
  -p 8080:8080 \
  -p 9090:9090 \
  -v caxton-data:/data \
  caxton/caxton:latest

# Verify container is running
docker ps | grep caxton
```

#### Docker Compose

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
      - ./config.yaml:/etc/caxton/config.yaml:ro
    environment:
      - CAXTON_CONFIG=/etc/caxton/config.yaml
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "caxton", "health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  caxton-data:
    driver: local
```

Start with:

```bash
docker-compose up -d
```

#### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: caxton
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
        - containerPort: 9090
        volumeMounts:
        - name: caxton-data
          mountPath: /data
        - name: config
          mountPath: /etc/caxton/config.yaml
          subPath: config.yaml
        env:
        - name: CAXTON_CONFIG
          value: /etc/caxton/config.yaml
        resources:
          limits:
            memory: "2Gi"
            cpu: "1000m"
          requests:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /api/v1/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /api/v1/health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
      volumes:
      - name: caxton-data
        persistentVolumeClaim:
          claimName: caxton-pvc
      - name: config
        configMap:
          name: caxton-config
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
      storage: 10Gi
```

### Building from Source

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

## Initial Setup

### 1. Create Configuration Directory

```bash
# System-wide installation
sudo mkdir -p /etc/caxton
sudo mkdir -p /var/lib/caxton
sudo mkdir -p /var/log/caxton

# User installation
mkdir -p ~/.config/caxton
mkdir -p ~/.local/share/caxton
mkdir -p ~/.local/share/caxton/logs
```

### 2. Generate Default Configuration

```bash
# Generate configuration with embedded memory backend
caxton config init

# Or specify location
caxton config init --config /etc/caxton/config.yaml
```

This creates a minimal configuration:

```yaml
server:
  host: 0.0.0.0
  port: 8080
  metrics_port: 9090

memory:
  backend: embedded
  embedded:
    database_path: "/var/lib/caxton/memory.db"
    embedding_model: "all-MiniLM-L6-v2"
    max_entities: 100000

observability:
  logging:
    level: info
    format: json
    file: "/var/log/caxton/caxton.log"
  metrics:
    enabled: true
```

### 3. Start Caxton Server

```bash
# Start server (foreground)
caxton server start

# Start server (background with systemd)
sudo systemctl start caxton
sudo systemctl enable caxton

# Start server (background without systemd)
caxton server start --daemon

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

### 4. Access Dashboard

Open your browser to http://localhost:8080/dashboard

You should see:

- **Server Status**: Health and version information
- **Memory System**: Embedded SQLite backend with 0 entities
- **Agent Registry**: Empty (ready for your first agent)
- **Capability Map**: No capabilities registered yet

## Configuration Options

### Embedded Memory System (Default)

No external dependencies required:

```yaml
memory:
  backend: embedded
  embedded:
    database_path: "./caxton.db"          # SQLite database location
    embedding_model: "all-MiniLM-L6-v2"  # Local embedding model (~23MB)
    max_entities: 100000                  # Scaling limit
    cleanup_interval: 1h                  # Automatic cleanup
    semantic_threshold: 0.6               # Similarity threshold
```

**Performance characteristics**:

- **Startup time**: ~5 seconds (loads embedding model)
- **Memory usage**: ~200MB baseline
- **Entity storage**: ~2.5KB per entity (including embeddings)
- **Query performance**: 10-50ms for semantic search up to 100K entities

### Optional External Memory Backends

For larger deployments, you can configure external backends:

#### Neo4j Backend

```yaml
memory:
  backend: neo4j
  neo4j:
    uri: "bolt://localhost:7687"
    username: "neo4j"
    password: "${NEO4J_PASSWORD}"
    database: "caxton"
```

#### Qdrant Backend

```yaml
memory:
  backend: qdrant
  qdrant:
    host: "localhost"
    port: 6333
    collection_name: "caxton_memory"
    vector_size: 384
```

### Server Configuration

Customize server behavior:

```yaml
server:
  host: 0.0.0.0
  port: 8080
  metrics_port: 9090
  dashboard_enabled: true

runtime:
  max_agents: 1000
  agent_timeout: 30s
  llm_provider: "anthropic"
  llm_model: "claude-3-haiku"

observability:
  logging:
    level: info
    format: json
  metrics:
    enabled: true
```

## System Service Setup

### Linux (systemd)

Create `/etc/systemd/system/caxton.service`:

```ini
[Unit]
Description=Caxton Multi-Agent System
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

# Security settings
NoNewPrivileges=true
PrivateTmp=true
PrivateDevices=true
ProtectHome=true
ProtectSystem=strict
ReadWritePaths=/var/lib/caxton /var/log/caxton

# Resource limits
MemoryLimit=4G
CPUQuota=200%

[Install]
WantedBy=multi-user.target
```

Create service user:

```bash
sudo useradd --system --home-dir /var/lib/caxton --shell /bin/false caxton
sudo chown -R caxton:caxton /var/lib/caxton /var/log/caxton
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable caxton
sudo systemctl start caxton
```

### macOS (launchd)

Create `~/Library/LaunchAgents/com.caxton.server.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.caxton.server</string>
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
</dict>
</plist>
```

Load and start:

```bash
launchctl load ~/Library/LaunchAgents/com.caxton.server.plist
launchctl start com.caxton.server
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
cat > test-agent.md << EOF
---
name: TestAgent
version: "1.0.0"
capabilities:
  - testing
system_prompt: |
  You are a test agent. Respond with "Hello, Caxton!" to any message.
---

## Test Agent
Simple agent for testing Caxton installation.
EOF

# Deploy test agent
caxton agent deploy test-agent.md

# Send test message
caxton message send \
  --capability "testing" \
  --performative request \
  --content '{"message": "test"}'

# Clean up
caxton agent remove TestAgent
```

### Performance Benchmarks

```bash
# Test embedded memory system performance
caxton benchmark memory --entities 1000

# Test agent deployment performance
caxton benchmark agents --concurrent 10

# Test messaging performance
caxton benchmark messaging --messages 100 --concurrent 5
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
sudo chown -R caxton:caxton /etc/caxton /var/lib/caxton
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

## Uninstallation

### Remove Caxton Binary

```bash
# Remove from system PATH
sudo rm /usr/local/bin/caxton

# Remove package manager installation
brew uninstall caxton           # Homebrew
sudo apt remove caxton          # Ubuntu/Debian
sudo yum remove caxton          # RHEL/CentOS
```

### Remove Configuration and Data

```bash
# Remove system-wide installation
sudo rm -rf /etc/caxton
sudo rm -rf /var/lib/caxton
sudo rm -rf /var/log/caxton

# Remove user installation
rm -rf ~/.config/caxton
rm -rf ~/.local/share/caxton

# Remove service files
sudo rm /etc/systemd/system/caxton.service
sudo systemctl daemon-reload
```

### Remove Container Installation

```bash
# Stop and remove Docker containers
docker-compose down -v

# Remove Docker images
docker rmi caxton/caxton:latest

# Remove Kubernetes resources
kubectl delete -f caxton-k8s.yaml
```

## Scaling and Production Considerations

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
- Embedded memory OK, consider external for performance

**Large deployment** (> 100 agents, > 100K entities):

- RAM: 16GB+
- CPU: 8 cores+
- Storage: 100GB+
- Use external memory backends (Neo4j, Qdrant)

### High Availability Setup

```yaml
# Load balancer configuration
upstream caxton {
    server caxton-1:8080;
    server caxton-2:8080;
    server caxton-3:8080;
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

# Custom Grafana dashboard
grafana-cli plugins install caxton-dashboard

# Alert rules
caxton alert rule create --name "high-memory-usage" --threshold 90%
caxton alert rule create --name "slow-responses" --threshold 5s
```

The zero-dependency embedded memory system makes Caxton incredibly easy to
install and get started, while supporting scaling to external backends as
your multi-agent system grows.
