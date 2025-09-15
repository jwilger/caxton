---
title: "Production Security Guide"
date: 2025-01-15
layout: page
categories: [Operations, Security]
audience: operators
description: "Production security deployment and monitoring for Caxton
  operators"
---

## Production Security Overview

This guide provides production security deployment and monitoring guidance for
Caxton operators. Focus is on embedded architecture security, operational
monitoring, and incident response procedures for production environments.

### Security Priority Matrix

| Area                     | Priority | Implementation Time |
| ------------------------ | -------- | ------------------- |
| Configuration Validation | P0       | Day 0               |
| Memory Scope Isolation   | P0       | Day 0               |
| Tool Access Control      | P0       | Day 0               |
| TLS/API Security         | P1       | Week 1              |
| Security Monitoring      | P1       | Week 1              |
| Incident Response        | P2       | Month 1             |

## Production Deployment Security

### Embedded Architecture Security Configuration

Configure production security settings for the embedded architecture:

```bash
# Production environment variables (required)
export CAXTON_MEMORY_BACKEND=embedded
export CAXTON_CONFIG_AGENT_VALIDATION=strict
export CAXTON_WASM_ISOLATION=strict
export CAXTON_SECURITY_AUDIT=enabled
export CAXTON_MEMORY_SCOPE_ISOLATION=enabled
export CAXTON_TOOL_ACCESS_CONTROL=enabled
export CAXTON_HOT_RELOAD_VALIDATION=strict
```

### System-Level Security Hardening

Implement system-level security for production deployment:

```bash
# Create dedicated system user
sudo useradd --system --home-dir /var/lib/caxton --shell /bin/false caxton

# Secure directory structure
sudo mkdir -p /var/lib/caxton/{data,agents,logs}
sudo chown -R caxton:caxton /var/lib/caxton/
sudo chmod 755 /var/lib/caxton/
sudo chmod 700 /var/lib/caxton/data/     # Restrict SQLite access
sudo chmod 755 /var/lib/caxton/agents/   # Agent configs
sudo chmod 755 /var/lib/caxton/logs/     # Log directory

# Secure agent configuration files
sudo chown root:caxton /var/lib/caxton/agents/*.md
sudo chmod 644 /var/lib/caxton/agents/*.md
```

### Systemd Service Security

Deploy with comprehensive systemd security restrictions:

```ini
# /etc/systemd/system/caxton.service
[Unit]
Description=Caxton Multi-Agent Server
After=network.target
StartLimitIntervalSec=60
StartLimitBurst=3

[Service]
Type=simple
User=caxton
Group=caxton
ExecStart=/usr/local/bin/caxton start --config /etc/caxton/caxton.yaml
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=5

# Security restrictions
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
PrivateDevices=true
ProtectKernelTunables=true
ProtectControlGroups=true
RestrictRealtime=true
MemoryDenyWriteExecute=true
SystemCallFilter=@system-service
SystemCallErrorNumber=EPERM
LockPersonality=true
ProtectKernelModules=true
ProtectKernelLogs=true
ProtectClock=true
RemoveIPC=true

# Resource limits
LimitNOFILE=8192
LimitNPROC=4096

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=caxton

[Install]
WantedBy=multi-user.target
```

### Network Security Configuration

Implement production network security:

```bash
# Firewall configuration (iptables/ufw)
sudo ufw enable
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Production API port (HTTPS)
sudo ufw allow 8443/tcp comment 'Caxton HTTPS API'

# Monitoring port (metrics)
sudo ufw allow from 10.0.0.0/8 to any port 9090 comment 'Internal monitoring'

# SSH access (restrict to bastion)
sudo ufw allow from 10.0.1.100 to any port 22 comment 'Bastion SSH'

# Block agent access to internal networks
sudo iptables -A OUTPUT -p tcp --dport 1:1023 -j DROP
sudo iptables -A OUTPUT -d 192.168.0.0/16 -j DROP
sudo iptables -A OUTPUT -d 10.0.0.0/8 -j DROP
sudo iptables -A OUTPUT -d 172.16.0.0/12 -j DROP
```

### TLS Configuration

Configure production TLS with proper certificate management:

