# Data Designer Testing Infrastructure

## 🎯 Quick Start

Run the comprehensive test suite with Elasticsearch logging:

```bash
# Start Elasticsearch (required for test logging)
./elasticsearch.sh start

# Run end-to-end tests with full tracing
cd tests && cargo test test_complete_data_pipeline_with_tracing

# Set up log autopurge (cleans logs older than 7 days)
./elasticsearch-autopurge-cron.sh setup
```

## 🏗️ Testing Strategy Overview

**Time invested in test suite is never wasted** - This comprehensive testing infrastructure provides:

- ✅ **Elasticsearch Integration**: All test events logged for debugging
- ✅ **End-to-End Tracing**: Complete request flow visualization
- ✅ **Service Isolation**: Separate test databases and mock services
- ✅ **Performance Monitoring**: Automated performance assertions
- ✅ **Error Propagation Testing**: Complete error handling verification

## 🧪 Test Categories

### 1. Unit Tests
- **Parser Tests**: DSL parsing, AST generation
- **Evaluator Tests**: Expression evaluation, type checking
- **Engine Tests**: Rule execution, dependency resolution
- **Database Tests**: CRUD operations, migrations

### 2. Integration Tests
- **gRPC Service Tests**: API endpoints, request/response validation
- **Database Integration**: Real DB operations with isolated schemas
- **Template API Tests**: Template management workflows
- **Elasticsearch Integration**: Logging, search, indexing

### 3. End-to-End Tests
- **Complete Pipeline**: Frontend → gRPC → Database → Response
- **User Journey Tests**: Full user interaction workflows
- **Performance Tests**: Load testing, response time validation
- **Error Scenarios**: Error handling across all services

## 🔍 Elasticsearch Test Logging

### View Test Logs in Real-Time

```bash
# Open Kibana dashboard
open http://localhost:5601

# Or query logs directly
curl "http://localhost:9200/test-logs-*/_search?q=test_name:complete_data_pipeline"
```

### Log Structure
```json
{
  "timestamp": "2024-10-18T10:30:00Z",
  "test_run_id": "uuid-test-run",
  "trace_id": "uuid-trace",
  "test_name": "complete_data_pipeline",
  "component": "grpc_server",
  "event_type": "GrpcCall",
  "level": "Info",
  "message": "Rule executed successfully",
  "data": {
    "execution_time_ms": 45,
    "rule_id": "age_category_rule"
  }
}
```

## 🚀 Running Tests

### Prerequisites
```bash
# Start required services
./elasticsearch.sh start
./database/setup.sh

# Verify setup
./elasticsearch-autopurge-cron.sh status
```

### Run Full Test Suite
```bash
# All tests with Elasticsearch logging
cargo test --all

# Specific test categories
cargo test --test integration     # Integration tests only
cargo test --test e2e             # End-to-end tests only
```

### Run Individual Test Examples

```bash
# Complete data pipeline test
cd tests && cargo test test_complete_data_pipeline_with_tracing

# Error propagation test
cd tests && cargo test test_error_propagation_and_logging

# Performance benchmark
cd tests && cargo test test_performance_benchmarking
```

## 📊 Test Debugging Workflow

### 1. Run Test with Tracing
```bash
cd tests && cargo test test_complete_data_pipeline_with_tracing -- --nocapture
```

### 2. Get Trace ID from Output
```
🎉 Complete data pipeline test passed!
   📋 Trace ID: 550e8400-e29b-41d4-a716-446655440000
   ⏱️  Total time: 245ms
   📊 Events logged: 12
```

### 3. Analyze in Elasticsearch
```bash
# Get all events for the trace
curl "http://localhost:9200/test-logs-*/_search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": {
      "term": { "trace_id": "550e8400-e29b-41d4-a716-446655440000" }
    },
    "sort": [{ "timestamp": "asc" }]
  }'
```

### 4. View in Kibana
- Open http://localhost:5601
- Create index pattern: `test-logs-*`
- Filter by `trace_id` for specific test run
- View timeline of events across components

## ⚡ Performance Testing

### Run Performance Benchmarks
```bash
cd tests && cargo test test_performance_benchmarking -- --nocapture
```

