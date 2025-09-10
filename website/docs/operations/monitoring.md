---
title: "Monitoring and Observability Guide"
layout: documentation
description: "Comprehensive monitoring and observability setup for Caxton multi-agent systems using OpenTelemetry, Prometheus, Jaeger, and custom dashboards"
date: 2025-09-10
categories: [Website]
---

This guide covers setting up comprehensive monitoring and observability for
Caxton multi-agent systems, including metrics collection, distributed tracing,
log aggregation, alerting, and performance monitoring.

## Observability Architecture

Caxton's observability stack is built on industry-standard tools:

- **OpenTelemetry**: Unified telemetry collection and export
- **Prometheus**: Metrics storage and alerting
- **Jaeger**: Distributed tracing
- **Grafana**: Visualization and dashboards
- **Fluentd/Loki**: Log aggregation
- **AlertManager**: Alert routing and management

## OpenTelemetry Integration

### Configuration

Caxton has built-in OpenTelemetry support that can be configured through the
main configuration file:

```toml
# caxton.toml
[observability]
# Enable OpenTelemetry instrumentation
enable_tracing = true
enable_metrics = true
enable_logging = true

# OTLP export configuration
otlp_endpoint = "http://otel-collector:4317"
otlp_timeout = "10s"
otlp_headers = { "api-key" = "your-api-key" }

# Sampling configuration
trace_sample_rate = 0.1  # Sample 10% of traces
metrics_export_interval = "10s"

# Resource attributes
service_name = "caxton-runtime"
service_version = "0.2.0"
deployment_environment = "production"
```

### OpenTelemetry Collector Configuration

```yaml
# otel-collector-config.yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 1s
    send_batch_size: 1024

  resource:
    attributes:
      - key: environment
        value: production
        action: upsert

  tail_sampling:
    decision_wait: 10s
    num_traces: 100
    expected_new_traces_per_sec: 10
    policies:
      - name: error_sampling
        type: status_code
        status_code: {status_codes: [ERROR]}
      - name: slow_requests
        type: latency
        latency: {threshold_ms: 1000}
      - name: random_sampling
        type: probabilistic
        probabilistic: {sampling_percentage: 10}

exporters:
  prometheus:
    endpoint: "0.0.0.0:8889"
    namespace: caxton
    const_labels:
      environment: production

  jaeger:
    endpoint: jaeger-collector:14250
    tls:
      insecure: true

  loki:
    endpoint: http://loki:3100/loki/api/v1/push
    tenant_id: caxton

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [resource, tail_sampling, batch]
      exporters: [jaeger]

    metrics:
      receivers: [otlp]
      processors: [resource, batch]
      exporters: [prometheus]

    logs:
      receivers: [otlp]
      processors: [resource, batch]
      exporters: [loki]
```

## Metrics Collection

### Core Metrics

Caxton automatically exposes the following metrics categories:

#### Runtime Metrics

- `caxton_agents_total`: Total number of agents
- `caxton_agents_active`: Currently active agents
- `caxton_agent_executions_total`: Total agent executions
- `caxton_agent_execution_duration_seconds`: Agent execution time
- `caxton_wasm_memory_usage_bytes`: WASM memory usage per agent
- `caxton_runtime_memory_usage_bytes`: Runtime memory usage
- `caxton_runtime_cpu_usage_ratio`: CPU utilization

#### Message Protocol Metrics

- `caxton_messages_sent_total`: FIPA messages sent
- `caxton_messages_received_total`: FIPA messages received
- `caxton_message_processing_duration_seconds`: Message processing time
- `caxton_message_queue_size`: Current message queue size
- `caxton_message_errors_total`: Message processing errors

#### HTTP API Metrics

- `caxton_http_requests_total`: HTTP requests by method/status
- `caxton_http_request_duration_seconds`: HTTP request duration
- `caxton_http_active_connections`: Active HTTP connections
- `caxton_websocket_connections`: Active WebSocket connections

#### System Resource Metrics

- `caxton_disk_usage_bytes`: Disk space usage
- `caxton_network_bytes_total`: Network I/O
- `caxton_file_descriptors`: Open file descriptors

### Custom Metrics Configuration

```toml
[observability.metrics]
# Custom histogram buckets for latency metrics
http_duration_buckets = [0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
agent_execution_buckets = [0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0]

# Metric labels to include
include_labels = ["agent_type", "agent_id", "message_type"]
exclude_labels = ["sensitive_data"]

# Export configuration
export_interval = "15s"
export_timeout = "5s"
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
  - job_name: 'caxton-runtime'
    static_configs:
      - targets: ['caxton-runtime:9090']
    scrape_interval: 10s
    metrics_path: /metrics

  - job_name: 'caxton-kubernetes'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
      - source_labels: [__address__, __meta_kubernetes_pod_annotation_prometheus_io_port]
        action: replace
        regex: ([^:]+)(?::\d+)?;(\d+)
        replacement: $1:$2
        target_label: __address__

  - job_name: 'otel-collector'
    static_configs:
      - targets: ['otel-collector:8889']
```