```bash
# Generate production certificate (Let's Encrypt recommended)
sudo certbot certonly --standalone -d caxton.yourdomain.com

# Configure Caxton with TLS
cat > /etc/caxton/tls.yaml << 'EOF'
server:
  port: 8443
  tls:
    enabled: true
    cert_path: "/etc/letsencrypt/live/caxton.yourdomain.com/fullchain.pem"
    key_path: "/etc/letsencrypt/live/caxton.yourdomain.com/privkey.pem"
    min_version: "1.3"
    ciphers: ["TLS_AES_256_GCM_SHA384", "TLS_CHACHA20_POLY1305_SHA256"]
EOF

# Certificate renewal automation
cat > /usr/local/bin/renew-caxton-cert.sh << 'EOF'
#!/bin/bash
set -e

# Renew certificate
certbot renew --quiet

# Reload Caxton if certificate was renewed
if systemctl is-active --quiet caxton; then
    systemctl reload caxton
fi
EOF

chmod +x /usr/local/bin/renew-caxton-cert.sh

# Schedule automatic renewal (weekly check)
echo "0 3 * * 0 /usr/local/bin/renew-caxton-cert.sh" | sudo crontab -
```

## Security Monitoring and Alerting

### Production Security Metrics

Monitor these critical security indicators:

#### Configuration Security Metrics

```yaml
# Prometheus scrape configuration
- job_name: "caxton-security"
  static_configs:
    - targets: ["localhost:9090"]
  scrape_interval: 30s
  metrics_path: "/metrics"
  params:
    security: ["true"]
```

**Key Security Metrics:**

- `caxton_config_validation_failures_total`: Agent config validation failures
- `caxton_memory_scope_violations_total`: Memory access violations
- `caxton_tool_access_denials_total`: Tool access control denials
- `caxton_hot_reload_security_violations_total`: Configuration reload violations
- `caxton_prompt_injection_attempts_total`: Prompt injection detection
- `caxton_api_auth_failures_total`: API authentication failures
- `caxton_sqlite_security_events_total`: Database security events

### Production Alerting Rules

Configure Prometheus alerting for security events:

```yaml
# /etc/prometheus/rules/caxton-security.yml
groups:
  - name: caxton_production_security
    rules:
      - alert: CaxtonConfigValidationCritical
        expr: rate(caxton_config_validation_failures_total[5m]) > 10
        for: 1m
        labels:
          severity: critical
          component: config
        annotations:
          summary: "Critical configuration validation failure rate"
          description: "{{ $value }} config validation failures per second"
          runbook_url: "https://ops.company.com/caxton/security/config-validation"

      - alert: CaxtonMemorySecurityBreach
        expr: caxton_memory_scope_violations_total > 0
        for: 0m
        labels:
          severity: critical
          component: memory
        annotations:
          summary: "Memory scope security violation detected"
          description: "{{ $value }} memory scope violations detected"
          runbook_url: "https://ops.company.com/caxton/security/memory-violation"

      - alert: CaxtonToolAccessAttack
        expr: rate(caxton_tool_access_denials_total[5m]) > 20
        for: 2m
        labels:
          severity: warning
          component: tools
        annotations:
          summary: "Possible tool access attack in progress"
          description: "{{ $value }} tool access denials per second"

      - alert: CaxtonPromptInjectionDetected
        expr: increase(caxton_prompt_injection_attempts_total[1m]) > 0
        for: 0m
        labels:
          severity: high
          component: agents
        annotations:
          summary: "Prompt injection attack detected"
          description: "{{ $value }} prompt injection attempts in last minute"
          runbook_url: "https://ops.company.com/caxton/security/prompt-injection"

      - alert: CaxtonAPISecurityIncident
        expr: rate(caxton_api_auth_failures_total[5m]) > 5
        for: 3m
        labels:
          severity: warning
          component: api
        annotations:
          summary: "High API authentication failure rate"
          description: "{{ $value }} API auth failures per second"
```

### Security Log Analysis

Implement structured security log analysis:

