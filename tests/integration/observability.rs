//! Observability and OpenTelemetry Integration Tests
//! 
//! Tests that verify comprehensive observability features:
//! - Structured logging with correlation IDs
//! - OpenTelemetry trace propagation
//! - Metrics collection and aggregation
//! - Event emission and handling
//! - Performance monitoring

use caxton::*;
use opentelemetry::trace::{Span, Tracer};
use opentelemetry_sdk::trace::{Config, TracerProvider};
use serial_test::serial;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{info, warn, error};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_test::traced_test;

/// Test structured logging with correlation IDs
#[tokio::test]
#[traced_test]
#[serial]
async fn test_structured_logging_correlation() {
    let runtime = setup_observability_runtime().await;
    let log_collector = setup_log_collector();
    
    // Spawn two agents to test correlation
    let agent1_id = runtime.spawn_agent(AgentConfig {
        name: "logging-agent-1".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["logging-test".to_string()],
        max_memory: Some(32 * 1024 * 1024),
        timeout: Some(Duration::from_secs(10)),
    }).await.expect("Failed to spawn logging agent 1");
    
    let agent2_id = runtime.spawn_agent(AgentConfig {
        name: "logging-agent-2".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["logging-test".to_string()],
        max_memory: Some(32 * 1024 * 1024),
        timeout: Some(Duration::from_secs(10)),
    }).await.expect("Failed to spawn logging agent 2");
    
    wait_for_agent_ready(&runtime, &agent1_id).await;
    wait_for_agent_ready(&runtime, &agent2_id).await;
    
    let conversation_id = ConversationId::new();
    
    // Send correlated messages
    let message1 = FipaMessage {
        performative: FipaPerformative::Request,
        sender: AgentId::system(),
        receiver: agent1_id.clone(),
        content: serde_json::json!({"action": "process_data", "data": "batch_1"}),
        conversation_id: Some(conversation_id.clone()),
        protocol: Some("test-correlation".to_string()),
        ..Default::default()
    };
    
    let message2 = FipaMessage {
        performative: FipaPerformative::Request,
        sender: agent1_id.clone(),
        receiver: agent2_id.clone(),
        content: serde_json::json!({"action": "validate_data", "data": "batch_1"}),
        conversation_id: Some(conversation_id.clone()),
        protocol: Some("test-correlation".to_string()),
        ..Default::default()
    };
    
    // Send messages and trigger logging
    runtime.send_message(message1).await.expect("Failed to send message 1");
    runtime.send_message(message2).await.expect("Failed to send message 2");
    
    // Wait for processing to complete
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Verify structured logs contain correlation IDs
    let logs = log_collector.get_logs().await;
    
    let correlated_logs: Vec<_> = logs.iter()
        .filter(|log| log.fields.get("conversation_id") == Some(&conversation_id.to_string()))
        .collect();
    
    assert!(correlated_logs.len() >= 4); // At least 2 sends + 2 receives
    
    // Verify all correlated logs have consistent structure
    for log in &correlated_logs {
        assert!(log.fields.contains_key("agent_id"));
        assert!(log.fields.contains_key("message_type"));
        assert!(log.fields.contains_key("conversation_id"));
        assert!(log.fields.contains_key("timestamp"));
        assert_eq!(log.fields.get("conversation_id").unwrap(), &conversation_id.to_string());
    }
    
    // Verify trace context propagation
    let trace_ids: std::collections::HashSet<_> = correlated_logs.iter()
        .filter_map(|log| log.fields.get("trace_id"))
        .collect();
    
    assert_eq!(trace_ids.len(), 1, "All correlated logs should share the same trace ID");
}