## Distributed Tracing

### Trace Context Propagation

Caxton automatically propagates trace context through:

- HTTP headers (W3C Trace Context)
- FIPA message metadata
- Internal agent communications
- Database operations

### Custom Spans

```rust
// Example: Adding custom spans in agent code
use opentelemetry::trace::Tracer;

#[tracing::instrument(name = "agent.execute_task")]
async fn execute_task(&self, task: Task) -> Result<TaskResult> {
    let span = tracer.start("task.processing");
    span.set_attribute("task.type", task.task_type.clone());
    span.set_attribute("task.priority", task.priority as i64);

    // Process task
    let result = self.process_task_internal(task).await;

    match &result {
        Ok(_) => span.set_status(Status::Ok),
        Err(e) => {
            span.set_status(Status::Error {
                description: e.to_string().into()
            });
        }
    }

    result
}
```

### Jaeger Configuration

```yaml
# jaeger-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: jaeger
spec:
  replicas: 1
  selector:
    matchLabels:
      app: jaeger
  template:
    metadata:
      labels:
        app: jaeger
    spec:
      containers:
      - name: jaeger
        image: jaegertracing/all-in-one:1.45
        ports:
        - containerPort: 16686  # UI
        - containerPort: 14268  # HTTP collector
        - containerPort: 14250  # gRPC collector
        - containerPort: 6831   # UDP agent
        env:
        - name: COLLECTOR_OTLP_ENABLED
          value: "true"
        - name: SPAN_STORAGE_TYPE
          value: elasticsearch
        - name: ES_SERVER_URLS
          value: http://elasticsearch:9200
        resources:
          requests:
            memory: 512Mi
            cpu: 250m
          limits:
            memory: 1Gi
            cpu: 500m
```

### Trace Sampling Strategies

```json
{
  "service_strategies": [
    {
      "service": "caxton-runtime",
      "type": "probabilistic",
      "param": 0.1,
      "max_traces_per_second": 100,
      "operation_strategies": [
        {
          "operation": "agent.execute",
          "type": "probabilistic",
          "param": 0.2
        },
        {
          "operation": "message.process",
          "type": "ratelimiting",
          "param": 50
        }
      ]
    }
  ],
  "default_strategy": {
    "type": "probabilistic",
    "param": 0.05
  }
}
```

## Log Aggregation

### Structured Logging Configuration

```toml
[observability.logging]
level = "info"
format = "json"
output = "stdout"

# Log correlation
include_trace_id = true
include_span_id = true

# Field configuration
timestamp_format = "rfc3339"
level_key = "level"
message_key = "message"
trace_id_key = "trace_id"
span_id_key = "span_id"

# Sampling for high-volume logs
debug_sample_rate = 0.01
trace_sample_rate = 0.1
```

### Fluentd Configuration

```yaml
# fluentd-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: fluentd-config
data:
  fluent.conf: |
    <source>
      @type tail
      @id caxton_logs
      path /var/log/containers/caxton-*.log
      pos_file /var/log/fluentd-caxton.log.pos
      tag caxton.*
      format json
      read_from_head true
    </source>

    <filter caxton.**>
      @type parser
      key_name message
      reserve_data true
      <parse>
        @type json
        json_parser_error_class JSONParserError
      </parse>
    </filter>

    <filter caxton.**>
      @type record_transformer
      <record>
        service "caxton-runtime"
        environment "production"
        cluster "#{ENV['CLUSTER_NAME']}"
      </record>
    </filter>

    <match caxton.**>
      @type copy
      <store>
        @type loki
        url http://loki:3100
        tenant ""
        extra_labels {"service":"caxton"}
        line_format json
        <label>
          level
          service
          environment
          agent_id
        </label>
      </store>
      <store>
        @type elasticsearch
        host elasticsearch
        port 9200
        index_name caxton-logs
        type_name _doc
        include_timestamp true
      </store>
    </match>
```

### Loki Configuration

