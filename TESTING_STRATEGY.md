# Data Designer Testing Strategy

## Overview

Comprehensive testing strategy for the Data Designer project, focusing on reliable end-to-end testing, service isolation, and debugging capabilities with Elasticsearch integration.

## 🎯 Testing Philosophy

**Time invested in test suite is never wasted** - Comprehensive testing prevents regressions, enables confident refactoring, and provides excellent debugging capabilities for complex data flows.

## 🏗️ Test Architecture

### 1. Test Organization (Separate Libraries)

```
data-designer/
├── tests/                          # Integration tests
│   ├── lib.rs                      # Main test harness
│   ├── common/                     # Shared test utilities
│   │   ├── fixtures.rs             # Test data fixtures
│   │   ├── helpers.rs              # Test helper functions
│   │   ├── elasticsearch.rs        # ES test logging
│   │   └── mock_services.rs        # Service mocking
│   ├── unit/                       # Unit test modules
│   │   ├── parser_tests.rs         # Parser unit tests
│   │   ├── evaluator_tests.rs      # Evaluator unit tests
│   │   ├── engine_tests.rs         # Engine unit tests
│   │   └── db_tests.rs             # Database unit tests
│   ├── integration/                # Integration test modules
│   │   ├── grpc_service_tests.rs   # gRPC service tests
│   │   ├── web_ui_tests.rs         # Web UI tests
│   │   ├── template_api_tests.rs   # Template API tests
│   │   └── elasticsearch_tests.rs  # ES integration tests
│   ├── e2e/                        # End-to-end test flows
│   │   ├── full_pipeline_tests.rs  # Complete workflow tests
│   │   ├── user_journey_tests.rs   # User interaction flows
│   │   └── performance_tests.rs    # Performance benchmarks
│   └── debug/                      # Debug and testing UI
│       ├── test_runner_ui.rs       # Interactive test runner
│       ├── data_flow_viewer.rs     # Data flow visualization
│       └── log_analyzer.rs         # Log analysis tools
└── test-libs/                      # Separate test libraries
    ├── test-harness/              # Core testing framework
    ├── test-data/                 # Test data management
    ├── test-services/             # Mock service implementations
    └── test-ui/                   # UI testing components
```

### 2. Test Categories

#### A. Unit Tests (Isolated Component Testing)
- **Parser Tests**: DSL parsing, AST generation, error handling
- **Evaluator Tests**: Expression evaluation, function calls, type checking
- **Engine Tests**: Rule execution, dependency resolution
- **Database Tests**: CRUD operations, query validation, migrations

#### B. Integration Tests (Service Interaction Testing)
- **gRPC Service Tests**: API endpoints, request/response validation
- **Database Integration**: Real DB operations with test data
- **Template API Tests**: Template CRUD, instantiation flows
- **Elasticsearch Integration**: Logging, search, data indexing

#### C. End-to-End Tests (Complete Workflow Testing)
- **Full Pipeline Tests**: Frontend → gRPC → Database → Response
- **User Journey Tests**: Complete user workflows
- **Performance Tests**: Load testing, response time validation
- **Error Propagation Tests**: Error handling across services

## 🔍 Test Data Management

### Test Fixtures Strategy
```rust
// tests/common/fixtures.rs
pub struct TestFixtures {
    pub sample_rules: Vec<Rule>,
    pub sample_templates: Vec<ResourceTemplate>,
    pub sample_dictionaries: Vec<DataDictionary>,
    pub elasticsearch_indices: Vec<String>,
}

impl TestFixtures {
    pub fn load() -> Self { /* Load from JSON/YAML files */ }
    pub fn clean_elasticsearch(&self) { /* Clean ES indices */ }
    pub fn reset_database(&self) { /* Reset test DB */ }
}
```

### Test Data Sources
- **JSON Fixtures**: Static test data for predictable scenarios
- **Generated Data**: Dynamic test data for edge cases
- **Real Data Samples**: Anonymized production-like data
- **Error Cases**: Malformed data for negative testing

## 📊 Elasticsearch Integration for Debugging

### Logging Strategy
```rust
// tests/common/elasticsearch.rs
pub struct TestLogger {
    es_client: elasticsearch::Elasticsearch,
    test_run_id: String,
    index_prefix: String,
}

#[derive(Serialize)]
pub struct TestLogEntry {
    timestamp: DateTime<Utc>,
    test_run_id: String,
    test_name: String,
    component: String,
    level: LogLevel,
    message: String,
    data: serde_json::Value,
    trace_id: String,
}
```