/// Test OpenTelemetry trace propagation across agents
#[tokio::test]
#[traced_test]
#[serial]
async fn test_opentelemetry_trace_propagation() {
    let runtime = setup_observability_runtime().await;
    let trace_collector = setup_trace_collector();
    
    // Create a distributed trace scenario
    let coordinator_id = runtime.spawn_agent(AgentConfig {
        name: "trace-coordinator".to_string(),
        agent_type: AgentType::Coordinator,
        capabilities: vec!["orchestration".to_string()],
        max_memory: Some(32 * 1024 * 1024),
        timeout: Some(Duration::from_secs(15)),
    }).await.expect("Failed to spawn trace coordinator");
    
    let worker_ids: Vec<_> = (0..3).map(|i| {
        runtime.spawn_agent(AgentConfig {
            name: format!("trace-worker-{}", i),
            agent_type: AgentType::Worker,
            capabilities: vec!["task-execution".to_string()],
            max_memory: Some(16 * 1024 * 1024),
            timeout: Some(Duration::from_secs(15)),
        })
    }).collect::<Vec<_>>();
    
    // Wait for all agents to be ready
    for agent_id in std::iter::once(&coordinator_id).chain(worker_ids.iter()) {
        wait_for_agent_ready(&runtime, agent_id).await;
    }
    
    // Create root span
    let tracer = runtime.get_tracer();
    let root_span = tracer.start("distributed-processing");
    let trace_id = root_span.span_context().trace_id();
    
    // Coordinator initiates distributed work
    let work_distribution = FipaMessage {
        performative: FipaPerformative::Request,
        sender: coordinator_id.clone(),
        receiver: worker_ids[0].clone(),
        content: serde_json::json!({
            "action": "distributed_task",
            "subtasks": ["task_a", "task_b", "task_c"],
            "trace_context": extract_trace_context(&root_span)
        }),
        protocol: Some("distributed-tracing".to_string()),
        ..Default::default()
    };
    
    runtime.send_message(work_distribution).await
        .expect("Failed to send work distribution");
    
    // Wait for distributed processing
    tokio::time::sleep(Duration::from_secs(2)).await;
    root_span.end();
    
    // Verify trace collection
    let traces = trace_collector.get_traces().await;
    let distributed_trace = traces.iter()
        .find(|t| t.trace_id == trace_id)
        .expect("Distributed trace not found");
    
    // Verify span hierarchy
    assert!(distributed_trace.spans.len() >= 4); // Root + 3 workers
    
    let root_spans: Vec<_> = distributed_trace.spans.iter()
        .filter(|s| s.parent_span_id.is_none())
        .collect();
    assert_eq!(root_spans.len(), 1);
    assert_eq!(root_spans[0].name, "distributed-processing");
    
    // Verify child spans from different agents
    let agent_spans: HashMap<AgentId, Vec<_>> = distributed_trace.spans.iter()
        .filter(|s| s.parent_span_id.is_some())
        .fold(HashMap::new(), |mut acc, span| {
            if let Some(agent_id) = span.attributes.get("agent_id") {
                acc.entry(agent_id.clone()).or_insert_with(Vec::new).push(span);
            }
            acc
        });
    
    assert!(agent_spans.len() >= 3); // At least 3 worker agents created spans
    
    // Verify span attributes contain agent context
    for spans_for_agent in agent_spans.values() {
        for span in spans_for_agent {
            assert!(span.attributes.contains_key("agent_id"));
            assert!(span.attributes.contains_key("agent_type"));
            assert!(span.attributes.contains_key("message_type"));
        }
    }
}

