# Installation Guide

Caxton is a standalone server application that orchestrates multi-agent systems. This guide covers all installation methods.

## System Requirements

- **Operating System**: Linux, macOS, or Windows (via WSL2)
- **Memory**: Minimum 2GB RAM (4GB+ recommended for production)
- **Disk Space**: 500MB for Caxton binary and runtime
- **Network**: Port 8080 (API) and 9090 (metrics) must be available

## Quick Install

### Linux/macOS (Recommended)

```bash
curl -sSL https://caxton.io/install.sh | sh
```

This script will:
1. Detect your operating system and architecture
2. Download the appropriate Caxton binary
3. Install it to `/usr/local/bin`
4. Set up systemd service (Linux) or launchd (macOS)

### Package Managers

#### Homebrew (macOS/Linux)

```bash
brew install caxton
```

#### APT (Ubuntu/Debian)

```bash
# Add Caxton repository
curl -fsSL https://caxton.io/apt/gpg | sudo apt-key add -
echo "deb https://caxton.io/apt stable main" | sudo tee /etc/apt/sources.list.d/caxton.list

# Install Caxton
sudo apt update
sudo apt install caxton
```

#### YUM/DNF (RHEL/Fedora/CentOS)

```bash
# Add Caxton repository
sudo curl -o /etc/yum.repos.d/caxton.repo https://caxton.io/yum/caxton.repo

# Install Caxton
sudo yum install caxton
# or
sudo dnf install caxton
```

### Docker

```bash
# Pull the latest image
docker pull caxton/caxton:latest

# Run Caxton server
docker run -d \
  -p 8080:8080 \
  -p 9090:9090 \
  --name caxton \
  caxton/caxton:latest
```

### Docker Compose

Create a `docker-compose.yml`:

```yaml
version: '3.8'

services:
  caxton:
    image: caxton/caxton:latest
    ports:
      - "8080:8080"  # Management API
      - "9090:9090"  # Metrics endpoint
    volumes:
      - caxton-data:/var/lib/caxton
    restart: unless-stopped

volumes:
  caxton-data:
```

Then run:

```bash
docker-compose up -d
```

## Building from Source

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git

### Build Steps

```bash
# Clone the repository
git clone https://github.com/caxton/caxton.git
cd caxton

# Build in release mode
cargo build --release

# Binary will be at target/release/caxton
sudo cp target/release/caxton /usr/local/bin/

# Make it executable
sudo chmod +x /usr/local/bin/caxton
```

## Verifying Installation

After installation, verify Caxton is working:

```bash
# Check version
caxton version

# Start the server
caxton server start

# Check server status
caxton server status

# View available commands
caxton --help
```

## Post-Installation Setup

### 1. Configure Caxton

Create a configuration file at `/etc/caxton/config.yaml`:

```yaml
server:
  host: 0.0.0.0
  port: 8080
  metrics_port: 9090

runtime:
  max_agents: 1000
  max_memory_per_agent: 100MB
  default_agent_timeout: 30s

observability:
  log_level: info
  enable_tracing: true
  tracing_endpoint: http://localhost:14268
```

### 2. Set up as System Service

#### Linux (systemd)

```bash
# Create service file
sudo cat > /etc/systemd/system/caxton.service << EOF
[Unit]
Description=Caxton Multi-Agent Orchestration Server
After=network.target

[Service]
Type=simple
User=caxton
ExecStart=/usr/local/bin/caxton server start
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl enable caxton
sudo systemctl start caxton
```

#### macOS (launchd)

```bash
# Create plist file
sudo cat > /Library/LaunchDaemons/io.caxton.server.plist << EOF
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
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
EOF

# Load and start service
sudo launchctl load /Library/LaunchDaemons/io.caxton.server.plist
```

### 3. Configure Firewall

Ensure the following ports are accessible:

- **8080**: Management API (gRPC/REST)
- **9090**: Metrics endpoint (Prometheus)

```bash
# UFW (Ubuntu/Debian)
sudo ufw allow 8080/tcp
sudo ufw allow 9090/tcp

# firewalld (RHEL/Fedora)
sudo firewall-cmd --permanent --add-port=8080/tcp
sudo firewall-cmd --permanent --add-port=9090/tcp
sudo firewall-cmd --reload
```

## Troubleshooting

### Common Issues

1. **Port already in use**
   ```bash
   # Check what's using the port
   sudo lsof -i :8080

   # Change port in config.yaml
   ```

2. **Permission denied**
   ```bash
   # Ensure caxton binary is executable
   chmod +x /usr/local/bin/caxton
   ```

3. **Service won't start**
   ```bash
   # Check logs
   journalctl -u caxton -f  # Linux
   tail -f /var/log/caxton.log  # macOS
   ```

## Next Steps

- [Quick Start Guide](quickstart.md) - Deploy your first agents
- [Configuration Reference](configuration.md) - Detailed configuration options
- [First Agent Tutorial](first-agent.md) - Build your first WebAssembly agent
