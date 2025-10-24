pub mod elasticsearch;
pub mod database;
pub mod fixtures;
pub mod grpc_testing;
pub mod trace;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub use elasticsearch::ElasticsearchTestClient;
pub use database::TestDatabase;
pub use fixtures::TestFixtures;
pub use grpc_testing::MockGrpcServices;
pub use trace::{RequestTrace, TraceId};

/// Main test harness that coordinates all testing infrastructure
pub struct TestHarness {
    pub elasticsearch: ElasticsearchTestClient,
    pub database: TestDatabase,
    pub grpc_services: MockGrpcServices,
    pub fixtures: TestFixtures,
    pub test_run_id: String,
}

impl TestHarness {
    /// Initialize the complete test harness with all services
    pub async fn setup() -> Result<Self> {
        tracing_subscriber::fmt::init();

        let test_run_id = Uuid::new_v4().to_string();

        let elasticsearch = ElasticsearchTestClient::new(&test_run_id).await?;
        let database = TestDatabase::new().await?;
        let grpc_services = MockGrpcServices::new().await?;
        let fixtures = TestFixtures::load()?;

        Ok(Self {
            elasticsearch,
            database,
            grpc_services,
            fixtures,
            test_run_id,
        })
    }

    /// Clean up all test resources
    pub async fn cleanup(&self) -> Result<()> {
        self.elasticsearch.cleanup().await?;
        self.database.cleanup().await?;
        self.grpc_services.stop().await?;
        Ok(())
    }

    /// Get complete trace for a request
    pub async fn get_request_trace(&self, trace_id: &str) -> Result<RequestTrace> {
        self.elasticsearch.get_request_trace(trace_id).await
    }

    /// Start tracing a new request
    pub fn start_trace(&self, test_name: &str) -> TraceId {
        TraceId::new(test_name, &self.test_run_id)
    }

    /// Log a test event to Elasticsearch
    pub async fn log_test_event(&self, event: TestEvent) -> Result<()> {
        self.elasticsearch.log_event(event).await
    }

    /// Get test metrics for the current run
    pub async fn get_test_metrics(&self) -> Result<TestMetrics> {
        self.elasticsearch.get_test_metrics(&self.test_run_id).await
    }
}

/// Test event logged to Elasticsearch
#[derive(Debug, Serialize, Deserialize)]
pub struct TestEvent {
    pub timestamp: DateTime<Utc>,
    pub test_run_id: String,
    pub trace_id: String,
    pub test_name: String,
    pub component: String,
    pub event_type: EventType,
    pub level: LogLevel,
    pub message: String,
    pub data: serde_json::Value,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EventType {
    TestStart,
    TestEnd,
    GrpcCall,
    DatabaseQuery,
    ParserOperation,
    EngineExecution,
    Error,
    Performance,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Test metrics collected during execution
#[derive(Debug, Serialize, Deserialize)]
pub struct TestMetrics {
    pub test_run_id: String,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub total_duration: chrono::Duration,
    pub component_metrics: HashMap<String, ComponentMetrics>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentMetrics {
    pub component: String,
    pub total_calls: usize,
    pub avg_duration_ms: f64,
    pub error_count: usize,
    pub slowest_operation_ms: u64,
}

/// Performance assertion helpers
pub struct PerformanceAssertions;

impl PerformanceAssertions {
    pub fn assert_max_duration(duration: chrono::Duration, max_ms: u64) {
        assert!(
            duration.num_milliseconds() <= max_ms as i64,
            "Operation took {}ms, expected max {}ms",
            duration.num_milliseconds(),
            max_ms
        );
    }

    pub fn assert_min_throughput(operations: usize, duration: chrono::Duration, min_ops_per_sec: f64) {
        let actual_throughput = operations as f64 / duration.num_milliseconds() as f64 * 1000.0;
        assert!(
            actual_throughput >= min_ops_per_sec,
            "Throughput was {:.2} ops/sec, expected min {:.2} ops/sec",
            actual_throughput,
            min_ops_per_sec
        );
    }
}

/// Test result assertions with Elasticsearch logging
pub struct TestAssertions<'h> {
    harness: &'h TestHarness,
    trace_id: String,
}

impl<'h> TestAssertions<'h> {
    pub fn new(harness: &'h TestHarness, trace_id: String) -> Self {
        Self { harness, trace_id }
    }

    pub async fn assert_components_called(&self, expected_components: Vec<&str>) -> Result<()> {
        let trace = self.harness.get_request_trace(&self.trace_id).await?;
        let actual_components: Vec<String> = trace.events.iter()
            .map(|e| e.component.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for component in expected_components {
            assert!(
                actual_components.contains(&component.to_string()),
                "Component '{}' was not called. Actual components: {:?}",
                component,
                actual_components
            );
        }

        Ok(())
    }

    pub async fn assert_no_errors(&self) -> Result<()> {
        let trace = self.harness.get_request_trace(&self.trace_id).await?;
        let errors: Vec<&TestEvent> = trace.events.iter()
            .filter(|e| matches!(e.level, LogLevel::Error))
            .collect();

        assert!(
            errors.is_empty(),
            "Found {} error(s) in trace: {:?}",
            errors.len(),
            errors
        );

        Ok(())
    }

    pub async fn assert_performance(&self, max_total_duration_ms: u64) -> Result<()> {
        let trace = self.harness.get_request_trace(&self.trace_id).await?;
        let total_duration = trace.total_duration();

        PerformanceAssertions::assert_max_duration(total_duration, max_total_duration_ms);
        Ok(())
    }
}