/// Test metrics collection and aggregation
#[tokio::test]
#[traced_test]
#[serial]
async fn test_metrics_collection() {
    let runtime = setup_observability_runtime().await;
    let metrics_collector = setup_metrics_collector();
    
    // Spawn agents for metrics testing
    let agent_ids: Vec<_> = (0..5).map(|i| {
        runtime.spawn_agent(AgentConfig {
            name: format!("metrics-agent-{}", i),
            agent_type: AgentType::Worker,
            capabilities: vec!["metrics-generation".to_string()],
            max_memory: Some(16 * 1024 * 1024),
            timeout: Some(Duration::from_secs(10)),
        })
    }).collect::<Vec<_>>();
    
    // Wait for all agents to be ready
    for agent_id in &agent_ids {
        wait_for_agent_ready(&runtime, agent_id).await;
    }
    
    // Generate various types of activity to collect metrics
    let mut message_count = 0;
    let start_time = Instant::now();
    
    // Send different types of messages
    for (i, agent_id) in agent_ids.iter().enumerate() {
        // Health checks
        let health_msg = create_test_message(agent_id, "health_check", serde_json::json!({}));
        runtime.send_message(health_msg).await.expect("Failed to send health check");
        message_count += 1;
        
        // CPU intensive tasks
        let cpu_msg = create_test_message(agent_id, "cpu_task", serde_json::json!({"iterations": 1000}));
        runtime.send_message(cpu_msg).await.expect("Failed to send CPU task");
        message_count += 1;
        
        // Memory allocation tasks
        let memory_msg = create_test_message(agent_id, "memory_task", serde_json::json!({"size": 1024 * i}));
        runtime.send_message(memory_msg).await.expect("Failed to send memory task");
        message_count += 1;
    }
    
    // Wait for all tasks to complete
    tokio::time::sleep(Duration::from_secs(3)).await;
    let test_duration = start_time.elapsed();
    
    // Collect and verify metrics
    let metrics = metrics_collector.get_metrics().await;
    
    // System-level metrics
    assert!(metrics.system.agent_count >= 5);
    assert!(metrics.system.total_messages >= message_count);
    assert!(metrics.system.uptime > Duration::from_secs(2));
    assert!(metrics.system.memory_used > 0);
    assert!(metrics.system.cpu_usage >= 0.0);
    
    // Message metrics
    assert_eq!(metrics.messages.total_sent, message_count);
    assert!(metrics.messages.total_received >= message_count);
    assert!(metrics.messages.avg_latency > Duration::from_nanos(0));
    assert_eq!(metrics.messages.failed_sends, 0);
    
    // Per-agent metrics
    assert_eq!(metrics.agents.len(), 5);
    
    for (agent_id, agent_metrics) in &metrics.agents {
        assert!(agent_ids.contains(agent_id));
        assert!(agent_metrics.messages_processed >= 3); // health + cpu + memory
        assert!(agent_metrics.cpu_time > Duration::from_nanos(0));
        assert!(agent_metrics.memory_used > 0);
        assert_eq!(agent_metrics.errors, 0);
        assert!(agent_metrics.uptime <= test_duration);
    }
    
    // Performance metrics
    let perf_metrics = runtime.get_performance_metrics().await;
    assert!(perf_metrics.messages_per_second > 0.0);
    assert!(perf_metrics.avg_response_time > Duration::from_nanos(0));
    assert!(perf_metrics.p95_response_time >= perf_metrics.avg_response_time);
    assert!(perf_metrics.throughput_bytes_per_sec >= 0);
}

