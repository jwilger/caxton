---
title: "Monitoring and Metrics Integration"
date: 2025-01-15
layout: page
categories: [Operations]
---

This guide documents Caxton's metrics aggregation and monitoring strategy using
Prometheus and OpenTelemetry, ensuring comprehensive observability across all
components.

## Architecture

### Metrics Pipeline

```text
Agents → OpenTelemetry Collector → Prometheus → Grafana
                ↓
          Alternative Backends
         (Datadog, New Relic, etc.)
```

## Prometheus Integration

### Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'caxton-orchestrator'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'

  - job_name: 'caxton-agents'
    static_configs:
      - targets: ['localhost:9091-9099']
    metrics_path: '/metrics'

  - job_name: 'opentelemetry-collector'
    static_configs:
      - targets: ['localhost:8888']
```

### Key Metrics

#### Orchestrator Metrics

```rust
// Core orchestrator metrics
pub static MESSAGES_PROCESSED: Counter = Counter::new(
    "caxton_messages_processed_total",
    "Total number of messages processed"
);

pub static MESSAGE_LATENCY: Histogram = Histogram::new(
    "caxton_message_latency_seconds",
    "Message processing latency in seconds"
);

pub static ACTIVE_AGENTS: Gauge = Gauge::new(
    "caxton_active_agents",
    "Number of currently active agents"
);

pub static AGENT_MEMORY_USAGE: Gauge = Gauge::new(
    "caxton_agent_memory_bytes",
    "Memory usage per agent in bytes"
);
```

#### Agent Metrics

```rust
// Per-agent metrics
pub static TASK_DURATION: Histogram = Histogram::new(
    "caxton_task_duration_seconds",
    "Task execution duration in seconds"
);

pub static TASK_SUCCESS_RATE: Gauge = Gauge::new(
    "caxton_task_success_rate",
    "Task success rate (0-1)"
);

pub static AGENT_CPU_USAGE: Gauge = Gauge::new(
    "caxton_agent_cpu_usage_percent",
    "CPU usage percentage per agent"
);
```

### Metric Labels and Cardinality

#### Best Practices

- Keep cardinality under control (< 10 label values per metric)
- Use consistent label names across metrics
- Avoid high-cardinality labels (user IDs, request IDs)

#### Standard Labels

```rust
pub struct StandardLabels {
    pub agent_id: String,      // Agent identifier
    pub agent_type: String,     // Agent type/capability
    pub conversation_id: String, // Conversation correlation
    pub environment: String,    // dev/staging/prod
    pub version: String,        // Software version
}
```

## OpenTelemetry Collector Configuration

### Collector Setup

```yaml
# otel-collector-config.yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

  prometheus:
    config:
      scrape_configs:
        - job_name: 'caxton-metrics'
          scrape_interval: 10s
          static_configs:
            - targets: ['localhost:9090']

processors:
  batch:
    timeout: 10s
    send_batch_size: 1024

  memory_limiter:
    check_interval: 1s
    limit_mib: 512
    spike_limit_mib: 128

  resource:
    attributes:
      - key: service.name
        value: "caxton"
      - key: service.version
        from_attribute: version

exporters:
  prometheus:
    endpoint: "0.0.0.0:8889"

  logging:
    loglevel: debug

  jaeger:
    endpoint: jaeger-collector:14250
    tls:
      insecure: true

service:
  pipelines:
    metrics:
      receivers: [otlp, prometheus]
      processors: [memory_limiter, batch, resource]
      exporters: [prometheus, logging]

    traces:
      receivers: [otlp]
      processors: [memory_limiter, batch, resource]
      exporters: [jaeger, logging]