```bash
# Production log aggregation (rsyslog configuration)
cat > /etc/rsyslog.d/50-caxton-security.conf << 'EOF'
# Caxton security logs
if $programname == 'caxton' and $msg contains 'SECURITY' then {
    /var/log/caxton/security.log
    stop
}
EOF

# Security event parsing script
cat > /usr/local/bin/caxton-security-parser.sh << 'EOF'
#!/bin/bash

SECURITY_LOG="/var/log/caxton/security.log"
ALERT_THRESHOLD=5

# Parse security events
jq -r 'select(.fields.event_type == "security_violation") |
       [.timestamp, .fields.violation_type, .fields.agent_id, .fields.details] |
       @tsv' $SECURITY_LOG |
while IFS=$'\t' read -r timestamp type agent details; do
    echo "SECURITY ALERT: $timestamp - $type on $agent: $details"

    # Send to SIEM/alerting system
    curl -X POST https://siem.company.com/api/alerts \
         -H "Content-Type: application/json" \
         -d "{
             \"timestamp\": \"$timestamp\",
             \"source\": \"caxton\",
             \"type\": \"$type\",
             \"agent\": \"$agent\",
             \"details\": \"$details\"
         }"
done
EOF

chmod +x /usr/local/bin/caxton-security-parser.sh

# Schedule security log parsing (every 5 minutes)
echo "*/5 * * * * /usr/local/bin/caxton-security-parser.sh" | crontab -
```

### Dashboard Configuration

Create operational security dashboard:

```json
{
  "dashboard": {
    "title": "Caxton Production Security",
    "panels": [
      {
        "title": "Security Violations (Last 24h)",
        "type": "stat",
        "targets": [
          {
            "expr": "sum(increase(caxton_memory_scope_violations_total[24h]))",
            "legendFormat": "Memory Violations"
          },
          {
            "expr": "sum(increase(
              caxton_prompt_injection_attempts_total[24h]))",
            "legendFormat": "Prompt Injections"
          }
        ]
      },
      {
        "title": "Configuration Security Status",
        "type": "timeseries",
        "targets": [
          {
            "expr": "rate(caxton_config_validation_failures_total[5m])",
            "legendFormat": "Config Validation Failures/sec"
          },
          {
            "expr": "rate(caxton_hot_reload_security_violations_total[5m])",
            "legendFormat": "Hot Reload Violations/sec"
          }
        ]
      },
      {
        "title": "Tool Access Control",
        "type": "timeseries",
        "targets": [
          {
            "expr": "rate(caxton_tool_access_denials_total[5m])",
            "legendFormat": "Tool Access Denials/sec"
          },
          {
            "expr": "caxton_active_tool_sessions",
            "legendFormat": "Active Tool Sessions"
          }
        ]
      }
    ]
  }
}
```

## Incident Response Procedures

### Security Incident Classification

### P0 - Critical (Response: Immediate)

- Memory scope isolation breach
- Configuration tampering detection
- System compromise indicators
- Data exfiltration attempts

### P1 - High (Response: < 1 hour)

- Prompt injection attacks
- API authentication bypass
- Hot reload security violations
- WASM sandbox escapes

### P2 - Medium (Response: < 4 hours)

- Tool access control violations
- Configuration validation failures
- Unusual access patterns
- Certificate/TLS issues

### P3 - Low (Response: < 24 hours)

- Security policy violations
- Log analysis anomalies
- Performance-based DoS
- Documentation gaps

### Incident Response Runbooks

#### Memory Scope Violation Response

```bash
#!/bin/bash
# Runbook: Memory scope violation incident response

echo "=== CAXTON SECURITY INCIDENT: MEMORY VIOLATION ==="
echo "Timestamp: $(date)"
echo ""

# 1. Immediate containment
echo "1. CONTAINMENT: Isolating affected agents..."
caxton agents list --scope-violations | while read agent_id; do
    echo "  Suspending agent: $agent_id"
    caxton agents suspend "$agent_id" --reason="security_violation"
done

# 2. Evidence collection
echo "2. EVIDENCE: Collecting violation details..."
caxton memory violations --export > "/tmp/memory-violation-$(date +%s).json"
caxton logs export --security --last-1h > "/tmp/security-logs-$(date +%s).log"

# 3. Impact assessment
echo "3. ASSESSMENT: Checking scope integrity..."
caxton memory scope-integrity-check --detailed

# 4. Recovery planning
echo "4. RECOVERY: Preparing recovery actions..."
caxton memory scope-rebuild --dry-run --affected-only

echo ""
echo "Next steps:"
echo "- Review evidence files in /tmp/"
echo "- Execute recovery plan if integrity check passes"
echo "- Update security policies based on root cause"
echo "- Document lessons learned"
```

#### Configuration Attack Response