/// Test event emission and handling
#[tokio::test]
#[traced_test]
#[serial]
async fn test_event_emission() {
    let runtime = setup_observability_runtime().await;
    let event_collector = setup_event_collector();
    
    let agent_id = runtime.spawn_agent(AgentConfig {
        name: "event-test-agent".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["event-generation".to_string()],
        max_memory: Some(32 * 1024 * 1024),
        timeout: Some(Duration::from_secs(10)),
    }).await.expect("Failed to spawn event test agent");
    
    wait_for_agent_ready(&runtime, &agent_id).await;
    
    // Trigger various events
    let events_to_trigger = vec![
        ("state_change", serde_json::json!({"from": "ready", "to": "processing"})),
        ("message_received", serde_json::json!({"type": "request", "size": 1024})),
        ("task_completed", serde_json::json!({"task_id": "task_123", "duration_ms": 250})),
        ("error_occurred", serde_json::json!({"error": "timeout", "context": "network_call"})),
        ("resource_usage", serde_json::json!({"memory_mb": 16, "cpu_percent": 25.5})),
    ];
    
    for (event_type, event_data) in &events_to_trigger {
        let event_msg = create_test_message(&agent_id, "emit_event", serde_json::json!({
            "event_type": event_type,
            "event_data": event_data
        }));
        
        runtime.send_message(event_msg).await
            .expect("Failed to send event trigger message");
        
        // Small delay between events
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Wait for event processing
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Verify events were emitted and collected
    let collected_events = event_collector.get_events().await;
    
    assert!(collected_events.len() >= events_to_trigger.len());
    
    // Verify each event type was emitted
    for (expected_type, expected_data) in &events_to_trigger {
        let matching_events: Vec<_> = collected_events.iter()
            .filter(|e| e.event_type == *expected_type && e.agent_id == agent_id)
            .collect();
        
        assert!(!matching_events.is_empty(), "No events found for type: {}", expected_type);
        
        let event = matching_events.first().unwrap();
        assert_eq!(event.agent_id, agent_id);
        assert!(event.timestamp > std::time::SystemTime::UNIX_EPOCH);
        assert!(event.trace_id.is_some());
        
        // Verify event data structure
        match expected_type {
            &"state_change" => {
                assert!(event.data.get("from").is_some());
                assert!(event.data.get("to").is_some());
            }
            &"message_received" => {
                assert!(event.data.get("type").is_some());
                assert!(event.data.get("size").is_some());
            }
            &"task_completed" => {
                assert!(event.data.get("task_id").is_some());
                assert!(event.data.get("duration_ms").is_some());
            }
            &"error_occurred" => {
                assert!(event.data.get("error").is_some());
                assert!(event.data.get("context").is_some());
            }
            &"resource_usage" => {
                assert!(event.data.get("memory_mb").is_some());
                assert!(event.data.get("cpu_percent").is_some());
            }
            _ => {}
        }
    }
    
    // Verify event ordering and timestamps
    let agent_events: Vec<_> = collected_events.iter()
        .filter(|e| e.agent_id == agent_id)
        .collect();
    
    // Events should be ordered by timestamp
    for window in agent_events.windows(2) {
        assert!(window[0].timestamp <= window[1].timestamp);
    }
}

/// Test performance monitoring and alerting
#[tokio::test] 
#[traced_test]
#[serial]
async fn test_performance_monitoring() {
    let runtime = setup_observability_runtime().await;
    let performance_monitor = setup_performance_monitor();
    
    // Create agents with different performance characteristics
    let fast_agent_id = runtime.spawn_agent(AgentConfig {
        name: "fast-agent".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["fast-processing".to_string()],
        max_memory: Some(16 * 1024 * 1024),
        timeout: Some(Duration::from_secs(5)),
    }).await.expect("Failed to spawn fast agent");
    
    let slow_agent_id = runtime.spawn_agent(AgentConfig {
        name: "slow-agent".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["slow-processing".to_string()],
        max_memory: Some(16 * 1024 * 1024),
        timeout: Some(Duration::from_secs(5)),
    }).await.expect("Failed to spawn slow agent");
    
    wait_for_agent_ready(&runtime, &fast_agent_id).await;
    wait_for_agent_ready(&runtime, &slow_agent_id).await;
    
    // Send performance test messages
    let num_messages = 10;
    let mut fast_latencies = Vec::new();
    let mut slow_latencies = Vec::new();
    
    for i in 0..num_messages {
        // Fast agent - simple task
        let start = Instant::now();
        let fast_msg = create_test_message(&fast_agent_id, "fast_task", serde_json::json!({"id": i}));
        runtime.send_message(fast_msg).await.expect("Failed to send fast message");
        fast_latencies.push(start.elapsed());
        
        // Slow agent - complex task
        let start = Instant::now();
        let slow_msg = create_test_message(&slow_agent_id, "slow_task", serde_json::json!({"id": i}));
        runtime.send_message(slow_msg).await.expect("Failed to send slow message");
        slow_latencies.push(start.elapsed());
        
        // Small delay between batches
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    // Wait for all processing to complete
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Analyze performance metrics
    let perf_data = performance_monitor.get_performance_data().await;
    
    // Verify agent-specific performance metrics
    let fast_metrics = perf_data.agents.get(&fast_agent_id).unwrap();
    let slow_metrics = perf_data.agents.get(&slow_agent_id).unwrap();
    
    // Fast agent should have better performance characteristics
    assert!(fast_metrics.avg_response_time < slow_metrics.avg_response_time);
    assert!(fast_metrics.throughput > slow_metrics.throughput);
    assert!(fast_metrics.p95_latency < slow_metrics.p95_latency);
    
    // Check for performance alerts
    let alerts = performance_monitor.get_alerts().await;
    
    // Should have alerts for slow performance
    let slow_agent_alerts: Vec<_> = alerts.iter()
        .filter(|a| a.agent_id == slow_agent_id && a.alert_type == "high_latency")
        .collect();
    
    assert!(!slow_agent_alerts.is_empty(), "Expected performance alerts for slow agent");
    
    // Verify alert details
    for alert in &slow_agent_alerts {
        assert!(alert.threshold_exceeded);
        assert!(alert.current_value > alert.threshold);
        assert!(alert.timestamp > std::time::SystemTime::UNIX_EPOCH);
    }
    
    // System-wide performance metrics
    assert!(perf_data.system.total_requests >= num_messages * 2);
    assert!(perf_data.system.requests_per_second > 0.0);
    assert!(perf_data.system.error_rate <= 0.05); // Less than 5% error rate
}

// Helper functions and test setup

async fn setup_observability_runtime() -> CaxtonRuntime {
    CaxtonRuntime::new(CaxtonConfig {
        max_agents: 20,
        default_timeout: Duration::from_secs(30),
        observability_enabled: true,
        tracing_enabled: true,
        metrics_enabled: true,
        event_emission_enabled: true,
        resource_limits: ResourceLimits::default(),
    }).await.expect("Failed to create observability test runtime")
}

fn setup_log_collector() -> Arc<MockLogCollector> {
    Arc::new(MockLogCollector::new())
}

fn setup_trace_collector() -> Arc<MockTraceCollector> {
    Arc::new(MockTraceCollector::new())
}

fn setup_metrics_collector() -> Arc<MockMetricsCollector> {
    Arc::new(MockMetricsCollector::new())
}

fn setup_event_collector() -> Arc<MockEventCollector> {
    Arc::new(MockEventCollector::new())
}

fn setup_performance_monitor() -> Arc<MockPerformanceMonitor> {
    Arc::new(MockPerformanceMonitor::new())
}

fn create_test_message(receiver: &AgentId, action: &str, data: serde_json::Value) -> FipaMessage {
    FipaMessage {
        performative: FipaPerformative::Request,
        sender: AgentId::system(),
        receiver: receiver.clone(),
        content: serde_json::json!({
            "action": action,
            "data": data
        }),
        protocol: Some("observability-test".to_string()),
        ..Default::default()
    }
}

fn extract_trace_context(span: &dyn Span) -> HashMap<String, String> {
    let mut context = HashMap::new();
    let span_context = span.span_context();
    context.insert("trace_id".to_string(), span_context.trace_id().to_string());
    context.insert("span_id".to_string(), span_context.span_id().to_string());
    context
}

async fn wait_for_agent_ready(runtime: &CaxtonRuntime, agent_id: &AgentId) {
    timeout(Duration::from_secs(5), async {
        loop {
            if runtime.get_agent_state(agent_id).await.unwrap() == AgentState::Ready {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }).await.expect("Agent never became ready");
}

// Mock collectors for testing (would be implemented with actual collection logic)

struct MockLogCollector {
    logs: Arc<Mutex<Vec<LogEntry>>>,
}

impl MockLogCollector {
    fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    async fn get_logs(&self) -> Vec<LogEntry> {
        self.logs.lock().unwrap().clone()
    }
}

struct MockTraceCollector {
    traces: Arc<Mutex<Vec<TraceData>>>,
}

impl MockTraceCollector {
    fn new() -> Self {
        Self {
            traces: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    async fn get_traces(&self) -> Vec<TraceData> {
        self.traces.lock().unwrap().clone()
    }
}

struct MockMetricsCollector {
    metrics: Arc<Mutex<SystemMetrics>>,
}

impl MockMetricsCollector {
    fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(SystemMetrics::default())),
        }
    }
    
    async fn get_metrics(&self) -> SystemMetrics {
        self.metrics.lock().unwrap().clone()
    }
}

struct MockEventCollector {
    events: Arc<Mutex<Vec<AgentEvent>>>,
}

impl MockEventCollector {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    async fn get_events(&self) -> Vec<AgentEvent> {
        self.events.lock().unwrap().clone()
    }
}

struct MockPerformanceMonitor {
    performance_data: Arc<Mutex<PerformanceData>>,
    alerts: Arc<Mutex<Vec<PerformanceAlert>>>,
}

impl MockPerformanceMonitor {
    fn new() -> Self {
        Self {
            performance_data: Arc::new(Mutex::new(PerformanceData::default())),
            alerts: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    async fn get_performance_data(&self) -> PerformanceData {
        self.performance_data.lock().unwrap().clone()
    }
    
    async fn get_alerts(&self) -> Vec<PerformanceAlert> {
        self.alerts.lock().unwrap().clone()
    }
}