```yaml
# loki-config.yaml
auth_enabled: false

server:
  http_listen_port: 3100
  grpc_listen_port: 9096

common:
  path_prefix: /tmp/loki
  storage:
    filesystem:
      chunks_directory: /tmp/loki/chunks
      rules_directory: /tmp/loki/rules
  replication_factor: 1
  ring:
    instance_addr: 127.0.0.1
    kvstore:
      store: inmemory

query_range:
  results_cache:
    cache:
      embedded_cache:
        enabled: true
        max_size_mb: 100

schema_config:
  configs:
    - from: 2020-10-24
      store: boltdb-shipper
      object_store: filesystem
      schema: v11
      index:
        prefix: index_
        period: 24h

ruler:
  alertmanager_url: http://alertmanager:9093

limits_config:
  ingestion_rate_mb: 16
  ingestion_burst_size_mb: 24
  max_streams_per_user: 10000
  max_line_size: 256KB
```

## Alerting Rules

### Prometheus Alert Rules

```yaml
# caxton_rules.yml
groups:
- name: caxton.rules
  rules:
  # Agent health alerts
  - alert: CaxtonAgentHighFailureRate
    expr: rate(caxton_agent_executions_total{status="error"}[5m]) / rate(caxton_agent_executions_total[5m]) > 0.05
    for: 2m
    labels:
      severity: warning
      service: caxton
    annotations:
      summary: "High agent execution failure rate"
      description: "Agent execution failure rate is {{ $value | humanizePercentage }} for {{ $labels.agent_type }}"

  - alert: CaxtonAgentMemoryLeak
    expr: increase(caxton_wasm_memory_usage_bytes[30m]) > 100*1024*1024
    for: 5m
    labels:
      severity: critical
      service: caxton
    annotations:
      summary: "Potential memory leak in agent"
      description: "Agent {{ $labels.agent_id }} memory usage increased by {{ $value | humanizeBytes }} in 30 minutes"

  # System resource alerts
  - alert: CaxtonHighCPUUsage
    expr: caxton_runtime_cpu_usage_ratio > 0.8
    for: 5m
    labels:
      severity: warning
      service: caxton
    annotations:
      summary: "High CPU usage"
      description: "CPU usage is {{ $value | humanizePercentage }}"

  - alert: CaxtonHighMemoryUsage
    expr: caxton_runtime_memory_usage_bytes / 1024/1024/1024 > 16
    for: 5m
    labels:
      severity: critical
      service: caxton
    annotations:
      summary: "High memory usage"
      description: "Memory usage is {{ $value | humanizeBytes }}"

  # API performance alerts
  - alert: CaxtonHighResponseTime
    expr: histogram_quantile(0.95, rate(caxton_http_request_duration_seconds_bucket[5m])) > 2.0
    for: 2m
    labels:
      severity: warning
      service: caxton
    annotations:
      summary: "High HTTP response time"
      description: "95th percentile response time is {{ $value }}s"

  - alert: CaxtonServiceDown
    expr: up{job="caxton-runtime"} == 0
    for: 1m
    labels:
      severity: critical
      service: caxton
    annotations:
      summary: "Caxton service is down"
      description: "Caxton runtime service is not responding"

  # Message processing alerts
  - alert: CaxtonMessageQueueBacklog
    expr: caxton_message_queue_size > 1000
    for: 5m
    labels:
      severity: warning
      service: caxton
    annotations:
      summary: "Large message queue backlog"
      description: "Message queue size is {{ $value }} messages"

  - alert: CaxtonMessageProcessingErrors
    expr: rate(caxton_message_errors_total[5m]) > 10
    for: 2m
    labels:
      severity: critical
      service: caxton
    annotations:
      summary: "High message processing error rate"
      description: "Message processing error rate is {{ $value }}/second"
```

### AlertManager Configuration

```yaml
# alertmanager.yml
global:
  smtp_smarthost: 'localhost:587'
  smtp_from: 'alerts@example.com'
  slack_api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'

route:
  group_by: ['alertname', 'service']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'web.hook'
  routes:
  - match:
      severity: critical
    receiver: 'critical-alerts'
  - match:
      service: caxton
    receiver: 'caxton-team'

receivers:
- name: 'web.hook'
  webhook_configs:
  - url: 'http://127.0.0.1:5001/'

- name: 'critical-alerts'
  email_configs:
  - to: 'oncall@example.com'
    subject: '[CRITICAL] Caxton Alert'
    body: |
      Alert: {{ .GroupLabels.alertname }}
      Summary: {{ .CommonAnnotations.summary }}
      Description: {{ .CommonAnnotations.description }}
  slack_configs:
  - channel: '#alerts-critical'
    title: 'Critical Alert: {{ .GroupLabels.alertname }}'
    text: '{{ .CommonAnnotations.summary }}'

- name: 'caxton-team'
  slack_configs:
  - channel: '#caxton-alerts'
    title: 'Caxton Alert: {{ .GroupLabels.alertname }}'
    text: '{{ .CommonAnnotations.summary }}'
```