### Log Categories
- **Request/Response Logs**: All gRPC calls with timing
- **Database Queries**: SQL queries with execution time
- **Parser Operations**: DSL parsing steps and AST generation
- **Engine Execution**: Rule evaluation with intermediate results
- **Error Traces**: Complete error propagation chains

### Debug Dashboard
- **Real-time Test Monitoring**: Live test execution with Kibana
- **Data Flow Visualization**: Request flow through services
- **Performance Metrics**: Response times, throughput
- **Error Analysis**: Error patterns and root cause analysis

## 🧪 Test Infrastructure

### Test Harness (Separate Library)
```rust
// test-libs/test-harness/src/lib.rs
pub struct TestHarness {
    pub elasticsearch: ElasticsearchTestClient,
    pub database: TestDatabase,
    pub grpc_services: MockGrpcServices,
    pub fixtures: TestFixtures,
}

impl TestHarness {
    pub async fn setup() -> Result<Self> { /* Initialize all services */ }
    pub async fn cleanup(&self) { /* Clean up all resources */ }
    pub fn trace_request(&self, trace_id: &str) -> RequestTrace { /* Full request tracing */ }
}
```

### Service Mocking
```rust
// test-libs/test-services/src/lib.rs
pub struct MockGrpcServer {
    responses: HashMap<String, Response>,
    delays: HashMap<String, Duration>,
    failures: HashMap<String, Error>,
}

impl MockGrpcServer {
    pub fn expect_call(&mut self, method: &str) -> CallExpectation { /* */ }
    pub fn simulate_delay(&mut self, method: &str, delay: Duration) { /* */ }
    pub fn simulate_failure(&mut self, method: &str, error: Error) { /* */ }
}
```

## 🔄 End-to-End Test Flows

### 1. Complete Data Pipeline Test
```rust
#[tokio::test]
async fn test_complete_data_pipeline() {
    let harness = TestHarness::setup().await.unwrap();
    let trace_id = Uuid::new_v4().to_string();

    // 1. Frontend sends DSL rule
    let dsl_rule = harness.fixtures.sample_rules[0].clone();
    let response = harness.grpc_client
        .create_rule(CreateRuleRequest { dsl: dsl_rule, trace_id: trace_id.clone() })
        .await.unwrap();

    // 2. Verify database persistence
    let stored_rule = harness.database
        .find_rule_by_id(&response.rule_id)
        .await.unwrap();

    // 3. Test rule execution
    let execution_result = harness.grpc_client
        .execute_rule(ExecuteRuleRequest {
            rule_id: response.rule_id,
            input_data: test_data,
            trace_id: trace_id.clone()
        })
        .await.unwrap();

    // 4. Verify Elasticsearch logging
    let logs = harness.elasticsearch
        .get_logs_for_trace(&trace_id)
        .await.unwrap();

    assert!(logs.len() > 5); // Minimum expected log entries
    assert_contains_components(&logs, vec!["grpc", "database", "engine", "parser"]);

    // 5. Performance assertions
    let total_time = logs.last().unwrap().timestamp - logs.first().unwrap().timestamp;
    assert!(total_time < Duration::from_millis(500)); // Max 500ms for complete flow
}
```

### 2. Error Propagation Test
```rust
#[tokio::test]
async fn test_error_propagation_flow() {
    let harness = TestHarness::setup().await.unwrap();

    // Inject database failure
    harness.database.simulate_failure("insert_rule", DatabaseError::ConnectionLost);

    let result = harness.grpc_client.create_rule(invalid_request).await;

    // Verify error propagation
    assert!(result.is_err());

    // Verify error logging in Elasticsearch
    let error_logs = harness.elasticsearch
        .get_error_logs_for_component("database")
        .await.unwrap();

    assert!(!error_logs.is_empty());
    assert_eq!(error_logs[0].error_type, "ConnectionLost");
}
```

## 🎛️ Interactive Debug UI