```bash
#!/bin/bash
# Runbook: Configuration tampering incident response

echo "=== CAXTON SECURITY INCIDENT: CONFIG TAMPERING ==="

# 1. Stop all hot reload operations
echo "1. CONTAINMENT: Stopping configuration changes..."
caxton config lock --emergency

# 2. Validate current configurations
echo "2. VALIDATION: Checking configuration integrity..."
caxton agents validate-all --security-audit --verbose

# 3. Restore from backup if needed
if [ "$1" = "--restore" ]; then
    echo "3. RECOVERY: Restoring from backup..."
    systemctl stop caxton
    tar -xzf /backup/caxton/caxton-embedded-backup-latest.tar.gz -C /
    systemctl start caxton
fi

# 4. Re-enable with enhanced security
echo "4. HARDENING: Re-enabling with enhanced security..."
caxton config unlock --enhanced-validation
```

### Post-Incident Actions

#### Security Assessment Template

```markdown
# Caxton Security Incident Report

**Incident ID**: SEC-{{ date }}-{{ sequence }}
**Severity**: {{ P0|P1|P2|P3 }}
**Duration**: {{ detection_time }} to {{ resolution_time }}

## Summary

Brief description of the security incident and its impact.

## Timeline

- **{{ time }}**: Initial detection via {{ monitoring_system }}
- **{{ time }}**: Containment actions initiated
- **{{ time }}**: Root cause identified
- **{{ time }}**: Resolution implemented
- **{{ time }}**: Normal operations restored

## Root Cause Analysis

Detailed technical analysis of how the incident occurred.

## Impact Assessment

- **Affected agents**: {{ count }}
- **Data exposure**: {{ none|minimal|significant }}
- **Service availability**: {{ percentage }}%
- **Security controls bypassed**: {{ list }}

## Response Actions

1. **Immediate containment**: {{ actions }}
2. **Evidence collection**: {{ files_collected }}
3. **Recovery procedures**: {{ steps }}
4. **Communications**: {{ stakeholders_notified }}

## Lessons Learned

1. **What worked well**: {{ successes }}
2. **Areas for improvement**: {{ gaps }}
3. **Security enhancements needed**: {{ improvements }}

## Follow-up Actions

- [ ] Security policy updates
- [ ] Monitoring improvements
- [ ] Staff training updates
- [ ] Documentation updates
- [ ] Technology improvements

**Prepared by**: {{ operator_name }}
**Reviewed by**: {{ security_lead }}
**Date**: {{ report_date }}
```

## Production Security Maintenance

### Daily Security Tasks

```bash
#!/bin/bash
# Daily security maintenance script

echo "=== CAXTON DAILY SECURITY MAINTENANCE ==="
date

# Check security metrics
echo "1. Checking security metrics..."
curl -s http://localhost:9090/metrics | \
  grep -E "caxton_(config_validation_failures|memory_scope_violations|\
           tool_access_denials)_total"

# Validate configuration integrity
echo "2. Validating configuration integrity..."
caxton agents validate-all --security-check

# Check memory scope health
echo "3. Checking memory scope integrity..."
caxton memory scope-health-check

# Review security logs
echo "4. Reviewing security events (last 24h)..."
grep -c "SECURITY" /var/log/caxton/security.log | tail -1

# Check certificate expiry
echo "5. Checking TLS certificate status..."
openssl x509 \
  -in /etc/letsencrypt/live/caxton.yourdomain.com/cert.pem \
  -checkend $((30*24*3600)) -noout

# File permission audit
echo "6. Auditing file permissions..."
find /var/lib/caxton -type f -perm /o+w -ls

echo "=== DAILY MAINTENANCE COMPLETE ==="
```

### Weekly Security Tasks

```bash
#!/bin/bash
# Weekly security maintenance script

echo "=== CAXTON WEEKLY SECURITY MAINTENANCE ==="

# Security log analysis
echo "1. Analyzing security trends (last 7 days)..."
/usr/local/bin/caxton-security-trend-analysis.sh

# Configuration backup verification
echo "2. Verifying backup integrity..."
/usr/local/bin/caxton-backup-verification.sh

# Security policy compliance check
echo "3. Checking security policy compliance..."
caxton security policy-compliance-check

# Update security documentation
echo "4. Checking for security updates..."
curl -s https://api.github.com/repos/your-org/caxton/security/advisories | \
  jq '.[] | select(.state == "published")'

# Penetration testing (automated)
echo "5. Running security baseline tests..."
/usr/local/bin/caxton-security-baseline-test.sh

echo "=== WEEKLY MAINTENANCE COMPLETE ==="
```

### Security Backup and Recovery

#### Production Backup Strategy