### View Performance Metrics
```bash
# Query performance events
curl "http://localhost:9200/test-logs-*/_search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": { "term": { "event_type": "Performance" } },
    "aggs": {
      "avg_duration": { "avg": { "field": "duration_ms" } },
      "max_duration": { "max": { "field": "duration_ms" } }
    }
  }'
```

## 🧹 Log Management

### Automatic Cleanup
```bash
# Set up daily cleanup at 2 AM
./elasticsearch-autopurge-cron.sh setup

# Check cleanup status
./elasticsearch-autopurge-cron.sh status

# Manual cleanup (dry run)
./elasticsearch-autopurge.sh --dry-run

# Manual cleanup (7 days retention)
./elasticsearch-autopurge.sh --retention-days 7
```

### Manual Log Queries
```bash
# View recent test runs
curl "http://localhost:9200/test-logs-*/_search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": { "term": { "event_type": "TestStart" } },
    "sort": [{ "timestamp": "desc" }],
    "size": 10
  }'

# Find failed tests
curl "http://localhost:9200/test-logs-*/_search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": { "term": { "level": "Error" } },
    "sort": [{ "timestamp": "desc" }]
  }'
```

## 🔧 Test Configuration

### Environment Variables
```bash
# Elasticsearch configuration
export TEST_ELASTICSEARCH_URL="http://localhost:9200"

# Database configuration
export TEST_DATABASE_URL="postgresql://adamtc007@localhost/data_designer_test"

# Test retention
export RETENTION_DAYS=7

# Enable debug logging
export RUST_LOG=debug
```

### Test Data Management
```bash
# Reset test database
cd database && ./setup.sh

# Validate test fixtures
cd tests && cargo test --lib test_utils::validate_fixtures

# Clean all test data
./elasticsearch.sh clean
```

## 🐛 Debugging Failed Tests

### 1. Check Test Output
```bash
# Run with full output
cargo test failing_test_name -- --nocapture

# Run with debug logging
RUST_LOG=debug cargo test failing_test_name
```

### 2. Query Error Logs
```bash
# Find errors for specific test
curl "http://localhost:9200/test-logs-*/_search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": {
      "bool": {
        "must": [
          { "term": { "test_name": "failing_test_name" } },
          { "term": { "level": "Error" } }
        ]
      }
    }
  }'
```

### 3. Analyze Component Flow
```bash
# Get component timeline for trace
curl "http://localhost:9200/test-logs-*/_search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": { "term": { "trace_id": "your-trace-id" } },
    "sort": [{ "timestamp": "asc" }],
    "aggs": {
      "components": {
        "terms": { "field": "component" },
        "aggs": {
          "avg_duration": { "avg": { "field": "duration_ms" } }
        }
      }
    }
  }'
```

## 📈 Monitoring & Alerts

### Key Metrics to Monitor
- **Test Success Rate**: Should be >95%
- **Average Test Duration**: Should be <500ms for E2E tests
- **Error Rate by Component**: Should be <1%
- **Log Storage Growth**: Managed by autopurge

### Performance Thresholds
- Complete pipeline: <500ms
- Database operations: <100ms
- Parser operations: <50ms
- gRPC calls: <200ms

## 🔗 Related Documentation

- [TESTING_STRATEGY.md](./TESTING_STRATEGY.md) - Comprehensive testing strategy
- [SCRIPTS.md](./SCRIPTS.md) - All utility scripts documentation
- [CLAUDE.md](./CLAUDE.md) - Main project documentation

## 🆘 Troubleshooting

### Elasticsearch Not Starting
```bash
# Check Docker status
docker ps | grep elasticsearch

# Restart Elasticsearch
./elasticsearch.sh restart

# Check logs
./elasticsearch.sh logs
```

### Test Database Issues
```bash
# Recreate test database
dropdb data_designer_test
createdb data_designer_test
cd database && ./setup.sh
```

### Test Fixtures Not Loading
```bash
# Validate fixtures
cd tests && cargo test --lib fixtures::validate_fixtures

# Reset fixtures
rm -rf test-data/generated/
cd tests && cargo test --lib fixtures::load
```