### Test Runner Interface
```rust
// tests/debug/test_runner_ui.rs
pub struct TestRunnerUI {
    elasticsearch: ElasticsearchClient,
    test_filter: String,
    selected_trace: Option<String>,
}

impl TestRunnerUI {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        // Test selection and filtering
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.test_filter);
            if ui.button("Run Selected Tests").clicked() {
                self.run_filtered_tests();
            }
        });

        // Real-time test progress
        self.render_test_progress(ui);

        // Live log streaming from Elasticsearch
        self.render_live_logs(ui);

        // Data flow visualization
        if let Some(trace_id) = &self.selected_trace {
            self.render_trace_visualization(ui, trace_id);
        }
    }
}
```

### Data Flow Visualization
- **Request Flow Graph**: Visual representation of request path
- **Timing Analysis**: Timeline of operations with bottlenecks
- **Data Transformation**: Show data changes at each stage
- **Error Injection Points**: Interactive failure simulation

## 📈 Performance Testing

### Benchmarks
```rust
// tests/e2e/performance_tests.rs
#[tokio::test]
async fn benchmark_rule_execution() {
    let harness = TestHarness::setup().await.unwrap();
    let rules = harness.fixtures.generate_rules(1000);

    let start_time = Instant::now();

    for rule in rules {
        harness.grpc_client.execute_rule(rule).await.unwrap();
    }

    let total_time = start_time.elapsed();
    let avg_time = total_time / 1000;

    // Performance assertions
    assert!(avg_time < Duration::from_millis(10)); // Max 10ms per rule

    // Log performance metrics to Elasticsearch
    harness.elasticsearch.log_performance_metrics(PerformanceMetrics {
        test_name: "rule_execution_benchmark",
        total_time,
        avg_time,
        throughput: 1000.0 / total_time.as_secs_f64(),
        memory_usage: get_memory_usage(),
    }).await.unwrap();
}
```

## 🚀 Implementation Plan

### Phase 1: Test Infrastructure (Week 1)
1. **Create test-harness library** with Elasticsearch integration
2. **Set up test database** with automated migrations
3. **Create test fixtures** and data management
4. **Implement basic service mocking**

### Phase 2: Unit Test Migration (Week 2)
1. **Move existing tests** to new structure
2. **Add comprehensive parser tests** with edge cases
3. **Create evaluator test suite** with error scenarios
4. **Add database operation tests** with transaction testing

### Phase 3: Integration Testing (Week 3)
1. **Implement gRPC service tests** with real client/server
2. **Add template API integration tests**
3. **Create Elasticsearch integration tests**
4. **Add database integration tests** with real DB operations

### Phase 4: End-to-End Testing (Week 4)
1. **Implement complete pipeline tests** with tracing
2. **Add user journey tests** with realistic workflows
3. **Create performance benchmarks** with alerting
4. **Build error propagation tests** with fault injection

### Phase 5: Debug UI (Week 5)
1. **Create interactive test runner** with live monitoring
2. **Build data flow visualization** with Kibana integration
3. **Add log analysis tools** with pattern detection
4. **Implement performance dashboards** with historical trends

## 🔧 Tools and Dependencies

### Required Crates
```toml
[dev-dependencies]
# Test Framework
tokio-test = "0.4"
criterion = "0.5"
proptest = "1.0"

# Elasticsearch Integration
elasticsearch = "8.5"
serde_json = "1.0"

# Mocking and Fixtures
mockall = "0.11"
wiremock = "0.5"

# UI Testing
egui = "0.24"
eframe = "0.24"

# Database Testing
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid"] }

# Async Testing
futures = "0.3"
tokio = { version = "1.0", features = ["full"] }
```

### External Services
- **Elasticsearch**: Log aggregation and search
- **Kibana**: Test monitoring dashboard
- **PostgreSQL**: Test database with isolation
- **Docker Compose**: Service orchestration for tests

## 📊 Success Metrics

### Test Coverage Goals
- **Unit Test Coverage**: >90% line coverage
- **Integration Test Coverage**: All API endpoints tested
- **E2E Test Coverage**: All major user workflows covered
- **Performance Tests**: All critical paths benchmarked

### Quality Metrics
- **Test Execution Time**: <2 minutes for full suite
- **Test Reliability**: <1% flaky test rate
- **Debug Efficiency**: <5 minutes to trace any issue
- **Deployment Confidence**: 100% automated testing before deploy

This comprehensive testing strategy ensures that the Data Designer project has robust, maintainable tests that provide excellent debugging capabilities and give confidence in the system's reliability.