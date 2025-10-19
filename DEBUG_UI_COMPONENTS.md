# Debug & Testing UI Components

## Overview

Comprehensive debugging and testing UI components have been integrated into the Data Designer web application to address the user's request for better debugging and testing capabilities for UI interactions and data flow tracing.

## Features Implemented

### 🧪 Test Runner Panel
- **Complete Pipeline Tests**: End-to-end testing from AI suggestions through template instantiation to DSL execution
- **Performance Benchmarks**: Load testing and response time validation
- **AI Integration Tests**: Testing AI provider connections and suggestions
- **Real-time Test Monitoring**: Live test execution with step-by-step progress tracking
- **Test History**: Comprehensive test results with timing and error information

### 📋 Elasticsearch Log Viewer
- **Real-time Log Streaming**: View logs from Elasticsearch with filtering capabilities
- **Multi-level Filtering**: Filter by component, message content, or trace ID
- **Structured Display**: Organized log entries with timestamp, level, component, and trace information
- **Integration with Test Harness**: Automatically captures test execution logs

### 🌐 gRPC Request Monitor
- **Request/Response Tracking**: Monitor all gRPC calls with request IDs and trace correlation
- **Performance Metrics**: Track request duration and success rates
- **Error Analysis**: Capture and display gRPC errors with detailed information
- **Payload Inspection**: View full request and response payloads for debugging

### 📊 Performance Metrics Dashboard
- **Live Performance Data**: Real-time metrics for average/max request times
- **Success Rate Monitoring**: Track request success rates with visual indicators
- **System Resource Usage**: Memory and CPU usage tracking
- **Benchmark Tools**: Built-in performance testing capabilities

### 🔍 End-to-End Trace Viewer
- **Complete Data Flow Visualization**: Trace requests across all system components
- **Component Timeline**: Visualize the flow of data through web UI → gRPC → database
- **Trace Correlation**: Link logs, requests, and responses by trace ID
- **Performance Analysis**: See exactly where time is spent in the pipeline

### ❌ Error Tracking & Analysis
- **Centralized Error Collection**: Capture errors from all components
- **Stack Trace Analysis**: Full stack traces for debugging
- **Error Categorization**: Organize errors by type and component
- **Trace Correlation**: Link errors to specific test runs and traces

## How to Use

### Accessing the Debug Interface
1. Click the "🔍 Debug" button in the main navigation
2. The comprehensive debug panel will open on the right side
3. Use the toggle buttons to enable different debug views:
   - 🧪 Test Runner
   - 📋 Logs
   - 🌐 gRPC Monitor
   - 📊 Performance
   - 🔍 Traces
   - ❌ Errors

### Running Tests
1. Open the Test Runner panel
2. Click "▶️ Run Complete Pipeline" to test full data flow
3. Click "🔄 Run Performance Test" for benchmark testing
4. Click "🧠 Run AI Integration Test" for AI provider testing
5. Monitor real-time progress and view detailed results

### Debugging Data Flow
1. Start a test that generates trace data
2. Open the Trace Viewer panel
3. Click "🔄 Refresh Traces" to see active traces
4. Expand component timelines to see detailed flow
5. Correlate with logs and errors using trace IDs

### Monitoring Performance
1. Open the Performance Metrics panel
2. Click "🔄 Update Metrics" for current stats
3. Click "📈 Run Benchmark" for comprehensive testing
4. Monitor success rates and response times

## Integration Points

### Elasticsearch Integration
- Automatically logs all test events to Elasticsearch
- Supports the comprehensive test harness created earlier
- Integrates with autopurge scripts for log management

### gRPC Client Integration
- Hooks into existing gRPC client for request monitoring
- Provides debugging methods: `log_grpc_request()`, `log_grpc_response()`
- Supports trace correlation across service boundaries

### Test Harness Integration
- Works with the separate test libraries created in `test-libs/`
- Supports the complete testing strategy documented in `TESTING_STRATEGY.md`
- Integrates with database isolation and mock services

## Technical Implementation

### Files Created/Modified
- **`web-ui/src/debug_ui.rs`**: Complete debug UI implementation
- **`web-ui/src/lib.rs`**: Added debug_ui module
- **`web-ui/src/app.rs`**: Integrated debug interface into main app
- **`web-ui/Cargo.toml`**: Added UUID dependency for trace generation

### Key Structures
- `DebugTestInterface`: Main debug interface coordinator
- `TestExecution`: Real-time test execution tracking
- `LogEntry`: Elasticsearch log entry representation
- `GrpcRequest/GrpcResponse`: gRPC call monitoring
- `TraceFlow`: End-to-end trace visualization
- `PerformanceMetrics`: System performance tracking

## Benefits Achieved

### "Time Invested in Test Suite is Never Wasted"
- Comprehensive test execution monitoring
- Detailed performance benchmarking
- Complete error tracking and analysis
- End-to-end data flow visualization

### "Debugging and Testing UI is Difficult"
- Visual test execution with real-time progress
- Interactive log filtering and analysis
- gRPC request/response monitoring
- Integrated performance metrics dashboard

### "Test the Flow of Calls and Data Front to Back"
- Complete trace visualization across components
- Request correlation from web UI through gRPC to database
- Performance analysis at each step
- Error tracking with full context

## Future Enhancements

The debug interface is designed to be extensible. Future additions could include:
- Real-time WebSocket integration for live log streaming
- Advanced performance profiling and flame graphs
- Database query analysis and optimization suggestions
- Automated test scheduling and reporting
- Integration with external monitoring services

This comprehensive debugging and testing UI addresses all the user's requirements for better testing infrastructure, making it easier to debug UI interactions and trace data flow throughout the entire system.