## Dashboard Setup

### Grafana Dashboards

#### Runtime Overview Dashboard

```json
{
  "dashboard": {
    "title": "Caxton Runtime Overview",
    "panels": [
      {
        "title": "Active Agents",
        "type": "stat",
        "targets": [
          {
            "expr": "caxton_agents_active",
            "legendFormat": "Active Agents"
          }
        ]
      },
      {
        "title": "Agent Execution Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(caxton_agent_executions_total[5m])",
            "legendFormat": "Executions/sec"
          }
        ]
      },
      {
        "title": "Memory Usage by Agent Type",
        "type": "graph",
        "targets": [
          {
            "expr": "sum by (agent_type) (caxton_wasm_memory_usage_bytes)",
            "legendFormat": "{{ agent_type }}"
          }
        ]
      },
      {
        "title": "Response Time Percentiles",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(caxton_http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          },
          {
            "expr": "histogram_quantile(0.50, rate(caxton_http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "50th percentile"
          }
        ]
      }
    ]
  }
}
```

### Performance Monitoring Dashboard

Key performance indicators to monitor:

1. **Throughput Metrics**:

   - Requests per second
   - Agent executions per second
   - Message processing rate

2. **Latency Metrics**:

   - Request response time
   - Agent execution time
   - Message processing delay

3. **Resource Utilization**:

   - CPU usage
   - Memory consumption
   - Disk I/O
   - Network I/O

4. **Error Rates**:

   - HTTP error responses
   - Agent execution failures
   - Message processing errors

### Custom Grafana Plugins

Install useful plugins for enhanced monitoring:

```bash
# Install Grafana plugins
grafana-cli plugins install grafana-polystat-panel
grafana-cli plugins install grafana-worldmap-panel
grafana-cli plugins install grafana-piechart-panel
```

## Performance Monitoring

### Benchmarking

Set up automated performance benchmarks:

```yaml
# benchmark-cronjob.yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: caxton-benchmark
spec:
  schedule: "0 2 * * *"  # Run daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: benchmark
            image: caxton/benchmark:latest
            command:
            - /bin/sh
            - -c
            - |
              /benchmark --target http://caxton-service:8080 \
                        --duration 300s \
                        --concurrent-users 100 \
                        --report-to-prometheus http://prometheus:9090
          restartPolicy: OnFailure
```

### Load Testing

Example load test configuration:

```javascript
// k6-load-test.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = {
  stages: [
    { duration: '2m', target: 100 },  // Ramp up to 100 users
    { duration: '5m', target: 100 },  // Stay at 100 users
    { duration: '2m', target: 200 },  // Ramp up to 200 users
    { duration: '5m', target: 200 },  // Stay at 200 users
    { duration: '2m', target: 0 },    // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<2000'], // 95% of requests under 2s
    http_req_failed: ['rate<0.1'],     // Error rate under 10%
  },
};

export default function() {
  const response = http.post('http://caxton.local:8080/api/agents', {
    agent_type: 'test-agent',
    config: { test: true }
  });

  check(response, {
    'status is 200': (r) => r.status === 200,
    'response time < 1000ms': (r) => r.timings.duration < 1000,
  });

  sleep(1);
}
```

### Continuous Profiling

Enable continuous profiling in production:

```toml
[profiling]
enabled = true
endpoint = "http://pyroscope:4040"
application_name = "caxton-runtime"
server_address = "0.0.0.0:6060"
sample_rate = 100  # Hz
profile_types = ["cpu", "alloc_objects", "alloc_space", "inuse_objects", "inuse_space"]
```

## Troubleshooting Monitoring

### Common Issues

1. **Missing Metrics**:

   - Check OpenTelemetry collector configuration
   - Verify network connectivity
   - Review Prometheus scrape configuration

2. **High Cardinality**:

   - Limit label values
   - Use recording rules for pre-aggregation
   - Implement metric sampling

3. **Trace Sampling Issues**:

   - Adjust sampling rates
   - Check trace context propagation
   - Verify Jaeger storage capacity

### Debug Commands

```bash
# Check metrics endpoint
curl http://caxton:9090/metrics | grep caxton_

# Validate OpenTelemetry export
curl -X POST http://otel-collector:4318/v1/traces \
  -H "Content-Type: application/json" \
  -d '{"test": "data"}'

# Query Prometheus
curl 'http://prometheus:9090/api/v1/query?query=up'

# Check Jaeger traces
curl http://jaeger:16686/api/traces?service=caxton-runtime
```

For more operational guidance, see the \[Deployment Guide\]({{
'/docs/operations/deployment/' | relative_url }}) and \[Security Guide\]({{
'/docs/operations/security/' | relative_url }}).