```

## Grafana Dashboard Configuration

### Core Dashboards

#### System Overview Dashboard

```json
{
  "dashboard": {
    "title": "Caxton System Overview",
    "panels": [
      {
        "title": "Message Throughput",
        "targets": [
          {
            "expr": "rate(caxton_messages_processed_total[5m])"
          }
        ]
      },
      {
        "title": "Active Agents",
        "targets": [
          {
            "expr": "caxton_active_agents"
          }
        ]
      },
      {
        "title": "Message Latency (p95)",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(caxton_message_latency_seconds_bucket[5m]))"
          }
        ]
      },
      {
        "title": "Error Rate",
        "targets": [
          {
            "expr": "rate(caxton_errors_total[5m])"
          }
        ]
      }
    ]
  }
}
```

#### Agent Performance Dashboard

```json
{
  "dashboard": {
    "title": "Agent Performance",
    "panels": [
      {
        "title": "Task Success Rate by Agent",
        "targets": [
          {
            "expr": "caxton_task_success_rate{}"
          }
        ]
      },
      {
        "title": "Agent Memory Usage",
        "targets": [
          {
            "expr": "caxton_agent_memory_bytes{}"
          }
        ]
      },
      {
        "title": "Task Duration Distribution",
        "targets": [
          {
            "expr": "histogram_quantile(0.5, rate(caxton_task_duration_seconds_bucket[5m]))"
          }
        ]
      }
    ]
  }
}
```

## Alert Rules

### Critical Alerts

```yaml
groups:
  - name: caxton_critical
    interval: 30s
    rules:
      - alert: HighErrorRate
        expr: rate(caxton_errors_total[5m]) > 0.01
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors/sec"

      - alert: OrchestratorDown
        expr: up{job="caxton-orchestrator"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Orchestrator is down"

      - alert: HighMemoryUsage
        expr: caxton_agent_memory_bytes > 1073741824  # 1GB
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Agent {{ $labels.agent_id }} high memory usage"
```

### Performance Alerts

```yaml
groups:
  - name: caxton_performance
    interval: 1m
    rules:
      - alert: HighLatency
        expr: histogram_quantile(0.95, rate(caxton_message_latency_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High message processing latency"
          description: "95th percentile latency is {{ $value }}s"

      - alert: LowThroughput
        expr: rate(caxton_messages_processed_total[5m]) < 10
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Low message throughput"
          description: "Processing only {{ $value }} messages/sec"
```

## Custom Metrics Implementation

### Adding New Metrics

```rust
use prometheus::{register_counter, register_histogram, register_gauge};

// Register custom metrics
lazy_static! {
    static ref CUSTOM_METRIC: Counter = register_counter!(
        "caxton_custom_metric_total",
        "Description of custom metric"
    ).unwrap();
}

// Use in code
CUSTOM_METRIC.inc();
```

### Metric Types Guide

- **Counter**: For monotonically increasing values (requests, errors)
- **Gauge**: For values that go up and down (memory, connections)
- **Histogram**: For distributions (latency, sizes)
- **Summary**: For pre-calculated quantiles (not recommended)

## Backend Alternatives

### Datadog Integration

```yaml
# For Datadog backend
exporters:
  datadog:
    api:
      key: ${DATADOG_API_KEY}
      site: datadoghq.com
    hostname: caxton-orchestrator
```

### New Relic Integration

```yaml
# For New Relic backend
exporters:
  newrelic:
    apikey: ${NEW_RELIC_API_KEY}
    timeout: 30s
```

### CloudWatch Integration

```yaml
# For AWS CloudWatch
exporters:
  awscloudwatchmetrics:
    namespace: Caxton
    region: us-west-2
```

## Performance Considerations

### Metric Collection Overhead

- Keep scrape intervals reasonable (15-30s for most metrics)
- Use histograms sparingly (higher storage cost)
- Batch metric updates where possible
- Consider sampling for high-volume metrics

### Storage and Retention

```yaml
# Prometheus storage configuration
storage:
  tsdb:
    path: /var/lib/prometheus
    retention.time: 30d
    retention.size: 10GB
    wal_compression: true
```

### Query Optimization

- Use recording rules for expensive queries
- Implement query result caching
- Optimize label cardinality
- Use downsampling for long-term storage

## Debugging Metrics Issues

### Common Problems and Solutions

#### Missing Metrics

```bash
# Check if metrics endpoint is accessible
curl http://localhost:9090/metrics

# Verify Prometheus scrape config
curl http://localhost:9090/api/v1/targets

# Check collector logs
docker logs otel-collector
```

#### High Cardinality

```promql
# Find high cardinality metrics
count by (__name__)({__name__=~".+"})

# Identify problematic labels
count by (label_name) (metric_name)
```

#### Performance Issues

```bash
# Profile Prometheus
curl http://localhost:9090/debug/pprof/profile?seconds=30 > profile.pb.gz

# Check TSDB stats
curl http://localhost:9090/api/v1/tsdb_status
```

## Best Practices Summary

1. **Use standard metrics libraries** - OpenTelemetry SDK preferred
2. **Keep cardinality low** - < 100k unique series
3. **Document all metrics** - Include unit and meaning
4. **Version metric names** - Include v1, v2 when breaking changes
5. **Test alerts locally** - Use Prometheus unit tests
6. **Monitor the monitoring** - Meta-metrics for observability stack
7. **Regular cleanup** - Remove unused metrics and dashboards

## References

- [Prometheus Best Practices](https://prometheus.io/docs/practices/)
- [OpenTelemetry Collector Docs](https://opentelemetry.io/docs/collector/)
- [Grafana Dashboard Best Practices](https://grafana.com/docs/grafana/latest/best-practices/)
- [ADR-0001: Observability-First Architecture](../adrs/0001-observability-first-architecture.md)
- [Performance Tuning Guide](performance-tuning.md)
- [Operational Runbook](operational-runbook.md)