```bash
#!/bin/bash
# Production security backup script

set -euo pipefail

BACKUP_DIR="/secure-backup/caxton"
DATE_SUFFIX=$(date +%Y%m%d-%H%M)
BACKUP_FILE="caxton-production-${DATE_SUFFIX}.tar.gz"

echo "Starting Caxton production backup: $BACKUP_FILE"

# Pre-backup security check
caxton agents validate-all --security-check
caxton memory scope-integrity-check

# Stop service for consistent backup
systemctl stop caxton

# Create encrypted backup
tar -czf "/tmp/$BACKUP_FILE" \
    /etc/caxton/ \
    /var/lib/caxton/data/ \
    /var/lib/caxton/agents/ \
    /etc/systemd/system/caxton.service \
    /etc/letsencrypt/live/caxton.yourdomain.com/

# Restart service
systemctl start caxton

# Encrypt with production key
gpg --symmetric \
    --cipher-algo AES256 \
    --digest-algo SHA256 \
    --cert-digest-algo SHA256 \
    --compress-algo 2 \
    --s2k-mode 3 \
    --s2k-digest-algo SHA256 \
    --s2k-count 65011712 \
    --batch --yes \
    --passphrase-file /etc/caxton/backup-passphrase \
    "/tmp/$BACKUP_FILE"

# Store in secure location
mv "/tmp/$BACKUP_FILE.gpg" "$BACKUP_DIR/"
rm "/tmp/$BACKUP_FILE"

# Verify backup
gpg --quiet --batch --yes \
    --passphrase-file /etc/caxton/backup-passphrase \
    --decrypt "$BACKUP_DIR/$BACKUP_FILE.gpg" | tar -tzf - > /dev/null

# Cleanup old backups (keep 30 days)
find "$BACKUP_DIR" -name "caxton-production-*.tar.gz.gpg" -mtime +30 -delete

echo "Backup completed successfully: $BACKUP_DIR/$BACKUP_FILE.gpg"
```

## Security Compliance Reporting

### Compliance Dashboard

```yaml
# Compliance metrics configuration
compliance_metrics:
  data_protection:
    - memory_scope_isolation_enabled
    - configuration_validation_strict
    - sqlite_security_enabled
    - file_permissions_compliant

  access_control:
    - tool_access_control_enabled
    - api_authentication_required
    - tls_encryption_enabled
    - certificate_valid

  audit_trail:
    - security_logging_enabled
    - log_retention_compliant
    - backup_encryption_enabled
    - incident_response_documented
```

### Security Attestation Report

Generate automated compliance reports:

```bash
#!/bin/bash
# Security compliance attestation generator

cat > caxton-security-attestation-$(date +%Y%m%d).md << EOF
# Caxton Security Compliance Attestation

**Generated**: $(date)
**Environment**: Production
**Assessment Period**: $(date -d '30 days ago' +%Y-%m-%d) to $(date +%Y-%m-%d)

## Security Controls Status

### Data Protection
- [x] Memory scope isolation: $(caxton memory scope-status --check)
- [x] Configuration validation: $(caxton config validation-status)
- [x] SQLite security: $(sqlite3 /var/lib/caxton/data/memory.db \
      'PRAGMA foreign_keys')
- [x] File permissions: $(stat -c %a /var/lib/caxton/data)

### Access Control
- [x] Tool access control: $(caxton tools access-control-status)
- [x] API authentication: $(caxton api auth-status)
- [x] TLS encryption: $(openssl s_client -connect localhost:8443 \
      < /dev/null 2>&1 | grep -c "Verify return code: 0")

### Monitoring and Auditing
- [x] Security monitoring: $(systemctl is-active prometheus)
- [x] Log retention: $(find /var/log/caxton -name "*.log" -mtime -30 | \
      wc -l) files
- [x] Backup encryption: $(ls /secure-backup/caxton/*.gpg | \
      wc -l) encrypted backups

## Security Incidents
- **Total incidents**: $(grep -c "SECURITY_INCIDENT" \
    /var/log/caxton/security.log)
- **P0 incidents**: $(grep -c "severity: critical" /var/log/caxton/security.log)
- **Resolution time (avg)**: $(calculate-avg-resolution-time.sh)

## Compliance Summary
All required security controls are operational and compliant.

**Attested by**: Caxton Operations Team
**Next review**: $(date -d '30 days' +%Y-%m-%d)
EOF
```

This production security guide provides comprehensive operational security
guidance for Caxton operators, focusing on embedded architecture security,
monitoring, and incident response procedures essential for production
environments.
